import { createRouter, createWebHistory, createWebHashHistory } from 'vue-router'
import { setupRouterGuard } from './guard'
import {asyncRoutes, basicRoutes, EMPTY_ROUTE} from './routes'
import {hasPermission} from "@/directives/permission";
import {useUserStore} from "@/stores";

const isHash = import.meta.env.VITE_USE_HASH === 'true'



export const router = createRouter({
  history: isHash ? createWebHashHistory('/') : createWebHistory('/'),
  routes: basicRoutes,
  scrollBehavior: () => ({ left: 0, top: 0 }),
})

export async function setupRouter(app) {
  await addDynamicRoutes()
  setupRouterGuard(router)
  app.use(router)
}

export async function resetRouter() {
  const basicRouteNames = getRouteNames(basicRoutes)
  router.getRoutes().forEach((route) => {
    const name = route.name
    if (!basicRouteNames.includes(name)) {
      router.removeRoute(name)
    }
  })
}

export function addDynamicRoutes() {
  const userStore = useUserStore()
  const uiPolicies = userStore.uiPolicies || []

  // 没有uiPolicies情况
  if (!uiPolicies) {
    router.addRoute(EMPTY_ROUTE)
    return
  }
  // 有uiPolicies的情况
  let accessibleRoutes = filterAccessibleRoutes(asyncRoutes, uiPolicies)
  accessibleRoutes.forEach((route) => {
    router.addRoute(route)
  })
  console.log(router.getRoutes())
}

export function getRouteNames(routes) {
  return routes.map((route) => getRouteName(route)).flat(1)
}

function getRouteName(route) {
  const names = [route.name]
  if (route.children && route.children.length) {
    names.push(...route.children.map((item) => getRouteName(item)).flat(1))
  }
  return names
}


function filterAccessibleRoutes(routesToFilter, userPermissions) { //
  const accessibleRoutes = [];

  routesToFilter.forEach(route => {
    const tempRoute = { ...route };

    let hasVisibleChildren = false;
    // 如果有子路由，则递归过滤
    if (tempRoute.children) {
      tempRoute.children = filterAccessibleRoutes(tempRoute.children, userPermissions);
      if (tempRoute.children.length > 0) {
        hasVisibleChildren = true;
      }
    }

    // 判断是否保留当前路由的条件
    const resource = tempRoute.meta?.resource;
    // 条件1: 它有可见的子路由（意味着它是一个需要显示的父菜单）
    // 条件2: 它本身就有权限被访问（适用于没有子路由的菜单项，或者父菜单本身也对应一个页面）
    const hasAccess = !resource || userPermissions.includes(resource);

    if (hasVisibleChildren || hasAccess) {
      // 如果它没有子路由了，就没必要保留 children 属性
      if (!hasVisibleChildren) {
        delete tempRoute.children;
      }
      accessibleRoutes.push(tempRoute);
    }
  });
  return accessibleRoutes;
}