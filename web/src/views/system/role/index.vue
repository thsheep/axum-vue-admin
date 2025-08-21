<script setup>
import {h, onMounted, ref, resolveDirective, withDirectives} from 'vue'
import {
  NButton,
  NForm,
  NFormItem,
  NInput,
  NPopconfirm,
  NTag,
  NTree,
  NDrawer,
  NDrawerContent,
  NTabs,
  NTabPane,
  NGrid,
  NGi,
} from 'naive-ui'

import CommonPage from '@/components/page/CommonPage.vue'
import QueryBarItem from '@/components/query-bar/QueryBarItem.vue'
import CrudModal from '@/components/table/CrudModal.vue'
import CrudTable from '@/components/table/CrudTable.vue'

import {formatDate, renderIcon} from '@/utils'
import {useRoleCrud } from '@/composables'
import TheIcon from '@/components/icon/TheIcon.vue'

defineOptions({name: '角色管理'})

const $table = ref(null)
const vPermission = resolveDirective('permission')

const roleCrud = useRoleCrud({refresh: () => $table.value?.handleSearch()})

const pattern = ref('')
const active = ref(false)
const role_id = ref(0)
const permissionSlugs = ref([])
const refPermissionTree = ref([])
const permissionTree = ref([])
const roleNameOptions = ref([])


onMounted(() => {
  $table.value?.handleSearch()
  roleCrud.loadData({pageSize: 999, fields: 'id,name'}).then((response) => {
    roleNameOptions.value = response.data.map(item => {
      return {
        label: item.name,
        value: item.id
      }
    })
  })
})


function buildPermissionTree(data) {
  const processedData = []
  const groupedData = {}

  data.forEach((item) => {
    const module = item['module']
    const slug = item['slug']
    if (!(module in groupedData)) {
      groupedData[module] = {key: module, label: module, children: []}
    }

    groupedData[module].children.push({
      id: item['id'],
      label: item['name'],
      key: slug,
    })
  })
  processedData.push(...Object.values(groupedData))
  return processedData
}


const columns = [
  {
    title: '角色名',
    key: 'name',
    width: 80,
    align: 'center',
    ellipsis: {tooltip: true},
    render(row) {
      return h(NTag, {type: 'info'}, {default: () => row.name})
    },
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
                    roleCrud.handleEdit(row)
                  },
                },
                {
                  default: () => '编辑',
                  icon: renderIcon('material-symbols:edit-outline', {size: 16}),
                }
            ),
            [[vPermission, 'button:role_update']]
        ),
        h(
            NPopconfirm,
            {
              onPositiveClick: () => roleCrud.handleDelete({id: row.id}, false),
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
                      [[vPermission, 'button:role_delete']]
                  ),
              default: () => h('div', {}, '确定删除该角色吗?'),
            }
        ),
      ]
    },
  },
]
</script>

<template>
  <CommonPage show-footer title="角色列表">
    <template #action>
      <NButton v-permission="'button:role_create'" type="primary" @click="roleCrud.handleAdd">
        <TheIcon icon="material-symbols:add" :size="18" class="mr-5"/>
        新建角色
      </NButton>
    </template>

    <CrudTable
        ref="$table"
        :query-items="roleCrud.state.searchParams"
        :columns="columns"
        :get-data="roleCrud.loadData"
        :pagination="roleCrud.state.pagination"
    >
      <template #queryBar>
        <QueryBarItem label="角色名" :label-width="50">
          <NInput
              v-model:value="roleCrud.state.searchParams.role_name"
              clearable
              type="text"
              placeholder="请输入角色名"
              @keypress.enter="$table?.handleSearch()"
          />
        </QueryBarItem>
      </template>
    </CrudTable>

    <CrudModal
        v-model:visible="roleCrud.state.modalVisible"
        :title="roleCrud.state.modalTitle"
        :loading="roleCrud.state.loading"
        @save="roleCrud.handleSave"
    >
      <NForm
          ref="modalFormRef"
          label-placement="left"
          label-align="left"
          :label-width="80"
          :model="roleCrud.state.currentItem"
          :disabled="roleCrud.state.modalType === 'view'"
      >
        <NFormItem
            label="角色名"
            path="name"
            :rule="{
            required: true,
            message: '请输入角色名称',
            trigger: ['input', 'blur'],
          }"
        >
          <NInput v-model:value="roleCrud.state.currentItem.name"/>
        </NFormItem>
        <NFormItem label="角色描述" path="description">
          <NInput v-model:value="roleCrud.state.currentItem.description"/>
        </NFormItem>
      </NForm>
    </CrudModal>

  </CommonPage>
</template>
