import {hasPermission} from "@/directives/permission";
import {useUserStore} from "@/store";
import {addDynamicRoutes} from "@/router";

const WHITE_LIST = ['/login', '/404', '/forgot-password', /^\/reset-password\/.*/]
export function createAuthGuard(router) {
  router.beforeEach(async (to) => {
    const userStore = useUserStore();
    /** 没有token的情况 */
    if (!userStore.accessToken) {
      // 检查目标路径是否在白名单中
      const isInWhiteList = WHITE_LIST.some((pattern) => {
        if (typeof pattern === 'string') {
          return pattern === to.path // 匹配普通路径
        }
        return pattern.test(to.path) // 匹配正则表达式
      })
      if (isInWhiteList) return true
      return { path: 'login', query: { ...to.query, redirect: to.path } }
    }

    if (!userStore.isDynamicRoutesAdded) { // 使用标志位防止重复添加

      addDynamicRoutes();

      userStore.setDynamicRoutesAdded(true); // 设置标志位
      return { ...to, replace: true }; // 重新导航以应用新路由
    }


    /** 有token的情况 */
    if (to.path === '/login') return { path: '/' }

    // 路由权限校验
    // 1. 获取目标路由的资源标识
    const resource = to.meta.resource;

    // 2. 如果路由没有定义资源标识，说明是公共页面（如403页），直接放行
    if (!resource) {
      return true;
    }

    // 3. 检查用户是否有权限访问该资源
    if (hasPermission(resource)) {
      // 有权限，放行
      return true;
    } else {
      // 无权限，重定向到403页面
      console.warn(`Access denied for resource: ${resource}`);
      return {path: "403"};
    }
  })
}
