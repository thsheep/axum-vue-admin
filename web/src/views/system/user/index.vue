<script setup>
import {h, onMounted, ref, resolveDirective, withDirectives} from 'vue'
import {
  NButton,
  NForm,
  NFormItem,
  NInput,
  NLayout,
  NLayoutContent,
  NLayoutSider,
  NPopconfirm, NPopover,
  NSpace,
  NSwitch,
  NTag,
  NTreeSelect,
} from 'naive-ui'

import CommonPage from '@/components/page/CommonPage.vue'
import QueryBarItem from '@/components/query-bar/QueryBarItem.vue'
import CrudModal from '@/components/table/CrudModal.vue'
import CrudTable from '@/components/table/CrudTable.vue'

import {formatDate, renderIcon} from '@/utils'
import {useDeptCrud, useRoleCrud, useUserCrud, useUserGroupCrud} from '@/composables'
import TheIcon from '@/components/icon/TheIcon.vue'

defineOptions({name: '用户管理'})

const $table = ref(null)
const vPermission = resolveDirective('permission')

let userCurd = useUserCrud({refresh: () => $table.value?.handleSearch()});
let roleCurd = useRoleCrud();
let userGroupCurd = useUserGroupCrud();

let deptCurd = useDeptCrud();

const deptOption = ref([]);
const roleOption = ref([]);
const userRoleInfo = ref([])
const userGroupOption = ref([]);
const showRoleModal = ref(false);
const currentUserId = ref(null);
const showPermissionModal = ref(false);
const userPermissionTreeOption = ref([]);
const userPermissionPattern = ref('')


onMounted(async () => {
  $table.value?.handleSearch()
  const [deptResponse, userGroupResponse] = await Promise.all([
    deptCurd.loadData(),
    userGroupCurd.loadData({pageSize: 999, fields: 'id,name'}),
  ]);
  deptOption.value = deptResponse.data;
  userGroupOption.value = userGroupResponse.data.map((item) => ({
    label: item.name,
    value: item.id,
  }))
})
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
    width: 80,
    align: 'center',
    fixed: 'right',
    render(row) {
      return [
        withDirectives(
            h(
                NButton,
                {
                  size: 'small',
                  type: 'primary',
                  style: 'margin-right: 8px;',
                  onClick: () => {
                    userCurd.handleEdit(row)
                    userCurd.state.currentItem.dept = row.dept?.id
                    userCurd.state.currentItem.groups = row.groups?.map((item) => item.id)
                  },
                },
                {
                  default: () => '编辑',
                  icon: renderIcon('material-symbols:edit', {size: 16}),
                }
            ),
            [[vPermission, 'button:user_update']]
        ),
        withDirectives(
            h(
                NButton,
                {
                  size: 'small',
                  type: 'primary',
                  style: 'margin-right: 8px;',
                  onClick: () => {
                    currentUserId.value = row.id
                    userCurd.getRelated(row.id, "roles").then((res) => {
                      userRoleInfo.value = res.data
                    })
                    roleCurd.loadData({pageSize: 999, fields: 'id,name'}).then((res) => {
                      roleOption.value = res.data.map((item) => ({
                        label: item.name,
                        value: item.id,
                      }))})
                    showRoleModal.value = true
                  },
                },
                {
                  default: () => '查看角色',
                  icon: renderIcon('material-symbols:visibility-outline', {size: 16}),
                }
            ),
            [[vPermission, 'button:user_view']]
        ),
        h(
            NPopconfirm,
            {
              onPositiveClick: async () => {
                await userCurd.handleDelete(row);
                await $table.value?.handleSearch();
              },
              onNegativeClick: () => {
              },
            },
            {
              trigger: () =>
                  withDirectives(
                      h(
                          NButton,
                          {
                            size: 'small',
                            type: 'error',
                            style: 'margin-right: 8px;',
                          },
                          {
                            default: () => '删除',
                            icon: renderIcon('material-symbols:delete-outline', {size: 16}),
                          }
                      ),
                      [[vPermission, 'button:user_delete']]
                  ),
              default: () => h('div', {}, '确定删除该用户吗?'),
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
    align: 'center',
    width: 150,
    fixed: 'right',
    render(row) {
      if (row.source === 'direct'){
        return [
          h(
              NPopconfirm,
              {
                onPositiveClick: async () => {
                  await userCurd.removeRelation(currentUserId.value, 'roles', row.id);
                  showRoleModal.value = false;
                },
                onNegativeClick: () => {},
              },
              {
                trigger: () =>
                    withDirectives(
                        h(
                            NButton,
                            {
                              size: 'small',
                              type: 'error',
                              style: 'margin-right: 8px;',
                            },
                            {
                              default: () => '删除',
                              icon: renderIcon('material-symbols:delete-outline', { size: 16 }),
                            }
                        ),
                        [[vPermission, 'button:user_delete']]
                    ),
                default: () => h('div', {}, '确定删除该角色吗?'),
              }
          )
        ]
      }
    }
  },
]

const nodeProps = ({option}) => {
  return {
    onClick() {
      if (userCurd.state.searchParams.dept_id === option.id) {
        delete userCurd.state.searchParams.dept_id
        $table.value?.handleSearch()
      } else {
        userCurd.state.searchParams.dept_id = option.id
        userCurd.loadData().then((res) => {
          $table.value.tableData = res.data
        })
      }
    },
  }
}

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
        if (!re.test(userCurd.state.currentItem.email)) {
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
  confirmPassword: [
    {
      required: true,
      message: '请再次输入密码',
      trigger: ['input'],
    },
    {
      trigger: ['blur'],
      validator: (rule, value, callback) => {
        if (value !== userCurd.state.currentItem.password) {
          callback('两次密码输入不一致')
          return
        }
        callback()
      },
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
}


function handleUserAddRoles(){
  if (!userCurd.state.currentItem.role_ids) {
    $message.warning('请先选择角色')
    return
  }
  userCurd.addRelation(currentUserId.value, 'roles', {ids: userCurd.state.currentItem.role_ids})
      .then(() => {
        showRoleModal.value = false
        delete userCurd.state.currentItem.role_ids
      }).catch((error) => {
        console.log(error)
        showRoleModal.value = false
  })
}

</script>

<template>
  <NLayout has-sider wh-full>
    <NLayoutSider
        bordered
        content-style="padding: 24px;"
        :collapsed-width="0"
        :width="240"
        show-trigger="arrow-circle"
    >
      <h1>部门列表</h1>
      <br/>
      <NTree
          block-line
          :data="deptOption"
          key-field="id"
          label-field="name"
          default-expand-all
          :node-props="nodeProps"
      >
      </NTree>
    </NLayoutSider>
    <NLayoutContent>
      <CommonPage show-footer title="用户列表">
        <template #action>
          <NButton v-permission="'button:user_create'" type="primary" @click="userCurd.handleAdd">
            <TheIcon icon="material-symbols:add" :size="18" class="mr-5"/>
            新建用户
          </NButton>
        </template>
        <!-- 表格 -->
        <CrudTable
            ref="$table"
            v-model:query-items="userCurd.state.searchParams"
            :columns="columns"
            :get-data="userCurd.loadData"
            :pagination="userCurd.state.pagination"
        >
          <template #queryBar>
            <QueryBarItem label="名称" :label-width="40">
              <NInput
                  v-model:value="userCurd.state.searchParams.username"
                  clearable
                  type="text"
                  placeholder="请输入用户名称"
                  @keypress.enter="$table?.handleSearch()"
              />
            </QueryBarItem>
            <QueryBarItem label="邮箱" :label-width="40">
              <NInput
                  v-model:value="userCurd.state.searchParams.email"
                  clearable
                  type="text"
                  placeholder="请输入邮箱"
                  @keypress.enter="$table?.handleSearch()"
              />
            </QueryBarItem>
          </template>
        </CrudTable>

        <!-- 新增/编辑 弹窗 -->
        <CrudModal
            v-model:visible="userCurd.state.modalVisible"
            :title="userCurd.state.modalTitle"
            :loading="userCurd.state.loading"
            @save="userCurd.handleSave"
        >
          <NForm
              ref="curd.state.currentItem"
              label-placement="left"
              label-align="left"
              :label-width="80"
              :model="userCurd.state.currentItem"
              :rules="validateAddUser"
          >
            <NFormItem label="用户名称" path="username">
              <NInput v-model:value="userCurd.state.currentItem.username" clearable/>
            </NFormItem>
            <NFormItem label="邮箱" path="email">
              <NInput v-model:value="userCurd.state.currentItem.email" clearable/>
            </NFormItem>
            <NFormItem v-if="userCurd.state.modalType === 'create'" label="密码" path="password">
              <NInput
                  v-model:value="userCurd.state.currentItem.password"
                  show-password-on="mousedown"
                  type="password"
                  clearable
                  placeholder="请输入密码"
              />
            </NFormItem>
            <NFormItem v-if="userCurd.state.modalType === 'create'" label="确认密码" path="confirmPassword">
              <NInput
                  v-model:value="userCurd.state.currentItem.confirmPassword"
                  show-password-on="mousedown"
                  type="password"
                  clearable
              />
            </NFormItem>
            <NFormItem label="用户组" path="group">
              <NSelect
                  :options="userGroupOption"
                  v-model:value="userCurd.state.currentItem.groups"
                  multiple
                  clearable/>
            </NFormItem>
            <NFormItem label="部门" path="dept">
              <NTreeSelect
                  v-model:value="userCurd.state.currentItem.dept"
                  :options="deptOption"
                  key-field="id"
                  label-field="name"
                  placeholder="请选择部门"
                  clearable
                  default-expand-all
              ></NTreeSelect>
            </NFormItem>
            <NFormItem label="禁用" path="is_active">
              <NSwitch
                  v-model:value="userCurd.state.currentItem.is_active"
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
                @close="showRoleModal = false">
          <NSpace vertical :size="12">
            <NSpace :size="12">
              <NSelect
                  :options="roleOption"
                  :consistent-menu-width="false"
                  v-model:value="userCurd.state.currentItem.role_ids"
                  style="min-width: 150px"
                  filterable
                  clearable
                  multiple/>
              <NButton @click="handleUserAddRoles">添加角色</NButton>
            </NSpace>
            <NDataTable
                :columns="userRoleColumns"
                :data="userRoleInfo"
            />
          </NSpace>
        </NModal>
        <!-- 权限 弹窗 -->
        <NModal
            :show="showPermissionModal"
            title="权限"
            preset="card"
            size="small"
            style="max-width: 500px"
            @close="showPermissionModal = false">
          <NSpace vertical :size="12">
            <NInput
                v-model:value="userPermissionPattern"
                placeholder="请输入权限名称"
                style="flex-grow: 1"
            ></NInput>
            <NTree
            :data="userPermissionTreeOption"
            :pattern="userPermissionPattern"
            />
          </NSpace>
        </NModal>
      </CommonPage>
    </NLayoutContent>
  </NLayout>
  <!-- 业务页面 -->
</template>
