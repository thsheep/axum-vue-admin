import { defineStore } from 'pinia';
import * as userApi from '@/api/modules/users';
import * as deptApi from '@/api/modules/departments';
import * as groupApi from '@/api/modules/userGroups';
import * as roleApi from '@/api/modules/roles';

export const useUserManagementStore = defineStore('userManagement', {
    state: () => ({
        users: [],
        departments: [],
        userGroups: [],
        roles: [],
        currentUserRoles: [],

        // Pagination 分页参数
        pagination: {
            page: 1,
            pageSize: 10,
            total: 0,
        },

        // Loading 状态
        isTableLoading: false,
        isDependenciesLoading: false,
    }),

    getters: {
        departmentOptions: (state) => state.departments,
        userGroupOptions: (state) => state.userGroups.map(item => ({ label: item.name, value: item.uuid })),
        roleOptions: (state) => state.roles.map(item => ({ label: item.name, value: item.uuid })),
    },

    actions: {
        async fetchUsers(params) {
            this.isTableLoading = true;
            try {
                const response = await userApi.getUsers(params);
                this.users = response.data;
                this.pagination.total = response.total;
                this.pagination.page = params.page;
                this.pagination.pageSize = params.pageSize;
            } finally {
                this.isTableLoading = false;
            }
        },

        async fetchDependencies() {
            this.isDependenciesLoading = true;
            try {
                const [deptRes, groupRes, roleRes] = await Promise.all([
                    deptApi.getDepartments(),
                    groupApi.getUserGroups({ pageSize: 999, fields: 'id,name' }),
                    roleApi.getRoles({ pageSize: 999, fields: 'id,name' })
                ]);
                this.departments = deptRes.data;
                this.userGroups = groupRes.data;
                this.roles = roleRes.data;
            } finally {
                this.isDependenciesLoading = false;
            }
        },

        // CRUD
        async createUser(data) {
            await userApi.createUser(data);
            await this.fetchUsers({ page: 1, pageSize: this.pagination.pageSize });
        },

        async updateUser(id, data) {
            await userApi.updateUser(id, data);
            await this.fetchUsers({ page: this.pagination.page, pageSize: this.pagination.pageSize });
        },

        async deleteUser(id) {
            await userApi.deleteUser(id);
            await this.fetchUsers({ page: this.pagination.page, pageSize: this.pagination.pageSize });
        },

        // Role actions
        async fetchUserRoles(userId) {
            const response = await userApi.getUserRoles(userId);
            this.currentUserRoles = response.data;
        },

        async addUserRoles(userId, roleIds) {
            await userApi.addUserRoles(userId, roleIds);
            await this.fetchUserRoles(userId);
        },

        async removeUserRole(userId, roleId) {
            await userApi.removeUserRole(userId, roleId);
            await this.fetchUserRoles(userId);
        }
    },
});