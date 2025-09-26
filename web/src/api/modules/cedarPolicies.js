import http from '../http';

// 获取策略列表 (支持分页和搜索)
export const getPolicies = (params) => {
    return http.get('/cedar_policies', { params });
};

// 创建新策略
export const createPolicy = (data) => {
    return http.post('/cedar_policies', data);
};

export const getPolicyByID = (id) => {
    return http.get(`/cedar_policies/${id}`);
};

// 更新策略
export const updatePolicy = (id, data) => {
    return http.put(`/cedar_policies/${id}`, data);
};

// 删除策略
export const deletePolicy = (id) => {
    return http.delete(`/cedar_policies/${id}`);
};

// --- 非标准操作 ---
export const syncPolicies = () => {
    return http.post('/cedar_policies/cache');
};