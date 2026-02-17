<script setup lang="ts">
import { ref, watch, onMounted, onBeforeUnmount, computed } from 'vue'
import { EditorView, basicSetup } from 'codemirror'
import { EditorState } from '@codemirror/state'
import { oneDark } from '@codemirror/theme-one-dark'
import { useHosts } from '../composables/useHosts'
import { useConfigStore } from '../stores/config'
import { storeToRefs } from 'pinia'
import { useI18n } from '../i18n'

const { selectedGroup, getGroupText, updateGroupFromText } = useHosts()
const configStore = useConfigStore()
const { config } = storeToRefs(configStore)
const { t } = useI18n()

const editorContainer = ref<HTMLElement>()
let editorView: EditorView | null = null
const hasEditorChanges = ref(false)
const loading = ref(false)

const isDark = computed(() => {
  if (config.value.theme === 'dark') return true
  if (config.value.theme === 'light') return false
  return window.matchMedia('(prefers-color-scheme: dark)').matches
})

async function loadText() {
  if (!selectedGroup.value) return
  loading.value = true
  try {
    const text = await getGroupText(selectedGroup.value.id)
    if (editorView) {
      editorView.dispatch({
        changes: {
          from: 0,
          to: editorView.state.doc.length,
          insert: text,
        },
      })
    }
    hasEditorChanges.value = false
  } finally {
    loading.value = false
  }
}

async function saveText() {
  if (!selectedGroup.value || !editorView) return
  loading.value = true
  try {
    const text = editorView.state.doc.toString()
    await updateGroupFromText(selectedGroup.value.id, text)
    hasEditorChanges.value = false
  } finally {
    loading.value = false
  }
}

function createEditor() {
  if (!editorContainer.value) return

  const extensions = [
    basicSetup,
    EditorView.lineWrapping,
    EditorView.updateListener.of((update) => {
      if (update.docChanged) {
        hasEditorChanges.value = true
      }
    }),
    EditorView.theme({
      '&': { height: '100%' },
      '.cm-scroller': { overflow: 'auto' },
      '.cm-content': { fontFamily: "'Cascadia Code', 'Fira Code', monospace", fontSize: '13px' },
      '.cm-gutters': { fontFamily: "'Cascadia Code', 'Fira Code', monospace", fontSize: '12px' },
    }),
  ]

  if (isDark.value) {
    extensions.push(oneDark)
  }

  const state = EditorState.create({
    doc: '',
    extensions,
  })

  editorView = new EditorView({
    state,
    parent: editorContainer.value,
  })
}

function destroyEditor() {
  if (editorView) {
    editorView.destroy()
    editorView = null
  }
}

watch(() => selectedGroup.value?.id, () => {
  loadText()
})

watch(isDark, () => {
  destroyEditor()
  createEditor()
  loadText()
})

onMounted(() => {
  createEditor()
  loadText()
})

onBeforeUnmount(() => {
  destroyEditor()
})
</script>

<template>
  <div class="codemirror-editor">
    <div class="editor-toolbar">
      <a-space size="small">
        <a-button size="small" @click="loadText" :loading="loading">
          {{ t('header.reload') }}
        </a-button>
        <a-button
          size="small"
          type="primary"
          :disabled="!hasEditorChanges"
          :loading="loading"
          @click="saveText"
        >
          {{ t('table.save') }}
        </a-button>
        <a-tag v-if="hasEditorChanges" size="small" color="orangered">*</a-tag>
      </a-space>
    </div>
    <div ref="editorContainer" class="editor-container" />
  </div>
</template>

<style scoped>
.codemirror-editor {
  display: flex;
  flex-direction: column;
  height: 100%;
}

.editor-toolbar {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  padding: 8px 0;
  flex-shrink: 0;
}

.editor-container {
  flex: 1;
  overflow: hidden;
  border: 1px solid var(--color-border);
  border-radius: 4px;
}

.editor-container :deep(.cm-editor) {
  height: 100%;
}
</style>
