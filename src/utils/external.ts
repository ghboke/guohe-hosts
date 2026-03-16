import { invoke } from '@tauri-apps/api/core'

export async function openExternalUrl(url: string) {
  await invoke('open_external_url', { url })
}
