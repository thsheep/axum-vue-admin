import { defineStore } from 'pinia'
import {authApi} from '@/composables/auth'

// * 后端路由相关函数

export const usePermissionsStore = defineStore('permissions', {
  state() {
    return {
      UIPolicies: {},
      accessPermissions: [],
    }
  },
  getters: {
    uiPolicies() {
      return this.UIPolicies
    },
    permissions() {
      return this.accessPermissions
    },
  },
  actions: {
    async getUIPolicies(){
      const res = await authApi.getCurrentUserUIPolicies()
      this.UIPolicies = res.data
      return this.UIPolicies
    },
    async getUserPermissions() {
      const res = await authApi.getCurrentUserPermissions()
      this.accessPermissions = res.data
      return this.accessPermissions
    },
    resetPermission() {
      this.$reset()
    },
  },
})
