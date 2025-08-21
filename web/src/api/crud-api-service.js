import axios from 'axios';
// import {getToken} from "@/utils";
import {useUserStore} from "@/store";
// import { resReject, resResolve, reqReject, reqResolve } from '@/api/interceptors'

export class CrudApiService {

    constructor(options = {}) {

        this.baseURL = options.baseURL || import.meta.env.VITE_BASE_API;
        // 标记是否正在刷新token
        this.isRefreshing = false;
        // 存储因token过期而失败的请求
        this.failedQueue = [];

        // 创建axios实例
        this.axios = axios.create({
            baseURL: this.baseURL,
            timeout: 10000,
            headers: {
                'Content-Type': 'application/json',
            },
            withCredentials: true,
        });

        // 设置请求拦截器
        this.axios.interceptors.request.use((config) =>this.reqResolve(config));

        this.axios.interceptors.response.use(
            (response) => {
                if (window.$loadingBar) {
                    window.$loadingBar.finish();
                }
                return response;
            },
            (error) => this.resReject(error));
    }

    reqResolve(config) {
        if (window.$loadingBar) {
            window.$loadingBar.start();
        }
        if (config.noNeedToken) {
            return config;
        }

        const userStore = useUserStore();
        const token = userStore.accessToken;
        if (token) {
            config.headers.Authorization = config.headers.Authorization || `Bearer ${token}`;
        }
        return config;
    }


    async resReject(error) {

        if (axios.isCancel(error)) {
            console.log('Request canceled:', error.message);
            return Promise.reject(error);
        }

        const originalRequest = error.config;
        const userStore = useUserStore();
        // 只有当错误是401且原始请求没有_retry标记时才处理
        if (error.response?.status === 401 && !originalRequest._retry) {

            originalRequest._retry = true;

            // 如果isRefreshing存在，说明正在刷新token，将当前请求加入队列
            if (this.isRefreshing) {
                return new Promise((resolve, reject) => {
                    this.failedQueue.push({
                        originalRequest,
                        resolve,
                        reject
                    })
                })
            }

            // 如果isRefreshing不存在，说明这是第一个触发401的请求
            if (!this.isRefreshing) {
                this.isRefreshing = true;
                return userStore.refreshToken().then((newAccessToken) => {
                    // 更新请求头中的token
                    originalRequest.headers['Authorization'] = `Bearer ${newAccessToken}`
                    // 执行失败的请求
                    this.failedQueue.forEach((item) => {
                        item.originalRequest.headers['Authorization'] = `Bearer ${newAccessToken}`;
                        item.resolve(this.axios(item.originalRequest)); 
                    });
                    // 重新发送原始请求
                    return this.axios(originalRequest);
                    }
                ).catch((refreshError) => {
                    this.failedQueue.forEach((item) => {
                        item.reject(refreshError);
                    });
                    if (refreshError.status === 401) {
                        userStore.logout();
                    }
                    return Promise.reject(refreshError);
                }).finally(
                    () => {
                        this.isRefreshing = false;
                        this.failedQueue = []
                    }
                )
            }
        }

        let errorMessage = "发生未知错误，稍后重试"
        if (error.response) {
            const status = error.response.status;
            const serverMessage = error.response.data?.msg;
            switch (status) {
                case 400:
                    errorMessage = serverMessage || '请求参数错误';
                    break;
                case 403:
                    errorMessage = '您没有权限执行此操作';
                    // 可选：有需要可以做点啥
                    // this.userStore.logout();
                    break;
                case 404:
                    errorMessage = `请求的资源未找到 (${originalRequest.url})`;
                    break;
                case 422:
                    // Unprocessable Entity，通常是表单验证错误
                    errorMessage = serverMessage || '提交的数据验证失败';
                    break;
                case 500:
                case 502:
                case 503:
                case 504:
                    errorMessage = '服务器开小差了，请稍后再试';
                    // 可以在这里将详细错误信息上报给监控系统（如 Sentry）
                    // Sentry.captureException(error);
                    break;
                default:
                    errorMessage = serverMessage || `请求失败，状态码：${status}`;
            }
        } else if (error.request) {
            // 请求已发出，但没有收到响应 (例如网络中断)
            errorMessage = '网络连接异常，请检查您的网络';
        }

        if (window.$message) {
            window.$message?.error(errorMessage, { keepAliveOnHover: true })
        } else {
            console.error(`ApiService Error: ${errorMessage}`, error);
        }

        if (window.$loadingBar) {
            window.$loadingBar.error();
        }
        return Promise.reject(error);
    }


    // 成功处理
    handleSuccess(message, data) {
        if (message && window.$message) {
            window.$message.success(message);
        }
        return data;
    }

    /**
     * 核心HTTP请求方法 - 集中处理所有请求
     */
    async request(method, url, data = null, config = {}, successMessage = null) {
        try {
            let response;

            switch (method.toLowerCase()) {
                case 'get':
                    response = await this.axios.get(url, config);
                    break;
                case 'post':
                    response = await this.axios.post(url, data, config);
                    break;
                case 'put':
                    response = await this.axios.put(url, data, config);
                    break;
                case 'delete':
                    response = await this.axios.delete(url, { ...config, data: data });
                    break;
                default:
                    throw new Error(`不支持的HTTP方法: ${method}`);
            }

            return this.handleSuccess(successMessage, response.data);
        } catch (error) {
            throw error; // 错误已在拦截器中处理
        }
    }

    /**
     * 便捷的HTTP方法封装
     */
    async get(url, params = {}, successMessage = null) {
        return this.request('get', url, null, { params }, successMessage);
    }

    async post(url, data = {}, successMessage = null) {
        return this.request('post', url, data, {}, successMessage);
    }

    async put(url, data = {}, successMessage = null) {
        return this.request('put', url, data, {}, successMessage);
    }

    async delete(url, data = null, successMessage = null) {
        return await this.request('delete', url, data, {}, successMessage);
    }

    /**
     * 基础CRUD操作
     */

    // 获取所有资源
    async getAll(resource, params = {}) {
        return this.get(`/${resource}`, params);
    }

    // 根据ID获取单个资源
    async getById(resource, id) {
        return this.get(`/${resource}/${id}`);
    }

    async updateCacheById(resource, id, data, successMessage) {
        return this.post(`/${resource}/${id}/cache`,
        data,
        successMessage || `${resource}缓存更新成功`);
    }

    async updateCacheAll(resource, data, successMessage) {
        return this.post(`/${resource}/cache`,
            data, successMessage|| `${resource}缓存更新成功`);
    }

    // 创建新资源
    async create(resource, data, successMessage) {
        return this.post(
            `/${resource}`,
            data,
            successMessage || `创建${resource}成功`
        );
    }

    // 更新资源
    async update(resource, id, data, successMessage) {
        return this.put(
            `/${resource}/${id}`,
            data,
            successMessage || `更新${resource}成功`
        );
    }

    // 删除资源
    async deleteResource(resource, id, successMessage) {
        return this.delete(
            `/${resource}/${id}`,
            null,
            successMessage || `删除${resource}成功`
        );
    }

    /**
     * 关联关系操作
     */

    // 获取资源的关联项
    async getRelated(resource, id, relation, params = {}) {
        return this.get(`/${resource}/${id}/${relation}`, params);
    }

    // 添加关联关系
    async addRelation(resource, id, relation, data, successMessage) {
        return this.post(
            `/${resource}/${id}/${relation}`,
            data,
            successMessage || `添加${resource}的${relation}关联成功`
        );
    }

    // 删除关联关系
    async removeRelation(resource, id, relation, relationId, successMessage) {
        return this.delete(
            `/${resource}/${id}/${relation}/${relationId}`,
            null,
            successMessage || `删除${resource}的${relation}关联成功`
        );
    }

    /**
     * 批量操作
     */

    // 批量删除
    async batchDelete(resource, ids, successMessage) {
        return this.delete(
            `/${resource}/batch`,
            { ids },
            successMessage || `批量删除${resource}成功`
        );
    }

    // 批量更新
    async batchUpdate(resource, updates, successMessage) {
        return this.put(
            `/${resource}/batch`,
            updates,
            successMessage || `批量更新${resource}成功`
        );
    }

    /**
     * 便捷方法 - 带确认的删除操作
     */
    async deleteWithConfirm(resource, id, options = {}) {
        const {
            title = '确认删除',
            content = '确定要删除这个项目吗？此操作不可恢复。',
            successMessage
        } = options;``

        if (!window.$dialog) {
            return this.deleteResource(resource, id, successMessage);
        }

        return new Promise((resolve) => {
            window.$dialog.warning({
                title,
                content,
                positiveText: '确定',
                negativeText: '取消',
                onPositiveClick: async () => {
                    const result = await this.deleteResource(resource, id, successMessage);
                    resolve(result);
                },
                onNegativeClick: () => {
                    resolve(null);
                }
            });
        });
    }
}

/**
 * 资源特定的API类
 */
export class ResourceApi {
    constructor(resourceName, crudService) {
        this.resource = resourceName;
        this.crud = crudService;
    }

    // 基础CRUD方法
    getAll(params) { return this.crud.getAll(this.resource, params); }
    getById(id) { return this.crud.getById(this.resource, id); }
    updateCacheById(id, data, message) { return this.crud.updateCacheById(this.resource, id, data, message); }
    updateCacheAll(data, message) { return this.crud.updateCacheAll(this.resource, data, message); }
    create(data, message) { return this.crud.create(this.resource, data, message); }
    update(id, data, message) { return this.crud.update(this.resource, id, data, message); }
    delete(id, message) { return this.crud.deleteResource(this.resource, id, message); }
    deleteWithConfirm(id, options) { return this.crud.deleteWithConfirm(this.resource, id, options); }

    // 关联关系方法
    getRelated(id, relation, params) {
        return this.crud.getRelated(this.resource, id, relation, params);
    }
    addRelation(id, relation, data, message) {
        return this.crud.addRelation(this.resource, id, relation, data, message);
    }
    removeRelation(id, relation, relationId, message) {
        return this.crud.removeRelation(this.resource, id, relation, relationId, message);
    }

    // 批量操作
    batchDelete(ids, message) { return this.crud.batchDelete(this.resource, ids, message); }
    batchUpdate(updates, message) { return this.crud.batchUpdate(this.resource, updates, message); }
}