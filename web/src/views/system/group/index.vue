<script setup>
import {h, onMounted, ref, reactive, resolveDirective, withDirectives} from 'vue';
import {
  NButton, NDrawer, NDrawerContent, NForm, NFormItem,
  NInput, NPopconfirm, NTree, NSpin, NTag
} from 'naive-ui';
import {storeToRefs} from 'pinia';
import {useUserGroupStore} from '@/stores';
import {formatDate, renderIcon} from '@/utils';
import CommonPage from '@/components/page/CommonPage.vue';
import TheIcon from '@/components/icon/TheIcon.vue';
import CrudTable from '@/components/table/CrudTable.vue';
import QueryBarItem from '@/components/query-bar/QueryBarItem.vue';
import CrudModal from '@/components/table/CrudModal.vue';

const $message = window.$message;

defineOptions({name: '用户组管理'});


const store = useUserGroupStore();
const {
  groups,
  pagination,
  isTableLoading,
  roleTreeOptions,
  checkedRoleKeys,
  isRoleTreeLoading,
} = storeToRefs(store);
const vPermission = resolveDirective('permission');


const showCrudModal = ref(false);
const modalType = ref('create');
const currentItem = ref({});
const isSubmitting = ref(false);

const showRoleDrawer = ref(false);
const currentGroupId = ref(null);
const roleFilterPattern = ref(''); // 角色树筛选文本


const queryParams = reactive({});

const loadData = (page, pageSize) => {
  store.fetchGroups({...queryParams, page, pageSize})
      .catch(error => $message.error(`加载用户组失败: ${error.message}`));
};

onMounted(() => {
  store.fetchAllRoles();
  loadData(1, 10);
});


const handlePageChange = (page) => loadData(page, pagination.value.pageSize);
const handlePageSizeChange = (pageSize) => loadData(1, pageSize);
const handleSearch = () => loadData(1, pagination.value.pageSize);
const handleResetSearch = () => {
  for (const key in queryParams) {
    delete queryParams[key]
  }
  handleSearch()
}

const handleAdd = () => {
  currentItem.value = {};
  modalType.value = 'create';
  showCrudModal.value = true;
};

const handleEdit = (row) => {
  currentItem.value = {...row};
  modalType.value = 'edit';
  showCrudModal.value = true;
};

const handleSave = async () => {
  isSubmitting.value = true;
  try {
    if (modalType.value === 'create') {
      await store.createGroup(currentItem.value);
      $message.success('创建成功');
    } else {
      await store.updateGroup(currentItem.value.uuid, currentItem.value);
      $message.success('更新成功');
    }
    showCrudModal.value = false;
  } catch (error) {
    $message.error(`保存失败: ${error.message}`);
  } finally {
    isSubmitting.value = false;
  }
};

const handleDelete = async (row) => {
  try {
    await store.deleteGroup(row.uuid);
    $message.success('删除成功');
  } catch (error) {
    $message.error(`删除失败: ${error.message}`);
  }
};


const handleSetRoles = async (row) => {
  currentGroupId.value = row.uuid;
  try {
    // 请求当前用户组的角色，Store会更新 checkedRoleKeys
    await store.fetchGroupRoles(row.uuid);
    showRoleDrawer.value = true;
  } catch (error) {
    $message.error(`获取角色信息失败: ${error.message}`);
  }
};

const handleRoleCheckChange = async (keys, option, meta) => {
  if (!currentGroupId.value) return;

  const roleId = meta.node.key;
  try {
    if (meta.action === 'check') {
      await store.assignRoleToGroup(currentGroupId.value, roleId);
    } else if (meta.action === 'uncheck') {
      await store.revokeRoleFromGroup(currentGroupId.value, roleId);
    }
  } catch (error) {
    $message.error(`操作失败: ${error.message}`);
    // 刷新一次角色列表，以回滚UI上的错误勾选
    await store.fetchGroupRoles(currentGroupId.value);
  }
};


const columns = [
  {
    title: '用户组',
    key: 'name',
    width: 80,
    align: 'center',
    ellipsis: {tooltip: true},
    render(row) {
      return h(NTag, {type: 'info'}, {default: () => row.name})
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
    width: 260,
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
                    handleEdit(row)
                  },
                },
                {
                  default: () => '编辑',
                  icon: renderIcon('material-symbols:edit-outline', {size: 16}),
                }
            ),
            [[vPermission, 'button:group_update']]
        ),
        h(NPopconfirm,
            {
              onPositiveClick: () => handleDelete(row)
            },
            {
              trigger: () => withDirectives(
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
                  [[vPermission, 'button:group_delete']]
              ),
              default: () => h('div', {}, '确定删除该用户组吗?'),
            }
        ),

        withDirectives(
            h(NButton, {
                  size: 'small',
                  type: 'primary',
                  onClick: () => handleSetRoles(row),
                },
                {default: () => '设置角色', icon: renderIcon('icon-park-outline:permissions', {size: 16})}
            ),
            [[vPermission, 'button:group_update']]
        ),
      ];
    },
  },
];
</script>

<template>
  <CommonPage show-footer title="用户组列表">
    <template #action>
      <NButton v-permission="'button:group_create'" type="primary" @click="handleAdd">
        <TheIcon icon="material-symbols:add" :size="18" class="mr-5"/>
        新建用户组
      </NButton>
    </template>

    <CrudTable
        :columns="columns"
        :data="groups"
        :loading="isTableLoading"
        :pagination="pagination"
        @update:page="handlePageChange"
        @update:page-size="handlePageSizeChange"
        @search="handleSearch"
        @reset="handleResetSearch"
    >
      <template #queryBar>
        <QueryBarItem label="用户组名" :label-width="60">
          <NInput
              v-model:value="queryParams.name"
              clearable
              type="text"
              placeholder="请输入角色名"
              @keypress.enter="handleSearch()"
          />
        </QueryBarItem>
      </template>
    </CrudTable>

    <CrudModal
        v-model:visible="showCrudModal"
        :title="modalType === 'create' ? '新建用户组' : '编辑用户组'"
        :loading="isSubmitting"
        @save="handleSave"
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
          <NInput v-model:value="currentItem.name" placeholder="请输入用户组名称"/>
        </NFormItem>
        <NFormItem label="请输入用户组描述" path="desc">
          <NInput v-model:value="currentItem.description" placeholder="请输入用户组描述"/>
        </NFormItem>
      </NForm>
    </CrudModal>

    <NDrawer v-model:show="showRoleDrawer" placement="right" :width="500">
      <NDrawerContent title="设置角色">
        <NSpace vertical :size="20">
          <NInput
              v-model:value="roleFilterPattern"
              placeholder="筛选角色"
              clearable
              size="large"
          />

          <NSpin :show="isRoleTreeLoading" description="正在加载或操作角色，请稍候..." size="medium">
            <div :style="{ minHeight: '300px', maxHeight: 'calc(100vh - 250px)', overflowY: 'auto' }">
              <NTree
                  block-line
                  checkable
                  cascade
                  :data="roleTreeOptions"
                  :pattern="roleFilterPattern"
                  key-field="uuid"
                  :show-irrelevant-nodes="false"
                  :checked-keys="checkedRoleKeys"
                  @update:checked-keys="handleRoleCheckChange"
              />
            </div>
          </NSpin>
        </NSpace>
      </NDrawerContent>
    </NDrawer>
  </CommonPage>
</template>