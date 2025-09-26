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
  `policy_uuid` char(36) CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci NOT NULL,
  `annotation` varchar(255) CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci NOT NULL COMMENT 'policy的注解',
  `policy_text` text CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci NOT NULL COMMENT '单条Policy;所有可用Policy 组和后给Cedar使用',
  `effect` varchar(10) CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci NOT NULL COMMENT '这条策略是许可策略 or 拒绝策略',
  `is_active` tinyint(1) NOT NULL DEFAULT '0' COMMENT '这条Policy是否激活可用',
  `description` varchar(255) CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci NOT NULL COMMENT '详细描述这条Policy作用',
  `policy_hash` varchar(64) COLLATE utf8mb4_general_ci NOT NULL COMMENT 'policy_text 的hash; 避免重复策略创建',
  `policy_type` varchar(10) CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci NOT NULL COMMENT '区分是静态策略还是模板, STATIC, TEMPLATE',
  `created_by` int NOT NULL COMMENT '创建策略的用户ID',
  `created_at` timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP,
  `updated_at` timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (`policy_id`,`policy_uuid`) USING BTREE,
  UNIQUE KEY `uniq_policy_hash` (`policy_hash`),
  UNIQUE KEY `policy_uuid` (`policy_uuid`) USING BTREE,
  KEY `created_by` (`created_by`),
  CONSTRAINT `cedar_policy_set_ibfk_1` FOREIGN KEY (`created_by`) REFERENCES `users` (`user_id`)
) ENGINE=InnoDB AUTO_INCREMENT=7 DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci COMMENT='用来记录Cedar Policy Set的，一条记录一条策略';

-- ----------------------------
-- Records of cedar_policy_set
-- ----------------------------
BEGIN;
INSERT INTO `cedar_policy_set` (`policy_id`, `policy_uuid`, `annotation`, `policy_text`, `effect`, `is_active`, `description`, `policy_hash`, `policy_type`, `created_by`, `created_at`, `updated_at`) VALUES (1, '4b6f6012-228e-491e-8f2e-385e2d533279', 'Preset roles are read-only', '@annotation(\"预设角色是只读的\")\nforbid (   \n	principal,   \n	action,   \n	resource\n) when {\n  resource in [\n    Role::\"6b929a6a-7d8b-4fe0-9426-2c4d536353d7\", //超级管理员\n    Role::\"d206bc7d-0b99-4d75-922d-b35e95d58874\", //用户管理员\n    Role::\"7b9a44c7-c220-4acb-89a4-aa6490857edc\", //策略管理员\n  ]\n}; \n', 'forbid', 1, '禁止操作预设角色', '2222', 'STATIC', 1, '2025-08-15 16:17:24', '2025-09-26 14:25:32');
INSERT INTO `cedar_policy_set` (`policy_id`, `policy_uuid`, `annotation`, `policy_text`, `effect`, `is_active`, `description`, `policy_hash`, `policy_type`, `created_by`, `created_at`, `updated_at`) VALUES (2, '5db4b0c5-79bf-44e7-a965-68997bf59f87', 'The SuperAdmin has full system privileges.', '@annotation(\"管理员可以执行permit所有操作\")\npermit (\n    principal in Role::\"6b929a6a-7d8b-4fe0-9426-2c4d536353d7\",\n    action,\n    resource\n);', 'permit', 1, '超级管理员可以执行所有操作', '3333', 'STATIC', 1, '2025-08-14 15:43:09', '2025-09-26 14:26:49');
INSERT INTO `cedar_policy_set` (`policy_id`, `policy_uuid`, `annotation`, `policy_text`, `effect`, `is_active`, `description`, `policy_hash`, `policy_type`, `created_by`, `created_at`, `updated_at`) VALUES (3, '054eb869-95a7-40bb-832e-881d0d2b0cd9', 'Policy Administrator', '@annotation(\"策略管理员\")\npermit (\n    principal in Role::\"7b9a44c7-c220-4acb-89a4-aa6490857edc\",\n    action in [\n        Action::\"ViewPolicy\",\n        Action::\"CreatePolicy\",\n        Action::\"UpdatePolicy\",\n        Action::\"DeletePolicy\",\n    ], \n    resource\n);', 'permit', 1, '策略管理', '4444', 'STATIC', 1, '2025-08-14 15:43:09', '2025-09-26 14:27:03');
INSERT INTO `cedar_policy_set` (`policy_id`, `policy_uuid`, `annotation`, `policy_text`, `effect`, `is_active`, `description`, `policy_hash`, `policy_type`, `created_by`, `created_at`, `updated_at`) VALUES (4, '73f8b80a-3d3b-45a9-bfe5-5abdfcdb7e78', 'User Administrator', '@annotation(\"用户管理员\")\npermit (\n    principal in Role::\"d206bc7d-0b99-4d75-922d-b35e95d58874\",\n    action in [\n        Action::\"ViewUser\",\n        Action::\"CreateUser\", \n        Action::\"UpdateUser\",\n        Action::\"DeleteUser\",\n        Action::\"ViewGroup\",\n        Action::\"CreateGroup\",\n        Action::\"UpdateGroup\",\n        Action::\"DeleteGroup\",\n        Action::\"ViewRole\",\n        Action::\"CreateRole\",\n        Action::\"UpdateRole\",\n        Action::\"DeleteRole\",\n        Action::\"AssignRole\",\n        Action::\"RevokeRole\",\n        Action::\"ViewDepartment\",\n        Action::\"CreateDepartment\",\n        Action::\"UpdateDepartment\",\n        Action::\"DeleteDepartment\",\n        ],\n    resource\n);', 'permit', 1, '用户管理', '55555', 'STATIC', 1, '2025-08-14 15:43:09', '2025-09-26 14:27:30');
INSERT INTO `cedar_policy_set` (`policy_id`, `policy_uuid`, `annotation`, `policy_text`, `effect`, `is_active`, `description`, `policy_hash`, `policy_type`, `created_by`, `created_at`, `updated_at`) VALUES (5, '9cb0c036-2601-469d-8bb7-a75b831b0f90', 'Preset Groups are read-only', '@annotation(\"预设用户组对于非SuperAdmin角色是只读的\")\nforbid (\n  principal,\n  action,\n  resource\n) when {\n  resource in [\n    Group::\"3a112a74-c801-44eb-b2db-b57a61a0c1fb\",  //用户管理组\n    Group::\"8c73306b-f407-434b-952f-c9f792ad7aa9\", // 策略管理组\n    ] \n    && \n    !(principal in Role::\"6b929a6a-7d8b-4fe0-9426-2c4d536353d7\")\n};', 'forbid', 1, '预设用户组对于非SuperAdmin角色是只读的', '666', 'STATIC', 1, '2025-08-19 22:55:01', '2025-09-26 20:12:13');
INSERT INTO `cedar_policy_set` (`policy_id`, `policy_uuid`, `annotation`, `policy_text`, `effect`, `is_active`, `description`, `policy_hash`, `policy_type`, `created_by`, `created_at`, `updated_at`) VALUES (6, '9c4804ae-6ea1-474e-a756-d9751929e881', 'Preset policies are read-only', '@annotation(\"禁止更新和删除预设角色\")\nforbid (   \n	principal,   \n	action in [\n    Action::\"UpdateRole\",\n    Action::\"DeleteRole\",\n  ],   \n	resource\n) when {\n  resource in [\n    Role::\"6b929a6a-7d8b-4fe0-9426-2c4d536353d7\", //超级管理员\n    Role::\"d206bc7d-0b99-4d75-922d-b35e95d58874\", //用户管理员\n    Role::\"7b9a44c7-c220-4acb-89a4-aa6490857edc\", //策略管理员\n    Role::\"847437fd-da90-4a52-b69c-e2b1d80a02bb\", \n    Role::\"4f443a1f-3237-4de7-85c4-0a097f8498b1\", \n    Role::\"49d873fc-7cd4-4f57-babb-15f3424bd7bd\", \n    ]\n}; ', 'forbid', 1, '禁止更新和删除预设角色', '11111111', 'STATIC', 1, '2025-08-14 15:43:09', '2025-09-26 20:16:26');
COMMIT;

-- ----------------------------
-- Table structure for cedar_schema
-- ----------------------------
DROP TABLE IF EXISTS `cedar_schema`;
CREATE TABLE `cedar_schema` (
  `schema_id` int NOT NULL AUTO_INCREMENT,
  `schema_uuid` char(36) COLLATE utf8mb4_general_ci NOT NULL,
  `schema` text COLLATE utf8mb4_general_ci NOT NULL,
  `description` varchar(255) COLLATE utf8mb4_general_ci NOT NULL,
  `is_active` tinyint(1) NOT NULL DEFAULT '0',
  `created_at` timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP,
  `updated_at` timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (`schema_id`)
) ENGINE=InnoDB AUTO_INCREMENT=2 DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci COMMENT='存储 Cedar Schema ';

-- ----------------------------
-- Records of cedar_schema
-- ----------------------------
BEGIN;
INSERT INTO `cedar_schema` (`schema_id`, `schema_uuid`, `schema`, `description`, `is_active`, `created_at`, `updated_at`) VALUES (1, 'e1c94cb0-3e26-4319-a35b-244a3b0c186d', '// 没有多租户的需求不设置命名空间\n\n// 定义别名组合\n\n\n// -------------------------------------------------\n// 1. 定义核心实体类型\n// -------------------------------------------------\n\n// 应用程序实体（全局权限检查）\nentity Application;\n\n// 用户 (User) 实体。\n// 这是我们系统中的主体（Principal）。\n// 一个用户可以是多个角色的成员。\"用户-角色\"(临时附加) \"用户-用户组-角色\"(常规情况)。\n\nentity User in [Role, Department, Group] = {\n    name: String\n};\n\n\n// 角色 (Role) 实体。\nentity Role = {\n    name: String\n};\n\n// 用户组\nentity Group = {\n    name: String\n};\n// 部门\nentity Department = {\n    name: String\n};\n\n// Cedar Policy\nentity Policy = {\n    name: String\n};\n\n// 资源 (Resource) 实体。\n// 这是被保护的对象，例如一篇文章、一个文件或一个API端点。\n// 为了增加灵活性，资源可以被分组到“资源组”中。\n\n\n\n// -------------------------------------------------\n// 2. 定义操作 (Actions)\n// -------------------------------------------------\n\n\n// 定义CRUD操作。\n// appliesTo 部分将这些操作与我们的核心实体关联起来。\n// 主体 (principal) 通常是用户。\n// 资源 (resource) 就是被保护的Resource。\n// 用户\n\naction \"ViewUser\" appliesTo {\n    principal: User,\n    resource: User\n};\n\naction \"CreateUser\" appliesTo {\n    principal: User,\n    resource: User\n};\n\naction \"UpdateUser\" appliesTo {\n    principal: User,\n    resource: User\n};\n\naction \"DeleteUser\" appliesTo {\n    principal: User,\n    resource: User\n};\n\n// 用户组\n\naction \"ViewGroup\" appliesTo {\n    principal: User,\n    resource: Group\n};\n\naction \"ViewGroupUsers\" appliesTo {\n    principal: User,\n    resource: Group\n};\n\naction \"CreateGroup\" appliesTo {\n    principal: User,\n    resource: Group\n};\n\naction \"UpdateGroup\" appliesTo {\n    principal: User,\n    resource: Group\n};\n\naction \"DeleteGroup\" appliesTo {\n    principal: User,\n    resource: Group\n};\n\n\n// 角色\n\naction \"ViewRole\" appliesTo {\n    principal: User,\n    resource: Role\n};\n\naction \"CreateRole\" appliesTo {\n    principal: User,\n    resource: Role\n};\n\naction \"UpdateRole\" appliesTo {\n    principal: User,\n    resource: Role\n};\n\naction \"DeleteRole\" appliesTo {\n    principal: User,\n    resource: Role\n};\n\naction \"AssignRole\" appliesTo {\n    principal: User,\n    resource: Role\n};\n\naction \"RevokeRole\" appliesTo {\n    principal: User,\n    resource: Role\n};\n\n// 部门\n\naction \"ViewDepartment\" appliesTo {\n    principal: User,\n    resource: Department\n};\n\naction \"ViewDepartmentUsers\" appliesTo {\n    principal: User,\n    resource: Department\n};\n\naction \"CreateDepartment\" appliesTo {\n    principal: User,\n    resource: Department\n};\n\naction \"UpdateDepartment\" appliesTo {\n    principal: User,\n    resource: Department\n};\n\naction \"DeleteDepartment\" appliesTo {\n    principal: User,\n    resource: Department\n};\n\n// 策略\n\naction \"ViewPolicy\" appliesTo {\n    principal: User,\n    resource: Policy\n};\n\naction \"CreatePolicy\" appliesTo {\n    principal: User,\n    resource: Policy\n};\n\naction \"UpdatePolicy\" appliesTo {\n    principal: User,\n    resource: Policy\n};\n\naction \"DeletePolicy\" appliesTo {\n    principal: User,\n    resource: Policy\n};', 'V1', 0, '2025-08-16 13:51:28', '2025-09-23 16:28:59');
COMMIT;

-- ----------------------------
-- Table structure for departments
-- ----------------------------
DROP TABLE IF EXISTS `departments`;
CREATE TABLE `departments` (
  `dept_id` int NOT NULL AUTO_INCREMENT,
  `dept_uuid` char(36) CHARACTER SET utf8mb4 COLLATE utf8mb4_0900_ai_ci NOT NULL,
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
INSERT INTO `departments` (`dept_id`, `dept_uuid`, `created_at`, `updated_at`, `name`, `desc`, `is_deleted`, `order`, `parent_id`) VALUES (1, 'e051303d-1e9f-40bf-979d-f760cdc1e968', '2024-08-28 08:46:32.820905', '2025-09-11 21:14:51.948397', 'AxumVueAdmin', '初始化的根节点', 0, 0, 0);
INSERT INTO `departments` (`dept_id`, `dept_uuid`, `created_at`, `updated_at`, `name`, `desc`, `is_deleted`, `order`, `parent_id`) VALUES (2, 'ceb895fc-875f-4c4b-8805-cd909d132458', '2024-08-28 20:47:54.235343', '2025-09-11 21:15:01.421582', 'CompanyOne', '', 0, 0, 1);
INSERT INTO `departments` (`dept_id`, `dept_uuid`, `created_at`, `updated_at`, `name`, `desc`, `is_deleted`, `order`, `parent_id`) VALUES (4, 'd18dff5e-ad3d-4d4c-932d-15f0cafce649', '2024-08-28 20:49:37.477063', '2025-09-11 21:15:03.558696', 'SupplyChain', '', 0, 0, 2);
INSERT INTO `departments` (`dept_id`, `dept_uuid`, `created_at`, `updated_at`, `name`, `desc`, `is_deleted`, `order`, `parent_id`) VALUES (5, '2bf69e26-d8f8-4444-af65-ee3fe8a1f88a', '2024-08-28 20:49:54.751697', '2025-09-11 21:15:17.420990', 'Engineer', '', 0, 0, 2);
INSERT INTO `departments` (`dept_id`, `dept_uuid`, `created_at`, `updated_at`, `name`, `desc`, `is_deleted`, `order`, `parent_id`) VALUES (6, '9dd9511a-b928-4598-8ca3-1c7ea17c9e7b', '2024-08-28 20:50:13.102046', '2025-09-11 21:15:22.370656', 'ProgramManagement', '', 0, 0, 2);
INSERT INTO `departments` (`dept_id`, `dept_uuid`, `created_at`, `updated_at`, `name`, `desc`, `is_deleted`, `order`, `parent_id`) VALUES (7, 'eb4f6794-6b36-478c-9cb5-b9a3baca352b', '2024-08-28 20:50:27.010850', '2025-09-11 21:15:27.397836', 'Sales', '', 0, 0, 2);
INSERT INTO `departments` (`dept_id`, `dept_uuid`, `created_at`, `updated_at`, `name`, `desc`, `is_deleted`, `order`, `parent_id`) VALUES (8, '40d91b92-c9db-40e6-bc93-385e33460f30', '2024-08-28 20:50:42.432557', '2025-09-26 16:23:24.317654', 'Credit', '', 0, 0, 2);
INSERT INTO `departments` (`dept_id`, `dept_uuid`, `created_at`, `updated_at`, `name`, `desc`, `is_deleted`, `order`, `parent_id`) VALUES (9, '04c3f5d4-7cfc-4d1b-957b-07128331e8a4', '2024-08-28 20:50:54.654242', '2025-09-11 21:15:40.141359', 'Purchase', '', 0, 0, 2);
INSERT INTO `departments` (`dept_id`, `dept_uuid`, `created_at`, `updated_at`, `name`, `desc`, `is_deleted`, `order`, `parent_id`) VALUES (10, '459c4e47-0182-4d96-ae31-a7a2ea2b393b', '2024-08-28 20:51:07.921094', '2025-09-11 21:15:46.675038', 'Manufacturing', '', 0, 0, 2);
INSERT INTO `departments` (`dept_id`, `dept_uuid`, `created_at`, `updated_at`, `name`, `desc`, `is_deleted`, `order`, `parent_id`) VALUES (11, '2d85f297-9bbf-4a61-8637-ae792b240cde', '2024-08-28 20:51:21.093665', '2025-09-11 21:15:57.139253', 'Warehouse', '', 0, 0, 2);
INSERT INTO `departments` (`dept_id`, `dept_uuid`, `created_at`, `updated_at`, `name`, `desc`, `is_deleted`, `order`, `parent_id`) VALUES (12, '2c47f01c-d5a4-4645-97a3-5b09f4ac272e', '2024-08-28 20:51:34.879295', '2025-09-11 21:16:05.725851', 'Traffic', '', 0, 0, 2);
INSERT INTO `departments` (`dept_id`, `dept_uuid`, `created_at`, `updated_at`, `name`, `desc`, `is_deleted`, `order`, `parent_id`) VALUES (13, 'eb991941-7fbf-48c8-af99-18c61d0737db', '2024-08-28 20:51:50.115774', '2025-09-11 21:16:13.500918', 'AP', '', 0, 0, 2);
INSERT INTO `departments` (`dept_id`, `dept_uuid`, `created_at`, `updated_at`, `name`, `desc`, `is_deleted`, `order`, `parent_id`) VALUES (14, '878c4716-1f40-454b-b2c5-254838b1c5a9', '2024-08-28 20:56:29.873771', '2025-09-11 21:16:18.343329', 'ProductSourcingManagement', '', 0, 0, 2);
INSERT INTO `departments` (`dept_id`, `dept_uuid`, `created_at`, `updated_at`, `name`, `desc`, `is_deleted`, `order`, `parent_id`) VALUES (15, 'eacb9d3a-1ac7-4a76-a38f-1257c2aec597', '2024-08-28 20:56:44.034183', '2025-09-18 09:56:54.229852', 'Marketing', '', 0, 0, 2);
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
  `role_uuid` char(36) COLLATE utf8mb4_general_ci NOT NULL,
  `role_name` varchar(50) COLLATE utf8mb4_general_ci NOT NULL,
  `description` varchar(255) COLLATE utf8mb4_general_ci DEFAULT NULL,
  `created_at` timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP,
  PRIMARY KEY (`role_id`),
  UNIQUE KEY `role_name` (`role_name`)
) ENGINE=InnoDB AUTO_INCREMENT=8 DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci;

-- ----------------------------
-- Records of roles
-- ----------------------------
BEGIN;
INSERT INTO `roles` (`role_id`, `role_uuid`, `role_name`, `description`, `created_at`) VALUES (1, '6b929a6a-7d8b-4fe0-9426-2c4d536353d7', 'SuperAdmin', '超级管理员，拥有系统所有权限', '2025-05-21 20:46:26');
INSERT INTO `roles` (`role_id`, `role_uuid`, `role_name`, `description`, `created_at`) VALUES (2, 'd206bc7d-0b99-4d75-922d-b35e95d58874', 'UserAdministrator', '用户管理', '2025-06-27 14:56:43');
INSERT INTO `roles` (`role_id`, `role_uuid`, `role_name`, `description`, `created_at`) VALUES (3, '7b9a44c7-c220-4acb-89a4-aa6490857edc', 'PolicyAdministrator', '策略管理员', '2025-08-19 21:10:49');
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
-- Table structure for template_links
-- ----------------------------
DROP TABLE IF EXISTS `template_links`;
CREATE TABLE `template_links` (
  `link_id` int NOT NULL AUTO_INCREMENT,
  `link_uuid` char(36) COLLATE utf8mb4_general_ci NOT NULL,
  `template_uuid` char(36) COLLATE utf8mb4_general_ci NOT NULL,
  `principal_uid` varchar(50) COLLATE utf8mb4_general_ci NOT NULL,
  `resource_uid` varchar(50) COLLATE utf8mb4_general_ci NOT NULL,
  `created_at` timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP,
  `updated_at` timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (`link_id`) USING BTREE,
  KEY `template_uuid` (`template_uuid`),
  CONSTRAINT `template_links_ibfk_1` FOREIGN KEY (`template_uuid`) REFERENCES `cedar_policy_set` (`policy_uuid`) ON DELETE CASCADE ON UPDATE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci;

-- ----------------------------
-- Records of template_links
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
INSERT INTO `user_group_members` (`user_id`, `group_id`, `assigned_at`) VALUES (46, 3, '2025-08-30 14:24:00');
INSERT INTO `user_group_members` (`user_id`, `group_id`, `assigned_at`) VALUES (48, 3, '2025-09-26 20:21:33');
COMMIT;

-- ----------------------------
-- Table structure for user_groups
-- ----------------------------
DROP TABLE IF EXISTS `user_groups`;
CREATE TABLE `user_groups` (
  `user_group_id` int NOT NULL AUTO_INCREMENT,
  `user_group_uuid` char(36) CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci NOT NULL COMMENT '用户组唯一ID/用户组唯一标识符',
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
INSERT INTO `user_groups` (`user_group_id`, `user_group_uuid`, `name`, `description`, `created_at`, `updated_at`) VALUES (1, '3a112a74-c801-44eb-b2db-b57a61a0c1fb', 'UserManagementGroup', '用户管理组', '2025-06-29 20:12:27', '2025-09-17 17:11:17');
INSERT INTO `user_groups` (`user_group_id`, `user_group_uuid`, `name`, `description`, `created_at`, `updated_at`) VALUES (2, '8c73306b-f407-434b-952f-c9f792ad7aa9', 'PolicyManagementGroup', '策略管理组', '2025-08-19 21:16:05', '2025-09-17 17:11:27');
INSERT INTO `user_groups` (`user_group_id`, `user_group_uuid`, `name`, `description`, `created_at`, `updated_at`) VALUES (3, '3af1326f-b71b-4ff1-90b6-e3c190f17bd0', 'UserGroup', '普通用户组', '2025-06-23 17:17:27', '2025-09-17 17:11:36');
COMMIT;

-- ----------------------------
-- Table structure for user_invitations
-- ----------------------------
DROP TABLE IF EXISTS `user_invitations`;
CREATE TABLE `user_invitations` (
  `id` int NOT NULL AUTO_INCREMENT,
  `inviter_user_id` int NOT NULL COMMENT '邀请人的用户ID',
  `invitee_email` varchar(255) CHARACTER SET utf8mb4 COLLATE utf8mb4_0900_ai_ci NOT NULL COMMENT '被邀请人的邮箱地址',
  `invitee_user_id` int DEFAULT NULL COMMENT '被邀请人接受邀请并注册后的用户ID',
  `invitation_code` varchar(255) CHARACTER SET utf8mb4 COLLATE utf8mb4_0900_ai_ci NOT NULL COMMENT '唯一的邀请码',
  `status` varchar(20) CHARACTER SET utf8mb4 COLLATE utf8mb4_0900_ai_ci NOT NULL DEFAULT 'pending' COMMENT '邀请的状态。pending: 待接受；accepted: 已接受；expired: 已过期；revoked: 已撤销',
  `expires_at` timestamp NOT NULL COMMENT '邀请的过期时间',
  `created_at` timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP,
  `updated_at` timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (`id`),
  UNIQUE KEY `uk_invitation_code` (`invitation_code`),
  KEY `idx_inviter_user_id` (`inviter_user_id`),
  KEY `idx_invitee_email` (`invitee_email`),
  KEY `fk_invitee_user_id` (`invitee_user_id`),
  CONSTRAINT `fk_invitee_user_id` FOREIGN KEY (`invitee_user_id`) REFERENCES `users` (`user_id`),
  CONSTRAINT `fk_inviter_user_id` FOREIGN KEY (`inviter_user_id`) REFERENCES `users` (`user_id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;

-- ----------------------------
-- Records of user_invitations
-- ----------------------------
BEGIN;
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
  `user_uuid` char(36) NOT NULL COMMENT '用户UUID',
  `created_at` datetime(6) NOT NULL DEFAULT CURRENT_TIMESTAMP(6),
  `updated_at` datetime(6) NOT NULL DEFAULT CURRENT_TIMESTAMP(6) ON UPDATE CURRENT_TIMESTAMP(6),
  `username` varchar(100) CHARACTER SET utf8mb4 COLLATE utf8mb4_0900_ai_ci NOT NULL COMMENT '用户名称',
  `alias` varchar(30) DEFAULT NULL COMMENT '姓名',
  `email` varchar(255) NOT NULL COMMENT '邮箱',
  `phone` varchar(20) DEFAULT NULL COMMENT '电话',
  `password` varchar(128) CHARACTER SET utf8mb4 COLLATE utf8mb4_0900_ai_ci NOT NULL COMMENT '密码',
  `dept_id` int NOT NULL COMMENT '部门ID',
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
) ENGINE=InnoDB AUTO_INCREMENT=49 DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;

-- ----------------------------
-- Records of users
-- ----------------------------
BEGIN;
INSERT INTO `users` (`user_id`, `user_uuid`, `created_at`, `updated_at`, `username`, `alias`, `email`, `phone`, `password`, `dept_id`, `is_active`, `avatar`, `last_login`, `reset_token`, `reset_triggered`) VALUES (1, '0dd34d6b-9ff6-4e5a-ab2b-3ae17d7b4f56', '2024-08-28 16:17:28.304153', '2025-09-26 17:18:45.714506', 'superadmin', NULL, 'superadmin@xxxx.com', NULL, '$argon2id$v=19$m=19456,t=2,p=1$2WGtjhokqiE6ToWGZQ1ukQ$rDYtzjjvewEDwav+H/IjyfkG6lAtKBLZocdwMCCRLJ0', 1, 1, 'https://avatars.githubusercontent.com/u/54677442?v=4', '2025-09-26 17:18:45.704852', NULL, '2025-08-19 01:22:50.017915');
INSERT INTO `users` (`user_id`, `user_uuid`, `created_at`, `updated_at`, `username`, `alias`, `email`, `phone`, `password`, `dept_id`, `is_active`, `avatar`, `last_login`, `reset_token`, `reset_triggered`) VALUES (43, 'e249bc31-1399-42e4-b9b7-ca13619bfc35', '2025-06-29 06:40:49.073780', '2025-09-17 17:41:00.484925', 'useradmin', NULL, 'admin@xxx.com', NULL, '$argon2id$v=19$m=19456,t=2,p=1$2WGtjhokqiE6ToWGZQ1ukQ$rDYtzjjvewEDwav+H/IjyfkG6lAtKBLZocdwMCCRLJ0', 1, 1, NULL, '2025-08-23 05:34:13.612591', '69ac5409-8d6c-4b89-96e5-678e03639086', '2025-08-18 15:31:01.186673');
INSERT INTO `users` (`user_id`, `user_uuid`, `created_at`, `updated_at`, `username`, `alias`, `email`, `phone`, `password`, `dept_id`, `is_active`, `avatar`, `last_login`, `reset_token`, `reset_triggered`) VALUES (44, '71c8834e-ad4d-4a78-a365-6783015f7488', '2025-08-21 03:55:55.400094', '2025-09-17 17:41:04.579539', 'policiadmin', NULL, 'policiadmin@xxx.com', NULL, '$argon2id$v=19$m=19456,t=2,p=1$S7MvIZk7rEUYOyuKZPtUsw$VM1rsQTYIC7w1uxcin4WXDroDoUS309O4bmuX+q4Aw4', 1, 1, NULL, '2025-08-21 04:27:23.900143', NULL, NULL);
INSERT INTO `users` (`user_id`, `user_uuid`, `created_at`, `updated_at`, `username`, `alias`, `email`, `phone`, `password`, `dept_id`, `is_active`, `avatar`, `last_login`, `reset_token`, `reset_triggered`) VALUES (46, 'a11fc672-f2ff-40e0-a948-6779b2bfa698', '2025-08-30 06:24:00.941481', '2025-09-17 17:41:09.533809', 'Darren.Tan', NULL, 'darren.tan@xxx.com', NULL, '$argon2id$v=19$m=19456,t=2,p=1$Qm7lrDz9rHkLkvi18ppw/w$tuXstCvNl/4l9G1rIrqUXTecctcOVOpQQ8aSBgbCm00', 13, 1, NULL, NULL, NULL, NULL);
INSERT INTO `users` (`user_id`, `user_uuid`, `created_at`, `updated_at`, `username`, `alias`, `email`, `phone`, `password`, `dept_id`, `is_active`, `avatar`, `last_login`, `reset_token`, `reset_triggered`) VALUES (47, 'be0345b9-71da-4e6e-9ccb-68fd6aa552cc', '2025-08-30 06:24:00.941481', '2025-09-17 17:41:10.876435', 'biubiu.biu', NULL, 'biubiu.biu@xxx.com', NULL, '$argon2id$v=19$m=19456,t=2,p=1$Qm7lrDz9rHkLkvi18ppw/w$tuXstCvNl/4l9G1rIrqUXTecctcOVOpQQ8aSBgbCm00', 13, 1, NULL, NULL, NULL, NULL);
INSERT INTO `users` (`user_id`, `user_uuid`, `created_at`, `updated_at`, `username`, `alias`, `email`, `phone`, `password`, `dept_id`, `is_active`, `avatar`, `last_login`, `reset_token`, `reset_triggered`) VALUES (48, 'e0e442f7-2783-401a-b44c-84a26862097d', '2025-09-26 12:21:33.522042', '2025-09-26 12:21:33.522042', 'test_user_uuid', NULL, 'test_user_uuid@xxx.com', NULL, '$argon2id$v=19$m=19456,t=2,p=1$mMKzGiCs8YQDg1HhLIAtiw$j7Ubnqh6I+ZF3tyejtHO89IftrK+wbxLGod677zziwA', 15, 1, NULL, NULL, NULL, NULL);
COMMIT;

SET FOREIGN_KEY_CHECKS = 1;
