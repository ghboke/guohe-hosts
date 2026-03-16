import { ref, computed } from 'vue'
import zhCN from './zh-CN'
import enUS from './en-US'

export type Locale = 'zh-CN' | 'en-US'

type Messages = typeof zhCN

const messages: Record<Locale, Messages> = {
  'zh-CN': zhCN,
  'en-US': enUS,
}

const currentLocale = ref<Locale>('zh-CN')

function getNestedValue(obj: Record<string, unknown>, path: string): string {
  const keys = path.split('.')
  let result: unknown = obj
  for (const key of keys) {
    if (result && typeof result === 'object' && key in result) {
      result = (result as Record<string, unknown>)[key]
    } else {
      return path
    }
  }
  return typeof result === 'string' ? result : path
}

export function useI18n() {
  const locale = computed(() => currentLocale.value)

  function t(key: string, params?: Record<string, string | number>): string {
    let text = getNestedValue(
      messages[currentLocale.value] as unknown as Record<string, unknown>,
      key
    )
    if (params) {
      for (const [k, v] of Object.entries(params)) {
        text = text.replace(`{${k}}`, String(v))
      }
    }
    return text
  }

  function setLocale(loc: Locale) {
    currentLocale.value = loc
  }

  return { t, locale, setLocale }
}
