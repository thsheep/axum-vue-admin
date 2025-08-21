import { useUserStore } from '@/store'

/**
 * 检查当前用户是否对某个UI资源有访问权限
 * @param {string} resourceIdentifier - 资源的唯一标识, e.g., 'menus:dashboard'
 * @returns {boolean}
 */
export function hasPermission(resourceIdentifier) {
  const user = useUserStore()

  const uiPolicies = user.uiPolicies || []
  // 1. 最高优先级：检查布尔值标识
  if (user && user.isSuperUser) {
    return true;
  }

  // 如果不是超级管理员，再走标准的权限检查流程
  if (!user || !user.uiPolicies) {
    return false;
  }

  // 5. 检查用户是否拥有该权限
  return uiPolicies.includes(resourceIdentifier);
}

export default function setupPermissionDirective(app) {
  function updateElVisible(el, resourceIdentifier) {
    if (!resourceIdentifier) {
      throw new Error(`need permissions: like v-permission="['user:create', 'user:update']"`)
    }
    if (!hasPermission(resourceIdentifier)) {
      el.parentElement?.removeChild(el)
    }
  }

  const permissionDirective = {
    mounted(el, binding) {
      updateElVisible(el, binding.value)
    },
    beforeUpdate(el, binding) {
      updateElVisible(el, binding.value)
    },
  }

  app.directive('permission', permissionDirective)
}
