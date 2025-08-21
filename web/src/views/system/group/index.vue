<script setup>
import {useRoleCrud, useUserGroupCrud} from "@/composables";
import {h, onMounted, ref, resolveDirective, withDirectives} from "vue";
import {
  NButton, NDrawer, NDrawerContent,
  NForm,
  NFormItem, NGi, NGrid,
  NInput,
  NPopconfirm, NTabPane, NTabs,
  NTag, NTree,
} from "naive-ui";
import {formatDate, renderIcon} from "@/utils";
import CommonPage from "@/components/page/CommonPage.vue";
import TheIcon from "@/components/icon/TheIcon.vue";
import CrudTable from "@/components/table/CrudTable.vue";
import QueryBarItem from "@/components/query-bar/QueryBarItem.vue";
import CrudModal from "@/components/table/CrudModal.vue";

const $table = ref(null)
const userGroupCrud = useUserGroupCrud({refresh: () => $table.value?.handleSearch()});
const roleCrud = useRoleCrud();

const vPermission = resolveDirective('permission')
const currentGroupId = ref(null)
const refGroupRoleTree = ref(null)
const roleOptions = ref([])
const checkedGroupRoleKeys = ref([])
const showSetRoleModal = ref(false)
const pattern = ref('')

onMounted(() => {
  $table.value?.handleSearch()
})

const columns = [
  {
    title: '用户组',
    key: 'name',
    width: 80,
    align: 'center',
    ellipsis: { tooltip: true },
    render(row) {
      return h(NTag, { type: 'info' }, { default: () => row.name })
    },
  },
  {
    title: '用户组描述',
    key: 'description',
    width: 80,
    align: 'center',
  },
  {
    title: '创建日期',
    key: 'created_at',
    width: 60,
    align: 'center',
    render(row) {
      return h('span', formatDate(row.created_at))
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
                    userGroupCrud.handleEdit(row)
                  },
                },
                {
                  default: () => '编辑',
                  icon: renderIcon('material-symbols:edit-outline', { size: 16 }),
                }
            ),
            [[vPermission, 'button:group_update']]
        ),
        h(
            NPopconfirm,
            {
              onPositiveClick: () => userGroupCrud.handleDelete({ id: row.id }),
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
                      [[vPermission, 'button:group_delete']]
                  ),
              default: () => h('div', {}, '确定删除该用户组吗?'),
            }
        ),
        withDirectives(
            h(
                NButton,
                {
                  size: 'small',
                  type: 'primary',
                  onClick: async () => {
                    currentGroupId.value = row.id
                    try {
                      await Promise.all([
                        roleCrud.loadData({pageSize: 999, fields: 'id,name'}),
                        userGroupCrud.getRelated( row.id, 'roles')
                      ]).then(([roleResponse, groupRoleResponse]) => {
                        roleOptions.value = roleResponse.data.map((item) => {
                          return {
                            label: item.name,
                            key: item.id,
                          }
                        })
                        checkedGroupRoleKeys.value = groupRoleResponse.data.map(item => item.id)
                      })

                      showSetRoleModal.value = true
                    } catch (error) {
                      // 错误处理
                      console.error('Error loading data:', error)
                    }
                  },
                },
                {
                  default: () => '设置角色',
                  icon: renderIcon('icon-park-outline:permissions', {size: 16}),
                }
            ),
            [[vPermission, 'button:group_update']]
        ),
      ]
    },
  },
]

const handleAssignAndRevokeRoleShow = ref(false)

function handleAssignAndRevokeRole(keys, option, meta) {
  if (meta.action === 'check') {
    handleAssignAndRevokeRoleShow.value = true
    userGroupCrud.addRelation(currentGroupId.value, 'roles', {role_id: meta.node.key})
        .then(() => {
          showSetRoleModal.value = false
          handleAssignAndRevokeRoleShow.value = false
        })
        .catch(() => {
          showSetRoleModal.value = false
          handleAssignAndRevokeRoleShow.value = false
        })
  }else if (meta.action === 'uncheck') {
    handleAssignAndRevokeRoleShow.value = true
    console.log(`取消勾选 Key: ${meta.node.key}`);
    userGroupCrud.removeRelation(currentGroupId.value, 'roles', meta.node.key)
        .then(() => {
          showSetRoleModal.value = false
          handleAssignAndRevokeRoleShow.value = false
        })
      .catch(() => {
        showSetRoleModal.value = false
        handleAssignAndRevokeRoleShow.value = false
      })
  } else {
    console.log(".....unKnow Action")
  }
}

</script>


<template>
  <CommonPage show-footer title="用户组列表">
    <template #action>
      <NButton v-permission="'button:group_create'" type="primary" @click="userGroupCrud.handleAdd">
        <TheIcon icon="material-symbols:add" :size="18" class="mr-5" />新建用户组
      </NButton>
    </template>

    <CrudTable
        ref="$table"
        v-model:query-items="userGroupCrud.state.searchParams"
        :columns="columns"
        :get-data="userGroupCrud.loadData"
        :pagination="userGroupCrud.state.pagination"
    >
      <template #queryBar>
        <QueryBarItem label="用户组名" :label-width="60">
          <NInput
              v-model:value="userGroupCrud.state.searchParams.name"
              clearable
              type="text"
              placeholder="请输入角色名"
              @keypress.enter="$table?.handleSearch()"
          />
        </QueryBarItem>
      </template>
    </CrudTable>

    <CrudModal
        v-model:visible="userGroupCrud.state.modalVisible"
        :title="userGroupCrud.state.modalTitle"
        :loading="userGroupCrud.state.loading"
        @save="userGroupCrud.handleSave"
    >
      <NForm
          ref="modalFormRef"
          label-placement="left"
          label-align="left"
          :label-width="80"
          :model="userGroupCrud.state.currentItem"
          :disabled="userGroupCrud.state.modalType === 'view'"
      >
        <NFormItem
            label="角色名"
            path="name"
            :rule="{
            required: true,
            message: '请输入用户组名称',
            trigger: ['input', 'blur'],
          }"
        >
          <NInput v-model:value="userGroupCrud.state.currentItem.name" placeholder="请输入用户组名称" />
        </NFormItem>
        <NFormItem label="角色描述" path="desc">
          <NInput v-model:value="userGroupCrud.state.currentItem.description" placeholder="请输入用户组描述" />
        </NFormItem>
      </NForm>
    </CrudModal>

    <NDrawer v-model:show="showSetRoleModal" placement="right" :width="500"
    >
      <NDrawerContent>
        <NGrid x-gap="16" cols="12">
          <NGi span="8">
            <NInput
                v-model:value="pattern"
                type="text"
                placeholder="筛选"
                style="flex-grow: 1"
            ></NInput>
          </NGi>
        </NGrid>
        <NTabs>
          <NTabPane name="role" tab="角色" display-directive="show">
            <NSpin :show="handleAssignAndRevokeRoleShow">
              <NTree
                  ref="refGroupRoleTree"
                  :data="roleOptions"
                  :pattern="pattern"
                  :showIrrelevantNodes="false"
                  :default-checked-keys="checkedGroupRoleKeys"
                  @update:checked-keys="handleAssignAndRevokeRole"
                  multiple
                  checkable
              />
              <template #description>
                请稍等.....
              </template>
            </NSpin>
          </NTabPane>
        </NTabs>
        <template #header> 设置角色</template>
      </NDrawerContent>
    </NDrawer>
  </CommonPage>
</template>

<style scoped lang="scss">

</style>