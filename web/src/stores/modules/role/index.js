
import { defineStore } from 'pinia';
import * as roleApi from '@/api/modules/roles';

export const useRoleStore = defineStore('role', {
    state: () => ({
        // 角色列表数据
        roles: [],

        // 分页信息
        pagination: {
            page: 1,
            pageSize: 10,
            total: 0,
        },

        // 整个表格的加载状态
        isTableLoading: false,
    }),

    actions: {
        // 获取角色列表
        async fetchRoles(params) {
            this.isTableLoading = true;
            try {
                const response = await roleApi.getRoles(params);
                this.roles = response.data;
                this.pagination.total = response.total;
                // 更新本地分页状态以保持同步
                this.pagination.page = params.page;
                this.pagination.pageSize = params.pageSize;
            } finally {
                this.isTableLoading = false;
            }
        },

        // 创建角色
        async createRole(data) {
            await roleApi.createRole(data);
            await this.fetchRoles({ page: 1, pageSize: this.pagination.pageSize });
        },

        // 更新角色
        async updateRole(id, data) {
            await roleApi.updateRole(id, data);
            await this.fetchRoles({ page: this.pagination.page, pageSize: this.pagination.pageSize });
        },

        // 删除角色
        async deleteRole(id) {
            await roleApi.deleteRole(id);
            await this.fetchRoles({ page: this.pagination.page, pageSize: this.pagination.pageSize });
        },
    },
});