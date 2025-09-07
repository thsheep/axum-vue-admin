<script setup>
import {h, onMounted, ref, reactive, resolveDirective, withDirectives, computed} from 'vue'
import {
  NButton, NForm, NFormItem, NInput, NLayout, NLayoutContent, NLayoutSider,
  NPopconfirm, NPopover, NSpace, NSwitch, NTag, NTreeSelect, NSelect, NModal, NDataTable, NTree
} from 'naive-ui'
import {storeToRefs} from 'pinia'
import {useUserManagementStore} from '@/stores'
import {formatDate, renderIcon, isEmail} from '@/utils'

import CommonPage from '@/components/page/CommonPage.vue'
import QueryBarItem from '@/components/query-bar/QueryBarItem.vue'
import CrudModal from '@/components/table/CrudModal.vue'
import CrudTable from '@/components/table/CrudTable.vue'
import TheIcon from '@/components/icon/TheIcon.vue'

defineOptions({name: '用户管理'})


const store = useUserManagementStore()
const {
  users,
  pagination,
  isTableLoading,
  departmentOptions,
  userGroupOptions,
  roleOptions,
  currentUserRoles,
} = storeToRefs(store)

const vPermission = resolveDirective('permission')


const showCrudModal = ref(false)
const modalType = ref('create')
const modalTitle = computed(() => modalType.value === 'create' ? '新建用户' : '编辑用户')
const currentItem = ref({})
const isSubmitting = ref(false)


const showRoleModal = ref(false)
const currentUserId = ref(null)
const selectedRoleIds = ref([])


const queryParams = reactive({})


onMounted(() => {
  store.fetchDependencies()
  handleSearch()
})

// --- 事件处理函数 ---
const loadData = () => {
  store.fetchUsers({
    ...queryParams,
    page: pagination.value.page,
    pageSize: pagination.value.pageSize,
  }).catch(error => {
    $message.error(`加载用户失败: ${error.message}`)
  })
}

// 处理来自 CrudTable 的搜索事件
const handleSearch = () => {
  pagination.value.page = 1 // 搜索时重置到第一页
  loadData()
}

// 处理的重置事件
const handleResetSearch = () => {
  for (const key in queryParams) {
    delete queryParams[key]
  }
  handleSearch()
}

// 处理的页码变更事件
const handlePageChange = (page) => {
  pagination.value.page = page
  loadData()
}

// 处理页面大小变更事件
const handlePageSizeChange = (pageSize) => {
  pagination.value.pageSize = pageSize
  pagination.value.page = 1
  loadData()
}

// CURD

const handleAdd = () => {
  currentItem.value = { is_active: true }
  modalType.value = 'create'
  showCrudModal.value = true
}

const handleEdit = (row) => {
  store.fetchUsers()
  currentItem.value = {
    ...row,
    dept: row.dept?.id,
    groups: row.groups?.map(g => g.id),
  }
  modalType.value = 'edit'
  showCrudModal.value = true
}

const handleSave = async () => {
  isSubmitting.value = true
  try {
    if (modalType.value === 'create') {
      await store.createUser(currentItem.value)
      $message.success('创建成功')
    } else {
      await store.updateUser(currentItem.value.id, currentItem.value)
      $message.success('更新成功')
    }
    showCrudModal.value = false
  } catch (error) {
    $message.error(`操作失败: ${error.message}`)
  } finally {
    isSubmitting.value = false
  }
}

const handleDelete = async (row) => {
  try {
    await store.deleteUser(row.id)
    $message.success('删除成功')
  } catch (error) {
    $message.error(`删除失败: ${error.message}`)
  }
}

// Role Modal
const handleViewRoles = async (row) => {
  currentUserId.value = row.id
  selectedRoleIds.value = []
  try {
    await store.fetchUserRoles(row.id)
    showRoleModal.value = true
  } catch (error) {
    $message.error(`获取角色失败: ${error.message}`)
  }
}

const handleAddRoles = async () => {
  if (!selectedRoleIds.value || selectedRoleIds.value.length === 0) {
    $message.warning('请选择要添加的角色');
    return;
  }
  try {
    await store.addUserRoles(currentUserId.value, selectedRoleIds.value);
    $message.success('角色添加成功');
    selectedRoleIds.value = [];
  } catch (error) {
    $message.error(`添加失败: ${error.message}`);
  }
}

const handleRemoveRole = async (roleId) => {
  try {
    await store.removeUserRole(currentUserId.value, roleId);
    $message.success('角色移除成功');
  } catch (error) {
    $message.error(`移除失败: ${error.message}`);
  }
}


const columns = [
  {
    title: '名称',
    key: 'username',
    width: 60,
    align: 'center',
    ellipsis: {tooltip: true},
  },
  {
    title: '邮箱',
    key: 'email',
    width: 60,
    align: 'center',
    ellipsis: {tooltip: true},
  },
  {
    title: '用户组',
    width: 100,
    align: 'center',
    render(row) {
      const groups = row.groups ?? []
      if (groups.length === 0) {
        return h('span', h(NTag, {type: 'info', style: {margin: '2px 3px'}}, {default: () => "No Group"}))
      }
      const MAX_VISIBLE = 1;
      const tagsToShow = groups.slice(0, MAX_VISIBLE);
      const hiddenCount = groups.length - MAX_VISIBLE;

      const triggerTags = tagsToShow.map(group =>
          h(NTag, { type: 'info', style: { marginRight: '6px' } }, { default: () => group.name })
      );

      if (hiddenCount > 0) {
        triggerTags.push(h(NTag, { type: 'default' }, { default: () => `+${hiddenCount}` }))
      }

      const allTags = groups.map(group =>
          h(NTag, { type: 'info', style: { margin: '2px' } }, { default: () => group.name })
      );

      if (hiddenCount <= 0) {
        return h('div', { style: { display: 'flex', justifyContent: 'center' } }, triggerTags)
      }

      return h(
          NPopover,
          {
            trigger: 'hover',
            placement: 'top',
            style: { maxWidth: '300px' }
          },
          {
            trigger: () => h('div', { style: { display: 'flex', alignItems: 'center', cursor: 'pointer' } }, triggerTags),
            default: () => h('div', { style: { display: 'flex', flexWrap: 'wrap', gap: '4px' } }, allTags)
          }
      )

    }
  },
  {
    title: '部门',
    key: 'dept.name',
    align: 'center',
    width: 40,
    ellipsis: {tooltip: true},
  },
  {
    title: '状态',
    key: 'is_active',
    width: 80,
    align: 'center',
    render(row) {
      return h(NSwitch,
          { defaultValue: row.is_active, disabled: true },
          {
            checked: () => '启用',
            unchecked: () => '禁用'
          })
    },
  },
  {
    title: '上次登录时间',
    key: 'last_login',
    align: 'center',
    width: 80,
    ellipsis: {tooltip: true},
    render(row) {
      return h(
          NButton,
          {size: 'small', type: 'text', ghost: true},
          {
            default: () => (row.last_login !== null ? formatDate(row.last_login) : "N/A"),
            icon: renderIcon('mdi:update', {size: 16}),
          }
      )
    },
  },
  {
    title: '操作',
    key: 'actions',
    width: 280,
    align: 'center',
    fixed: 'right',
    render(row) {
      return [
        withDirectives(
            h(NButton,
                {
                  size: 'small',
                  type: 'primary',
                  style: 'margin-right: 8px;',
                  onClick: () => handleEdit(row)},
                {default: () => '编辑', icon: renderIcon('material-symbols:edit', {size: 16})}),
            [[vPermission, 'button:user_update']]
        ),
        withDirectives(
            h(NButton, {
                  size: 'small',
                  type: 'primary',
                  style: 'margin-right: 8px;',
                  onClick: () => handleViewRoles(row)
                },
                {default: () => '查看角色', icon: renderIcon('material-symbols:visibility-outline', {size: 16})}),
            [[vPermission, 'button:user_view']]
        ),
        h(NPopconfirm, {onPositiveClick: () => handleDelete(row)},
            {
              trigger: () => withDirectives(
                  h(NButton, {size: 'small', type: 'error'},
                      {default: () => '删除', icon: renderIcon('material-symbols:delete-outline', {size: 16})}),
                  [[vPermission, 'button:user_delete']]
              ),
              default: () => '确定删除该用户吗?'
            }
        ),
      ]
    },
  },
]

const userRoleColumns = [
  {
    title: '角色',
    key: 'role_name',
    align: 'center',
    width: 150,
  },
  {
    title: '来源',
    key: 'source',
    align: 'center',
    width: 150,
  },
  {
    title: '用户组',
    key: 'group_name',
    align: 'center',
    width: 150,
  },
  {
    title: '操作',
    key: 'actions',
    render(row) {
      if (row.source === 'direct') {
        return h(NPopconfirm, {onPositiveClick: () => handleRemoveRole(row.id)},
            {
              trigger: () => withDirectives(
                  h(NButton,
                      {
                        size: 'small',
                        type: 'error'},
                      {
                        default: () => '删除',
                        icon: renderIcon('material-symbols:delete-outline',{size: 16})}),
                  [[vPermission, 'button:user_delete']]
              ),
              default: () => '确定删除该用户的直接角色吗?',
            }
        )
      }
    }
  }
]

const validateAddUser = {
  username: [
    {
      required: true,
      message: '请输入名称',
      trigger: ['input', 'blur'],
    },
  ],
  email: [
    {
      required: true,
      message: '请输入邮箱地址',
      trigger: ['input', 'change'],
    },
    {
      trigger: ['blur'],
      validator: (rule, value, callback) => {
        const re = /^[a-zA-Z0-9_-]+@[a-zA-Z0-9_-]+(\.[a-zA-Z0-9_-]+)+$/
        if (!re.test(currentItem.email)) {
          callback('邮箱格式错误')
          return
        }
        callback()
      },
    },
  ],
  password: [
    {
      required: true,
      message: '请输入密码',
      trigger: ['input', 'blur', 'change'],
    },
  ],
  dept_ids: [
    {
      type: 'array',
      required: true,
      message: '请选择一个部门',
      trigger: ['blur', 'change'],
    },
  ],
  confirmPassword: [
    {
      required: true,
      message: '请再次输入密码',
      trigger: ['input'],
    },
    {
      trigger: ['blur'],
      validator: (rule, value) => {
        if (value !== currentItem.value.password) {
          return new Error('两次密码输入不一致');
        }
        return true;
      },
    },
  ],
}

// 左侧的部门选择
const nodeProps = ({option}) => {
  return {
    onClick() {
      if (queryParams.dept_id === option.id) {
        delete queryParams.dept_id
        handleSearch()
      } else {
        queryParams.dept_id=option.id
        handleSearch()
      }
    },
  }
}
</script>

<template>
  <NLayout has-sider wh-full>
    <NLayoutSider bordered content-style="padding: 24px;" :width="240" show-trigger="arrow-circle">
      <h1>部门列表</h1>
      <br/>
      <NTree
          block-line
          :data="departmentOptions"
          key-field="id"
          label-field="name"
          default-expand-all
          :node-props="nodeProps"
      />
    </NLayoutSider>
    <NLayoutContent>
      <CommonPage show-footer title="用户列表">
        <template #action>
          <NButton v-permission="'button:user_create'" type="primary" @click="handleAdd">
            <TheIcon icon="material-symbols:add" :size="18" class="mr-5"/>
            新建用户
          </NButton>
        </template>

        <CrudTable
            :columns="columns"
            :data="users"
            :loading="isTableLoading"
            :pagination="pagination"
            @update:page="handlePageChange"
            @update:page-size="handlePageSizeChange"
            @search="handleSearch"
            @reset="handleResetSearch"
        >
          <template #queryBar>
            <QueryBarItem label="名称" :label-width="40">
              <NInput
                  v-model:value="queryParams.username"
                  clearable
                  type="text"
                  placeholder="请输入用户名称"
                  @keypress.enter="handleSearch()"
              />
            </QueryBarItem>
            <QueryBarItem label="邮箱" :label-width="40">
              <NInput
                  v-model:value="queryParams.email"
                  clearable
                  type="text"
                  placeholder="请输入邮箱"
                  @keypress.enter="handleSearch()"
              />
            </QueryBarItem>
            <!-- ... -->
          </template>
        </CrudTable>

        <CrudModal
            v-model:visible="showCrudModal"
            :title="modalTitle"
            :loading="isSubmitting"
            @save="handleSave"
        >
          <NForm :model="currentItem" :rules="validateAddUser" label-placement="left" label-width="auto">
            <NFormItem label="用户名称" path="username">
              <NInput v-model:value="currentItem.username"/>
            </NFormItem>
            <NFormItem label="邮箱" path="email">
              <NInput v-model:value="currentItem.email" clearable/>
            </NFormItem>
            <NFormItem v-if="modalType === 'create'" label="密码" path="password">
              <NInput
                  v-model:value="currentItem.password"
                  show-password-on="mousedown"
                  type="password"
                  clearable
                  placeholder="请输入密码"
              />
            </NFormItem>
            <NFormItem v-if="modalType === 'create'" label="确认密码" path="confirmPassword">
              <NInput
                  v-model:value="currentItem.confirmPassword"
                  show-password-on="mousedown"
                  type="password"
                  clearable
              />
            </NFormItem>
            <NFormItem label="用户组" path="group">
              <NSelect
                  :options="userGroupOptions"
                  v-model:value="currentItem.groups"
                  multiple
                  clearable/>
            </NFormItem>
            <NFormItem label="部门" path="dept">
              <NTreeSelect
                  v-model:value="currentItem.dept"
                  :options="departmentOptions"
                  key-field="id"
                  label-field="name"
                  placeholder="请选择部门"
                  clearable
                  default-expand-all
              ></NTreeSelect>
            </NFormItem>
            <NFormItem label="禁用" path="is_active">
              <NSwitch
                  v-model:value="currentItem.is_active"
                  :checked-value="false"
                  :unchecked-value="true"
                  :default-value="true"
              />
            </NFormItem>
          </NForm>
        </CrudModal>
        <!-- 角色 弹窗 -->
        <NModal :show="showRoleModal"
                title="角色"
                preset="card"
                size="large"
                style="width: 700px"
                @close="showRoleModal = false">
          <NSpace vertical :size="12">
            <NSpace :size="12">
              <NSelect
                  :options="roleOptions"
                  :consistent-menu-width="false"
                  v-model:value="currentItem.ids"
                  style="min-width: 150px"
                  filterable
                  clearable
                  multiple/>
              <NButton @click="handleAddRoles">添加角色</NButton>
            </NSpace>
            <NDataTable :columns="userRoleColumns" :data="currentUserRoles"/>
          </NSpace>
        </NModal>

      </CommonPage>
    </NLayoutContent>
  </NLayout>
</template>