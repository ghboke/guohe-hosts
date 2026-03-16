<script setup lang="ts">
import { ref, watch } from 'vue'
import { Modal } from '@arco-design/web-vue'
import { useDraggable } from 'vue-draggable-plus'
import { useHosts } from '../../composables/useHosts'
import { useI18n } from '../../i18n'
import type { HostGroup } from '../../types/host'

const {
  groups,
  selectedGroupId,
  selectGroup,
  addGroup,
  deleteGroup,
  activateGroup,
  renameGroup,
  reorderGroups,
} = useHosts()

const { t } = useI18n()

const emit = defineEmits<{
  openImport: []
  openExport: []
}>()

const showAddGroupModal = ref(false)
const newGroupName = ref('')
const showRenameModal = ref(false)
const renamingGroupId = ref<string | null>(null)
const renamingName = ref('')
const groupListRef = ref<HTMLElement | null>(null)
const contextMenuVisible = ref(false)
const contextMenuGroup = ref<{ id: string; name: string } | null>(null)
const contextMenuStyle = ref({ top: '0px', left: '0px' })

// Local mutable copy for SortableJS — never let it mutate the store directly
const sortableGroups = ref<HostGroup[]>([])

watch(groups, (val) => {
  sortableGroups.value = [...val]
}, { immediate: true })

useDraggable(groupListRef, sortableGroups, {
  animation: 150,
  handle: '.drag-handle',
  ghostClass: 'sortable-ghost',
  forceFallback: true,
  onUpdate() {
    reorderGroups(sortableGroups.value.map((g) => g.id))
  },
})

function openAddGroupModal() {
  newGroupName.value = ''
  showAddGroupModal.value = true
}

function handleAddGroup() {
  const name = newGroupName.value.trim()
  if (name) {
    addGroup(name)
    showAddGroupModal.value = false
  }
}

function openRenameModal(groupId: string, currentName: string) {
  renamingGroupId.value = groupId
  renamingName.value = currentName
  showRenameModal.value = true
}

function handleRename() {
  const name = renamingName.value.trim()
  if (name && renamingGroupId.value) {
    renameGroup(renamingGroupId.value, name)
    showRenameModal.value = false
  }
}

function handleDelete(groupId: string, groupName: string) {
  Modal.confirm({
    title: t('sidebar.deleteConfirmTitle'),
    content: t('sidebar.deleteConfirm', { name: groupName }),
    okText: t('common.confirm'),
    cancelText: t('common.cancel'),
    okButtonProps: { status: 'danger' },
    onOk: () => deleteGroup(groupId),
  })
}

function handleContextMenu(action: string, group: { id: string; name: string }) {
  if (action === 'rename') {
    openRenameModal(group.id, group.name)
  } else if (action === 'delete') {
    handleDelete(group.id, group.name)
  }
}

function openContextMenu(e: MouseEvent, group: { id: string; name: string }) {
  e.preventDefault()
  contextMenuGroup.value = group
  contextMenuStyle.value = { top: e.clientY + 'px', left: e.clientX + 'px' }
  contextMenuVisible.value = true
}

function handleContextMenuAction(action: string) {
  contextMenuVisible.value = false
  if (contextMenuGroup.value) {
    handleContextMenu(action, contextMenuGroup.value)
  }
}

function closeContextMenu() {
  contextMenuVisible.value = false
}
</script>

<template>
  <aside class="sidebar">
    <div class="sidebar-header">
      <span class="sidebar-title">{{ t('sidebar.groups') }}</span>
      <a-space size="mini">
        <a-button size="mini" @click="emit('openImport')">
          {{ t('sidebar.importTitle') }}
        </a-button>
        <a-button size="mini" type="primary" @click="openAddGroupModal">+</a-button>
      </a-space>
    </div>

    <a-modal
      v-model:visible="showAddGroupModal"
      :title="t('sidebar.addGroupTitle')"
      :ok-text="t('table.add')"
      :cancel-text="t('common.cancel')"
      @ok="handleAddGroup"
    >
      <a-form :model="{}" layout="vertical">
        <a-form-item :label="t('sidebar.groupName')" required>
          <a-input
            v-model="newGroupName"
            :placeholder="t('sidebar.newGroupPlaceholder')"
            @press-enter="handleAddGroup"
          />
        </a-form-item>
      </a-form>
    </a-modal>

    <a-modal
      v-model:visible="showRenameModal"
      :title="t('sidebar.rename')"
      :ok-text="t('common.confirm')"
      :cancel-text="t('common.cancel')"
      @ok="handleRename"
    >
      <a-form :model="{}" layout="vertical">
        <a-form-item :label="t('sidebar.groupName')" required>
          <a-input
            v-model="renamingName"
            :placeholder="t('sidebar.newGroupPlaceholder')"
            @press-enter="handleRename"
          />
        </a-form-item>
      </a-form>
    </a-modal>

    <div ref="groupListRef" class="group-list">
      <div
        v-for="group in sortableGroups"
        :key="group.id"
        class="group-item"
        :class="{ active: group.id === selectedGroupId }"
        @click="selectGroup(group.id)"
        @contextmenu="openContextMenu($event, group)"
      >
        <div class="group-item-left">
          <span class="drag-handle" @click.stop>&#x2630;</span>
          <a-radio
            :model-value="group.enabled"
            @click.stop="activateGroup(group.id)"
          />
          <span class="group-name">
            {{ group.name }}
          </span>
        </div>
        <div class="group-item-right">
          <span class="entry-count">{{ group.entries.length }}</span>
        </div>
      </div>
    </div>

    <teleport to="body">
      <div v-if="contextMenuVisible" class="context-menu-overlay" @click="closeContextMenu" @contextmenu.prevent="closeContextMenu">
        <div class="context-menu" :style="contextMenuStyle" @click.stop>
          <div class="context-menu-item" @click="handleContextMenuAction('rename')">{{ t('sidebar.rename') }}</div>
          <div class="context-menu-item context-menu-danger" @click="handleContextMenuAction('delete')">{{ t('sidebar.delete') }}</div>
        </div>
      </div>
    </teleport>
  </aside>
</template>

<style scoped>
.sidebar {
  width: 220px;
  min-width: 220px;
  background-color: var(--color-bg-1);
  border-right: 1px solid var(--color-border);
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.sidebar-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 12px 8px;
}

.sidebar-title {
  font-size: 12px;
  font-weight: 600;
  text-transform: uppercase;
  color: var(--color-text-3);
  letter-spacing: 0.5px;
}

.group-list {
  overflow-y: auto;
  flex: 1;
}

.group-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 6px 12px;
  cursor: pointer;
  transition: background 0.1s ease;
}

.group-item:hover {
  background: var(--color-fill-2);
}

.group-item.active {
  background: rgb(var(--primary-6));
  color: #fff;
}

.group-item.active .entry-count {
  color: rgba(255, 255, 255, 0.7);
}

.group-item-left {
  display: flex;
  align-items: center;
  gap: 6px;
  flex: 1;
  min-width: 0;
}

.drag-handle {
  cursor: grab;
  color: var(--color-text-4);
  font-size: 10px;
  user-select: none;
  opacity: 0;
  transition: opacity 0.15s;
}

.group-item:hover .drag-handle {
  opacity: 1;
}

.group-name {
  font-size: 13px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.group-item-right {
  display: flex;
  align-items: center;
  gap: 4px;
  flex-shrink: 0;
}

.entry-count {
  font-size: 11px;
  color: var(--color-text-3);
}

.sortable-ghost {
  opacity: 0.4;
  background: var(--color-fill-3);
}
</style>

<style>
.context-menu-overlay {
  position: fixed;
  top: 0; left: 0; right: 0; bottom: 0;
  z-index: 1000;
}

.context-menu {
  position: fixed;
  min-width: 120px;
  background: var(--color-bg-popup, #fff);
  border: 1px solid var(--color-border);
  border-radius: 4px;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.12);
  padding: 4px 0;
  z-index: 1001;
}

.context-menu-item {
  padding: 6px 16px;
  font-size: 13px;
  cursor: pointer;
  transition: background 0.1s;
}

.context-menu-item:hover {
  background: var(--color-fill-2);
}

.context-menu-danger {
  color: rgb(var(--danger-6));
}
</style>
