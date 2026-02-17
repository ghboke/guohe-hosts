export interface AppConfig {
  hostsPath: string
  autoSave: boolean
  theme: 'light' | 'dark' | 'system'
  locale: string
  flushDnsOnSave: boolean
  activeGroupId: string
}
