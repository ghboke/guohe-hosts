export interface UpdateCheckResult {
  checked: boolean
  hasUpdate: boolean
  latestVersion: string | null
  latestVersionCode: number | null
  changelog: string | null
  forceUpdate: boolean
  updateUrl: string | null
  minCheckIntervalSeconds: number
}
