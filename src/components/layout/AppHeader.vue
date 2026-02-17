<script setup lang="ts">
import { getCurrentWindow } from '@tauri-apps/api/window'
import { useI18n } from '../../i18n'

const { t } = useI18n()
const appWindow = getCurrentWindow()

async function handleDrag(e: MouseEvent) {
  const target = e.target as HTMLElement
  if (target.closest('button, [role="button"], input, .arco-btn')) {
    return
  }
  if (e.button === 0) {
    await appWindow.startDragging()
  }
}

async function handleMinimize() {
  await appWindow.minimize()
}

async function handleMaximize() {
  await appWindow.toggleMaximize()
}

async function handleClose() {
  await appWindow.hide()
}
</script>

<template>
  <header class="app-header" @mousedown="handleDrag">
    <div class="header-left">
      <h1 class="app-title">{{ t('app.title') }}</h1>
    </div>
    <div class="window-controls">
      <button class="win-btn" @click="handleMinimize" title="Minimize">
        <svg width="10" height="1" viewBox="0 0 10 1"><rect width="10" height="1" fill="currentColor"/></svg>
      </button>
      <button class="win-btn" @click="handleMaximize" title="Maximize">
        <svg width="10" height="10" viewBox="0 0 10 10"><rect x="0.5" y="0.5" width="9" height="9" fill="none" stroke="currentColor" stroke-width="1"/></svg>
      </button>
      <button class="win-btn win-close" @click="handleClose" title="Close">
        <svg width="10" height="10" viewBox="0 0 10 10"><line x1="0" y1="0" x2="10" y2="10" stroke="currentColor" stroke-width="1.2"/><line x1="10" y1="0" x2="0" y2="10" stroke="currentColor" stroke-width="1.2"/></svg>
      </button>
    </div>
  </header>
</template>

<style scoped>
.app-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  height: 40px;
  padding: 0 0 0 16px;
  background-color: var(--color-bg-2);
  border-bottom: 1px solid var(--color-border);
  user-select: none;
}

.header-left {
  display: flex;
  align-items: center;
  gap: 12px;
  flex-shrink: 0;
}

.app-title {
  font-size: 13px;
  font-weight: 600;
}

.window-controls {
  display: flex;
  align-items: stretch;
  height: 100%;
  flex-shrink: 0;
}

.win-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 46px;
  height: 100%;
  border: none;
  background: transparent;
  color: var(--color-text-2);
  cursor: pointer;
  transition: background 0.1s;
  outline: none;
}

.win-btn:hover {
  background: var(--color-fill-3);
}

.win-close:hover {
  background: #e81123;
  color: #fff;
}
</style>
