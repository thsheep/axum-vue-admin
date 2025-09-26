import { defineStore } from 'pinia';
import * as policyApi from '@/api/modules/cedarPolicies';

export const usePolicyStore = defineStore('policy', {
    state: () => ({
        policies: [],
        pagination: {
            page: 1,
            pageSize: 10,
            total: 0,
        },
        isTableLoading: false,
        isSyncing: false,
    }),

    actions: {
        async fetchPolicies(params) {
            this.isTableLoading = true;
            try {
                const response = await policyApi.getPolicies(params);
                this.policies = response.data;
                this.pagination.total = response.total;
                this.pagination.page = params.page;
                this.pagination.pageSize = params.pageSize;
            } finally {
                this.isTableLoading = false;
            }
        },

        async createPolicy(data) {
            await policyApi.createPolicy(data);
            await this.fetchPolicies({ page: 1, pageSize: this.pagination.pageSize });
        },

        async fetchPolicyByID(id) {
            this.isTableLoading = true;
            try {
                const response = await policyApi.getPolicyByID(id);
                return response.data;
            } finally {
                this.isTableLoading = false;
            }
        },

        async updatePolicy(id, data) {
            await policyApi.updatePolicy(id, data);
            await this.fetchPolicies({ page: this.pagination.page, pageSize: this.pagination.pageSize });
        },

        async deletePolicy(id) {
            await policyApi.deletePolicy(id);
            await this.fetchPolicies({ page: this.pagination.page, pageSize: this.pagination.pageSize });
        },

        async syncPolicies() {
            this.isSyncing = true;
            try {
                await policyApi.syncPolicies();
            } finally {
                this.isSyncing = false;
            }
        },
    },
});