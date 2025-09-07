import http from '../http';

// 获取用户列表 (支持分页和搜索)
export const getUsers = (params) => {
    return http.get('/users', { params });
};

// 获取单个用户详情
export const getUserById = (id) => {
    return http.get(`/users/${id}`);
};

// 创建新用户
export const createUser = (data) => {
    return http.post('/users', data);
};

// 更新用户
// 注意：这里我们传递完整的更新数据，而不是在UI层计算差异。
// 后端应该处理只更新接收到的字段。
export const updateUser = (id, data) => {
    return http.put(`/users/${id}`, data);
};

// 删除用户
export const deleteUser = (id) => {
    return http.delete(`/users/${id}`);
};

// 批量删除用户
export const batchDeleteUsers = (ids) => {
    return http.delete('/users/batch', { data: { ids } });
};

// 关联操作
export const getUserRoles = (userId, params) => {
    return http.get(`/users/${userId}/roles`, { params });
}
export const addUserRoles = (userId, roleIds) => {
    return http.post(`/users/${userId}/roles`, { ids: roleIds });
}
export const removeUserRole = (userId, roleId) => {
    return http.delete(`/users/${userId}/roles/${roleId}`);
}