'use client'

import { useI18n } from '@/lib/i18n'

export function LanguageToggle() {
  const { locale, setLocale } = useI18n()

  return (
    <button
      onClick={() => setLocale(locale === 'zh' ? 'en' : 'zh')}
      className="btn btn-ghost btn-sm rounded-lg px-2 font-medium text-xs opacity-60 hover:opacity-100 transition-opacity"
      aria-label="切换语言 / Switch Language"
    >
      {locale === 'zh' ? 'EN' : '中文'}
    </button>
  )
}
