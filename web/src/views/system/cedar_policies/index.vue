<script setup>
import {h, onMounted, ref, reactive, resolveDirective, withDirectives, computed} from 'vue';
import {
  NButton, NFlex, NForm, NFormItem, NInput, NPopconfirm, NSwitch, NTag, NSelect
} from 'naive-ui';
import {storeToRefs} from 'pinia';
import {usePolicyStore} from '@/stores';
import {useRouter} from 'vue-router';
import {renderIcon} from '@/utils';
import CommonPage from '@/components/page/CommonPage.vue';
import QueryBarItem from '@/components/query-bar/QueryBarItem.vue';
import CrudModal from '@/components/table/CrudModal.vue';
import CrudTable from '@/components/table/CrudTable.vue';
import TheIcon from '@/components/icon/TheIcon.vue';

const $message = window.$message;
const $dialog = window.$dialog;

defineOptions({name: '策略管理'});


const store = usePolicyStore();
const {policies, pagination, isTableLoading, isSyncing} = storeToRefs(store);
const router = useRouter();
const vPermission = resolveDirective('permission');


const showModal = ref(false);
const modalType = ref('create');
const modalTitle = computed(() => (modalType.value === 'create' ? '新建策略' : '编辑策略'));
const currentItem = ref({});
const isSubmitting = ref(false);

const queryParams = reactive({});


onMounted(() => {
  setTimeout(() => {
    $dialog.warning({
      title: '注意!!!',
      content: '请慎重操作,错误的修改可能会导致权限失效！',
      positiveText: '我明白',
      negativeText: '不改了',
      maskClosable: false,
      closable: false,
      onNegativeClick: () => router.push('/'),
    });
  }, 500);
  loadData(1, 10)
});


const loadData = (page, pageSize) => {
  store.fetchPolicies({...queryParams, page, pageSize})
      .catch(error => $message.error(`加载策略失败: ${error.message}`));
};

const handlePageChange = (page) => loadData(page, pagination.value.pageSize);
const handlePageSizeChange = (pageSize) => loadData(1, pageSize);
const handleSearch = () => loadData(1, pagination.value.pageSize);
const handleResetSearch = () => {
  for (const key in queryParams) {
    delete queryParams[key]
  }
  handleSearch()
}

// CRUD 处理器
const handleAdd = () => {
  currentItem.value = {is_active: true};
  modalType.value = 'create';
  showModal.value = true;
};

const handleEdit = (row) => {
  currentItem.value = {...row};
  modalType.value = 'edit';
  showModal.value = true;
};

const handleSave = async () => {
  isSubmitting.value = true;
  try {
    if (modalType.value === 'create') {
      await store.createPolicy(currentItem.value);
      $message.success('创建成功');
    } else {
      await store.updatePolicy(currentItem.value.id, currentItem.value);
      $message.success('更新成功');
    }
    showModal.value = false;
  } catch (error) {
    $message.error(`保存失败: ${error.message}`);
  } finally {
    isSubmitting.value = false;
  }
};

const handleDelete = async (row) => {
  try {
    await store.deletePolicy(row.id);
    $message.success('删除成功');
  } catch (error) {
    $message.error(`删除失败: ${error.message}`);
  }
};

// 特殊操作处理器
const handleSyncPolicies = async () => {
  try {
    await store.syncPolicies();
    $message.success('同步成功，策略已生效');
  } catch (error) {
    $message.error(`同步失败: ${error.message}`);
  }
};


const columns = [
  {
    title: '注释',
    key: 'policy_str_id',
    width: 450,
    align: 'center',
    ellipsis: {tooltip: true},
  },
  {
    title: '效果',
    key: 'effect',
    width: 80,
    align: 'center',
    ellipsis: {tooltip: true},
  },
  {
    title: '状态',
    key: 'is_active',
    width: 100,
    align: 'center',
    render(row) {
      return h(NSwitch,
          {defaultValue: row.is_active, disabled: true},
          {
            checked: () => '启用',
            unchecked: () => '禁用'
          })
    },
  },
  {
    title: '描述',
    key: 'description',
    width: 150,
    align: 'center',
  },
  {
    title: '操作',
    key: 'actions',
    width: 350,
    align: 'center',
    render(row) {
      return [
        withDirectives(
            h(NButton,
                {
                  size: 'small',
                  type: 'primary',
                  style: 'margin-right: 8px;',
                  onClick: () => handleEdit(row)
                },
                {
                  default: () => '编辑',
                  icon: renderIcon('material-symbols:edit-outline', {size: 16}),
                }
            ),
            [[vPermission, 'button:policy_update']]
        ),
        h(NPopconfirm,
                {onPositiveClick: () => handleDelete(row)},
                {
                  trigger: () => withDirectives( h(
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
                      [[vPermission, 'button:policy_delete']]
                  ),
                  default: () => h('div', {}, '确定删除该权限吗?')
                }
            ),
      ];
    },
  },
];
</script>

<template>
  <CommonPage show-footer title="访问策略列表">
    <template #action>
      <NFlex>
        <NButton v-permission="'button:policy_create'" type="primary" @click="handleSyncPolicies" :loading="isSyncing">
          <TheIcon icon="material-symbols:sync" :size="18" class="mr-5"/>
          立即生效
        </NButton>
        <NButton v-permission="'button:policy_create'" type="primary" @click="handleAdd">
          <TheIcon icon="material-symbols:add" :size="18" class="mr-5"/>
          新建策略
        </NButton>
      </NFlex>
    </template>

    <CrudTable
        :columns="columns"
        :data="policies"
        :loading="isTableLoading"
        :pagination="pagination"
        @update:page="handlePageChange"
        @update:page-size="handlePageSizeChange"
        @search="handleSearch"
        @reset="handleResetSearch"
    >
      <template #queryBar>
        <NFlex>
          <NFlex>
            <QueryBarItem label="注释" :label-width="50">
              <NInput
                  v-model:value="queryParams.name"
                  clearable
                  type="text"
                  @keypress.enter="handleSearch()"
              />
            </QueryBarItem>
            <QueryBarItem label="效果" :label-width="50">
              <NSelect
                  min-w-sm
                  v-model:value="queryParams.effect"
                  :options="[{label: '允许', value: 'permit'}, {label: '禁止', value: 'forbid'}]"
                  clearable
              />
            </QueryBarItem>
            <QueryBarItem label="状态" :label-width="50">
              <NSelect
                  min-w-sm
                  v-model:value="queryParams.is_active"
                  :options="[{label: '已启用', value: 1}, {label: '未启用', value: 0}]"
                  clearable
              />
            </QueryBarItem>
          </NFlex>
        </NFlex>
      </template>
    </CrudTable>

    <CrudModal
        v-model:visible="showModal"
        :title="modalTitle"
        :loading="isSubmitting"
        @save="handleSave"
    >
      <NForm
          label-placement="left"
          label-align="left"
          :label-width="80"
          :model="currentItem"
          :disabled="modalType === 'view'"
      >
        <NFormItem label="策略内容" path="policy_text" required>
          <NInput
              type="textarea"
              v-model:value="currentItem.policy_text"
          />
        </NFormItem>
        <NFormItem label="描述" path="description" required>
          <NInput
              v-model:value="currentItem.description"
          />
        </NFormItem>
        <NFormItem label="状态" path="is_active">
          <NSwitch :value="currentItem.is_active"
                   :default-value="true">
            <template #checked>
              启用
            </template>
            <template #unchecked>
              禁用
            </template>
          </NSwitch>
        </NFormItem>
      </NForm>
    </CrudModal>
  </CommonPage>
</template>

  