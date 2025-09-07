import http from '../http';

// --- 用户组 CRUD ---
export const getUserGroups = (params) => http.get('/groups', { params });
export const createUserGroup = (data) => http.post('/groups', data);
export const updateUserGroup = (id, data) => http.put(`/groups/${id}`, data);
export const deleteUserGroup = (id) => http.delete(`/groups/${id}`);

// --- 用户组与角色的关联操作 ---
// 获取某个用户组已拥有的角色
export const getGroupRoles = (groupId) => http.get(`/groups/${groupId}/roles`);

// 为用户组添加一个角色
export const addRoleToGroup = (groupId, roleId) => http.post(`/groups/${groupId}/roles`, { role_id: roleId });

// 从用户组移除一个角色
export const removeRoleFromGroup = (groupId, roleId) => http.delete(`/groups/${groupId}/roles/${roleId}`);