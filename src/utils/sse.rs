// SSE推送消息

use serde::{Deserialize, Serialize};
use crate::config::state::AppState;
use crate::errors::app_error::AppError;

#[derive(Deserialize, Serialize, Debug)]
pub struct SSEPushPayload {
    pub message_source: String, // 通知类型
    pub message_level: String, // 消息等级
    pub message: String, // 消息内容
}

pub async fn sse_push_message(
    state: &AppState,
    user_uuid: String,
    payload: SSEPushPayload
) -> Result<(), AppError> {
    let senders = state.sse_senders.lock().await;
    let message = serde_json::to_value(&payload).unwrap().to_string();

    if let Some(sender) = senders.get(&user_uuid) {
        // 尝试发送消息
        // 用户在线，尝试发送消息
        if sender.send(message.clone()).await.is_ok() {
            // 发送成功
            tracing::debug!("消息[{:?}]已发送给在线用户 {}", &message, user_uuid);
            return Ok(());
        } else {
            // 发送失败，说明用户刚刚断开连接，我们需要将此消息转为离线消息
            tracing::info!("用户 {} 刚刚断开连接。正在存储消息。", user_uuid);
            // 我们现在只需要存储消息即可。
        }
    } else {
        // 用户不在线
        tracing::info!("用户 {} 处于离线状态。正在存储消息。", user_uuid);
    }

    drop(senders);
    // 如果用户不在线，则存储消息
    let cache_key = redis_offline_key(user_uuid);

    let redis_conn = &mut state.redis.get_multiplexed_async_connection().await?;
    let _: () = redis::cmd("LPUSH")
        .arg(cache_key)
        .arg(message)
        .exec_async(redis_conn).await?;

    Ok(())

}


pub fn redis_offline_key(user_uuid: String) -> String {
    format!("offline:messages:{}", user_uuid)
}