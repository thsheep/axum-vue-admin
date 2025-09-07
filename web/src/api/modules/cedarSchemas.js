
import http from '../http';

// 获取 Schema 列表
export const getSchemas = (params) => {
    return http.get('/cedar_schema', { params });
};

// 更新 Schema
export const updateSchema = (id, data) => {
    return http.put(`/cedar_schema/${id}`, data);
};