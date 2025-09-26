import axios from 'axios';
import { useUserStore } from '@/stores';

const http = axios.create({
    baseURL: import.meta.env.VITE_BASE_API,
    timeout: 10000,
    headers: { 'Content-Type': 'application/json' },
});


function cleanObject(obj) {
    if (obj === null || typeof obj !== 'object') {
        return obj;
    }

    const cleaned = Array.isArray(obj) ? [] : {};

    for (const key in obj) {
        if (Object.prototype.hasOwnProperty.call(obj, key)) {
            const value = obj[key];

            if (typeof value === 'object' && value !== null) {
                const cleanedValue = cleanObject(value);

                if (Object.keys(cleanedValue).length > 0) {
                    cleaned[key] = cleanedValue;
                }

            }
            else if (value !== null && value !== undefined && value !== '') {
                cleaned[key] = value;
            }
        }
    }

    return cleaned;
}

// --- 请求拦截器 ---
http.interceptors.request.use(
    (config) => {
        // window.$loadingBar?.start();

        if (config.method === 'post' || config.method === 'put') {
            if (config.data && typeof config.data === 'object' && !(config.data instanceof FormData)) {
                config.data = cleanObject(config.data);
            }
        }

        if (config.noNeedToken) {
            return config;
        }

        const userStore = useUserStore();
        const token = userStore.accessToken;
        if (token) {
            config.headers.Authorization = `Bearer ${token}`;
        }
        return config;
    },
    (error) => {
        return Promise.reject(error);
    }
);


// --- 响应拦截器 ---

// 用于管理刷新 Token 的逻辑，防止并发请求重复刷新
let isRefreshing = false;
let failedQueue = [];

const processQueue = (error, token = null) => {
    failedQueue.forEach(prom => {
        if (error) {
            prom.reject(error);
        } else {
            prom.resolve(token);
        }
    });
    failedQueue = [];
};

http.interceptors.response.use(
    (response) => {
        // window.$loadingBar?.finish();
        return response.data;
    },
    async (error) => {
        // window.$loadingBar?.error();
        const originalRequest = error.config;
        const userStore = useUserStore();

        // 处理 Token 刷新逻辑
        if (error.response?.status === 401 && !originalRequest._retry) {
            if (isRefreshing) {
                return new Promise(function (resolve, reject) {
                    failedQueue.push({ resolve, reject });
                }).then(token => {
                    originalRequest.headers['Authorization'] = 'Bearer ' + token;
                    return http(originalRequest);
                }).catch(err => {
                    return Promise.reject(err);
                });
            }

            originalRequest._retry = true;
            isRefreshing = true;

            try {
                const newAccessToken = await userStore.refreshToken();
                originalRequest.headers['Authorization'] = `Bearer ${newAccessToken}`;
                processQueue(null, newAccessToken);
                return http(originalRequest);
            } catch (refreshError) {
                processQueue(refreshError, null);
                await userStore.logout();
                // 返回一个更具体的错误，而不是原始的刷新错误
                return Promise.reject({ message: '会话已过期，请重新登录。' });
            } finally {
                isRefreshing = false;
            }
        }

        const customError = {
            message: '发生未知错误',
            statusCode: error.response?.status,
            originalError: error,
            data: error.response?.data,
        };

        if (error.response) {
            const serverMessage = error.response.data?.message || error.response.data?.message;
            switch (error.response.status) {
                case 400: customError.message = serverMessage || '请求参数错误'; break;
                case 403: customError.message = serverMessage || '您没有权限执行此操作'; break;
                case 404: customError.message = serverMessage || `请求的资源未找到 (${originalRequest.url})`; break;
                case 500: customError.message = serverMessage || '服务器内部错误'; break;
                default: customError.message = serverMessage || `请求失败 (${error.response.status})`;
            }
        } else if (error.request) {
            customError.message = '网络连接异常，请检查您的网络';
        } else if (axios.isCancel(error)) {
            console.log('Request canceled:', error.message);
            customError.message = '请求已取消';
        }

        // 将格式化后的错误 reject 出去，由调用方处理
        return Promise.reject(customError);
    }
);

export default http;