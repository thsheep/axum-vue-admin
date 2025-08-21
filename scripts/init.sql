
SET NAMES utf8mb4;
SET FOREIGN_KEY_CHECKS = 0;

-- ----------------------------
-- Table structure for auditlog
-- ----------------------------
DROP TABLE IF EXISTS `auditlog`;
CREATE TABLE `auditlog` (
  `id` bigint NOT NULL AUTO_INCREMENT,
  `created_at` datetime(6) NOT NULL DEFAULT CURRENT_TIMESTAMP(6),
  `updated_at` datetime(6) NOT NULL DEFAULT CURRENT_TIMESTAMP(6) ON UPDATE CURRENT_TIMESTAMP(6),
  `user_id` int NOT NULL COMMENT '用户ID',
  `username` varchar(64) NOT NULL DEFAULT '' COMMENT '用户名称',
  `module` varchar(64) NOT NULL DEFAULT '' COMMENT '功能模块',
  `summary` varchar(128) NOT NULL DEFAULT '' COMMENT '请求描述',
  `method` varchar(10) NOT NULL DEFAULT '' COMMENT '请求方法',
  `path` varchar(255) NOT NULL DEFAULT '' COMMENT '请求路径',
  `status` int NOT NULL DEFAULT '-1' COMMENT '状态码',
  `response_time` int NOT NULL DEFAULT '0' COMMENT '响应时间(单位ms)',
  PRIMARY KEY (`id`),
  KEY `idx_auditlog_user_id_4b93fa` (`user_id`),
  KEY `idx_auditlog_usernam_b187b3` (`username`),
  KEY `idx_auditlog_module_04058b` (`module`),
  KEY `idx_auditlog_summary_3e27da` (`summary`),
  KEY `idx_auditlog_method_4270a2` (`method`),
  KEY `idx_auditlog_path_b99502` (`path`),
  KEY `idx_auditlog_status_2a72d2` (`status`)
) ENGINE=InnoDB AUTO_INCREMENT=267072 DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;

-- ----------------------------
-- Records of auditlog
-- ----------------------------
BEGIN;
COMMIT;

-- ----------------------------
-- Table structure for cedar_policy_set
-- ----------------------------
DROP TABLE IF EXISTS `cedar_policy_set`;
CREATE TABLE `cedar_policy_set` (
  `policy_id` int NOT NULL AUTO_INCREMENT,
  `policy_str_id` varchar(255) CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci NOT NULL COMMENT 'policy的注解ID',
  `policy_text` text CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci NOT NULL COMMENT '单条Policy;所有可用Policy 组和后给Cedar使用',
  `effect` varchar(10) CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci NOT NULL COMMENT '这条策略是许可策略 or 拒绝策略',
  `is_active` tinyint(1) NOT NULL DEFAULT '0' COMMENT '这条Policy是否激活可用',
  `description` varchar(255) CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci NOT NULL COMMENT '详细描述这条Policy作用',
  `policy_hash` varchar(64) COLLATE utf8mb4_general_ci NOT NULL COMMENT 'policy_text 的hash; 避免重复策略创建',
  `created_by` int NOT NULL COMMENT '创建策略的用户ID',
  `created_at` timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP,
  `updated_at` timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (`policy_id`) USING BTREE,
  UNIQUE KEY `uniq_policy_hash` (`policy_hash`),
  KEY `created_by` (`created_by`),
  CONSTRAINT `cedar_policy_set_ibfk_1` FOREIGN KEY (`created_by`) REFERENCES `users` (`user_id`)
) ENGINE=InnoDB AUTO_INCREMENT=7 DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci COMMENT='用来记录Cedar Policy Set的，一条记录一条策略';

-- ----------------------------
-- Records of cedar_policy_set
-- ----------------------------
BEGIN;
INSERT INTO `cedar_policy_set` (`policy_id`, `policy_str_id`, `policy_text`, `effect`, `is_active`, `description`, `policy_hash`, `created_by`, `created_at`, `updated_at`) VALUES (1, 'Preset roles are read-only', '@id(\"Preset roles are read-only\")\nforbid (   \n	principal,   \n	action,   \n	resource\n) when {\n  resource in [Role::\"1\", Role::\"2\", Role::\"3\"]\n}; ', 'forbid', 1, '禁止操作预设角色', '2222', 1, '2025-08-15 16:17:24', '2025-08-19 21:29:44');
INSERT INTO `cedar_policy_set` (`policy_id`, `policy_str_id`, `policy_text`, `effect`, `is_active`, `description`, `policy_hash`, `created_by`, `created_at`, `updated_at`) VALUES (2, 'The SuperAdmin has full system privileges.', '@id(\"The SuperAdmin has full system privileges.\")\npermit (\n    principal in Role::\"1\",\n    action,\n    resource\n);', 'permit', 1, '超级管理员可以执行所有操作', '3333', 1, '2025-08-14 15:43:09', '2025-08-14 16:28:57');
INSERT INTO `cedar_policy_set` (`policy_id`, `policy_str_id`, `policy_text`, `effect`, `is_active`, `description`, `policy_hash`, `created_by`, `created_at`, `updated_at`) VALUES (3, 'Policy Administrator', '@id(\"Policy Administrator\")\npermit (\n    principal in Role::\"3\",\n    action in [\n        Action::\"ViewPolicies\",\n        Action::\"CreatePolicies\",\n        Action::\"UpdatePolicies\",\n        Action::\"DeletePolicies\",\n    ], \n    resource\n);', 'permit', 1, '策略管理', '4444', 1, '2025-08-14 15:43:09', '2025-08-21 12:26:13');
INSERT INTO `cedar_policy_set` (`policy_id`, `policy_str_id`, `policy_text`, `effect`, `is_active`, `description`, `policy_hash`, `created_by`, `created_at`, `updated_at`) VALUES (4, 'User Administrator', '@id(\"User Administrator\")\npermit (\n    principal in Role::\"2\",\n    action in [\n        Action::\"ViewUser\",\n        Action::\"CreateUser\", \n        Action::\"UpdateUser\",\n        Action::\"DeleteUser\",\n        Action::\"ViewGroup\",\n        Action::\"CreateGroup\",\n        Action::\"UpdateGroup\",\n        Action::\"DeleteGroup\",\n        Action::\"ViewRole\",\n        Action::\"CreateRole\",\n        Action::\"UpdateRole\",\n        Action::\"DeleteRole\",\n        Action::\"AssignRole\",\n        Action::\"RevokeRole\",\n        Action::\"ViewDepartment\",\n        Action::\"CreateDepartment\",\n        Action::\"UpdateDepartment\",\n        Action::\"DeleteDepartment\",\n        ],\n    resource\n);', 'permit', 1, '用户管理', '55555', 1, '2025-08-14 15:43:09', '2025-08-21 11:09:26');
INSERT INTO `cedar_policy_set` (`policy_id`, `policy_str_id`, `policy_text`, `effect`, `is_active`, `description`, `policy_hash`, `created_by`, `created_at`, `updated_at`) VALUES (5, 'Preset Groups are read-only', '@id(\"Preset Groups are read-only\")\nforbid (\n  principal,\n  action,\n  resource\n) when {\n  resource in [Group::\"1\", Group::\"2\"]\n};', 'forbid', 1, '禁止操作预设组', '666', 1, '2025-08-19 22:55:01', '2025-08-19 22:55:40');
INSERT INTO `cedar_policy_set` (`policy_id`, `policy_str_id`, `policy_text`, `effect`, `is_active`, `description`, `policy_hash`, `created_by`, `created_at`, `updated_at`) VALUES (6, 'Preset policies are read-only', '@id(\"Preset policies are read-only\")\nforbid (\n  principal,\n  action,\n  resource\n) when {\n  resource in [Policy::\"1\", Policy::\"2\", Policy::\"3\", Policy::\"4\", Policy::\"5\", Policy::\"6\"]\n};', 'forbid', 1, '禁止操作预设策略', '11111111', 1, '2025-08-14 15:43:09', '2025-08-19 22:55:19');
COMMIT;

-- ----------------------------
-- Table structure for cedar_schema
-- ----------------------------
DROP TABLE IF EXISTS `cedar_schema`;
CREATE TABLE `cedar_schema` (
  `schema_id` int NOT NULL AUTO_INCREMENT,
  `schema` text COLLATE utf8mb4_general_ci NOT NULL,
  `description` varchar(255) COLLATE utf8mb4_general_ci NOT NULL,
  `created_at` timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP,
  `updated_at` timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (`schema_id`)
) ENGINE=InnoDB AUTO_INCREMENT=2 DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci COMMENT='存储 Cedar Schema ';

-- ----------------------------
-- Records of cedar_schema
-- ----------------------------
BEGIN;
INSERT INTO `cedar_schema` (`schema_id`, `schema`, `description`, `created_at`, `updated_at`) VALUES (1, '// 没有多租户的需求不设置命名空间\n\n// 定义别名组合\n\n\n// -------------------------------------------------\n// 1. 定义核心实体类型\n// -------------------------------------------------\n\n// 应用程序实体（全局权限检查）\nentity Application;\n\n// 用户 (User) 实体。\n// 这是我们系统中的主体（Principal）。\n// 一个用户可以是多个角色的成员。\"用户-角色\"(临时附加) \"用户-用户组-角色\"(常规情况)。\n\nentity User in [Role, Department, Group] = {\n    name: String\n};\n\n\n// 角色 (Role) 实体。\nentity Role = {\n    name: String\n};\n\n// 用户组\nentity Group = {\n    name: String\n};\n// 部门\nentity Department = {\n    name: String\n};\n\n// Cedar Policy\nentity Policy = {\n    name: String\n};\n\n// 资源 (Resource) 实体。\n// 这是被保护的对象，例如一篇文章、一个文件或一个API端点。\n// 为了增加灵活性，资源可以被分组到“资源组”中。\n\n\n\n// -------------------------------------------------\n// 2. 定义操作 (Actions)\n// -------------------------------------------------\n\n\n// 定义CRUD操作。\n// appliesTo 部分将这些操作与我们的核心实体关联起来。\n// 主体 (principal) 通常是用户。\n// 资源 (resource) 就是被保护的Resource。\n// 用户\n\naction \"ViewUser\" appliesTo {\n    principal: User,\n    resource: User\n};\n\naction \"CreateUser\" appliesTo {\n    principal: User,\n    resource: User\n};\n\naction \"UpdateUser\" appliesTo {\n    principal: User,\n    resource: User\n};\n\naction \"DeleteUser\" appliesTo {\n    principal: User,\n    resource: User\n};\n\n// 用户组\n\naction \"ViewGroup\" appliesTo {\n    principal: User,\n    resource: Group\n};\n\naction \"ViewGroupUsers\" appliesTo {\n    principal: User,\n    resource: Group\n};\n\naction \"CreateGroup\" appliesTo {\n    principal: User,\n    resource: Group\n};\n\naction \"UpdateGroup\" appliesTo {\n    principal: User,\n    resource: Group\n};\n\naction \"DeleteGroup\" appliesTo {\n    principal: User,\n    resource: Group\n};\n\n\n// 角色\n\naction \"ViewRole\" appliesTo {\n    principal: User,\n    resource: Role\n};\n\naction \"CreateRole\" appliesTo {\n    principal: User,\n    resource: Role\n};\n\naction \"UpdateRole\" appliesTo {\n    principal: User,\n    resource: Role\n};\n\naction \"DeleteRole\" appliesTo {\n    principal: User,\n    resource: Role\n};\n\naction \"AssignRole\" appliesTo {\n    principal: User,\n    resource: Role\n};\n\naction \"RevokeRole\" appliesTo {\n    principal: User,\n    resource: Role\n};\n\n// 部门\n\naction \"ViewDepartment\" appliesTo {\n    principal: User,\n    resource: Department\n};\n\naction \"ViewDepartmentUsers\" appliesTo {\n    principal: User,\n    resource: Department\n};\n\naction \"CreateDepartment\" appliesTo {\n    principal: User,\n    resource: Department\n};\n\naction \"UpdateDepartment\" appliesTo {\n    principal: User,\n    resource: Department\n};\n\naction \"DeleteDepartment\" appliesTo {\n    principal: User,\n    resource: Department\n};\n\n// 策略\n\naction \"ViewPolicy\" appliesTo {\n    principal: User,\n    resource: Policy\n};\n\naction \"CreatePolicy\" appliesTo {\n    principal: User,\n    resource: Policy\n};\n\naction \"UpdatePolicy\" appliesTo {\n    principal: User,\n    resource: Policy\n};\n\naction \"DeletePolicy\" appliesTo {\n    principal: User,\n    resource: Policy\n};', 'V1', '2025-08-16 13:51:28', '2025-08-16 13:51:28');
COMMIT;


-- ----------------------------
-- Table structure for departments
-- ----------------------------
DROP TABLE IF EXISTS `departments`;
CREATE TABLE `departments` (
  `dept_id` int NOT NULL AUTO_INCREMENT,
  `created_at` datetime(6) NOT NULL DEFAULT CURRENT_TIMESTAMP(6),
  `updated_at` datetime(6) NOT NULL DEFAULT CURRENT_TIMESTAMP(6) ON UPDATE CURRENT_TIMESTAMP(6),
  `name` varchar(50) CHARACTER SET utf8mb4 COLLATE utf8mb4_0900_ai_ci NOT NULL COMMENT '部门名称',
  `desc` varchar(500) DEFAULT NULL COMMENT '备注',
  `is_deleted` tinyint(1) NOT NULL DEFAULT '0' COMMENT '软删除标记',
  `order` int NOT NULL DEFAULT '0' COMMENT '排序',
  `parent_id` int NOT NULL DEFAULT '0' COMMENT '父部门ID',
  PRIMARY KEY (`dept_id`) USING BTREE,
  UNIQUE KEY `name_and_parent_id` (`name`,`parent_id`) USING BTREE
) ENGINE=InnoDB AUTO_INCREMENT=37 DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;

-- ----------------------------
-- Records of departments
-- ----------------------------
BEGIN;
INSERT INTO `departments` (`dept_id`, `created_at`, `updated_at`, `name`, `desc`, `is_deleted`, `order`, `parent_id`) VALUES (1, '2024-08-28 08:46:32.820905', '2025-08-19 15:09:31.910086', 'AxumVueAdmin', '初始化的根节点', 0, 0, 0);
INSERT INTO `departments` (`dept_id`, `created_at`, `updated_at`, `name`, `desc`, `is_deleted`, `order`, `parent_id`) VALUES (2, '2024-08-28 20:47:54.235343', '2025-06-29 16:55:59.933909', 'CompanyOne', '', 0, 0, 1);
INSERT INTO `departments` (`dept_id`, `created_at`, `updated_at`, `name`, `desc`, `is_deleted`, `order`, `parent_id`) VALUES (4, '2024-08-28 20:49:37.477063', '2024-08-28 20:49:37.477072', 'SupplyChain', '', 0, 0, 2);
INSERT INTO `departments` (`dept_id`, `created_at`, `updated_at`, `name`, `desc`, `is_deleted`, `order`, `parent_id`) VALUES (5, '2024-08-28 20:49:54.751697', '2024-08-28 20:49:54.751706', 'Engineer', '', 0, 0, 2);
INSERT INTO `departments` (`dept_id`, `created_at`, `updated_at`, `name`, `desc`, `is_deleted`, `order`, `parent_id`) VALUES (6, '2024-08-28 20:50:13.102046', '2024-08-28 20:50:13.102057', 'ProgramManagement', '', 0, 0, 2);
INSERT INTO `departments` (`dept_id`, `created_at`, `updated_at`, `name`, `desc`, `is_deleted`, `order`, `parent_id`) VALUES (7, '2024-08-28 20:50:27.010850', '2024-08-28 20:50:27.010860', 'Sales', '', 0, 0, 2);
INSERT INTO `departments` (`dept_id`, `created_at`, `updated_at`, `name`, `desc`, `is_deleted`, `order`, `parent_id`) VALUES (8, '2024-08-28 20:50:42.432557', '2024-08-28 20:50:42.432577', 'Credit', '', 0, 0, 2);
INSERT INTO `departments` (`dept_id`, `created_at`, `updated_at`, `name`, `desc`, `is_deleted`, `order`, `parent_id`) VALUES (9, '2024-08-28 20:50:54.654242', '2024-08-28 20:50:54.654252', 'Purchase', '', 0, 0, 2);
INSERT INTO `departments` (`dept_id`, `created_at`, `updated_at`, `name`, `desc`, `is_deleted`, `order`, `parent_id`) VALUES (10, '2024-08-28 20:51:07.921094', '2024-08-28 20:51:07.921108', 'Manufacturing', '', 0, 0, 2);
INSERT INTO `departments` (`dept_id`, `created_at`, `updated_at`, `name`, `desc`, `is_deleted`, `order`, `parent_id`) VALUES (11, '2024-08-28 20:51:21.093665', '2024-08-28 20:51:21.093675', 'Warehouse', '', 0, 0, 2);
INSERT INTO `departments` (`dept_id`, `created_at`, `updated_at`, `name`, `desc`, `is_deleted`, `order`, `parent_id`) VALUES (12, '2024-08-28 20:51:34.879295', '2024-08-28 20:51:34.879304', 'Traffic', '', 0, 0, 2);
INSERT INTO `departments` (`dept_id`, `created_at`, `updated_at`, `name`, `desc`, `is_deleted`, `order`, `parent_id`) VALUES (13, '2024-08-28 20:51:50.115774', '2024-08-28 20:51:50.115787', 'AP', '', 0, 0, 2);
INSERT INTO `departments` (`dept_id`, `created_at`, `updated_at`, `name`, `desc`, `is_deleted`, `order`, `parent_id`) VALUES (14, '2024-08-28 20:56:29.873771', '2024-08-28 20:56:29.873784', 'ProductSourcingManagement', '', 0, 0, 2);
INSERT INTO `departments` (`dept_id`, `created_at`, `updated_at`, `name`, `desc`, `is_deleted`, `order`, `parent_id`) VALUES (15, '2024-08-28 20:56:44.034183', '2024-08-28 20:56:44.034197', 'Marketing', '', 0, 0, 2);
COMMIT;

-- ----------------------------
-- Table structure for group_roles
-- ----------------------------
DROP TABLE IF EXISTS `group_roles`;
CREATE TABLE `group_roles` (
  `group_id` int NOT NULL,
  `role_id` int NOT NULL,
  PRIMARY KEY (`group_id`,`role_id`),
  KEY `role_id` (`role_id`),
  CONSTRAINT `group_roles_ibfk_1` FOREIGN KEY (`group_id`) REFERENCES `user_groups` (`user_group_id`) ON DELETE CASCADE,
  CONSTRAINT `group_roles_ibfk_2` FOREIGN KEY (`role_id`) REFERENCES `roles` (`role_id`) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci COMMENT='用户组与角色的关联表 (M:N)';

-- ----------------------------
-- Records of group_roles
-- ----------------------------
BEGIN;
INSERT INTO `group_roles` (`group_id`, `role_id`) VALUES (1, 2);
INSERT INTO `group_roles` (`group_id`, `role_id`) VALUES (2, 3);
COMMIT;

-- ----------------------------
-- Table structure for roles
-- ----------------------------
DROP TABLE IF EXISTS `roles`;
CREATE TABLE `roles` (
  `role_id` int NOT NULL AUTO_INCREMENT,
  `role_name` varchar(50) COLLATE utf8mb4_general_ci NOT NULL,
  `description` varchar(255) COLLATE utf8mb4_general_ci DEFAULT NULL,
  `created_at` timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP,
  PRIMARY KEY (`role_id`),
  UNIQUE KEY `role_name` (`role_name`)
) ENGINE=InnoDB AUTO_INCREMENT=5 DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci;

-- ----------------------------
-- Records of roles
-- ----------------------------
BEGIN;
INSERT INTO `roles` (`role_id`, `role_name`, `description`, `created_at`) VALUES (1, 'SuperAdmin', '超级管理员，拥有系统所有权限', '2025-05-21 20:46:26');
INSERT INTO `roles` (`role_id`, `role_name`, `description`, `created_at`) VALUES (2, 'UserAdministrator', '用户管理', '2025-06-27 14:56:43');
INSERT INTO `roles` (`role_id`, `role_name`, `description`, `created_at`) VALUES (3, 'PolicyAdministrator', '策略管理员', '2025-08-19 21:10:49');
COMMIT;

-- ----------------------------
-- Table structure for systems
-- ----------------------------
DROP TABLE IF EXISTS `systems`;
CREATE TABLE `systems` (
  `system_id` int unsigned NOT NULL AUTO_INCREMENT COMMENT '系统唯一ID/系统唯一编码, 用于程序识别',
  `name` varchar(100) COLLATE utf8mb4_general_ci NOT NULL COMMENT '系统名称',
  `description` text COLLATE utf8mb4_general_ci COMMENT '系统描述',
  `created_at` timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP,
  `updated_at` timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (`system_id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci COMMENT='系统信息表,用于实现多租户';

-- ----------------------------
-- Records of systems
-- ----------------------------
BEGIN;
COMMIT;

-- ----------------------------
-- Table structure for user_group_members
-- ----------------------------
DROP TABLE IF EXISTS `user_group_members`;
CREATE TABLE `user_group_members` (
  `user_id` int NOT NULL,
  `group_id` int NOT NULL,
  `assigned_at` timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP,
  PRIMARY KEY (`user_id`,`group_id`),
  KEY `group_id` (`group_id`),
  CONSTRAINT `user_group_members_ibfk_1` FOREIGN KEY (`user_id`) REFERENCES `users` (`user_id`) ON DELETE CASCADE,
  CONSTRAINT `user_group_members_ibfk_2` FOREIGN KEY (`group_id`) REFERENCES `user_groups` (`user_group_id`) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci COMMENT='用户与用户组的关联表 (M:N)';

-- ----------------------------
-- Records of user_group_members
-- ----------------------------
BEGIN;
INSERT INTO `user_group_members` (`user_id`, `group_id`, `assigned_at`) VALUES (43, 1, '2025-08-19 23:13:36');
INSERT INTO `user_group_members` (`user_id`, `group_id`, `assigned_at`) VALUES (44, 2, '2025-08-21 11:55:55');
COMMIT;

-- ----------------------------
-- Table structure for user_groups
-- ----------------------------
DROP TABLE IF EXISTS `user_groups`;
CREATE TABLE `user_groups` (
  `user_group_id` int NOT NULL AUTO_INCREMENT COMMENT '用户组唯一ID/用户组唯一标识符',
  `name` varchar(100) COLLATE utf8mb4_general_ci NOT NULL COMMENT '用户组名称, 如: 项目A核心开发组',
  `description` text COLLATE utf8mb4_general_ci COMMENT '用户组描述',
  `created_at` timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP,
  `updated_at` timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (`user_group_id`)
) ENGINE=InnoDB AUTO_INCREMENT=4 DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci COMMENT='用户组定义表';

-- ----------------------------
-- Records of user_groups
-- ----------------------------
BEGIN;
INSERT INTO `user_groups` (`user_group_id`, `name`, `description`, `created_at`, `updated_at`) VALUES (1, 'UserManagementGroup', '用户管理组', '2025-06-29 20:12:27', '2025-08-19 21:14:47');
INSERT INTO `user_groups` (`user_group_id`, `name`, `description`, `created_at`, `updated_at`) VALUES (2, 'PolicyManagementGroup', '策略管理组', '2025-08-19 21:16:05', '2025-08-19 21:16:05');
INSERT INTO `user_groups` (`user_group_id`, `name`, `description`, `created_at`, `updated_at`) VALUES (3, 'UserGroup', '普通用户', '2025-06-23 17:17:27', '2025-08-19 21:15:06');
COMMIT;

-- ----------------------------
-- Table structure for user_roles
-- ----------------------------
DROP TABLE IF EXISTS `user_roles`;
CREATE TABLE `user_roles` (
  `user_id` int NOT NULL,
  `role_id` int NOT NULL,
  `assigned_at` timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP,
  PRIMARY KEY (`user_id`,`role_id`),
  KEY `role_id` (`role_id`),
  CONSTRAINT `user_roles_ibfk_1` FOREIGN KEY (`user_id`) REFERENCES `users` (`user_id`) ON DELETE CASCADE,
  CONSTRAINT `user_roles_ibfk_2` FOREIGN KEY (`role_id`) REFERENCES `roles` (`role_id`) ON DELETE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci;

-- ----------------------------
-- Records of user_roles
-- ----------------------------
BEGIN;
INSERT INTO `user_roles` (`user_id`, `role_id`, `assigned_at`) VALUES (1, 1, '2025-05-21 20:49:42');
COMMIT;

-- ----------------------------
-- Table structure for users
-- ----------------------------
DROP TABLE IF EXISTS `users`;
CREATE TABLE `users` (
  `user_id` int NOT NULL AUTO_INCREMENT,
  `created_at` datetime(6) NOT NULL DEFAULT CURRENT_TIMESTAMP(6),
  `updated_at` datetime(6) NOT NULL DEFAULT CURRENT_TIMESTAMP(6) ON UPDATE CURRENT_TIMESTAMP(6),
  `username` varchar(100) CHARACTER SET utf8mb4 COLLATE utf8mb4_0900_ai_ci NOT NULL COMMENT '用户名称',
  `alias` varchar(30) DEFAULT NULL COMMENT '姓名',
  `email` varchar(255) NOT NULL COMMENT '邮箱',
  `phone` varchar(20) DEFAULT NULL COMMENT '电话',
  `password` varchar(128) CHARACTER SET utf8mb4 COLLATE utf8mb4_0900_ai_ci NOT NULL COMMENT '密码',
  `dept_id` int DEFAULT NULL COMMENT '部门ID',
  `is_active` tinyint(1) NOT NULL DEFAULT '1' COMMENT '是否激活',
  `avatar` varchar(255) DEFAULT NULL COMMENT '头像地址',
  `last_login` datetime(6) DEFAULT NULL COMMENT '最后登录时间',
  `reset_token` varchar(128) DEFAULT NULL COMMENT '重置密码的token',
  `reset_triggered` datetime(6) DEFAULT NULL COMMENT '重置触发时间',
  PRIMARY KEY (`user_id`) USING BTREE,
  UNIQUE KEY `username` (`username`),
  UNIQUE KEY `email` (`email`),
  KEY `users_ibfk_1` (`dept_id`),
  CONSTRAINT `users_ibfk_1` FOREIGN KEY (`dept_id`) REFERENCES `departments` (`dept_id`) ON DELETE CASCADE ON UPDATE RESTRICT
) ENGINE=InnoDB AUTO_INCREMENT=46 DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;

-- ----------------------------
-- Records of users
-- ----------------------------
BEGIN;
INSERT INTO `users` (`user_id`, `created_at`, `updated_at`, `username`, `alias`, `email`, `phone`, `password`, `dept_id`, `is_active`, `avatar`, `last_login`, `reset_token`, `reset_triggered`) VALUES (1, '2024-08-28 16:17:28.304153', '2025-08-21 15:41:02.499337', 'superadmin', NULL, 'superadmin@xxxx.com', NULL, '$argon2id$v=19$m=19456,t=2,p=1$2WGtjhokqiE6ToWGZQ1ukQ$rDYtzjjvewEDwav+H/IjyfkG6lAtKBLZocdwMCCRLJ0', 1, 1, 'https://avatars.githubusercontent.com/u/54677442?v=4', '2025-08-21 07:27:14.685289', NULL, '2025-08-19 01:22:50.017915');
INSERT INTO `users` (`user_id`, `created_at`, `updated_at`, `username`, `alias`, `email`, `phone`, `password`, `dept_id`, `is_active`, `avatar`, `last_login`, `reset_token`, `reset_triggered`) VALUES (43, '2025-06-29 06:40:49.073780', '2025-08-20 09:18:58.524516', 'useradmin', NULL, 'admin@xxx.com', NULL, '$argon2id$v=19$m=19456,t=2,p=1$2WGtjhokqiE6ToWGZQ1ukQ$rDYtzjjvewEDwav+H/IjyfkG6lAtKBLZocdwMCCRLJ0', 1, 1, NULL, '2025-08-20 09:18:58.516522', '69ac5409-8d6c-4b89-96e5-678e03639086', '2025-08-18 15:31:01.186673');
INSERT INTO `users` (`user_id`, `created_at`, `updated_at`, `username`, `alias`, `email`, `phone`, `password`, `dept_id`, `is_active`, `avatar`, `last_login`, `reset_token`, `reset_triggered`) VALUES (44, '2025-08-21 03:55:55.400094', '2025-08-21 04:27:23.909319', 'policiadmin', NULL, 'policiadmin@xxx.com', NULL, '$argon2id$v=19$m=19456,t=2,p=1$S7MvIZk7rEUYOyuKZPtUsw$VM1rsQTYIC7w1uxcin4WXDroDoUS309O4bmuX+q4Aw4', 1, 1, NULL, '2025-08-21 04:27:23.900143', NULL, NULL);
COMMIT;

SET FOREIGN_KEY_CHECKS = 1;
