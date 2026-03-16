import { defineStore } from 'pinia'
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { AppConfig } from '../types/config'
import { useI18n, type Locale } from '../i18n'

export const useConfigStore = defineStore('config', () => {
  const config = ref<AppConfig>({
    hostsPath: '',
    autoSave: false,
    theme: 'system',
    locale: 'zh-CN',
    flushDnsOnSave: true,
    activeGroupId: '',
    shortcutToggleWindow: 'Alt+H',
  })

  const { setLocale } = useI18n()

  async function loadConfig() {
    try {
      const result = await invoke<AppConfig>('get_config')
      config.value = result
      applyTheme(result.theme)
      setLocale(result.locale as Locale)
    } catch {
      try {
        const hostsPath = await invoke<string>('get_system_hosts_path')
        config.value.hostsPath = hostsPath
      } catch {
        // ignore when backend is not ready
      }
      applyTheme(config.value.theme)
    }
  }

  async function saveConfig(newConfig: AppConfig) {
    try {
      await invoke('save_config', { config: newConfig })
      config.value = newConfig
      applyTheme(newConfig.theme)
      setLocale(newConfig.locale as Locale)
    } catch (e) {
      throw e
    }
  }

  function applyTheme(theme: string) {
    if (theme === 'dark') {
      document.body.setAttribute('arco-theme', 'dark')
    } else if (theme === 'light') {
      document.body.removeAttribute('arco-theme')
    } else {
      const isDark = window.matchMedia('(prefers-color-scheme: dark)').matches
      if (isDark) {
        document.body.setAttribute('arco-theme', 'dark')
      } else {
        document.body.removeAttribute('arco-theme')
      }
    }
  }

  return { config, loadConfig, saveConfig, applyTheme }
})
