import { ref, onUnmounted } from 'vue';
import { useNotification, useDialog } from "naive-ui";
import { fetchEventSource } from '@microsoft/fetch-event-source';
import { getToken } from '@/utils'

export function useSSE(url) {
    // 使用 ref 来保存接收到的消息
    const messages = ref([]);
    // 使用 ref 来保存 AbortController，以便可以随时取消连接
    const eventSourceController = ref(null);

    /**
     * 启动 SSE 连接
     */
    const start = () => {
        // 如果已有连接，先断开
        if (eventSourceController.value) {
            eventSourceController.value.abort();
        }

        const ctrl = new AbortController();
        eventSourceController.value = ctrl;

        const authToken = getToken();
        if (!authToken) {
            console.error('无法启动SSE连接：用户未登录或Token不存在');
            return;
        }

        console.log('正在启动 SSE 连接...');

        fetchEventSource(url, {
            method: 'GET', // 或 'POST'
            headers: {
                'Authorization': `Bearer ${authToken}`,
                'Content-Type': 'application/json',
            },
            signal: ctrl.signal,

            onopen: async (response) => {
                if (response.ok) {
                    console.log('SSE 连接成功建立');
                } else if (response.status === 401) {
                    console.error('认证失败，请重新登录');
                    // 认证失败，停止并不再重试
                    ctrl.abort();
                } else {
                    console.error('SSE 连接发生错误', response.status);
                    // 这里可以根据状态码决定是否重试，库默认会处理
                }
            },

            onmessage(ev) {
                console.log('收到服务器消息:', { event: ev.event, data: ev.data });
                try {
                    const parsedData = JSON.parse(ev.data);
                    if (parsedData.message_type.lowerCase() === "notification"){
                        console.log("收听到通知消息", parsedData);
                    }
                } catch (error) {
                    console.error('解析JSON失败:', error);
                }
            },

            onclose() {
                console.log('SSE 连接已关闭');
                // 如果不是我们主动关闭的，可以进行一些提示或重连逻辑
                if (!ctrl.signal.aborted) {
                    console.warn('警告：SSE 连接意外关闭，库将尝试自动重连。');
                }
            },

            onerror(err) {
                console.error('SSE 连接发生致命错误:', err);
                // 默认情况下，库会尝试重连。如果想停止重连，可以手动调用
                // ctrl.abort();
                // 或者直接在这里抛出错误以停止
                throw err;
            },
        });
    };

    /**
     * 手动停止 SSE 连接
     */
    const stop = () => {
        if (eventSourceController.value) {
            console.log('正在手动断开 SSE 连接...');
            eventSourceController.value.abort();
            eventSourceController.value = null;
        }
    };

    // 当组件卸载时，确保关闭连接
    onUnmounted(() => {
        stop();
    });

    return {
        messages,
        start,
        stop,
    };
}