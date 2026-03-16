<script setup lang="ts">
import { ref } from 'vue'
import { useHosts } from '../composables/useHosts'
import { useI18n } from '../i18n'

const { importFromText, exportGroup, selectedGroup } = useHosts()
const { t } = useI18n()

const props = defineProps<{
  visible: boolean
  mode: 'import' | 'export'
}>()

const emit = defineEmits<{
  'update:visible': [value: boolean]
}>()

const importText = ref('')
const importGroupName = ref('')
const exportText = ref('')
const copied = ref(false)
const loading = ref(false)

async function handleImport() {
  const text = importText.value.trim()
  const groupName = importGroupName.value.trim()
  if (!text || !groupName) return

  loading.value = true
  try {
    await importFromText(text, groupName)
    importText.value = ''
    importGroupName.value = ''
    emit('update:visible', false)
  } finally {
    loading.value = false
  }
}

async function loadExportText() {
  if (props.mode === 'export' && selectedGroup.value) {
    loading.value = true
    try {
      exportText.value = await exportGroup(selectedGroup.value.id)
    } finally {
      loading.value = false
    }
  }
}

async function handleCopy() {
  try {
    await navigator.clipboard.writeText(exportText.value)
    copied.value = true
    setTimeout(() => { copied.value = false }, 2000)
  } catch {
    // fallback
    const textarea = document.createElement('textarea')
    textarea.value = exportText.value
    document.body.appendChild(textarea)
    textarea.select()
    document.execCommand('copy')
    document.body.removeChild(textarea)
    copied.value = true
    setTimeout(() => { copied.value = false }, 2000)
  }
}

function handleOpen() {
  if (props.mode === 'export') {
    loadExportText()
  }
}
</script>

<template>
  <a-modal
    :visible="visible"
    :title="mode === 'import' ? t('importExport.importTitle') : t('importExport.exportTitle')"
    :width="mode === 'export' ? 760 : 640"
    :ok-text="mode === 'import' ? t('importExport.import') : undefined"
    :cancel-text="t('common.cancel')"
    :hide-cancel="mode === 'export'"
    :footer="mode === 'export' ? false : undefined"
    @ok="handleImport"
    @cancel="emit('update:visible', false)"
    @open="handleOpen"
  >
    <!-- Import mode -->
    <template v-if="mode === 'import'">
      <a-form :model="{}" layout="vertical">
        <a-form-item :label="t('importExport.importGroupName')" required>
          <a-input
            v-model="importGroupName"
            :placeholder="t('importExport.importGroupNamePlaceholder')"
          />
        </a-form-item>
        <a-form-item :label="t('importExport.importText')" required>
          <a-textarea
            v-model="importText"
            :placeholder="t('importExport.importTextPlaceholder')"
            :auto-size="{ minRows: 6, maxRows: 12 }"
            class="host-textarea"
            style="font-family: 'Cascadia Code', 'Fira Code', monospace; font-size: 12px"
          />
        </a-form-item>
      </a-form>
    </template>

    <!-- Export mode -->
    <template v-else>
      <a-spin :loading="loading" class="export-spin">
        <a-textarea
          :model-value="exportText"
          readonly
          :auto-size="{ minRows: 6, maxRows: 16 }"
          class="host-textarea"
          style="font-family: 'Cascadia Code', 'Fira Code', monospace; font-size: 12px"
        />
        <div style="margin-top: 12px; text-align: right">
          <a-button type="primary" size="small" @click="handleCopy">
            {{ copied ? t('importExport.copied') : t('importExport.copy') }}
          </a-button>
        </div>
      </a-spin>
    </template>
  </a-modal>
</template>

<style scoped>
.host-textarea {
  width: 100%;
}

.export-spin {
  display: block;
  width: 100%;
}

.export-spin :deep(.arco-spin-content) {
  width: 100%;
}
</style>
