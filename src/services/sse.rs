// 一个全局SSE推送服务

use std::sync::{Arc, Mutex};
use axum::{
    response::{
        sse::{Event, KeepAlive, Sse},
    },
};
use redis::AsyncCommands;
use tokio::sync::mpsc;
use tokio_stream::{Stream, StreamExt, wrappers::ReceiverStream};
use crate::errors::app_error::AppError;
use crate::schemas::auth::CurrentUser;
use crate::config::state::{AppState, SSESenders};
use crate::utils::sse::redis_offline_key;


// 负责在 Stream 被 Drop 时，自动从在线用户列表中移除 sender
struct SseStreamCleanup {
    user_uuid: String,
    sse_senders: SSESenders,
}

impl Drop for SseStreamCleanup {
    fn drop(&mut self) {
        let user_uuid = self.user_uuid.clone();
        let sse_senders = self.sse_senders.clone();

        // 不能在 drop 中直接 .await，所以需要生成一个新任务来执行异步的清理操作
        tokio::spawn(async move {
            sse_senders.lock().await.remove(&user_uuid);
            tracing::debug!("用户 {} 的 SSE 连接已关闭，sender 已被自动清理。", user_uuid);
        });
    }
}

// 全局消息推送
#[derive(Clone)]
pub struct SSEService {
    app_state: AppState,
}

impl SSEService {
    pub fn new(app_state: AppState) -> Self {
        Self { app_state }
    }

    async fn send_offline_messages(
        state_clone: AppState,
        user_uuid: String,
        tx: mpsc::Sender<String>,
    ){
        // 1. 获取 Redis 连接
        let mut conn = match state_clone.redis.get_multiplexed_async_connection().await {
            Ok(conn) => conn,
            Err(e) => {
                tracing::error!("[用户 {}] 无法获取离线消息的 Redis 连接：{}”", user_uuid, e);
                return;
            }
        };

        let key = redis_offline_key(user_uuid.clone());

        // 2. 使用 LRANGE 获取所有消息
        // LRANGE key 0 -1  获取列表中的所有元素
        let messages: Vec<String> = match conn.lrange(&key, 0, -1).await {
            Ok(messages) => messages,
            Err(e) => {
                tracing::error!("[用户 {}] 无法从 Redis 键“{}”进行 LRANGE：{}", user_uuid, key, e);
                return;
            }
        };

        // 如果没有离线消息，就提前结束
        if messages.is_empty() {
            return;
        }

        tracing::debug!("[用户 {}] 在 Redis 中发现 {} 条离线消息。正在发送...", user_uuid, messages.len());

        // 3. 倒序发送消息，以保持原始顺序
        // 因为 LPUSH 是 "Left Push"，所以最新的消息在列表的最左边（索引0）。
        // 我们从后往前遍历，就能按时间顺序（旧 -> 新）发送。
        for msg in messages.iter().rev() {
            if tx.send(msg.clone()).await.is_err() {
                // 如果在发送离线消息时客户端就断开了，我们就不再继续发送，
                // 也不删除 Redis 中的消息，等待用户下次上线。
                tracing::error!("[用户 {}] 客户端在发送离线消息时断开连接。正在中止。", user_uuid.clone());
                return; // 退出任务
            }
        }

        // 4. 所有消息都成功发送后，删除 Redis 中的键
        tracing::debug!("[用户 {}] 所有离线消息已发送。正在删除 Redis 键“{}”。", user_uuid, key);

        let _: () = conn.del(&key).await.unwrap_or_else(|e| {
            tracing::error!("[用户 {}] 无法删除 Redis 键“{}”：{} . 可能会导致重复。", user_uuid, key, e);
        });

    }

    pub async fn global_message(
        &self,
        current_user: CurrentUser,
    )-> Sse<impl Stream<Item = Result<Event, AppError>> + use<>> {
        // 创建一个新的 MPSC 通道，用于此用户的 SSE 事件
        let user_uuid = current_user.uuid;
        let (tx, rx) = mpsc::channel(100); // 缓冲区大小为100
        let state_clone = self.app_state.clone();
        let tx_clone = tx.clone();

        // -----------------发送离线消息开始 -----------------
        tokio::spawn(Self::send_offline_messages(state_clone, user_uuid.clone(), tx_clone));

        // -----------------发送离线消息结束 -----------------

        // 将发送端 (tx) 存储到共享状态中，与 user_id 关联
        // 需要一个 `lock` 来安全地修改 HashMap
        self.app_state.sse_senders.lock().await.insert(user_uuid.clone(), tx.clone());
        tracing::debug!("为用户 {} 建立了 SSE 连接", user_uuid.clone());

        let cleanup_guard = SseStreamCleanup {
            user_uuid,
            sse_senders: self.app_state.sse_senders.clone(),
        };

        let stream = ReceiverStream::new(rx)
            .map(|data| Ok(Event::default().data(data)))
            //  使用 `map` 或者 `inspect` 来确保 cleanup_guard 的生命周期和流绑定
            .map(move |res| {
                // 这个闭包借用了 cleanup_guard，只要流还存活，它就存活
                // 当流被 drop 时，这个闭包和它捕获的 cleanup_guard 会一起被 drop
                let _ = &cleanup_guard;
                res
            });

        Sse::new(stream).keep_alive(KeepAlive::default())
    }
}