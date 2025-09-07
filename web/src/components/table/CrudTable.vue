<template>
  <QueryBar v-if="$slots.queryBar" mb-30 @search="$emit('search')" @reset="$emit('reset')">
    <slot name="queryBar" />
  </QueryBar>

  <n-data-table
      :remote="remote"
      :loading="loading"
      :columns="columns"
      :data="data"
      :scroll-x="scrollX"
      :row-key="(row) => row[rowKey]"
      :pagination="paginationConfig"
      @update:checked-row-keys="onChecked"
      @update:page="onPageChange"
      @update:page-size="onPageSizeChange"
  />
</template>

<script setup>
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import QueryBar from "@/components/query-bar/QueryBar.vue";

const { t } = useI18n({ useScope: 'global' })

const props = defineProps({
  data: {
    type: Array,
    required: true,
  },
  loading: {
    type: Boolean,
    default: false,
  },
  pagination: {
    type: [Object, Boolean],
    default: () => ({}),
  },


  columns: {
    type: Array,
    required: true,
  },
  remote: {
    type: Boolean,
    default: true,
  },
  scrollX: {
    type: Number,
    default: 1200,
  },
  rowKey: {
    type: String,
    default: 'id',
  },
})

const emit = defineEmits([
  'update:page',
  'update:pageSize',
  'onChecked',
  'search',
  'reset',
])


const paginationConfig = computed(() => {
  if (props.pagination === false || typeof props.pagination !== 'object') {
    return false
  }
  return {
    page: props.pagination.page,
    pageSize: props.pagination.pageSize,
    itemCount: props.pagination.total,
    pageSizes: [10, 20, 50, 100],
    showSizePicker: true,
    prefix({ itemCount }) {
      return t('common.text.pagination', { itemCount })
    },
  }
})


function onPageChange(page) {
  emit('update:page', page)
}

function onPageSizeChange(pageSize) {
  emit('update:pageSize', pageSize)
}

function onChecked(rowKeys) {
  // 仅当存在选择列时才发出事件
  if (props.columns.some((item) => item.type === 'selection')) {
    emit('onChecked', rowKeys)
  }
}
</script>