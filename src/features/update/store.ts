import { defineStore } from 'pinia'
import { h, ref } from 'vue'
import { Message, Modal, Notification } from '@arco-design/web-vue'
import { useI18n } from '../../i18n'
import { openExternalUrl } from '../../utils/external'
import { UPDATE_FEATURE_ENABLED } from './config'
import type { UpdateCheckResult } from './types'
import { invoke } from '@tauri-apps/api/core'

export const useUpdateStore = defineStore('update', () => {
  const checking = ref(false)
  const lastResult = ref<UpdateCheckResult | null>(null)
  const notifiedVersion = ref<string | null>(null)
  const { t } = useI18n()

  async function openUpdateUrl(url: string) {
    await openExternalUrl(url)
  }

  function showUpdateDialog(result: UpdateCheckResult) {
    const version = result.latestVersion ?? ''
    const content = result.changelog
      ? `${t('update.dialogContent', { version })}\n\n${t('update.changelog')}: ${result.changelog}`
      : t('update.dialogContent', { version })

    Modal.confirm({
      title: t('update.dialogTitle'),
      content,
      okText: t('update.download'),
      cancelText: t('common.cancel'),
      onOk: async () => {
        if (result.updateUrl) {
          try {
            await openUpdateUrl(result.updateUrl)
          } catch {
            Message.error(t('update.openFailed'))
          }
        } else {
          Message.warning(t('update.noUrl'))
        }
      },
    })
  }

  function notifyUpdate(result: UpdateCheckResult) {
    const version = result.latestVersion
    if (!version || notifiedVersion.value === version) {
      return
    }

    notifiedVersion.value = version
    Notification.info({
      id: 'app-update',
      title: t('update.availableTitle'),
      content: () =>
        h(
          'div',
          {
            style: {
              cursor: 'pointer',
            },
            onClick: () => showUpdateDialog(result),
          },
          t('update.availableContent', { version })
        ),
      position: 'bottomRight',
      duration: 0,
      closable: true,
    })
  }

  async function checkForUpdates(force = false) {
    if (!UPDATE_FEATURE_ENABLED) {
      lastResult.value = {
        checked: false,
        hasUpdate: false,
        latestVersion: null,
        latestVersionCode: null,
        changelog: null,
        forceUpdate: false,
        updateUrl: null,
        minCheckIntervalSeconds: 0,
      }
      return lastResult.value
    }

    if (checking.value) {
      return lastResult.value
    }

    checking.value = true
    try {
      const result = await invoke<UpdateCheckResult>('check_update', { force })
      lastResult.value = result

      if (result.checked && result.hasUpdate && result.updateUrl) {
        notifyUpdate(result)
      }

      return result
    } finally {
      checking.value = false
    }
  }

  return {
    checking,
    lastResult,
    checkForUpdates,
    showUpdateDialog,
    openUpdateUrl,
  }
})
