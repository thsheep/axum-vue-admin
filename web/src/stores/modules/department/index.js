import { defineStore } from 'pinia';
import * as deptApi from '@/api/modules/departments';

export const useDepartmentStore = defineStore('department', {
    state: () => ({
        departments: [],

        // 加载状态
        isLoading: false,
    }),

    getters: {
        departmentOptions: (state) => state.departments,
    },

    actions: {
        // 获取完整的部门树
        async fetchDepartments(params) {
            this.isLoading = true;
            try {
                const response = await deptApi.getDepartments(params);
                this.departments = response.data;
            } finally {
                this.isLoading = false;
            }
        },

        // 创建部门
        async createDepartment(data) {
            await deptApi.createDepartment(data);
            // 成功后，必须重新获取整个树以保证结构正确
            await this.fetchDepartments();
        },

        // 更新部门
        async updateDepartment(id, data) {
            await deptApi.updateDepartment(id, data);
            await this.fetchDepartments();
        },

        // 删除部门
        async deleteDepartment(id) {
            await deptApi.deleteDepartment(id);
            await this.fetchDepartments();
        },
    },
});