<script setup>
import { h, onMounted, ref, resolveDirective, withDirectives } from 'vue'
import {
  NButton,
  NForm,
  NFormItem,
  NInput
} from 'naive-ui'

import { useRouter } from 'vue-router'

import CommonPage from '@/components/page/CommonPage.vue'
import CrudModal from '@/components/table/CrudModal.vue'
import CrudTable from '@/components/table/CrudTable.vue'

import { renderIcon } from '@/utils'
import { useResourceSchemaCrud } from '@/composables'

defineOptions({ name: 'Schema管理' })

const $table = ref(null)
const vPermission = resolveDirective('permission')
const router = useRouter()


let resourceSchemaCurd = useResourceSchemaCrud({refresh: () => $table.value?.handleSearch()});

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
                    resourceSchemaCurd.handleEdit(row)
                  },
                },
                {
                  default: () => '编辑',
                  icon: renderIcon('material-symbols:edit-outline', { size: 16 }),
                }
            ),
            [[vPermission, 'button:policy_update']]
        )
      ]
    },
  },
]

</script>

<template>
  <CommonPage show-footer title="Schema列表">

    <CrudTable
        ref="$table"
        v-model:query-items="resourceSchemaCurd.state.searchParams"
        :columns="columns"
        :pagination="resourceSchemaCurd.state.pagination"
        :get-data="resourceSchemaCurd.loadData"
    >
    </CrudTable>

    <CrudModal
        v-model:visible="resourceSchemaCurd.state.modalVisible"
        :title="resourceSchemaCurd.state.modalTitle"
        :loading="resourceSchemaCurd.state.loading"
        @save="resourceSchemaCurd.handleSave"
    >
      <NForm
          label-placement="left"
          label-align="left"
          :label-width="80"
          :model="resourceSchemaCurd.state.currentItem"
          :disabled="resourceSchemaCurd.state.modalType === 'view'"
      >
        <NFormItem label="Schema" path="schema" required>
          <NInput
              type="textarea"
              v-model:value="resourceSchemaCurd.state.currentItem.schema"
          />
        </NFormItem>
        <NFormItem label="描述" path="description" required>
          <NInput
              v-model:value="resourceSchemaCurd.state.currentItem.description"
          />
        </NFormItem>
      </NForm>
    </CrudModal>
  </CommonPage>
</template>
