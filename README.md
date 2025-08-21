<p align="center">
  <a href="https://github.com/mizhexiaoxiao/vue-fastapi-admin">
    <img alt="Axum Vue Admin Logo" width="200" src="https://github.com/mizhexiaoxiao/vue-fastapi-admin/blob/main/deploy/sample-picture/logo.svg">
  </a>
</p>

<h1 align="center">Axum Vue Admin</h1>

[English](./README-en.md) | 简体中文


基于
[Axum](https://github.com/tokio-rs/axum)、
[SeaORM](https://www.sea-ql.org/)、
[CedarPolicy](https://github.com/cedar-policy/cedar)、
[Vue3](https://vuejs.org/)、
[Naive UI](https://www.naiveui.com/)、 的现代化前后端分离开发平台，融合了 PBAC 权限管理、动态路由和 JWT 鉴权，助力中小型`Rust Web`应用快速搭建，用于学习参考。

>前端修改自 [vue-fastapi-admin](https://github.com/mizhexiaoxiao/vue-fastapi-admin)

### 特性
- **动态路由**：后端动态路由，结合 PBAC（Policy-Based Access Control）权限模型，提供精细的菜单路由控制。
- **JWT鉴权**：使用 JSON Web Token（JWT）、双Token，进行身份验证和授权，增强应用的安全性。
- **JWT黑名单**：针对性废弃Token.
- **CedarPolicy授权**: 基于策略的访问控制，实现高度灵活和细粒度访问控制。
- **细粒度权限控制**：实现按钮和接口级别的权限控制，确保不同用户或角色在界面操作和接口访问时具有不同的权限限制。
- **重置密码**: 邮件重置密码.

### 在线预览
- 待部署
- username: admin
- password: 123456

### 登录页

![image](https://github.com/thsheep/axum-vue-admin/blob/main/screenshot/login.png)
### 工作台

![image](https://github.com/thsheep/axum-vue-admin/blob/main/screenshot/workbench.png)

### 用户管理

![image](https://github.com/thsheep/axum-vue-admin/blob/main/screenshot/user.png)

### 用户组管理

![image](https://github.com/thsheep/axum-vue-admin/blob/main/screenshot/user_group.png)

### 角色管理

![image](https://github.com/thsheep/axum-vue-admin/blob/main/screenshot/role.png)

### Cedar Policy管理

![image](https://github.com/thsheep/axum-vue-admin/blob/main/screenshot/cedar_policy.png)

### Cedar Schema管理

![image](https://github.com/thsheep/axum-vue-admin/blob/main/screenshot/cedar_schema.png)


### 忘记密码

![image](https://github.com/thsheep/axum-vue-admin/blob/main/screenshot/forgot_password.png)

### 重置密码

![image](https://github.com/thsheep/axum-vue-admin/blob/main/screenshot/reset_password.png)


### 快速开始

#### 后端服务



启动项目需要以下环境：
- MYSQL
- Redis
- rust

**将scripts/init.sql导入数据库**

```shell


git clone --depth 1 https://github.com/thsheep/axum-vue-admin.git

cd axum-vue-admin

# 生成配置文件。 会生成一个 config.toml 配置文件,酌情修改配置.
cargo run -- -g

# 运行
cargo run

```

服务现在应该正在运行，访问 http://localhost:9999/swagger-ui 查看API文档


#### 前端服务

启动项目需要以下环境：
- node v18.8.0+

```shell
cd web

## 安装依赖(建议使用pnpm: https://pnpm.io/zh/installation)
npm i -g pnpm # 已安装可忽略
pnpm i # 或者 npm i

pnpm dev

```

##### 访问

http://localhost:3100

username：superadmin / useradmin / policiadmin

password：Qjv+L5NX#tF-
