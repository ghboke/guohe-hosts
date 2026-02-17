<script setup lang="ts">
import { ref, watch } from 'vue'
import { useConfigStore } from '../stores/config'
import { storeToRefs } from 'pinia'
import { useI18n } from '../i18n'
import type { AppConfig } from '../types/config'

const configStore = useConfigStore()
const { config } = storeToRefs(configStore)
const { t } = useI18n()

const props = defineProps<{
  visible: boolean
}>()

const emit = defineEmits<{
  'update:visible': [value: boolean]
}>()

const form = ref<AppConfig>({ ...config.value })

watch(() => props.visible, (val) => {
  if (val) {
    form.value = { ...config.value }
  }
})

async function handleSave() {
  await configStore.saveConfig({ ...form.value })
  emit('update:visible', false)
}
</script>

<template>
  <a-modal
    :visible="visible"
    :title="t('settings.title')"
    :ok-text="t('table.save')"
    :cancel-text="t('common.cancel')"
    @ok="handleSave"
    @cancel="emit('update:visible', false)"
  >
    <a-form :model="form" layout="vertical">
      <a-form-item :label="t('settings.hostsPath')">
        <a-input v-model="form.hostsPath" />
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
  </a-modal>
</template>

<style scoped>
.setting-desc {
  font-size: 12px;
  color: var(--color-text-3);
}
</style>
