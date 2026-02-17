import { useHostsStore } from '../stores/hosts'
import { storeToRefs } from 'pinia'

export function useHosts() {
  const store = useHostsStore()
  const {
    groups,
    selectedGroupId,
    selectedGroup,
    loading,
    error,
    hasUnsavedChanges,
    totalEntries,
    enabledEntries,
  } = storeToRefs(store)

  return {
    // State
    groups,
    selectedGroupId,
    selectedGroup,
    loading,
    error,
    hasUnsavedChanges,
    totalEntries,
    enabledEntries,

    // Actions
    loadHosts: store.loadHosts,
    saveHosts: store.saveHosts,
    addGroup: store.addGroup,
    deleteGroup: store.deleteGroup,
    activateGroup: store.activateGroup,
    renameGroup: store.renameGroup,
    reorderGroups: store.reorderGroups,
    addEntry: store.addEntry,
    updateEntry: store.updateEntry,
    deleteEntry: store.deleteEntry,
    toggleEntry: store.toggleEntry,
    reorderEntries: store.reorderEntries,
    flushDns: store.flushDns,
    createBackup: store.createBackup,
    listBackups: store.listBackups,
    restoreBackup: store.restoreBackup,
    deleteBackup: store.deleteBackup,
    importFromText: store.importFromText,
    exportGroup: store.exportGroup,
    getGroupText: store.getGroupText,
    updateGroupFromText: store.updateGroupFromText,
    selectGroup: store.selectGroup,
    clearError: store.clearError,
  }
}
