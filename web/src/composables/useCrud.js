import {computed, reactive, ref} from 'vue'
import {CrudApiService, ResourceApi} from '@/api/crud-api-service'

// 创建API服务实例
const apiService = new CrudApiService()

/**
 * 通用CRUD组合式函数
 * @param {string} resourceName - 资源名称 (如: 'users', 'roles', 'menus')
 * @param {Object} options - 配置选项
 */
export function useCrud(resourceName, options = {}) {
    const {
        // 初始化选项
        autoLoad = false,           // 是否自动加载数据
        pageSize = 10,            // 分页大小

        // 自定义消息
        messages = options.messages || {
            create: `创建${resourceName}成功`,
            update: `更新${resourceName}成功`,
            delete: `删除${resourceName}成功`,
            batchDelete: `批量删除${resourceName}成功`
        },

        refresh = options.refresh || null, // 刷新函数

        // 自定义验证函数
        validateCreate = null,
        validateUpdate = null,

        // 数据转换函数
        transformData = null,

        // 自定义API实例
        customApi = null
    } = options

    // 创建资源API实例
    const api = customApi || new ResourceApi(resourceName, apiService)

    // ===== 响应式数据 =====
    const state = reactive({
        // 数据列表
        list: [],

        // 分页信息
        pagination: {
            page: 1,
            pageSize: 10,
            total: 0,
            showSizePicker: true,
            pageSizes: [10, 20, 50, 100, 200, 500],
            showQuickJumper: true,
            prefix({ itemCount }) {
                return `共 ${itemCount} 条`
            },
        },

        // 搜索参数
        searchParams: {},

        // 选中的项
        selectedIds: [],

        // 当前编辑的项
        currentItem: {},

        // 模态框状态
        modalVisible: false,
        modalType: 'create', // 'create' | 'edit' | 'view'
        modalTitle: '',

        // 加载状态
        loading: false,
        submitting: false
    })

    // ===== 计算属性 =====
    const hasSelected = computed(() => state.selectedIds.length > 0)
    const selectedCount = computed(() => state.selectedIds.length)
    const isEditing = computed(() => state.modalType === 'edit')
    const isCreating = computed(() => state.modalType === 'create')

    // ===== 数据加载 =====
    const loadData = async (params = {}) => {
        try {
            state.loading = true
            const queryParams = {
                page: state.pagination.page,
                pageSize: state.pagination.pageSize,
                ...state.searchParams,
                ...params
            }
            return await api.getAll(queryParams)

            // 根据后端返回的数据结构调整
            // if (response.data && response.pagination) {
            //     state.list = response.data
            //     state.pagination.total = response.pagination.total
            // } else if (Array.isArray(response)) {
            //     state.list = response
            // } else {
            //     state.list = response.data || []
            //     state.pagination.total = response.total || 0
            // }

        } catch (error) {
            console.error(`加载${resourceName}失败:`, error)
            state.list = []
        } finally {
            state.loading = false
        }
    }

    // ===== CRUD操作 =====

    // 新增
    const handleAdd = () => {
        state.currentItem = {}
        state.modalType = 'create'
        state.modalVisible = true
        state.modalTitle = "新建"
    }

    // 编辑
    const handleEdit = async (item) => {
        try {
            // 如果传入的是ID，先获取完整数据
            if (typeof item === 'number' || typeof item === 'string') {
                const response = await api.getById(item)
                state.currentItem = { ...response }
            } else {
                state.currentItem = { ...item }
            }
            console.log(state.currentItem)
            state.modalType = 'edit'
            state.modalVisible = true
            state.modalTitle = "编辑"
        } catch (error) {
            console.error(`获取${resourceName}详情失败:`, error)
        }
    }

    // 查看详情
    const handleView = async (item) => {
        try {
            if (typeof item === 'number' || typeof item === 'string') {
                const response = await api.getById(item)
                state.currentItem = { ...response }
            } else {
                state.currentItem = { ...item }
            }

            state.modalType = 'view'
            state.modalVisible = true
            state.modalTitle = "查看"
        } catch (error) {
            console.error(`获取${resourceName}详情失败:`, error)
        }
    }

    // 保存（新增或更新）
    const handleSave = async (formData=null) => {
        try {
            // 自定义验证
            if (isCreating.value && validateCreate) {
                const validation = await validateCreate(formData)
                if (!validation.valid) {
                    throw new Error(validation.message)
                }
            }

            if (isEditing.value && validateUpdate) {
                const validation = await validateUpdate(formData)
                if (!validation.valid) {
                    throw new Error(validation.message)
                }
            }

            if (!formData) {
                formData = { ...state.currentItem}
            }
            // 数据转换
            const dataToSave = transformData ? transformData(formData) : formData

            state.submitting = true

            let result
            if (isCreating.value) {
                result = await api.create(dataToSave, messages.create)
            } else {
                result = await api.update(state.currentItem.id, dataToSave, messages.update)
            }

            // 保存成功后的处理
            state.modalVisible = false
            if (refresh) {
                refresh() // 重新加载数据
            }
            return result
        } catch (error) {
            console.error(`保存${resourceName}失败:`, error)
            throw error
        } finally {
            state.submitting = false
        }
    }

    // 删除单个
    const handleDelete = async (item) => {
        try {
            const id = typeof item === 'object' ? item.id : item
            await api.delete(id)
            if (refresh) {
                refresh() // 重新加载数据
            }
        } catch (error) {
            if (error) { // 如果不是用户取消
                console.error(`删除${resourceName}失败:`, error)
            }
        }
    }
    // 删除单个 - 确认弹窗
    const handleDeleteConfirm = async (item) => {
        try {
            const id = typeof item === 'object' ? item.id : item
            await api.deleteWithConfirm(id, {
                title: `删除${resourceName}`,
                content: `确定要删除这个${resourceName}吗？此操作不可恢复。`,
                successMessage: messages.delete
            })
            if (refresh) {
                refresh() // 重新加载数据
            }
        } catch (error) {
            if (error) { // 如果不是用户取消
                console.error(`删除${resourceName}失败:`, error)
            }
        }
    }

    // 批量删除
    const handleBatchDelete = async () => {
        if (state.selectedIds.length === 0) {
            window.$message?.warning('请先选择要删除的项目')
            return
        }

        try {
            await api.batchDelete(state.selectedIds, messages.batchDelete)
            state.selectedIds = []
            if (refresh) {
                refresh() // 重新加载数据
            }
        } catch (error) {
            console.error(`批量删除${resourceName}失败:`, error)
        }
    }

    // ===== 关联操作 =====
    const getRelated = (id, relation, params) => {
        return api.getRelated(id, relation, params)
    }
    
    const addRelation = (id, relation, data, message) => {
        return api.addRelation(id, relation, data, message)
    }
    
    const removeRelation = (id, relation, relationId, message) => {
        return api.removeRelation(id, relation, relationId, message)
    }

    // ===== 搜索和分页 =====

    // 搜索
    const handleSearch = (searchParams = {}) => {
        state.searchParams = { ...searchParams }
        state.pagination.page = 1 // 重置到第一页
        loadData()
    }

    // 重置搜索
    const handleResetSearch = () => {
        state.searchParams = {}
        state.pagination.page = 1
        loadData()
    }

    // 分页改变
    const handlePageChange = (page) => {
        state.pagination.page = page
        loadData()
    }

    // 页面大小改变
    const handlePageSizeChange = (pageSize) => {
        state.pagination.pageSize = pageSize
        state.pagination.page = 1
        loadData()
    }

    // ===== 选择操作 =====

    // 切换单个选择
    const toggleSelection = (id) => {
        const index = state.selectedIds.indexOf(id)
        if (index > -1) {
            state.selectedIds.splice(index, 1)
        } else {
            state.selectedIds.push(id)
        }
    }

    // 全选/取消全选
    const toggleSelectAll = (checked) => {
        if (checked) {
            state.selectedIds = state.list.map(item => item.id)
        } else {
            state.selectedIds = []
        }
    }

    // 关闭模态框
    const closeModal = () => {
        state.modalVisible = false
        state.currentItem = null
    }

    // ===== 初始化 =====
    if (autoLoad) {
        loadData()
    }

    // ===== 返回的API =====
    return {
        // 状态
        state,

        // 计算属性
        hasSelected,
        selectedCount,
        isEditing,
        isCreating,

        // 数据操作
        loadData,
        handleAdd,
        handleEdit,
        handleView,
        handleSave,
        handleDelete,
        handleDeleteConfirm,
        handleBatchDelete,
        
        // 关联操作
        getRelated,
        addRelation,
        removeRelation,

        // 搜索和分页
        handleSearch,
        handleResetSearch,
        handlePageChange,
        handlePageSizeChange,

        // 选择操作
        toggleSelection,
        toggleSelectAll,

        // 模态框
        closeModal,

        // API实例（用于扩展）
        api
    }
}

/**
 * 专门的资源管理组合式函数
 * 为常用资源提供预配置的CRUD功能
 */

// 用户管理
export function useUserCrud(options={}) {
    return useCrud('users', {
        messages: {
            create: '创建用户成功',
            update: '更新用户信息成功',
            delete: '删除用户成功',
            batchDelete: '批量删除用户成功'
        },
        ...options,
    })
}

// 角色管理
export function useRoleCrud(options={}) {
    return useCrud('roles', {
        messages: {
            create: '创建角色成功',
            update: '更新角色信息成功',
            delete: '删除角色成功',
            batchDelete: '批量删除角色成功'
        },
        ...options,
    })
}


// 部门管理
export function useDeptCrud(options={}) {
    return useCrud('departments', {
        messages: {
            create: '创建部门成功',
            update: '更新部门信息成功',
            delete: '删除部门成功',
            batchDelete: '批量删除部门成功'
        },
        ...options,
    })
}

// 用户组管理
export function useUserGroupCrud(options={}) {
    return useCrud('groups', {
        messages: {
            create: '创建用户组成功',
            update: '更新用户组信息成功',
            delete: '删除用户组成功',
            batchDelete: '批量删除用户组成功'
        },
        ...options
    })
}

// 访问策略管理
export function useResourcePoliciesCrud(options={}) {
    return useCrud('cedar_policies', {
        messages: {
            create: '创建访问策略成功',
            update: '更新访问策略信息成功',
            delete: '删除访问策略成功',
            batchDelete: '批量删除访问策略成功'
        },
        ...options
    })
}

// Schema
export function useResourceSchemaCrud(options={}) {
    return useCrud('cedar_schema', {
        messages: {
            create: '创建访问策略成功',
            update: '更新访问策略信息成功',
            delete: '删除访问策略成功',
            batchDelete: '批量删除访问策略成功'
        },
        ...options
    })
}