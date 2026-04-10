import type { Metadata } from 'next'
import { Bricolage_Grotesque, DM_Sans } from 'next/font/google'
import { I18nProvider } from '@/lib/i18n'
import './globals.css'

const displayFont = Bricolage_Grotesque({
  subsets: ['latin'],
  variable: '--font-display',
  weight: ['400', '500', '600', '700', '800'],
  display: 'swap',
})

const bodyFont = DM_Sans({
  subsets: ['latin'],
  variable: '--font-body',
  weight: ['300', '400', '500'],
  display: 'swap',
})

export const metadata: Metadata = {
  title: 'AI Accounts Hub — 统一管理你的 AI 账号',
  description:
    '一个 macOS 桌面工具，统一管理 Claude、Codex、Gemini 多个账号，一键切换登录态，实时查看配额。',
  keywords: ['AI', 'Claude', 'Codex', 'Gemini', '账号管理', 'macOS', '配额监控'],
}

// Inline script to set theme before paint to avoid flash
const themeScript = `
(function() {
  try {
    var saved = localStorage.getItem('theme');
    if (saved) {
      document.documentElement.setAttribute('data-theme', saved);
    } else if (window.matchMedia('(prefers-color-scheme: dark)').matches) {
      document.documentElement.setAttribute('data-theme', 'luxury');
    } else {
      document.documentElement.setAttribute('data-theme', 'bumblebee');
    }
  } catch(e) {}
})();
`

export default function RootLayout({ children }: { children: React.ReactNode }) {
  return (
    <html lang="zh-CN" data-theme="bumblebee" suppressHydrationWarning>
      <head>
        <script dangerouslySetInnerHTML={{ __html: themeScript }} />
      </head>
      <body className={`${displayFont.variable} ${bodyFont.variable}`}>
        <I18nProvider>{children}</I18nProvider>
      </body>
    </html>
  )
}
