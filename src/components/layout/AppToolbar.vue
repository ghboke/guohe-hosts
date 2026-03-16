<script setup lang="ts">
import { ref } from 'vue'
import { Message } from '@arco-design/web-vue'
import {
  IconRefresh,
  IconSave,
} from '@arco-design/web-vue/es/icon'
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

async function handleSave() {
  try {
    await saveHosts()
    Message.success(t('header.saveSuccess'))
  } catch {
    Message.error(t('header.saveFailed'))
  }
}

function handleToolsSelect(key: string | number | Record<string, unknown>) {
  switch (key) {
    case 'flush_dns':
      handleFlushDns()
      break
    case 'backup':
      emit('openBackup')
      break
    case 'settings':
      emit('openSettings')
      break
  }
}
</script>

<template>
  <div class="app-toolbar">
    <div class="toolbar-left">
      <a-dropdown @select="handleToolsSelect" trigger="click">
        <button class="menu-trigger">
          {{ t('header.tools') }}
          <svg class="menu-arrow" width="8" height="5" viewBox="0 0 8 5"><path d="M0.5 0.5L4 4L7.5 0.5" stroke="currentColor" stroke-width="1.2" fill="none"/></svg>
        </button>
        <template #content>
          <a-doption value="flush_dns">
            <template #icon>
              <svg width="14" height="14" viewBox="0 0 16 16"><path d="M8 2v4l3-2-3-2zM8 14v-4l-3 2 3 2zM2 8h4L4 5 2 8zM14 8h-4l2 3 2-3z" fill="currentColor" opacity="0.8"/></svg>
            </template>
            {{ t('header.flushDns') }}
          </a-doption>
          <a-doption value="backup">
            <template #icon>
              <svg width="14" height="14" viewBox="0 0 16 16"><path d="M3 2h10a1 1 0 011 1v10a1 1 0 01-1 1H3a1 1 0 01-1-1V3a1 1 0 011-1zm1 2v3h8V4H4zm0 5v3h8V9H4z" fill="currentColor" opacity="0.8"/></svg>
            </template>
            {{ t('header.backup') }}
          </a-doption>
          <a-doption value="settings">
            <template #icon>
              <svg width="14" height="14" viewBox="0 0 16 16"><path d="M8 10a2 2 0 100-4 2 2 0 000 4zm5.3-1.7l1.2.7a.5.5 0 01.2.6l-1 1.8a.5.5 0 01-.6.2l-1.2-.5a4.5 4.5 0 01-1 .6l-.2 1.3a.5.5 0 01-.5.4H6.8a.5.5 0 01-.5-.4l-.2-1.3a4.5 4.5 0 01-1-.6l-1.2.5a.5.5 0 01-.6-.2l-1-1.8a.5.5 0 01.2-.6l1.2-.7V8v-.3l-1.2-.7a.5.5 0 01-.2-.6l1-1.8a.5.5 0 01.6-.2l1.2.5a4.5 4.5 0 011-.6l.2-1.3a.5.5 0 01.5-.4h2.4a.5.5 0 01.5.4l.2 1.3a4.5 4.5 0 011 .6l1.2-.5a.5.5 0 01.6.2l1 1.8a.5.5 0 01-.2.6l-1.2.7V8v.3z" fill="currentColor" opacity="0.8"/></svg>
            </template>
            {{ t('header.settings') }}
          </a-doption>
        </template>
      </a-dropdown>
    </div>

    <div class="toolbar-right">
      <a-button
        size="small"
        :disabled="loading"
        @click="loadHosts"
      >
        <template #icon><icon-refresh /></template>
        {{ t('header.reload') }}
      </a-button>
      <a-button
        type="primary"
        size="small"
        :disabled="loading || !hasUnsavedChanges"
        @click="handleSave"
      >
        <template #icon><icon-save /></template>
        {{ hasUnsavedChanges ? t('header.saveUnsaved') : t('header.save') }}
      </a-button>
    </div>
  </div>
</template>

<style scoped>
.app-toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  height: 36px;
  padding: 0 12px;
  background-color: var(--color-bg-2);
  border-bottom: 1px solid var(--color-border);
}

.toolbar-left {
  display: flex;
  align-items: center;
  gap: 8px;
}

.toolbar-right {
  display: flex;
  align-items: center;
  gap: 8px;
}

.menu-trigger {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  height: 24px;
  padding: 0 10px;
  border: none;
  border-radius: 4px;
  background: transparent;
  color: var(--color-text-2);
  font-size: 13px;
  cursor: pointer;
  transition: background 0.15s, color 0.15s;
  outline: none;
}

.menu-trigger:hover {
  background: var(--color-fill-2);
  color: var(--color-text-1);
}

.menu-trigger:active {
  background: var(--color-fill-3);
}

.menu-arrow {
  margin-top: 1px;
}
</style>
