<script setup lang="ts">
import { ref } from 'vue'
import type { HostEntry } from '../types/host'
import { useI18n } from '../i18n'

const { t } = useI18n()

const props = defineProps<{
  entry: HostEntry
  groupEnabled: boolean
  searchQuery: string
}>()

const emit = defineEmits<{
  toggle: [entryId: string, enabled: boolean]
  update: [entryId: string, ip: string, hostname: string, comment: string]
  delete: [entryId: string]
}>()

const editing = ref(false)
const editIp = ref('')
const editHostname = ref('')
const editComment = ref('')

function startEdit() {
  editIp.value = props.entry.ip
  editHostname.value = props.entry.hostname
  editComment.value = props.entry.comment
  editing.value = true
}

function saveEdit() {
  const ip = editIp.value.trim()
  const hostname = editHostname.value.trim()
  if (ip && hostname) {
    emit('update', props.entry.id, ip, hostname, editComment.value.trim())
  }
  editing.value = false
}

function cancelEdit() {
  editing.value = false
}

function escapeHtml(text: string): string {
  return text
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
}

function escapeRegExp(str: string): string {
  return str.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')
}

function highlightText(text: string, query: string): string {
  if (!query) return escapeHtml(text)
  const escaped = escapeHtml(text)
  const escapedQuery = escapeHtml(query)
  const regex = new RegExp(`(${escapeRegExp(escapedQuery)})`, 'gi')
  return escaped.replace(regex, '<mark class="highlight">$1</mark>')
}
</script>

<template>
  <tr class="entry-row" :class="{ disabled: !entry.enabled || !groupEnabled }">
    <td class="col-drag">
      <span class="drag-handle" @click.stop>&#x2630;</span>
    </td>
    <td class="col-toggle">
      <a-switch
        size="small"
        :model-value="entry.enabled"
        :disabled="!groupEnabled"
        @change="emit('toggle', entry.id, $event as boolean)"
      />
    </td>
    <template v-if="editing">
      <td class="col-ip">
        <a-input
          v-model="editIp"
          size="small"
          placeholder="IP"
          @press-enter="saveEdit"
          @keyup.escape="cancelEdit"
        />
      </td>
      <td class="col-hostname">
        <a-input
          v-model="editHostname"
          size="small"
          placeholder="Hostname"
          @press-enter="saveEdit"
          @keyup.escape="cancelEdit"
        />
      </td>
      <td class="col-comment">
        <a-input
          v-model="editComment"
          size="small"
          :placeholder="t('table.comment')"
          @press-enter="saveEdit"
          @keyup.escape="cancelEdit"
        />
      </td>
      <td class="col-actions">
        <a-space size="mini">
          <a-button size="mini" type="primary" @click="saveEdit">{{ t('table.save') }}</a-button>
          <a-button size="mini" @click="cancelEdit">{{ t('table.cancel') }}</a-button>
        </a-space>
      </td>
    </template>
    <template v-else>
      <td class="col-ip mono" v-html="highlightText(entry.ip, searchQuery)" />
      <td class="col-hostname mono" v-html="highlightText(entry.hostname, searchQuery)" />
      <td class="col-comment" v-html="highlightText(entry.comment, searchQuery)" />
      <td class="col-actions">
        <a-space size="mini" class="action-btns">
          <a-button size="mini" type="text" @click="startEdit">{{ t('table.edit') }}</a-button>
          <a-button size="mini" type="text" status="danger" @click="emit('delete', entry.id)">{{ t('table.delete') }}</a-button>
        </a-space>
      </td>
    </template>
  </tr>
</template>

<style scoped>
.entry-row {
  transition: opacity 0.15s ease;
}

.entry-row.disabled {
  opacity: 0.5;
}

.entry-row td {
  padding: 6px 8px;
  border-bottom: 1px solid var(--color-border);
  font-size: 13px;
  vertical-align: middle;
}

.col-drag {
  width: 24px;
  text-align: center;
}

.drag-handle {
  cursor: grab;
  color: var(--color-text-4);
  font-size: 10px;
  user-select: none;
  opacity: 0;
  transition: opacity 0.15s;
}

.entry-row:hover .drag-handle {
  opacity: 1;
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

.col-comment {
  color: var(--color-text-3);
  font-size: 12px;
}

.col-actions {
  width: 120px;
  text-align: right;
  white-space: nowrap;
}

.mono {
  font-family: 'Cascadia Code', 'Fira Code', monospace;
}

.action-btns {
  opacity: 0;
  transition: opacity 0.1s ease;
}

.entry-row:hover .action-btns {
  opacity: 1;
}

:deep(.highlight) {
  background: rgba(var(--warning-6), 0.3);
  color: inherit;
  padding: 0 1px;
  border-radius: 2px;
}
</style>
