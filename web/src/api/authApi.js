// src/api/authApi.js
import http from './http'; // 导入我们重构后的中心化 http 客户端

export const authApi = {
    /**
     * 用户登录
     * @param {object} credentials - { username, password }
     * @returns {Promise<object>} - 包含 token 等信息的响应
     */
    login: (credentials) => {
        return http.post('/auth/login', credentials);
    },

    /**
     * 用户注销
     * @returns {Promise<any>}
     */
    logout: () => {
        return http.post('/auth/logout');
    },

    /**
     * 刷新 Access Token
     * @returns {Promise<object>} - 包含新 token 的响应
     */
    refreshToken: () => {
        return http.post('/auth/refresh_token');
    },

    /**
     * 获取当前登录用户的信息
     * @returns {Promise<object>} - 用户信息
     */
    getCurrentUserProfile: () => {
        return http.get('/me/profile');
    },

    /**
     * 更新当前登录用户的信息
     * @param {object} userData - 需要更新的用户数据
     * @returns {Promise<object>} - 更新后的用户信息
     */
    updateCurrentUserProfile: (userData) => {
        return http.put('/me/profile', userData);
    },

    /**
     * 修改当前用户的密码
     * @param {object} passwordData - { oldPassword, newPassword }
     * @returns {Promise<any>}
     */
    changePassword: (passwordData) => {
        return http.put('/me/password', passwordData);
    },

    /**
     * 忘记密码（请求重置）
     * @param {object} resetData - { email: '', language: '' }
     * @returns {Promise<any>}
     */
    forgotPassword: (resetData) => {
        return http.post('/password-resets', resetData);
    },

    /**
     * 根据 token 重置密码
     * @param {string} resetToken
     * @param {object} resetData - { password }
     * @returns {Promise<any>}
     */
    resetPassword: (resetToken, resetData) => {
        return http.post(`/password-resets/${resetToken}`, resetData);
    },
};