<script setup>
import {h, onMounted, ref, reactive, resolveDirective, withDirectives, computed} from 'vue';
import {NButton, NForm, NFormItem, NInput, NInputNumber, NPopconfirm, NTreeSelect} from 'naive-ui';
import {storeToRefs} from 'pinia';
import {useDepartmentStore} from '@/stores';
import {renderIcon} from '@/utils';
import CommonPage from '@/components/page/CommonPage.vue';
import QueryBarItem from '@/components/query-bar/QueryBarItem.vue';
import CrudModal from '@/components/table/CrudModal.vue';
import CrudTable from '@/components/table/CrudTable.vue';
import TheIcon from '@/components/icon/TheIcon.vue';

const $message = window.$message;

defineOptions({name: '部门管理'});


const store = useDepartmentStore();
const {departments, isLoading, departmentOptions} = storeToRefs(store);
const vPermission = resolveDirective('permission');


const showModal = ref(false);
const modalType = ref('create');
const modalTitle = computed(() => modalType.value === 'create' ? '新建部门' : '编辑部门');
const currentItem = ref({}); // 表单数据
const isSubmitting = ref(false);
const isParentDeptDisabled = ref(false); // 控制父级部门选择器是否禁用

const queryParams = reactive({});


const loadData = () => {
  store.fetchDepartments(queryParams)
      .catch(error => $message.error(`加载部门失败: ${error.message}`));
};

onMounted(() => {
      loadData()
    }
);


const handleSearch = () => loadData();

const handleResetSearch = () => {
  for (const key in queryParams) {
    delete queryParams[key]
  }
  handleSearch()
}

const handleAdd = () => {
  currentItem.value = {order: 0};
  modalType.value = 'create';
  isParentDeptDisabled.value = false;
  showModal.value = true;
};

const handleEdit = (row) => {
  currentItem.value = {...row};
  modalType.value = 'edit';
  isParentDeptDisabled.value = !row.parent_id;
  showModal.value = true;
};

const handleSave = async () => {
  isSubmitting.value = true;
  try {
    if (modalType.value === 'create') {
      await store.createDepartment(currentItem.value);
      $message.success('创建成功');
    } else {
      await store.updateDepartment(currentItem.value.uuid, currentItem.value);
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
    await store.deleteDepartment(row.uuid);
  } catch (error) {
    console.log(error);
  }
};


const deptRules = {
  name: [{required: true, message: '请输入部门名称', trigger: ['input', 'blur']}],
};

const columns = [
  {
    title: '部门名称',
    key: 'name',
    align: 'center',
  },
  {
    title: '备注',
    key: 'desc',
    align: 'center',
  },
  {
    title: '操作',
    key: 'actions',
    align: 'center',
    render(row) {
      return [
        withDirectives(
            h(NButton, {size: 'small', type: 'primary', style: 'margin-left: 8px;', onClick: () => handleEdit(row)},
                {default: () => '编辑', icon: renderIcon('material-symbols:edit', {size: 16})}
            ),
            [[vPermission, 'button:dept_update']]
        ),
        withDirectives(
            h(NPopconfirm, {onPositiveClick: () => handleDelete(row)},
                {
                  trigger: () => h(
                      NButton,
                      {
                        size: 'small',
                        type: 'error',
                        style: 'margin-left: 8px;',
                      },
                      {
                        default: () => '删除',
                        icon: renderIcon('material-symbols:delete-outline', {size: 16}),
                      }
                  ),
                  default: () => '确定删除该部门吗?',
                }
            ),
            [[vPermission, 'button:dept_delete']]
        ),

      ];
    },
  },
];
</script>

<template>
  <CommonPage show-footer title="部门列表">
    <template #action>
      <NButton v-permission="'button:dept_create'" type="primary" @click="handleAdd">
        <TheIcon icon="material-symbols:add" :size="18" class="mr-5"/>
        新建部门
      </NButton>
    </template>

    <CrudTable
        :columns="columns"
        :data="departments"
        :loading="isLoading"
        row-key="uuid"
        :pagination="false"
        @search="handleSearch"
        @reset="handleResetSearch"
    >
      <template #queryBar>
        <QueryBarItem label="部门名称" :label-width="80">
          <NInput v-model:value="queryParams.name" @keypress.enter="handleSearch"/>
        </QueryBarItem>
      </template>
    </CrudTable>

    <CrudModal
        v-model:visible="showModal"
        :title="modalTitle"
        :loading="isSubmitting"
        @save="handleSave"
    >
      <NForm :model="currentItem" :rules="deptRules" label-placement="left" label-width="auto">
        <NFormItem label="父级部门" path="parent_id" required>
          <NTreeSelect
              v-model:value="currentItem.parent_uuid"
              :options="departmentOptions"
              key-field="id"
              label-field="name"
              placeholder="请选择父级部门"
              clearable
              default-expand-all
              :disabled="isParentDeptDisabled"
          />
        </NFormItem>
        <NFormItem label="部门名称" path="name">
          <NInput v-model:value="currentItem.name"/>
        </NFormItem>
        <NFormItem label="备注" path="desc">
          <NInput v-model:value="currentItem.desc"/>
        </NFormItem>
        <NFormItem label="排序" path="order">
          <NInputNumber v-model:value="currentItem.order" min="0"/>
        </NFormItem>
      </NForm>
    </CrudModal>
  </CommonPage>
</template>