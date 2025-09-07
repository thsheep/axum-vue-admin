
import http from '../http';

// 获取角色列表 (支持分页和搜索)
export const getRoles = (params) => {
    return http.get('/roles', { params });
};

// 获取单个角色详情 
export const getRoleById = (id) => {
    return http.get(`/roles/${id}`);
};

// 创建新角色
export const createRole = (data) => {
    return http.post('/roles', data);
};

// 更新角色
export const updateRole = (id, data) => {
    return http.put(`/roles/${id}`, data);
};

// 删除角色
export const deleteRole = (id) => {
    return http.delete(`/roles/${id}`);
};