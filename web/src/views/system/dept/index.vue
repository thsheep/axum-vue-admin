<script setup>
import { h, onMounted, ref, resolveDirective, withDirectives } from 'vue'
import { NButton, NForm, NFormItem, NInput, NInputNumber, NPopconfirm, NTreeSelect } from 'naive-ui'

import CommonPage from '@/components/page/CommonPage.vue'
import QueryBarItem from '@/components/query-bar/QueryBarItem.vue'
import CrudModal from '@/components/table/CrudModal.vue'
import CrudTable from '@/components/table/CrudTable.vue'
import TheIcon from '@/components/icon/TheIcon.vue'

import { renderIcon } from '@/utils'

import {useDeptCrud} from '@/composables'

defineOptions({ name: '部门管理' })

const $table = ref(null)
const vPermission = resolveDirective('permission')

const deptOption = ref([])
const isDisabled = ref(false)

const deptCrud = useDeptCrud({refresh: () => $table.value?.handleSearch()});

onMounted(() => {
  $table.value?.handleSearch()
  deptCrud.loadData().then((res) => (deptOption.value = res.data))
})

const deptRules = {
  name: [
    {
      required: true,
      message: '请输入部门名称',
      trigger: ['input', 'blur', 'change'],
    },
  ],
}

async function addDepts() {
  isDisabled.value = false
  handleAdd()
}

const columns = [
  {
    title: '部门名称',
    key: 'name',
    width: 'auto',
    align: 'center',
    ellipsis: { tooltip: true },
  },
  {
    title: '备注',
    key: 'desc',
    align: 'center',
    width: 'auto',
    ellipsis: { tooltip: true },
  },
  {
    title: '操作',
    key: 'actions',
    width: 'auto',
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
              style: 'margin-left: 8px;',
              onClick: () => {
                console.log('row', row.parent_id)
                isDisabled.value = row.parent_id === 0;
                deptCrud.handleEdit(row)
              },
            },
            {
              default: () => '编辑',
              icon: renderIcon('material-symbols:edit', { size: 16 }),
            }
          ),
          [[vPermission, 'button:dept_update']]
        ),
        h(
          NPopconfirm,
          {
            onPositiveClick: () => deptCrud.handleDelete({ id: row.id }, false),
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
                    style: 'margin-left: 8px;',
                  },
                  {
                    default: () => '删除',
                    icon: renderIcon('material-symbols:delete-outline', { size: 16 }),
                  }
                ),
                [[vPermission, 'button:dept_delete']]
              ),
            default: () => h('div', {}, '确定删除该部门吗?'),
          }
        ),
      ]
    },
  },
]
</script>

<template>
  <!-- 业务页面 -->
  <CommonPage show-footer title="部门列表">
    <template #action>
      <div>
        <NButton
          v-permission="'button:dept_create'"
          class="float-right mr-15"
          type="primary"
          @click="deptCrud.handleAdd"
        >
          <TheIcon icon="material-symbols:add" :size="18" class="mr-5" />新建部门
        </NButton>
      </div>
    </template>
    <!-- 表格 -->
    <CrudTable
      ref="$table"
      :query-items="deptCrud.state.searchParams"
      :columns="columns"
      :get-data="deptCrud.loadData"
    >
      <template #queryBar>
        <QueryBarItem label="部门名称" :label-width="80">
          <NInput
            v-model:value="deptCrud.state.searchParams.name"
            clearable
            type="text"
            placeholder="请输入部门名称"
            @keypress.enter="$table?.handleSearch()"
          />
        </QueryBarItem>
      </template>
    </CrudTable>

    <!-- 新增/编辑 弹窗 -->
    <CrudModal
      v-model:visible="deptCrud.state.modalVisible"
      :title="deptCrud.state.modalTitle"
      :loading="deptCrud.state.loading"
      @save="deptCrud.handleSave"
    >
      <NForm
        ref="modalFormRef"
        label-placement="left"
        label-align="left"
        :label-width="80"
        :model="deptCrud.state.currentItem"
        :rules="deptRules"
      >
        <NFormItem label="父级部门" path="parent_id">
          <NTreeSelect
            v-model:value="deptCrud.state.currentItem.parent_id"
            :options="deptOption"
            key-field="id"
            label-field="name"
            placeholder="请选择父级部门"
            clearable
            default-expand-all
            :disabled="isDisabled"
          ></NTreeSelect>
        </NFormItem>
        <NFormItem label="部门名称" path="name">
          <NInput v-model:value="deptCrud.state.currentItem.name" clearable placeholder="请输入部门名称" />
        </NFormItem>
        <NFormItem label="备注" path="desc">
          <NInput v-model:value="deptCrud.state.currentItem.desc" type="textarea" clearable />
        </NFormItem>
        <NFormItem label="排序" path="order">
          <NInputNumber v-model:value="deptCrud.state.currentItem.order" min="0"></NInputNumber>
        </NFormItem>
      </NForm>
    </CrudModal>
  </CommonPage>
</template>
