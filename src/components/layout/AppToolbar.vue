<script setup lang="ts">
import { ref } from 'vue'
import { Message } from '@arco-design/web-vue'
import { useHosts } from '../../composables/useHosts'
import { useI18n } from '../../i18n'

const { loading, hasUnsavedChanges, saveHosts, loadHosts, flushDns } = useHosts()
const { t } = useI18n()

const emit = defineEmits<{
  openSettings: []
  openBackup: []
}>()

const flushing = ref(false)

async function handleFlushDns() {
  flushing.value = true
  try {
    await flushDns()
    Message.success(t('dns.flushSuccess'))
  } catch {
    Message.error(t('dns.flushError'))
  } finally {
    flushing.value = false
  }
}
</script>

<template>
  <div class="app-toolbar">
    <div class="toolbar-actions">
      <a-button
        size="small"
        :loading="flushing"
        @click="handleFlushDns"
      >
        {{ t('header.flushDns') }}
      </a-button>
      <a-button
        size="small"
        @click="emit('openBackup')"
      >
        {{ t('header.backup') }}
      </a-button>
      <a-button
        size="small"
        @click="emit('openSettings')"
      >
        {{ t('header.settings') }}
      </a-button>
      <a-divider direction="vertical" />
      <a-button
        size="small"
        :disabled="loading"
        @click="loadHosts"
      >
        {{ t('header.reload') }}
      </a-button>
      <a-button
        type="primary"
        size="small"
        :disabled="loading || !hasUnsavedChanges"
        @click="saveHosts"
      >
        {{ hasUnsavedChanges ? t('header.saveUnsaved') : t('header.save') }}
      </a-button>
    </div>
  </div>
</template>

<style scoped>
.app-toolbar {
  display: flex;
  align-items: center;
  height: 36px;
  padding: 0 12px;
  background-color: var(--color-bg-2);
  border-bottom: 1px solid var(--color-border);
}

.toolbar-actions {
  display: flex;
  align-items: center;
  gap: 8px;
}
</style>
