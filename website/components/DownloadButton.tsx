'use client'

import { useState, useEffect } from 'react'

export const GITHUB_URL = 'https://github.com/murongg/ai-accounts-hub'
export const RELEASES_URL = `${GITHUB_URL}/releases/latest`

const RELEASES_API = 'https://api.github.com/repos/murongg/ai-accounts-hub/releases/latest'

type Arch = 'arm' | 'x64'
type Platform = 'macos' | 'windows' | 'linux' | 'other'

function detectPlatform(): Platform {
  if (typeof navigator === 'undefined') return 'other'
  const ua = navigator.userAgent
  if (/Mac/i.test(ua)) return 'macos'
  if (/Win/i.test(ua)) return 'windows'
  if (/Linux/i.test(ua)) return 'linux'
  return 'other'
}

function detectArch(): Arch {
  // Chrome 90+ exposes architecture via userAgentData
  const uaData = (navigator as unknown as {
    userAgentData?: { getHighEntropyValues?: (h: string[]) => Promise<{ architecture?: string }> }
  }).userAgentData

  if (uaData?.getHighEntropyValues) {
    // Will be resolved async — caller handles this
    return 'arm'
  }
  // Fallback: check for ARM hint in userAgent (Firefox on Apple Silicon may include it)
  if (/arm/i.test(navigator.userAgent)) return 'arm'
  // Default to arm since most modern Macs are Apple Silicon
  return 'arm'
}

async function detectArchAsync(): Promise<Arch> {
  try {
    const uaData = (navigator as unknown as {
      userAgentData?: { getHighEntropyValues?: (h: string[]) => Promise<{ architecture?: string }> }
    }).userAgentData

    if (uaData?.getHighEntropyValues) {
      const data = await uaData.getHighEntropyValues(['architecture'])
      if (data.architecture === 'x86') return 'x64'
      return 'arm'
    }
  } catch {
    // ignore
  }
  return detectArch()
}

interface ReleaseAsset {
  name: string
  browser_download_url: string
}

async function fetchLatestRelease(): Promise<{ version: string; assets: ReleaseAsset[] } | null> {
  try {
    const res = await fetch(RELEASES_API, {
      headers: { Accept: 'application/vnd.github+json' },
      next: { revalidate: 3600 },
    })
    if (!res.ok) return null
    const data = await res.json()
    return { version: data.tag_name, assets: data.assets ?? [] }
  } catch {
    return null
  }
}

function findDmg(assets: ReleaseAsset[], arch: Arch): string | null {
  const pattern = arch === 'arm' ? /aarch64\.dmg$/ : /x64\.dmg$/
  const asset = assets.find((a) => pattern.test(a.name))
  return asset?.browser_download_url ?? null
}

// ── Download Icon ────────────────────────────────────────────────────────────
function DownloadIcon() {
  return (
    <svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor"
      strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round">
      <path d="M21 15v4a2 2 0 01-2 2H5a2 2 0 01-2-2v-4M7 10l5 5 5-5M12 15V3"/>
    </svg>
  )
}

// ── Primary download button (for Hero / Navbar) ───────────────────────────────
export function DownloadButton({ size = 'md' }: { size?: 'sm' | 'md' | 'lg' }) {
  const [href, setHref] = useState<string>(RELEASES_URL)
  const [label, setLabel] = useState('免费下载')
  const [ready, setReady] = useState(false)
  const [isMac, setIsMac] = useState(true)

  useEffect(() => {
    const platform = detectPlatform()
    setIsMac(platform === 'macos')

    async function init() {
      const arch = await detectArchAsync()
      const release = await fetchLatestRelease()

      if (release) {
        const dmg = findDmg(release.assets, arch)
        if (dmg) {
          setHref(dmg)
          const archLabel = arch === 'arm' ? 'Apple Silicon' : 'Intel'
          setLabel(`下载 for macOS (${archLabel})`)
        } else {
          setHref(RELEASES_URL)
          setLabel('前往下载页')
        }
      } else {
        setHref(RELEASES_URL)
        setLabel(platform === 'macos' ? '下载 for macOS' : '前往下载页')
      }
      setReady(true)
    }

    if (platform === 'macos') {
      init()
    } else {
      setLabel('前往下载页')
      setHref(RELEASES_URL)
      setReady(true)
    }
  }, [])

  const sizeClass = size === 'sm' ? 'btn-sm' : size === 'lg' ? 'btn-lg' : ''

  return (
    <div className="flex flex-col gap-1.5 items-start">
      <a
        href={href}
        target={href === RELEASES_URL ? '_blank' : undefined}
        rel="noopener noreferrer"
        className={`btn btn-primary rounded-2xl gap-2 px-6 shadow-lg shadow-primary/30 ${sizeClass} ${!ready ? 'opacity-80' : ''}`}
      >
        <DownloadIcon />
        {label}
      </a>
      {ready && !isMac && (
        <span className="text-[11px] opacity-40 pl-1">此应用仅支持 macOS</span>
      )}
    </div>
  )
}

// ── Inverse button (for CTA section on colored background) ───────────────────
export function DownloadButtonInverse({ size = 'md' }: { size?: 'sm' | 'md' | 'lg' }) {
  const [href, setHref] = useState<string>(RELEASES_URL)
  const [label, setLabel] = useState('下载最新版本')
  const [ready, setReady] = useState(false)
  const [isMac, setIsMac] = useState(true)

  useEffect(() => {
    const platform = detectPlatform()
    setIsMac(platform === 'macos')

    async function init() {
      const arch = await detectArchAsync()
      const release = await fetchLatestRelease()

      if (release) {
        const dmg = findDmg(release.assets, arch)
        if (dmg) {
          setHref(dmg)
          const archLabel = arch === 'arm' ? 'Apple Silicon' : 'Intel'
          setLabel(`下载 for macOS (${archLabel})`)
        } else {
          setHref(RELEASES_URL)
          setLabel('前往下载页')
        }
      } else {
        setHref(RELEASES_URL)
        setLabel(platform === 'macos' ? '下载 for macOS' : '前往下载页')
      }
      setReady(true)
    }

    if (platform === 'macos') {
      init()
    } else {
      setLabel('前往下载页')
      setHref(RELEASES_URL)
      setReady(true)
    }
  }, [])

  const sizeClass = size === 'sm' ? 'btn-sm' : size === 'lg' ? 'btn-lg' : ''

  return (
    <div className="flex flex-col gap-2 items-center">
      <a
        href={href}
        target={href === RELEASES_URL ? '_blank' : undefined}
        rel="noopener noreferrer"
        className={`btn rounded-2xl gap-2 px-8 shadow-xl download-cta-primary-btn ${sizeClass} ${!ready ? 'opacity-80' : ''}`}
      >
        <DownloadIcon />
        {label}
      </a>
      {ready && !isMac && (
        <p className="text-xs text-primary-content/50">此应用仅支持 macOS</p>
      )}
    </div>
  )
}
