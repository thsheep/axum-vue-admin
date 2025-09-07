import { defineStore } from 'pinia';
import * as schemaApi from '@/api/modules/cedarSchemas';

export const useSchemaStore = defineStore('schema', {
    state: () => ({
        // Schema 列表
        schemas: [],
        // 加载状态
        isLoading: false,
    }),

    actions: {
        // 获取 Schema 列表
        async fetchSchemas() {
            this.isLoading = true;
            try {
                const response = await schemaApi.getSchemas();
                this.schemas = response.data;
            } finally {
                this.isLoading = false;
            }
        },

        // 更新 Schema
        async updateSchema(id, data) {
            await schemaApi.updateSchema(id, data);
            await this.fetchSchemas();
        },
    },
});