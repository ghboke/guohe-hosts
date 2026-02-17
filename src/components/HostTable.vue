<script setup lang="ts">
import { ref, computed, watch, onMounted } from 'vue'
import { useDraggable } from 'vue-draggable-plus'
import { useHosts } from '../composables/useHosts'
import { useI18n } from '../i18n'
import type { HostEntry } from '../types/host'
import HostEntryRow from './HostEntryRow.vue'
import CodeMirrorEditor from './CodeMirrorEditor.vue'

const {
  selectedGroup,
  addEntry,
  updateEntry,
  deleteEntry,
  toggleEntry,
  reorderEntries,
} = useHosts()

const { t } = useI18n()

const emit = defineEmits<{
  openExport: []
}>()

const showAddModal = ref(false)
const newIp = ref('')
const newHostname = ref('')
const newComment = ref('')

const searchQuery = ref('')
const viewMode = ref<'table' | 'text'>('table')
const tbodyRef = ref<HTMLElement | null>(null)
const entries = ref<HostEntry[]>([])

// Sync entries when selected group changes (shallow copy for SortableJS)
watch(
  () => selectedGroup.value?.id,
  () => {
    const group = selectedGroup.value
    entries.value = group ? [...group.entries] : []
  },
  { immediate: true }
)

// Also sync when entries within the same group change (add/delete/toggle)
watch(
  () => selectedGroup.value?.entries,
  (newEntries) => {
    if (newEntries) {
      entries.value = [...newEntries]
    }
  }
)

let draggable: ReturnType<typeof useDraggable> | null = null

onMounted(() => {
  draggable = useDraggable(tbodyRef, entries, {
    immediate: false,
    animation: 150,
    handle: '.drag-handle',
    ghostClass: 'sortable-ghost',
    forceFallback: true,
    onUpdate() {
      if (selectedGroup.value) {
        reorderEntries(selectedGroup.value.id, entries.value.map((e) => e.id))
      }
    },
  })
})

// tbodyRef is behind v-if, re-init Sortable whenever the element appears
watch(tbodyRef, (el) => {
  if (el && draggable) {
    draggable.start()
  }
})

const filteredEntries = computed(() => {
  const q = searchQuery.value.toLowerCase()
  if (!q) return entries.value
  return entries.value.filter(
    (e) =>
      e.ip.toLowerCase().includes(q) ||
      e.hostname.toLowerCase().includes(q) ||
      e.comment.toLowerCase().includes(q)
  )
})

function openAddModal() {
  newIp.value = ''
  newHostname.value = ''
  newComment.value = ''
  showAddModal.value = true
}

function handleAdd() {
  const ip = newIp.value.trim()
  const hostname = newHostname.value.trim()
  if (!ip || !hostname || !selectedGroup.value) return
  addEntry(selectedGroup.value.id, ip, hostname, newComment.value.trim())
  showAddModal.value = false
}

function handleUpdate(entryId: string, ip: string, hostname: string, comment: string) {
  if (!selectedGroup.value) return
  updateEntry(selectedGroup.value.id, entryId, ip, hostname, comment)
}

function handleDelete(entryId: string) {
  if (!selectedGroup.value) return
  deleteEntry(selectedGroup.value.id, entryId)
}

function handleToggle(entryId: string, enabled: boolean) {
  if (!selectedGroup.value) return
  toggleEntry(selectedGroup.value.id, entryId, enabled)
}
</script>

<template>
  <div v-if="selectedGroup" class="host-table-container">
    <div class="table-toolbar">
      <div class="toolbar-left">
        <h2 class="group-title">{{ selectedGroup.name }}</h2>
        <a-tag size="small" color="arcoblue">
          {{ t('table.entries', { count: selectedGroup.entries.length }) }}
        </a-tag>
        <a-radio-group v-model="viewMode" size="small" type="button">
          <a-radio value="table">{{ t('table.tableView') }}</a-radio>
          <a-radio value="text">{{ t('table.textView') }}</a-radio>
        </a-radio-group>
      </div>
      <div class="toolbar-right">
        <template v-if="viewMode === 'table'">
          <a-input-search
            v-model="searchQuery"
            size="small"
            :placeholder="t('table.search')"
            style="width: 200px"
          />
          <a-button size="small" @click="emit('openExport')">
            {{ t('sidebar.exportTitle') }}
          </a-button>
          <a-button
            size="small"
            type="primary"
            @click="openAddModal"
          >
            {{ t('table.add') }}
          </a-button>
        </template>
      </div>
    </div>

    <a-modal
      v-model:visible="showAddModal"
      :title="t('table.addEntryTitle')"
      :ok-text="t('table.add')"
      :cancel-text="t('common.cancel')"
      @ok="handleAdd"
    >
      <a-form :model="{}" layout="vertical">
        <a-form-item :label="t('table.ipAddress')" required>
          <a-input v-model="newIp" :placeholder="t('table.ipPlaceholder')" @press-enter="handleAdd" />
        </a-form-item>
        <a-form-item :label="t('table.hostname')" required>
          <a-input v-model="newHostname" :placeholder="t('table.hostnamePlaceholder')" @press-enter="handleAdd" />
        </a-form-item>
        <a-form-item :label="t('table.comment')">
          <a-input v-model="newComment" :placeholder="t('table.commentPlaceholder')" @press-enter="handleAdd" />
        </a-form-item>
      </a-form>
    </a-modal>

    <!-- Table view -->
    <template v-if="viewMode === 'table'">
      <div class="table-wrapper">
        <table class="host-table">
          <thead>
            <tr>
              <th class="col-drag"></th>
              <th class="col-toggle">{{ t('table.on') }}</th>
              <th class="col-ip">{{ t('table.ipAddress') }}</th>
              <th class="col-hostname">{{ t('table.hostname') }}</th>
              <th class="col-comment">{{ t('table.comment') }}</th>
              <th class="col-actions">{{ t('table.actions') }}</th>
            </tr>
          </thead>
          <tbody v-if="!searchQuery" ref="tbodyRef">
            <HostEntryRow
              v-for="entry in entries"
              :key="entry.id"
              :entry="entry"
              :group-enabled="selectedGroup.enabled"
              :search-query="searchQuery"
              @toggle="handleToggle"
              @update="handleUpdate"
              @delete="handleDelete"
            />
          </tbody>
          <tbody v-else>
            <HostEntryRow
              v-for="entry in filteredEntries"
              :key="entry.id"
              :entry="entry"
              :group-enabled="selectedGroup.enabled"
              :search-query="searchQuery"
              @toggle="handleToggle"
              @update="handleUpdate"
              @delete="handleDelete"
            />
          </tbody>
        </table>
        <a-empty v-if="filteredEntries.length === 0" style="padding: 40px 0">
          <template #description>
            {{ searchQuery ? t('table.noMatch') : t('table.noEntries') }}
          </template>
        </a-empty>
      </div>
    </template>

    <!-- Text editor view -->
    <template v-else>
      <CodeMirrorEditor />
    </template>
  </div>
  <div v-else class="no-selection">
    <a-empty>
      <template #description>{{ t('table.selectGroup') }}</template>
    </a-empty>
  </div>
</template>

<style scoped>
.host-table-container {
  display: flex;
  flex-direction: column;
  height: 100%;
}

.table-toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 12px;
  flex-shrink: 0;
}

.toolbar-left {
  display: flex;
  align-items: center;
  gap: 8px;
}

.group-title {
  font-size: 16px;
  font-weight: 600;
}

.toolbar-right {
  display: flex;
  align-items: center;
  gap: 8px;
}

.table-wrapper {
  flex: 1;
  overflow: auto;
}

.host-table {
  width: 100%;
  border-collapse: collapse;
  table-layout: auto;
}

.host-table thead th {
  padding: 6px 8px;
  font-size: 11px;
  font-weight: 600;
  text-transform: uppercase;
  color: var(--color-text-3);
  border-bottom: 2px solid var(--color-border);
  text-align: left;
  position: sticky;
  top: 0;
  background: var(--color-bg-1);
  z-index: 1;
}

.col-drag {
  width: 24px;
}

.col-toggle {
  width: 60px;
  text-align: center;
}

.col-ip {
  width: 160px;
}

.col-hostname {
  min-width: 200px;
}

.col-actions {
  width: 120px;
  text-align: right;
}

.no-selection {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 100%;
}

:deep(.sortable-ghost) {
  opacity: 0.4;
  background: var(--color-fill-3);
}
</style>
