<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { Message } from '@arco-design/web-vue'
import { IconInfoCircle, IconSettings } from '@arco-design/web-vue/es/icon'
import { invoke } from '@tauri-apps/api/core'
import { useConfigStore } from '../stores/config'
import { UPDATE_FEATURE_ENABLED, useUpdateStore } from '../features/update'
import { storeToRefs } from 'pinia'
import { useI18n } from '../i18n'
import type { AppConfig } from '../types/config'
import { useHosts } from '../composables/useHosts'
import {
  APP_NAME,
  APP_REPOSITORY_URL,
  APP_VERSION,
  APP_VERSION_CODE,
} from '../constants/app'
import { openExternalUrl } from '../utils/external'

const configStore = useConfigStore()
const updateStore = useUpdateStore()
const { config } = storeToRefs(configStore)
const { checking, lastResult } = storeToRefs(updateStore)
const { loadHosts } = useHosts()
const { t } = useI18n()

const isRecordingShortcut = ref(false)
const shortcutDisplay = ref('')

const props = defineProps<{
  visible: boolean
}>()

const emit = defineEmits<{
  'update:visible': [value: boolean]
}>()

type SettingsSection = 'general' | 'about'

const form = ref<AppConfig>({ ...config.value })
const activeSection = ref<SettingsSection>('general')

watch(() => props.visible, (val) => {
  if (val) {
    form.value = { ...config.value }
    activeSection.value = 'general'
    shortcutDisplay.value = config.value.shortcutToggleWindow || ''
    isRecordingShortcut.value = false
  }
})

function handleShortcutKeydown(e: KeyboardEvent) {
  e.preventDefault()
  e.stopPropagation()

  // Ignore modifier-only presses
  if (['Control', 'Alt', 'Shift', 'Meta'].includes(e.key)) {
    return
  }

  const parts: string[] = []
  if (e.ctrlKey) parts.push('Ctrl')
  if (e.altKey) parts.push('Alt')
  if (e.shiftKey) parts.push('Shift')
  if (e.metaKey) parts.push('Meta')

  // Normalize key name
  let key = e.key
  if (key.length === 1) {
    key = key.toUpperCase()
  } else if (key.startsWith('F') && /^F\d{1,2}$/.test(key)) {
    // F1-F12, keep as is
  } else if (/^\d$/.test(key)) {
    // digit keys
  } else {
    // Unsupported key
    return
  }

  parts.push(key)

  // Must have at least one modifier
  if (!e.ctrlKey && !e.altKey && !e.shiftKey && !e.metaKey) {
    return
  }

  const shortcutStr = parts.join('+')
  shortcutDisplay.value = shortcutStr
  form.value = { ...form.value, shortcutToggleWindow: shortcutStr }
  isRecordingShortcut.value = false
}

function handleClearShortcut() {
  shortcutDisplay.value = ''
  form.value = { ...form.value, shortcutToggleWindow: '' }
  isRecordingShortcut.value = false
}

function handleStartRecording() {
  isRecordingShortcut.value = true
}

function handleStopRecording() {
  isRecordingShortcut.value = false
}

async function handleSave() {
  const previousHostsPath = config.value.hostsPath
  const previousShortcut = config.value.shortcutToggleWindow

  // Re-register global shortcut if changed
  const newShortcut = form.value.shortcutToggleWindow
  if (newShortcut !== previousShortcut) {
    try {
      await invoke('update_global_shortcut', {
        oldShortcut: previousShortcut || '',
        newShortcut: newShortcut || '',
      })
    } catch {
      Message.error(t('settings.shortcutRegisterFailed'))
      return
    }
  }

  await configStore.saveConfig({ ...form.value })
  if (form.value.hostsPath !== previousHostsPath) {
    await loadHosts()
  }
  emit('update:visible', false)
}

async function handleRestoreSystemHosts() {
  try {
    const systemHostsPath = await invoke<string>('get_system_hosts_path')
    const nextConfig = { ...form.value, hostsPath: systemHostsPath }
    form.value = nextConfig
    await configStore.saveConfig(nextConfig)
    await loadHosts()
    Message.success(t('settings.restoreSystemHostsSuccess'))
  } catch {
    Message.error(t('settings.restoreSystemHostsFailed'))
  }
}

async function handleCheckUpdate() {
  if (!UPDATE_FEATURE_ENABLED) {
    Message.info(t('update.disabled'))
    return
  }

  try {
    const result = await updateStore.checkForUpdates(true)
    if (!result) return

    if (result.hasUpdate) {
      updateStore.showUpdateDialog(result)
    } else {
      Message.success(t('update.latest'))
    }
  } catch {
    Message.error(t('update.checkFailed'))
  }
}

function handleOpenUpdate() {
  const result = lastResult.value
  if (result?.hasUpdate) {
    updateStore.showUpdateDialog(result)
  }
}

async function handleOpenRepository() {
  await openExternalUrl(APP_REPOSITORY_URL)
}

const hasUpdate = computed(() => Boolean(lastResult.value?.hasUpdate))
const latestVersionText = computed(() => lastResult.value?.latestVersion ?? '-')
</script>

<template>
  <a-modal
    :visible="visible"
    :title="t('settings.title')"
    width="760px"
    @cancel="emit('update:visible', false)"
  >
    <div class="settings-layout">
      <div class="settings-menu">
        <button
          class="menu-item"
          :class="{ active: activeSection === 'general' }"
          @click="activeSection = 'general'"
        >
          <icon-settings />
          <span>{{ t('settings.general') }}</span>
        </button>
        <button
          class="menu-item"
          :class="{ active: activeSection === 'about' }"
          @click="activeSection = 'about'"
        >
          <icon-info-circle />
          <span>{{ t('settings.about') }}</span>
        </button>
      </div>

      <div class="settings-content">
        <a-form v-if="activeSection === 'general'" :model="form" layout="vertical">
          <a-form-item :label="t('settings.hostsPath')">
            <div class="hosts-path-row">
              <a-input v-model="form.hostsPath" />
              <a-button @click="handleRestoreSystemHosts">
                {{ t('settings.restoreSystemHosts') }}
              </a-button>
            </div>
          </a-form-item>

          <a-form-item :label="t('settings.theme')">
            <a-radio-group v-model="form.theme">
              <a-radio value="light">{{ t('settings.themeLight') }}</a-radio>
              <a-radio value="dark">{{ t('settings.themeDark') }}</a-radio>
              <a-radio value="system">{{ t('settings.themeSystem') }}</a-radio>
            </a-radio-group>
          </a-form-item>

          <a-form-item :label="t('settings.language')">
            <a-radio-group v-model="form.locale">
              <a-radio value="zh-CN">中文</a-radio>
              <a-radio value="en-US">English</a-radio>
            </a-radio-group>
          </a-form-item>

          <a-form-item :label="t('settings.shortcutToggleWindow')">
            <div class="shortcut-row">
              <div
                class="shortcut-input"
                :class="{ recording: isRecordingShortcut }"
                tabindex="0"
                @click="handleStartRecording"
                @keydown="handleShortcutKeydown"
                @blur="handleStopRecording"
              >
                <span v-if="isRecordingShortcut" class="shortcut-recording">
                  {{ t('settings.shortcutPlaceholder') }}
                </span>
                <span v-else-if="shortcutDisplay" class="shortcut-keys">
                  <kbd v-for="(key, idx) in shortcutDisplay.split('+')" :key="idx">{{ key }}</kbd>
                </span>
                <span v-else class="shortcut-none">{{ t('settings.shortcutNone') }}</span>
              </div>
              <a-button size="small" @click="handleClearShortcut">
                {{ t('settings.shortcutClear') }}
              </a-button>
            </div>
            <div class="setting-desc" style="margin-top: 4px">
              {{ t('settings.shortcutToggleWindowDesc') }}
            </div>
          </a-form-item>

          <a-form-item>
            <a-space direction="vertical" fill>
              <a-checkbox v-model="form.autoSave">
                {{ t('settings.autoSave') }}
                <template #description>
                  <span class="setting-desc">{{ t('settings.autoSaveDesc') }}</span>
                </template>
              </a-checkbox>
              <a-checkbox v-model="form.flushDnsOnSave">
                {{ t('settings.flushDnsOnSave') }}
                <template #description>
                  <span class="setting-desc">{{ t('settings.flushDnsOnSaveDesc') }}</span>
                </template>
              </a-checkbox>
            </a-space>
          </a-form-item>
        </a-form>

        <div v-else class="about-panel">
          <div class="about-header">
            <div class="about-title">{{ APP_NAME }}</div>
            <a-tag color="arcoblue">v{{ APP_VERSION }}</a-tag>
          </div>

          <div class="about-meta">
            <div class="meta-row">
              <span class="meta-label">{{ t('settings.appName') }}</span>
              <span>{{ APP_NAME }}</span>
            </div>
            <div class="meta-row">
              <span class="meta-label">{{ t('settings.appVersion') }}</span>
              <span>v{{ APP_VERSION }}</span>
            </div>
            <div class="meta-row">
              <span class="meta-label">{{ t('settings.versionCode') }}</span>
              <span>{{ APP_VERSION_CODE }}</span>
            </div>
            <div class="meta-row repo-row">
              <span class="meta-label">{{ t('settings.openSource') }}</span>
              <a-link @click="handleOpenRepository">{{ APP_REPOSITORY_URL }}</a-link>
            </div>
          </div>

          <div v-if="UPDATE_FEATURE_ENABLED" class="update-card" :class="{ highlight: hasUpdate }">
            <div class="update-top">
              <div>
                <div class="update-title">{{ t('settings.checkUpdate') }}</div>
                <div class="update-desc">
                  {{
                    hasUpdate
                      ? t('update.availableContent', { version: latestVersionText })
                      : t('update.currentVersion', { version: APP_VERSION })
                  }}
                </div>
              </div>
              <a-tag v-if="hasUpdate" color="orangered">{{ t('update.availableShort') }}</a-tag>
            </div>

            <div v-if="hasUpdate" class="update-extra">
              {{ t('update.latestVersion', { version: latestVersionText }) }}
            </div>

            <div class="update-actions">
              <a-button @click="handleOpenRepository">
                {{ t('settings.viewSource') }}
              </a-button>
              <a-button type="primary" :loading="checking" @click="handleCheckUpdate">
                {{ t('settings.checkUpdate') }}
              </a-button>
              <a-button v-if="hasUpdate" @click="handleOpenUpdate">
                {{ t('update.download') }}
              </a-button>
            </div>
          </div>

          <div v-else class="update-disabled">
            {{ t('update.disabled') }}
          </div>
        </div>
      </div>
    </div>

    <template #footer>
      <div class="modal-footer">
        <a-button @click="emit('update:visible', false)">
          {{ t('common.close') }}
        </a-button>
        <a-button
          v-if="activeSection === 'general'"
          type="primary"
          @click="handleSave"
        >
          {{ t('table.save') }}
        </a-button>
      </div>
    </template>
  </a-modal>
</template>

<style scoped>
.settings-layout {
  display: flex;
  min-height: 360px;
}

.settings-menu {
  width: 150px;
  padding-right: 16px;
  border-right: 1px solid var(--color-border);
  display: flex;
  flex-direction: column;
  gap: 8px;
  flex-shrink: 0;
}

.menu-item {
  display: flex;
  align-items: center;
  gap: 8px;
  width: 100%;
  padding: 10px 12px;
  border: none;
  border-radius: 6px;
  background: transparent;
  color: var(--color-text-2);
  cursor: pointer;
  transition: all 0.15s ease;
}

.menu-item:hover {
  background: var(--color-fill-2);
}

.menu-item.active {
  background: rgb(var(--primary-1));
  color: rgb(var(--primary-6));
}

.settings-content {
  flex: 1;
  min-width: 0;
  padding-left: 16px;
}

.hosts-path-row {
  display: flex;
  align-items: center;
  gap: 8px;
  width: 100%;
}

.hosts-path-row :deep(.arco-input-wrapper) {
  flex: 1;
  min-width: 0;
}

.hosts-path-row :deep(.arco-btn) {
  flex-shrink: 0;
}

.setting-desc {
  font-size: 12px;
  color: var(--color-text-3);
}

.about-panel {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.about-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.about-title {
  font-size: 18px;
  font-weight: 600;
}

.about-meta {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.meta-row {
  display: flex;
  justify-content: space-between;
  gap: 16px;
  font-size: 13px;
}

.repo-row {
  align-items: center;
}

.meta-label {
  color: var(--color-text-3);
}

.update-card {
  padding: 16px;
  border: 1px solid var(--color-border);
  border-radius: 8px;
  background: var(--color-fill-1);
}

.update-card.highlight {
  border-color: rgb(var(--warning-6));
  background: rgb(var(--warning-1));
}

.update-top {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 12px;
}

.update-title {
  font-size: 14px;
  font-weight: 600;
  margin-bottom: 4px;
}

.update-desc,
.update-extra {
  font-size: 12px;
  color: var(--color-text-3);
}

.update-extra {
  margin-top: 10px;
}

.update-actions {
  display: flex;
  gap: 8px;
  margin-top: 14px;
}

.modal-footer {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
}

.update-disabled {
  padding: 14px 16px;
  border: 1px dashed var(--color-border);
  border-radius: 8px;
  font-size: 12px;
  color: var(--color-text-3);
}

.shortcut-row {
  display: flex;
  align-items: center;
  gap: 8px;
}

.shortcut-input {
  flex: 1;
  min-height: 32px;
  padding: 4px 12px;
  border: 1px solid var(--color-border);
  border-radius: 6px;
  background: var(--color-fill-2);
  cursor: pointer;
  display: flex;
  align-items: center;
  transition: border-color 0.15s;
  outline: none;
}

.shortcut-input:hover {
  border-color: rgb(var(--primary-5));
}

.shortcut-input.recording {
  border-color: rgb(var(--primary-6));
  box-shadow: 0 0 0 2px rgba(var(--primary-6), 0.15);
}

.shortcut-recording {
  color: rgb(var(--primary-6));
  font-size: 13px;
  animation: blink 1s infinite;
}

@keyframes blink {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.5; }
}

.shortcut-keys {
  display: flex;
  gap: 4px;
}

.shortcut-keys kbd {
  display: inline-block;
  padding: 2px 8px;
  border: 1px solid var(--color-border);
  border-radius: 4px;
  background: var(--color-bg-2);
  font-size: 12px;
  font-family: inherit;
  line-height: 1.4;
  box-shadow: 0 1px 0 var(--color-border);
}

.shortcut-none {
  color: var(--color-text-4);
  font-size: 13px;
}
</style>
