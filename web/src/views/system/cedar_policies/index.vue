<script setup>
import { h, onMounted, ref, resolveDirective, withDirectives } from 'vue'
import {
  NButton,
  NForm,
  NFormItem,
  NInput,
  NPopconfirm, NSwitch,
  NTag,
} from 'naive-ui'

import CommonPage from '@/components/page/CommonPage.vue'
import QueryBarItem from '@/components/query-bar/QueryBarItem.vue'
import CrudModal from '@/components/table/CrudModal.vue'
import CrudTable from '@/components/table/CrudTable.vue'

import { formatDate, renderIcon } from '@/utils'
import { useResourcePoliciesCrud } from '@/composables'
import TheIcon from '@/components/icon/TheIcon.vue'
import {useRouter} from "vue-router";

defineOptions({ name: '策略管理' })

const $table = ref(null)
const vPermission = resolveDirective('permission')
const router = useRouter()


let resourcePoliciesCurd = useResourcePoliciesCrud({refresh: () => $table.value?.handleSearch()});

onMounted(() => {
  $table.value?.handleSearch();
  setTimeout(() => {
    window.$dialog.warning({
      title: '注意!!!',
      content: () => h('div', [
        h('p', '请慎重操作,错误的修改可能会导致权限失效！'),
      ]),
      positiveText: '我明白',
      negativeText: '不改了',
      maskClosable: false,
      closable: false,
      onPositiveClick: () => {
        console.log('我明白')
      },
      onNegativeClick: () => {
        router.push('/')
      }
    })
  }, 500)
})



const columns = [
  {
    title: '注释',
    key: 'policy_str_id',
    width: 80,
    align: 'center',
    ellipsis: { tooltip: true },
  },
  {
    title: '效果',
    key: 'effect',
    width: 80,
    align: 'center',
    ellipsis: { tooltip: true },
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
    title: '描述',
    key: 'description',
    width: 80,
    align: 'center',
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
                    resourcePoliciesCurd.handleEdit(row)
                  },
                },
                {
                  default: () => '编辑',
                  icon: renderIcon('material-symbols:edit-outline', { size: 16 }),
                }
            ),
            [[vPermission, 'button:policy_update']]
        ),
        h(
            NPopconfirm,
            {
              onPositiveClick: () => resourcePoliciesCurd.handleDelete({ id: row.id }, false),
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
                      [[vPermission, 'button:policy_delete']]
                  ),
              default: () => h('div', {}, '确定删除该权限吗?'),
            }
        )
      ]
    },
  },
]

const handelSyncResourcePoliciesEvent = () => {
  resourcePoliciesCurd.api.updateCacheAll().then((res) => {
    window.$message.success('同步成功')
  }).catch((err) => {
    window.$message.error('同步失败')
  })
}

</script>

<template>
  <CommonPage show-footer title="访问策略列表">
    <template #action>
      <NSpace>
        <NButton v-permission="'button:policy_create'" type="primary" @click="handelSyncResourcePoliciesEvent">
          <TheIcon icon="material-symbols:sync" :size="18" class="mr-5"/> 立即生效
        </NButton>
        <NButton v-permission="'button:policy_create'" type="primary" @click="resourcePoliciesCurd.handleAdd">
          <TheIcon icon="material-symbols:add" :size="18" class="mr-5" />新建策略
        </NButton>
      </NSpace>
    </template>

    <CrudTable
        ref="$table"
        v-model:query-items="resourcePoliciesCurd.state.searchParams"
        :columns="columns"
        :pagination="resourcePoliciesCurd.state.pagination"
        :get-data="resourcePoliciesCurd.loadData"
    >
      <template #queryBar>
        <NFlex>
        <QueryBarItem label="注释" :label-width="50">
          <NInput
              v-model:value="resourcePoliciesCurd.state.searchParams.name"
              clearable
              type="text"
              @keypress.enter="$table?.handleSearch()"
          />
        </QueryBarItem>
        <QueryBarItem label="效果" :label-width="50">
          <NSelect
              min-w-sm
              v-model:value="resourcePoliciesCurd.state.searchParams.effect"
              :options="[{label: '允许', value: 'permit'}, {label: '禁止', value: 'forbid'}]"
              clearable
          />
        </QueryBarItem>
        <QueryBarItem label="状态" :label-width="50">
          <NSelect
              min-w-sm
              v-model:value="resourcePoliciesCurd.state.searchParams.is_active"
              :options="[{label: '已启用', value: 1}, {label: '未启用', value: 0}]"
              clearable
          />
        </QueryBarItem>
        </NFlex>
      </template>
    </CrudTable>

    <CrudModal
        v-model:visible="resourcePoliciesCurd.state.modalVisible"
        :title="resourcePoliciesCurd.state.modalTitle"
        :loading="resourcePoliciesCurd.state.loading"
        @save="resourcePoliciesCurd.handleSave"
    >
      <NForm
          label-placement="left"
          label-align="left"
          :label-width="80"
          :model="resourcePoliciesCurd.state.currentItem"
          :disabled="resourcePoliciesCurd.state.modalType === 'view'"
      >
        <NFormItem label="策略内容" path="policy_text" required>
          <NInput
              type="textarea"
              v-model:value="resourcePoliciesCurd.state.currentItem.policy_text"
          />
        </NFormItem>
        <NFormItem label="描述" path="description" required>
          <NInput
              v-model:value="resourcePoliciesCurd.state.currentItem.description"
          />
        </NFormItem>
        <NFormItem label="状态" path="is_active">
          <NSwitch :value="resourcePoliciesCurd.state.currentItem.is_active"
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
