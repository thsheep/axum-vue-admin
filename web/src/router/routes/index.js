import i18n from '@/i18n'

const {t} = i18n.global

const Layout = () => import('@/layout/index.vue')

/*
路由不应该由后端配置，会造成以下问题：
1.前后端强耦合
2.前端极差的开发体验
3.后端逻辑复杂

正确的做法是：
1.前端定义全量路由
2.使用meta字段标记资源
3.后端提供数据
    UI资源的策略定义 (uiPolicies)。
4.前端进行过滤和保护


icons: https://icones.js.org/
*/
export const basicRoutes = [
    {
        name: 'index',
        path: '/',
        redirect: '/workbench', // 默认跳转到首页
        meta: {
            order: 0,
            isHidden: true
        },
    },
    {
        name: t('views.workbench.label_workbench'),
        path: '/workbench',
        component: Layout,
        children: [
            {
                path: '',
                component: () => import('@/views/workbench/index.vue'),
                name: t('views.workbench.label_workbench'),
                meta: {
                    title: t('views.workbench.label_workbench'),
                    icon: 'icon-park-outline:workbench',
                    affix: true,
                },
            },
        ],
        meta: {order: 1},
    },
    {
        path: '/login',
        name: 'Login',
        component: () => import('@/views/login/index.vue'),
        meta: {isHidden: true},
    },
    {
        name: 'ForGotPassword',
        path: '/forgot-password',
        component: () => import('@/views/forgot-password/index.vue'),
        meta: {
            isHidden: true,
            title: t('views.forget_password.forget_password'),
        },
    },
    {
        name: 'RestPassword',
        path: '/reset-password/:resetToken',
        component: () => import('@/views/reset-password/index.vue'),
        isHidden: true,
        props: true,
        meta: {
            isHidden: true,
            title: t('views.resetPassword.resetPassword'),
        },
    },
    {
        path: '/error-page',
        name: 'ErrorPage',
        component: Layout,
        redirect: '/error-page/404',
        meta: {isHidden: true},
        children: [
            {
                path: '401',
                name: 'ERROR-401',
                component: () => import('@/views/error-page/401.vue'),
                meta: {
                    title: '401',
                    icon: 'material-symbols:authenticator',
                },
            },
            {
                path: '403',
                name: 'ERROR-403',
                component: () => import('@/views/error-page/403.vue'),
                meta: {
                    title: '403',
                    icon: 'solar:forbidden-circle-line-duotone',
                },
            },
            {
                path: '404',
                name: 'ERROR-404',
                component: () => import('@/views/error-page/404.vue'),
                meta: {
                    title: '404',
                    icon: 'tabler:error-404',
                },
            },
            {
                path: '500',
                name: 'ERROR-500',
                component: () => import('@/views/error-page/500.vue'),
                meta: {
                    title: '500',
                    icon: 'clarity:rack-server-outline-alerted',
                },
            },
        ],
    },
    // 404 路由必须放在最后
    {
        path: '/:pathMatch(.*)*',
        name: 'NotFound',
        redirect: '/error-page/404',
        meta: {isHidden: true},
    },
]

// 需要过滤鉴权的路由放这儿
export const asyncRoutes = [
    {
        name: "系统管理",
        path: "/system",
        component: Layout,
        "redirect": "user",
        meta: {
            title: "系统管理",
            icon: "carbon:gui-management",
            order: 2,
            keepAlive: true,
            isHidden: false,
        },
        children: [
            {
                name: "用户管理",
                path: "user",
                component: () => import('@/views/system/user/index.vue'),
                meta: {
                    title: "用户管理",
                    icon: "material-symbols:person-outline-rounded",
                    order: 1,
                    keepAlive: true,
                    isHidden: false,
                    resource: "menus:user_management",
                }
            },
            {
                name: "用户组管理",
                path: "group",
                component: () => import('@/views/system/group/index.vue'),
                meta: {
                    title: "用户组管理",
                    icon: "mdi:account-group-outline",
                    order: 2,
                    keepAlive: true,
                    isHidden: false,
                    resource: "menus:group_management",
                }
            },
            {
                name: "角色管理",
                path: "role",
                component: () => import('@/views/system/role/index.vue'),
                meta: {
                    title: "角色管理",
                    icon: "carbon:user-role",
                    order: 3,
                    keepAlive: true,
                    isHidden: false,
                    resource: "menus:role_management",
                }
            },
            {
                name: "部门管理",
                path: "dept",
                component: () => import('@/views/system/dept/index.vue'),
                meta: {
                    title: "部门管理",
                    icon: "mingcute:department-line",
                    order: 4,
                    keepAlive: true,
                    isHidden: false,
                    resource: "menus:dept_management",
                }
            },
            {
                name: "CedarPolicy管理",
                path: "cedar_policies",
                component: () => import('@/views/system/cedar_policies/index.vue'),
                meta: {
                    title: "CedarPolicy管理",
                    icon: "carbon:policy",
                    order: 5,
                    keepAlive: true,
                    isHidden: false,
                    resource: "menus:policies_management",
                }
            },{
                name: "CedarSchema管理",
                path: "cedar_schema",
                component: () => import('@/views/system/cedar_schema/index.vue'),
                meta: {
                    title: "CedarSchema管理",
                    icon: "carbon:schematics",
                    order: 6,
                    keepAlive: true,
                    isHidden: false,
                    resource: "menus:policies_management",
                }
            },
            {
                name: "审计日志",
                path: "auditlog",
                component: () => import('@/views/system/auditlog/index.vue'),
                meta: {
                    title: "审计日志",
                    icon: "ph:clipboard-text-bold",
                    order: 7,
                    keepAlive: true,
                    isHidden: false,
                    resource: "menus:auditlog_management",
                }
            }
        ]
    },
    {
        name: t('views.profile.label_profile'),
        path: '/profile',
        component: Layout,
        children: [
            {
                path: '',
                component: () => import('@/views/profile/index.vue'),
                name: `${t('views.profile.label_profile')}Default`,
                meta: {
                    title: t('views.profile.label_profile'),
                    icon: 'user',
                    affix: true,
                },
            },
        ],
        meta: {
            order: 99,
            isHidden: true,
        },
    },
]

export const NOT_FOUND_ROUTE = {
    name: 'NotFound',
    path: '/:pathMatch(.*)*',
    redirect: '/404',
    isHidden: true,
}

export const EMPTY_ROUTE = {
    name: 'Empty',
    path: '/:pathMatch(.*)*',
    component: null,
}

// const modules = import.meta.glob('@/views/**/route.js', { eager: true })
// Object.keys(modules).forEach((key) => {
//   asyncRoutes.push(modules[key].default)
// })
