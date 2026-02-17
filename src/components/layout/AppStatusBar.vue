<script setup lang="ts">
import { useHosts } from '../../composables/useHosts'
import { useI18n } from '../../i18n'

const { totalEntries, enabledEntries, error, hasUnsavedChanges, clearError } = useHosts()
const { t } = useI18n()
</script>

<template>
  <footer class="status-bar">
    <div class="status-left">
      <span v-if="error" class="status-error" @click="clearError">
        {{ t('statusBar.error') }}: {{ error }}
      </span>
      <span v-else class="status-info">
        {{ t('statusBar.entriesActive', { enabled: enabledEntries, total: totalEntries }) }}
      </span>
    </div>
    <div class="status-right">
      <a-tag v-if="hasUnsavedChanges" color="orangered" size="small">
        {{ t('statusBar.unsaved') }}
      </a-tag>
      <a-tag v-else color="green" size="small">
        {{ t('statusBar.saved') }}
      </a-tag>
    </div>
  </footer>
</template>

<style scoped>
.status-bar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  height: 28px;
  padding: 0 12px;
  background-color: var(--color-bg-2);
  border-top: 1px solid var(--color-border);
  font-size: 11px;
  color: var(--color-text-3);
}

.status-error {
  color: rgb(var(--danger-6));
  cursor: pointer;
}
</style>
