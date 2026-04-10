'use client'

import { createContext, useContext, useState, useEffect } from 'react'
import { zh } from '@/messages/zh'
import { en } from '@/messages/en'
import type { Messages } from '@/messages/zh'

export type Locale = 'zh' | 'en'

const messages: Record<Locale, Messages> = { zh, en }

interface I18nContextValue {
  locale: Locale
  t: Messages
  setLocale: (l: Locale) => void
}

const I18nContext = createContext<I18nContextValue>({
  locale: 'zh',
  t: zh,
  setLocale: () => {},
})

export function I18nProvider({ children }: { children: React.ReactNode }) {
  const [locale, setLocaleState] = useState<Locale>('zh')

  useEffect(() => {
    try {
      const saved = localStorage.getItem('locale') as Locale | null
      if (saved === 'zh' || saved === 'en') {
        setLocaleState(saved)
      } else if (navigator.language.toLowerCase().startsWith('en')) {
        setLocaleState('en')
      }
    } catch {
      // ignore
    }
  }, [])

  const setLocale = (l: Locale) => {
    setLocaleState(l)
    try { localStorage.setItem('locale', l) } catch { /* ignore */ }
  }

  return (
    <I18nContext.Provider value={{ locale, t: messages[locale], setLocale }}>
      {children}
    </I18nContext.Provider>
  )
}

export const useI18n = () => useContext(I18nContext)
