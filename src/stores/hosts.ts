import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { HostEntry, HostGroup, BackupInfo } from '../types/host'

export const useHostsStore = defineStore('hosts', () => {
  const groups = ref<HostGroup[]>([])
  const selectedGroupId = ref<string | null>(null)
  const loading = ref(false)
  const error = ref<string | null>(null)
  const hasUnsavedChanges = ref(false)

  const selectedGroup = computed(() =>
    groups.value.find((g) => g.id === selectedGroupId.value) ?? null
  )

  const totalEntries = computed(() =>
    groups.value.reduce((sum, g) => sum + g.entries.length, 0)
  )

  const enabledEntries = computed(() =>
    groups.value.reduce(
      (sum, g) =>
        sum +
        (g.enabled
          ? g.entries.filter((e) => e.enabled).length
          : 0),
      0
    )
  )

  async function loadHosts() {
    loading.value = true
    error.value = null
    const previousName = selectedGroup.value?.name ?? null
    try {
      const result = await invoke<HostGroup[]>('read_hosts_file')
      groups.value = result
      if (previousName) {
        const match = result.find((g) => g.name === previousName)
        selectedGroupId.value = match?.id ?? result[0]?.id ?? null
      } else if (result.length > 0 && !selectedGroupId.value) {
        const activeGroup = result.find((g) => g.enabled)
        selectedGroupId.value = activeGroup?.id ?? result[0]!.id
      }
      hasUnsavedChanges.value = false
    } catch (e) {
      error.value = String(e)
    } finally {
      loading.value = false
    }
  }

  async function saveHosts() {
    loading.value = true
    error.value = null
    try {
      await invoke('write_hosts_file')
      hasUnsavedChanges.value = false
    } catch (e) {
      error.value = String(e)
    } finally {
      loading.value = false
    }
  }

  async function addGroup(name: string) {
    try {
      const group = await invoke<HostGroup>('add_group', { name })
      groups.value = [...groups.value, group]
      selectedGroupId.value = group.id
      hasUnsavedChanges.value = true
    } catch (e) {
      error.value = String(e)
    }
  }

  async function deleteGroup(groupId: string) {
    try {
      const updated = await invoke<HostGroup[]>('delete_group', { groupId })
      groups.value = updated
      if (selectedGroupId.value === groupId) {
        selectedGroupId.value = updated[0]?.id ?? null
      }
      hasUnsavedChanges.value = true
    } catch (e) {
      error.value = String(e)
    }
  }

  async function activateGroup(groupId: string) {
    const current = groups.value.find((g) => g.enabled)
    if (current?.id === groupId) return
    try {
      const updated = await invoke<HostGroup[]>('activate_group', { groupId })
      groups.value = updated
      hasUnsavedChanges.value = false
    } catch (e) {
      error.value = String(e)
    }
  }

  async function renameGroup(groupId: string, name: string) {
    try {
      const updated = await invoke<HostGroup>('rename_group', { groupId, name })
      groups.value = groups.value.map((g) => (g.id === groupId ? updated : g))
      hasUnsavedChanges.value = true
    } catch (e) {
      error.value = String(e)
    }
  }

  async function reorderGroups(groupIds: string[]) {
    try {
      await invoke('reorder_groups', { groupIds })
      const ordered = groupIds
        .map((id) => groups.value.find((g) => g.id === id))
        .filter((g): g is HostGroup => g !== undefined)
      groups.value = ordered
      hasUnsavedChanges.value = true
    } catch (e) {
      error.value = String(e)
    }
  }

  async function addEntry(groupId: string, ip: string, hostname: string, comment: string) {
    try {
      const entry = await invoke<HostEntry>('add_entry', { groupId, ip, hostname, comment })
      groups.value = groups.value.map((g) => {
        if (g.id !== groupId) return g
        return { ...g, entries: [...g.entries, entry] }
      })
      hasUnsavedChanges.value = true
    } catch (e) {
      error.value = String(e)
    }
  }

  async function updateEntry(
    groupId: string,
    entryId: string,
    ip: string,
    hostname: string,
    comment: string
  ) {
    try {
      const updated = await invoke<HostEntry>('update_entry', {
        groupId,
        entryId,
        ip,
        hostname,
        comment,
      })
      groups.value = groups.value.map((g) => {
        if (g.id !== groupId) return g
        return {
          ...g,
          entries: g.entries.map((e) => (e.id === entryId ? updated : e)),
        }
      })
      hasUnsavedChanges.value = true
    } catch (e) {
      error.value = String(e)
    }
  }

  async function deleteEntry(groupId: string, entryId: string) {
    try {
      await invoke('delete_entry', { groupId, entryId })
      groups.value = groups.value.map((g) => {
        if (g.id !== groupId) return g
        return { ...g, entries: g.entries.filter((e) => e.id !== entryId) }
      })
      hasUnsavedChanges.value = true
    } catch (e) {
      error.value = String(e)
    }
  }

  async function toggleEntry(groupId: string, entryId: string, enabled: boolean) {
    try {
      const updated = await invoke<HostEntry>('toggle_entry', { groupId, entryId, enabled })
      groups.value = groups.value.map((g) => {
        if (g.id !== groupId) return g
        return {
          ...g,
          entries: g.entries.map((e) => (e.id === entryId ? updated : e)),
        }
      })
      hasUnsavedChanges.value = true
    } catch (e) {
      error.value = String(e)
    }
  }

  async function reorderEntries(groupId: string, entryIds: string[]) {
    try {
      await invoke('reorder_entries', { groupId, entryIds })
      groups.value = groups.value.map((g) => {
        if (g.id !== groupId) return g
        const ordered = entryIds
          .map((id) => g.entries.find((e) => e.id === id))
          .filter((e): e is HostEntry => e !== undefined)
        return { ...g, entries: ordered }
      })
      hasUnsavedChanges.value = true
    } catch (e) {
      error.value = String(e)
    }
  }

  // DNS
  async function flushDns(): Promise<string> {
    const result = await invoke<string>('flush_dns')
    return result
  }

  // Backup
  async function createBackup(): Promise<string> {
    return await invoke<string>('create_backup')
  }

  async function listBackups(): Promise<BackupInfo[]> {
    return await invoke<BackupInfo[]>('list_backups')
  }

  async function restoreBackup(backupPath: string): Promise<void> {
    const result = await invoke<HostGroup[]>('restore_backup', { backupPath })
    groups.value = result
    if (result.length > 0 && !result.find((g) => g.id === selectedGroupId.value)) {
      selectedGroupId.value = result[0]!.id
    }
    hasUnsavedChanges.value = false
  }

  async function deleteBackup(backupPath: string): Promise<void> {
    await invoke('delete_backup', { backupPath })
  }

  // Import / Export
  async function importFromText(text: string, groupName: string): Promise<HostGroup> {
    const group = await invoke<HostGroup>('import_from_text', { text, groupName })
    groups.value = [...groups.value, group]
    selectedGroupId.value = group.id
    hasUnsavedChanges.value = true
    return group
  }

  async function exportGroup(groupId: string): Promise<string> {
    return await invoke<string>('export_group', { groupId })
  }

  // CodeMirror
  async function getGroupText(groupId: string): Promise<string> {
    return await invoke<string>('get_group_text', { groupId })
  }

  async function updateGroupFromText(groupId: string, text: string): Promise<HostGroup> {
    const updated = await invoke<HostGroup>('update_group_from_text', { groupId, text })
    groups.value = groups.value.map((g) => (g.id === groupId ? updated : g))
    hasUnsavedChanges.value = true
    return updated
  }

  function selectGroup(groupId: string) {
    selectedGroupId.value = groupId
  }

  function clearError() {
    error.value = null
  }

  return {
    groups,
    selectedGroupId,
    selectedGroup,
    loading,
    error,
    hasUnsavedChanges,
    totalEntries,
    enabledEntries,
    loadHosts,
    saveHosts,
    addGroup,
    deleteGroup,
    activateGroup,
    renameGroup,
    reorderGroups,
    addEntry,
    updateEntry,
    deleteEntry,
    toggleEntry,
    reorderEntries,
    flushDns,
    createBackup,
    listBackups,
    restoreBackup,
    deleteBackup,
    importFromText,
    exportGroup,
    getGroupText,
    updateGroupFromText,
    selectGroup,
    clearError,
  }
})
