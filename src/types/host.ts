export interface HostEntry {
  id: string
  ip: string
  hostname: string
  comment: string
  enabled: boolean
}

export interface HostGroup {
  id: string
  name: string
  enabled: boolean
  entries: HostEntry[]
  createdAt: string
  updatedAt: string
}

export interface NewEntryForm {
  ip: string
  hostname: string
  comment: string
}

export interface BackupInfo {
  name: string
  path: string
  modified: string
  size: number
}
