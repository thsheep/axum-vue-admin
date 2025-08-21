
/**
 * 认证和用户相关API类
 */

import {CrudApiService, ResourceApi} from '@/api/crud-api-service';
import  i18n  from '@/i18n'

const { t } = i18n.global

class AuthApi {
  constructor(crudService) {
    this.crud = crudService;
  }

  // 用户登录
  async login(credentials) {
    return this.crud.post('/auth/login', credentials, t('views.login.message_login_success'));
  }

  // 忘记密码
  async forgotPassword(resetData) {
    return this.crud.post('/password-resets', resetData);
  }

  // 重置密码
  async resetPassword(resetToken, resetData) {
    return this.crud.post(`/password-resets/${resetToken}`, resetData);
  }

  // 获取当前用户信息
  async getCurrentUserProfile() {
    return this.crud.get('/me/profile');
  }

  // 用户注销
  async logout() {
    return this.crud.post('/auth/logout', {}, '注销成功');
  }

  // 刷新Token
  async refreshToken() {
    // 请求不再需要发送body，浏览器会自动携带cookie
    return this.crud.post('/auth/refresh_token');
  }

  // 修改密码
  async changePassword(passwordData) {
    return this.crud.put('/me/password', passwordData, '密码修改成功');
  }

  // 更新当前用户信息
  async updateCurrentUser(userData) {
    return this.crud.put('/me/info', userData, '用户信息更新成功');
  }
}


// 创建默认实例
const apiService = new CrudApiService();

// 创建特定资源的API实例
export const authApi = new AuthApi(apiService);

/*
使用示例：

// 基础使用
const users = await userApi.getAll({ page: 1, limit: 10 });
const user = await userApi.getById(1);
const newUser = await userApi.create({ name: '张三', email: 'zhang@example.com' });

// 认证相关
await authApi.login({ username: 'admin', password: '123456' });
const currentUser = await authApi.getCurrentUser();

// 带确认的删除
await userApi.deleteWithConfirm(1, {
  title: '删除用户',
  content: '确定要删除这个用户吗？'
});

// 批量操作
await userApi.batchDelete([1, 2, 3], '批量删除用户成功');
*/