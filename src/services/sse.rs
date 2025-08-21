// 一个全局SSE推送服务

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
use crate::config::state::AppState;
use crate::utils::sse::redis_offline_key;


// 全局消息推送

#[derive(Clone)]
pub struct SSEService {
    app_state: AppState,
}

impl SSEService {
    pub fn new(app_state: AppState) -> Self {
        Self { app_state }
    }

    pub async fn global_message(
        &self,
        current_user: CurrentUser,
    )-> Sse<impl Stream<Item = Result<Event, AppError>> + use<>> {
        // 创建一个新的 MPSC 通道，用于此用户的 SSE 事件
        let user_id = current_user.user_id as u32;
        let (tx, rx) = mpsc::channel(100); // 缓冲区大小为100
        let state_clone = self.app_state.clone();
        let tx_clone = tx.clone();

        // -----------------发送离线消息开始 -----------------
        tokio::spawn(async move {
            // 1. 获取 Redis 连接
            let mut conn = match state_clone.redis.get_multiplexed_async_connection().await {
                Ok(conn) => conn,
                Err(e) => {
                    tracing::error!("[用户 {}] 无法获取离线消息的 Redis 连接：{}”", user_id, e);
                    return; // 无法连接 Redis，直接退出任务
                }
            };

            let key = redis_offline_key(user_id);

            // 2. 使用 LRANGE 获取所有消息
            // LRANGE key 0 -1  获取列表中的所有元素
            let messages: Vec<String> = match conn.lrange(&key, 0, -1).await {
                Ok(messages) => messages,
                Err(e) => {
                    tracing::error!("[用户 {}] 无法从 Redis 键“{}”进行 LRANGE：{}", user_id, key, e);
                    return;
                }
            };

            // 如果没有离线消息，就提前结束
            if messages.is_empty() {
                return;
            }

            tracing::debug!("[用户 {}] 在 Redis 中发现 {} 条离线消息。正在发送...", user_id, messages.len());

            // 3. 倒序发送消息，以保持原始顺序
            // 因为 LPUSH 是 "Left Push"，所以最新的消息在列表的最左边（索引0）。
            // 我们从后往前遍历，就能按时间顺序（旧 -> 新）发送。
            for msg in messages.iter().rev() {
                if tx_clone.send(msg.clone()).await.is_err() {
                    // 如果在发送离线消息时客户端就断开了，我们就不再继续发送，
                    // 也不删除 Redis 中的消息，等待用户下次上线。
                    tracing::error!("[用户 {}] 客户端在发送离线消息时断开连接。正在中止。", user_id);
                    return; // 退出任务
                }
            }

            // 4. 所有消息都成功发送后，删除 Redis 中的键
            tracing::debug!("[用户 {}] 所有离线消息已发送。正在删除 Redis 键“{}”。", user_id, key);

            let _: () = conn.del(&key).await.unwrap_or_else(|e| {
                tracing::error!("[用户 {}] 无法删除 Redis 键“{}”：{} . 可能会导致重复。", user_id, key, e);
            });

        });

        // -----------------发送离线消息结束 -----------------

        // 将发送端 (tx) 存储到共享状态中，与 user_id 关联
        // 需要一个 `lock` 来安全地修改 HashMap
        self.app_state.sse_senders.lock().await.insert(user_id, tx.clone());
        tracing::debug!("为用户 {} 建立了 SSE 连接", user_id);

        // 将接收端 (rx) 转换为一个 Stream
        let stream = ReceiverStream::new(rx).map(|data| Ok(Event::default().data(data)));

        let sse_senders = self.app_state.sse_senders.clone();

        let stream_with_cleanup = stream.then(move |res| {

            let sse_senders = sse_senders.clone();
            async move {
                // 当客户端断开时，从在线列表中移除用户
                if res.is_err() {
                    tracing::debug!("用户 {} 的流已结束。正在清理sender。", user_id);
                    sse_senders.lock().await.remove(&user_id);
                }
                res
            }
        });
        Sse::new(stream_with_cleanup).keep_alive(KeepAlive::default())
    }
}