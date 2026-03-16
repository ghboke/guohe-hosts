<script setup lang="ts">
import { ref, onMounted, onBeforeUnmount } from 'vue'
import { Message } from '@arco-design/web-vue'
import AppHeader from './components/layout/AppHeader.vue'
import AppToolbar from './components/layout/AppToolbar.vue'
import AppSidebar from './components/layout/AppSidebar.vue'
import AppStatusBar from './components/layout/AppStatusBar.vue'
import HostTable from './components/HostTable.vue'
import SettingsModal from './components/SettingsModal.vue'
import BackupDrawer from './components/BackupDrawer.vue'
import ImportExportModal from './components/ImportExportModal.vue'
import { useHosts } from './composables/useHosts'
import { useConfigStore } from './stores/config'
import { useI18n } from './i18n'
import { UPDATE_FEATURE_ENABLED, useUpdateStore } from './features/update'

const { loadHosts, saveHosts, hasUnsavedChanges, groups } = useHosts()
const { t } = useI18n()
const configStore = useConfigStore()
const updateStore = useUpdateStore()

const showSettings = ref(false)
const showBackup = ref(false)
const showImportExport = ref(false)
const importExportMode = ref<'import' | 'export'>('import')

function openImport() {
  importExportMode.value = 'import'
  showImportExport.value = true
}

function openExport() {
  importExportMode.value = 'export'
  showImportExport.value = true
}

async function handleKeydown(e: KeyboardEvent) {
  if (e.ctrlKey && e.key === 's') {
    e.preventDefault()
    try {
      await saveHosts()
      Message.success(t('header.saveSuccess'))
    } catch {
      Message.error(t('header.saveFailed'))
    }
  }
  if (e.ctrlKey && e.key === 'r') {
    e.preventDefault()
    loadHosts()
  }
}

async function initApp(): Promise<void> {
  await configStore.loadConfig().catch(() => {})

  // Load hosts with retry + timeout (backend may not be ready yet)
  for (let attempt = 0; attempt < 10; attempt++) {
    try {
      await Promise.race([
        loadHosts(),
        new Promise<void>((_, reject) =>
          setTimeout(() => reject(new Error('timeout')), 3000)
        ),
      ])
    } catch {
      // timeout or invoke error - will retry
    }
    if (groups.value.length > 0) {
      return
    }
    await new Promise((r) => setTimeout(r, 500))
  }
}

onMounted(async () => {
  await initApp()
  if (UPDATE_FEATURE_ENABLED) {
    updateStore.checkForUpdates().catch(() => {})
  }

  // Listen for system theme changes
  window.matchMedia('(prefers-color-scheme: dark)').addEventListener('change', () => {
    if (configStore.config.theme === 'system') {
      configStore.applyTheme('system')
    }
  })

  window.addEventListener('keydown', handleKeydown)
})

onBeforeUnmount(() => {
  window.removeEventListener('keydown', handleKeydown)
})
</script>

<template>
  <div id="guohe-hosts-app">
    <AppHeader />
    <AppToolbar
      @open-settings="showSettings = true"
      @open-backup="showBackup = true"
    />
    <div class="app-body">
      <AppSidebar
        @open-import="openImport"
        @open-export="openExport"
      />
      <main class="app-main">
        <HostTable @open-export="openExport" />
      </main>
    </div>
    <AppStatusBar />

    <SettingsModal v-model:visible="showSettings" />
    <BackupDrawer v-model:visible="showBackup" />
    <ImportExportModal
      v-model:visible="showImportExport"
      :mode="importExportMode"
    />
  </div>
</template>

<style>
* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

body {
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
  background: transparent;
  color: var(--color-text-1);
  overflow: hidden;
}

#guohe-hosts-app {
  width: 100vw;
  height: 100vh;
  display: flex;
  flex-direction: column;
  background: var(--color-bg-1);
}

.app-body {
  flex: 1;
  display: flex;
  overflow: hidden;
}

.app-main {
  flex: 1;
  overflow: hidden;
  padding: 16px;
  display: flex;
  flex-direction: column;
}
</style>
