import { defineStore } from 'pinia';
import { resetRouter } from '@/router';
import { useTagsStore } from '@/stores';
import { toLogin } from '@/utils';
import { authApi } from '@/api/authApi';


export const useUserStore = defineStore('user', {
    state: () => ({
        accessToken: null,
        userInfo: {},
        uiPolicies: [],
        roles: [],
        departments: {},
        groups: [],
        // 跟踪初始认证状态，防止路由守卫在检查完成前执行
        authStatusChecked: false,
        isDynamicRoutesAdded: false,
    }),

    getters: {
        isAuthenticated: (state) => !!state.accessToken,
        userId: (state) => state.userInfo?.id,
        name: (state) => state.userInfo?.username,
        isSuperUser: (state) => state.userInfo?.is_superuser,
    },

    actions: {
        /**
         * 登录操作
         * @param {object} credentials
         */
        async login(credentials) {
            try {
                const response = await authApi.login(credentials);
                this.accessToken = response.data.access_token;
                await this.getUserProfile();
            } catch (error) {
                $message.error(error.message || '登录失败');
                console.error('Login failed:', error);
                throw error;
            }
        },

        /**
         * 获取当前登录用户的信息
         */
        async getUserProfile() {
            try {
                const response = await authApi.getCurrentUserProfile();
                const { id, username, email, avatar, is_superuser, is_active } = response.data.info;
                this.userInfo = { id, username, email, avatar, is_superuser, is_active };
                this.uiPolicies = response.data.ui_policies;
                this.roles = response.data.roles;
                this.departments = response.data.departments;
                this.groups = response.data.groups;
            } catch (error) {
                console.error('Failed to get user profile:', error);
                // 如果获取用户信息失败（例如 token 彻底失效），可以选择在这里触发登出
                await this.logout();
                throw error;
            }
        },

        /**
         * 刷新 Token。主要由 http 拦截器调用。
         */
        async refreshToken() {
            try {
                const response = await authApi.refreshToken();
                this.accessToken = response.data.access_token;
                return response.access_token;
            } catch (error) {
                console.error('Failed to refresh token:', error);
                // 刷新失败意味着会话已完全失效，清理所有数据并跳转到登录页
                this.clearAuthData();
                toLogin();
                throw error;
            }
        },

        /**
         * 登出操作
         */
        async logout() {
            try {
                // 只有在有 token 的情况下才需要调用后端登出接口
                if (this.accessToken) {
                    await authApi.logout();
                }
            } catch (error) {
                console.error('Logout API call failed:', error);
            } finally {
                const { resetTags } = useTagsStore();
                if (resetTags) resetTags();

                this.$reset();

                sessionStorage.removeItem(this.$id);

                await resetRouter();
                toLogin();
            }
        },

        async checkAuthStatus() {
            try {
                // 如果 session storage 中有 token，尝试获取用户信息
                if (this.accessToken) {
                    await this.getUserProfile();
                }
            } catch (error) {
                // 捕获到错误意味着会话无效，静默处理，用户将保持未登录状态
                console.log("No valid session found or failed to restore session.");
                // 清理可能存在的无效 token
                this.accessToken = null;
            } finally {
                // 无论成功与否，都标记为检查已完成
                this.authStatusChecked = true;
            }
        },

        setDynamicRoutesAdded(isAdded) {
            this.isDynamicRoutesAdded = isAdded;
        },

        clearAuthData() {
            this.accessToken = null;
            this.userInfo = {};
            this.UIPolicies = [];
            this.roles = [];
            // ... 重置所有与认证相关的 state
        },
    },

    persist: {
        // Pinia 持久化插件的配置
        storage: sessionStorage,
    },
});