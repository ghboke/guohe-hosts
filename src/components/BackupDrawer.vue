<script setup lang="ts">
import { ref, watch } from 'vue'
import { IconFile } from '@arco-design/web-vue/es/icon'
import { useHosts } from '../composables/useHosts'
import { useI18n } from '../i18n'
import type { BackupInfo } from '../types/host'

const { listBackups, createBackup, restoreBackup, deleteBackup, loadHosts } = useHosts()
const { t } = useI18n()

const props = defineProps<{
  visible: boolean
}>()

const emit = defineEmits<{
  'update:visible': [value: boolean]
}>()

const backups = ref<BackupInfo[]>([])
const loading = ref(false)

watch(() => props.visible, async (val) => {
  if (val) {
    await refreshBackups()
  }
})

async function refreshBackups() {
  loading.value = true
  try {
    backups.value = await listBackups()
  } finally {
    loading.value = false
  }
}

async function handleCreate() {
  loading.value = true
  try {
    await createBackup()
    await refreshBackups()
  } finally {
    loading.value = false
  }
}

async function handleRestore(backup: BackupInfo) {
  if (!confirm(t('backup.confirmRestore'))) return
  loading.value = true
  try {
    await restoreBackup(backup.path)
    await loadHosts()
    emit('update:visible', false)
  } finally {
    loading.value = false
  }
}

async function handleDelete(backup: BackupInfo) {
  if (!confirm(t('backup.confirmDelete'))) return
  loading.value = true
  try {
    await deleteBackup(backup.path)
    await refreshBackups()
  } finally {
    loading.value = false
  }
}

function formatSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`
}
</script>

<template>
  <a-drawer
    :visible="visible"
    :title="t('backup.title')"
    :width="400"
    @cancel="emit('update:visible', false)"
  >
    <div style="margin-bottom: 12px; display: flex; justify-content: flex-end;">
      <a-button type="primary" size="small" :loading="loading" @click="handleCreate">
        {{ t('backup.create') }}
      </a-button>
    </div>

    <a-spin :loading="loading" style="width: 100%">
      <a-empty v-if="backups.length === 0" style="padding: 40px 0">
        <template #description>{{ t('backup.empty') }}</template>
      </a-empty>

      <a-list v-else :bordered="false">
        <a-list-item v-for="backup in backups" :key="backup.path">
          <a-list-item-meta :title="backup.name" :description="backup.modified">
            <template #avatar>
              <a-avatar :size="32" shape="square" style="background: var(--color-fill-3)">
                <icon-file />
              </a-avatar>
            </template>
          </a-list-item-meta>
          <template #actions>
            <a-space size="mini">
              <a-tag size="small" color="gray">{{ formatSize(backup.size) }}</a-tag>
              <a-button size="mini" type="text" @click="handleRestore(backup)">
                {{ t('backup.restore') }}
              </a-button>
              <a-button size="mini" type="text" status="danger" @click="handleDelete(backup)">
                {{ t('backup.delete') }}
              </a-button>
            </a-space>
          </template>
        </a-list-item>
      </a-list>
    </a-spin>
  </a-drawer>
</template>
