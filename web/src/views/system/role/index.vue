<script setup>
import {h, onMounted, ref, reactive, resolveDirective, withDirectives, computed} from 'vue'
import {
  NButton,
  NForm,
  NFormItem,
  NInput,
  NPopconfirm,
  NTag,
} from 'naive-ui'
import {storeToRefs} from 'pinia'
import {useRoleStore} from '@/stores'
import {formatDate, renderIcon} from '@/utils'

import CommonPage from '@/components/page/CommonPage.vue'
import QueryBarItem from '@/components/query-bar/QueryBarItem.vue'
import CrudModal from '@/components/table/CrudModal.vue'
import CrudTable from '@/components/table/CrudTable.vue'
import TheIcon from '@/components/icon/TheIcon.vue'

const $message = window.$message

defineOptions({name: '角色管理'})

// =============================
const roleStore = useRoleStore()
const {roles, pagination, isTableLoading} = storeToRefs(roleStore)
const vPermission = resolveDirective('permission')

const showModal = ref(false)
const modalType = ref('create') // 'create' | 'edit'
const modalTitle = computed(() => (modalType.value === 'create' ? '新建角色' : '编辑角色'))
const currentItem = ref({}) // 用于表单 v-model 绑定
const isSubmitting = ref(false)

// 搜索参数
const queryParams = reactive({})


const loadData = () => {
  roleStore.fetchRoles({
    ...queryParams,
    page: pagination.value.page,
    pageSize: pagination.value.pageSize,
  })
      .catch(error => {
        $message.error(`加载角色失败: ${error.message}`)
      })
}

onMounted(() => {
  loadData()
});

const handleSearch = () => {
  pagination.value.page = 1
  loadData()
}

const handleResetSearch = () => {
  for (const key in queryParams) {
    delete queryParams[key]
  }
  handleSearch()
}

const handlePageChange = (page) => {
  loadData(page, pagination.value.pageSize);
}

const handlePageSizeChange = (pageSize) => {
  loadData(1, pageSize);
}

// CRUD 操作
const handleAdd = () => {
  currentItem.value = {}
  modalType.value = 'create'
  showModal.value = true
}

const handleEdit = (row) => {
  currentItem.value = {...row}
  modalType.value = 'edit'
  showModal.value = true
}

const handleSave = async () => {
  isSubmitting.value = true
  try {
    if (modalType.value === 'create') {
      await roleStore.createRole(currentItem.value)
      $message.success('创建成功')
    } else {
      await roleStore.updateRole(currentItem.value.id, currentItem.value)
      $message.success('更新成功')
    }
    showModal.value = false
  } catch (error) {
    $message.error(`保存失败: ${error.message}`)
  } finally {
    isSubmitting.value = false
  }
}

const handleDelete = async (row) => {
  try {
    await roleStore.deleteRole(row.id)
    $message.success('删除成功')
  } catch (error) {
    $message.error(`删除失败: ${error.message}`)
  }
}

const columns = [
  {
    title: '角色名',
    key: 'name',
    width: 80,
    align: 'center',
    ellipsis: {tooltip: true},
    render: (row) => h(NTag, {type: 'info'}, {default: () => row.name}),
  },
  {
    title: '角色描述',
    key: 'description',
    width: 80,
    align: 'center',
  },
  {
    title: '创建日期',
    key: 'created_at',
    width: 60,
    align: 'center',
    render: (row) => h('span', formatDate(row.created_at)),
  },
  {
    title: '操作',
    key: 'actions',
    width: 180,
    align: 'center',
    fixed: 'right',
    render(row) {
      return [
        withDirectives(
            h(NButton, {
                  size: 'small',
                  type: 'primary',
                  style: 'margin-right: 8px;',
                  onClick: () => handleEdit(row),
                },
                {default: () => '编辑', icon: renderIcon('material-symbols:edit-outline', {size: 16})}),
            [[vPermission, 'button:role_update']]
        ),
        h(NPopconfirm,
            {onPositiveClick: () => handleDelete(row)},
            {
              trigger: () => withDirectives(
                  h(NButton, {
                        size: 'small',
                        type: 'error',
                      },
                      {default: () => '删除', icon: renderIcon('material-symbols:delete-outline', {size: 16})}),
                  [[vPermission, 'button:role_delete']]
              ),
              default: () => '确定删除该角色吗?',
            }),
      ]
    },
  },
]
</script>

<template>
  <CommonPage show-footer title="角色列表">
    <template #action>
      <NButton v-permission="'button:role_create'" type="primary" @click="handleAdd">
        <TheIcon icon="material-symbols:add" :size="18" class="mr-5"/>
        新建角色
      </NButton>
    </template>

    <!-- 使用重构后的 CrudTable -->
    <CrudTable
        :columns="columns"
        :data="roles"
        :loading="isTableLoading"
        :pagination="pagination"
        @update:page="handlePageChange"
        @update:page-size="handlePageSizeChange"
        @reset="handleResetSearch"
        @search="handleSearch"
    >
      <template #queryBar>
        <QueryBarItem label="角色名" :label-width="50">
          <NInput
              v-model:value="queryParams.role_name"
              clearable
              type="text"
              placeholder="请输入角色名"
              @keypress.enter="handleSearch"
          />
        </QueryBarItem>
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
      >
        <NFormItem
            label="角色名"
            path="name"
            :rule="{ required: true, message: '请输入角色名称', trigger: ['input', 'blur'] }"
        >
          <NInput v-model:value="currentItem.name"/>
        </NFormItem>
        <NFormItem label="角色描述" path="description">
          <NInput v-model:value="currentItem.description"/>
        </NFormItem>
      </NForm>
    </CrudModal>
  </CommonPage>
</template>