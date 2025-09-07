import { defineStore } from 'pinia';
import * as groupApi from '@/api/modules/userGroups';
import * as roleApi from '@/api/modules/roles'; // 复用角色API

export const useUserGroupStore = defineStore('userGroup', {
    state: () => ({
        // 用户组列表
        groups: [],
        pagination: {
            page: 1,
            pageSize: 10,
            total: 0,
        },
        isTableLoading: false,

        // 角色分配抽屉所需的状态
        allRoles: [], // 所有可用角色的列表
        currentGroupRoles: [], // 当前正在编辑的用户组所拥有的角色列表
        isRoleTreeLoading: false, // 角色树的加载状态（例如，勾选/取消勾选时）
    }),

    getters: {
        // 将角色列表转换为 NTree 需要的数据格式
        roleTreeOptions: (state) => {
            return state.allRoles.map(role => ({
                key: role.id,
                label: role.name,
            }));
        },
        // 从当前用户组的角色列表中计算出需要勾选的 key
        checkedRoleKeys: (state) => {
            return state.currentGroupRoles.map(role => role.id);
        },
    },

    actions: {
        async fetchGroups(params) {
            this.isTableLoading = true;
            try {
                const response = await groupApi.getUserGroups(params);
                this.groups = response.data;
                this.pagination.total = response.total;
                this.pagination.page = params.page;
                this.pagination.pageSize = params.pageSize;
            } finally {
                this.isTableLoading = false;
            }
        },
        async createGroup(data) {
            await groupApi.createUserGroup(data);
            await this.fetchGroups({ page: 1, pageSize: this.pagination.pageSize });
        },
        async updateGroup(id, data) {
            await groupApi.updateUserGroup(id, data);
            await this.fetchGroups({ page: this.pagination.page, pageSize: this.pagination.pageSize });
        },
        async deleteGroup(id) {
            await groupApi.deleteUserGroup(id);
            await this.fetchGroups({ page: this.pagination.page, pageSize: this.pagination.pageSize });
        },


        // 获取所有角色，用于填充权限树
        async fetchAllRoles() {
            // 避免重复获取
            if (this.allRoles.length > 0) return;
            const response = await roleApi.getRoles({ pageSize: 999, fields: 'id,name' });
            this.allRoles = response.data;
        },

        // 获取特定用户组的角色
        async fetchGroupRoles(groupId) {
            const response = await groupApi.getGroupRoles(groupId);
            this.currentGroupRoles = response.data;
        },

        // 为用户组分配角色 (对应勾选操作)
        async assignRoleToGroup(groupId, roleId) {
            this.isRoleTreeLoading = true;
            try {
                await groupApi.addRoleToGroup(groupId, roleId);
                await this.fetchGroupRoles(groupId);
            } finally {
                this.isRoleTreeLoading = false;
            }
        },

        // 从用户组撤销角色 (对应取消勾选操作)
        async revokeRoleFromGroup(groupId, roleId) {
            this.isRoleTreeLoading = true;
            try {
                await groupApi.removeRoleFromGroup(groupId, roleId);
                await this.fetchGroupRoles(groupId);
            } finally {
                this.isRoleTreeLoading = false;
            }
        },
    },
});