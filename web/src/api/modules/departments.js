
import http from '../http';

export const getDepartments = (params) => {
    return http.get('/departments', { params });
};

// 创建新部门
export const createDepartment = (data) => {
    return http.post('/departments', data);
};

// 更新部门
export const updateDepartment = (id, data) => {
    return http.put(`/departments/${id}`, data);
};

// 删除部门
export const deleteDepartment = (id) => {
    return http.delete(`/departments/${id}`);
};