<script setup>
import { h, onMounted, ref, resolveDirective, withDirectives } from 'vue';
import { NButton, NForm, NFormItem, NInput } from 'naive-ui';
import { useRouter } from 'vue-router';
import { storeToRefs } from 'pinia';
import { useSchemaStore } from '@/stores';
import { renderIcon } from '@/utils';
import CommonPage from '@/components/page/CommonPage.vue';
import CrudModal from '@/components/table/CrudModal.vue';
import CrudTable from '@/components/table/CrudTable.vue';

const $message = window.$message;
const $dialog = window.$dialog;

defineOptions({ name: 'Schema管理' });


const store = useSchemaStore();
const { schemas, isLoading } = storeToRefs(store);
const router = useRouter();
const vPermission = resolveDirective('permission');


const showModal = ref(false);
const modalTitle = ref('编辑Schema');
const currentItem = ref({});
const isSubmitting = ref(false);


onMounted(() => {
  store.fetchSchemas().catch(error => {
    $message.error(`加载Schema失败: ${error.message}`);
  });


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
});


const handleEdit = (row) => {
  currentItem.value = { ...row };
  showModal.value = true;
};

const handleSave = async () => {
  isSubmitting.value = true;
  try {
    await store.updateSchema(currentItem.value.id, currentItem.value);
    $message.success('更新成功');
    showModal.value = false;
  } catch (error) {
    $message.error(`更新失败: ${error.message}`);
  } finally {
    isSubmitting.value = false;
  }
};


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
                  onClick: () => handleEdit(row),
                },
                {
                  default: () => '编辑',
                  icon: renderIcon('material-symbols:edit-outline', { size: 16 }),
                }
            ),
            [[vPermission, 'button:policy_update']]
        ),
      ];
    },
  },
];
</script>

<template>
  <CommonPage show-footer title="Schema列表">
    <CrudTable
        :columns="columns"
        :data="schemas"
        :loading="isLoading"
        :pagination="false"
    />

    <CrudModal
        v-model:visible="showModal"
        :title="modalTitle"
        :loading="isSubmitting"
        @save="handleSave"
    >
      <NForm :model="currentItem">
        <NFormItem label="Schema" path="schema" required>
          <NInput
              type="textarea"
              v-model:value="currentItem.schema"
              :autosize="{ minRows: 10, maxRows: 20 }"
          />
        </NFormItem>
        <NFormItem label="描述" path="description" required>
          <NInput v-model:value="currentItem.description" />
        </NFormItem>
      </NForm>
    </CrudModal>
  </CommonPage>
</template>