import {defineStore} from 'pinia'
import {resetRouter} from '@/router'
import {useTagsStore} from '@/store'
import {toLogin} from '@/utils'
import {authApi} from '@/composables/auth'
import {ref} from 'vue'

export const useUserStore = defineStore('user', {
    state: () => (
        {
            accessToken: ref(null),
            userInfo: {},
            UIPolicies: [],
            roles: [],
            departments: {},
            groups: [],
            // 添加一个状态来跟踪认证是否已检查完毕
            authStatusChecked: false,
            isDynamicRoutesAdded: ref(false),
        }
    ),
    getters: {
        isAuthenticated() {
            !!this.accessToken
        },
        userId() {
            return this.userInfo?.id
        },
        name() {
            return this.userInfo?.username
        },
        email() {
            return this.userInfo?.email
        },
        avatar() {
            return this.userInfo?.avatar
        },
        isSuperUser() {
            return this.userInfo?.is_superuser
        },
        isActive() {
            return this.userInfo?.is_active
        },
        uiPolicies() {
            return this.UIPolicies
        },
        permissions() {
            return this.accessPermissions
        },
    },
    actions: {

        async getUserProfile() {
            try {
                const res = await authApi.getCurrentUserProfile()
                if (res.code === 401) {
                    await this.logout()
                    return
                }
                const {id, username, email, avatar, is_superuser, is_active} = res.data.info
                this.userInfo = {id, username, email, avatar, is_superuser, is_active}
                this.UIPolicies = res.data.ui_policies
                this.roles = res.data.roles
                this.departments = res.data.departments
                this.groups = res.data.groups
            } catch (error) {
                return error
            }
        },

        async login(credentials) {
            try {
                const res = await authApi.login(credentials)
                this.accessToken = res.data.access_token
                await this.getUserProfile()
            } catch (error) {
                console.error('Login failed:', error);
            }
        },

        async refreshToken() {
            try {
                console.log('Attempting to refresh token from HttpOnly cookie...');
                const res = await authApi.refreshToken();
                this.accessToken = res.data.access_token
                return res.data.access_token
            } catch (error) {
                console.error('Failed to refresh token:', error);
                this.clearAuthData();
                toLogin();
                throw error;
            }
        },

        async logout() {
            try {
                await authApi.logout()
            } catch (error) {
                console.error('Logout failed:', error);
            } finally {
                const {resetTags} = useTagsStore()
                resetTags()
                await resetRouter()
                this.$reset()
                toLogin()
            }
        },

        setUserInfo(userInfo = {}) {
            this.userInfo = {...this.userInfo, ...userInfo}
        },

        setDynamicRoutesAdded(tag) {
          this.isDynamicRoutesAdded = tag
        },

      clearAuthData() {
            this.accessToken = null;
        },

        async checkAuthStatus() {
            try {
                await this.refreshToken();
            } catch (error) {
                console.log("No valid session found.");
            } finally {
                this.authStatusChecked = true;
            }
        }
    },
    persist: {
        storage: sessionStorage,
    },
})
