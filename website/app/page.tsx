'use client'

import { useEffect, useState } from 'react'
import { ThemeToggle } from '@/components/ThemeToggle'
import { LanguageToggle } from '@/components/LanguageToggle'
import { DownloadButton, DownloadButtonInverse, GITHUB_URL, RELEASES_URL } from '@/components/DownloadButton'
import { useI18n } from '@/lib/i18n'

// ─── Brand Logo ────────────────────────────────────────────────────────────────
function HubLogo({ size = 36 }: { size?: number }) {
  return (
    <svg width={size} height={size} viewBox="0 0 1024 1024" fill="none">
      <path
        d="M195.2 534.4l147.2-147.2c73.6-1.6 140.8 28.8 203.2 91.2 62.4 62.4 92.8 129.6 91.2 203.2l-147.2 147.2c-81.6 81.6-212.8 81.6-294.4 0s-81.6-212.8 0-294.4z"
        fill="currentColor" className="text-primary"
      />
      <path
        d="M217.6 556.8c-68.8 68.8-68.8 180.8 0 249.6s180.8 68.8 249.6 0l339.2-339.2c68.8-68.8 68.8-180.8 0-249.6s-180.8-68.8-249.6 0L217.6 556.8z m-22.4-22.4l339.2-339.2c81.6-81.6 212.8-81.6 294.4 0s81.6 212.8 0 294.4L489.6 828.8c-81.6 81.6-212.8 81.6-294.4 0s-81.6-212.8 0-294.4z"
        fill="currentColor" className="text-base-content"
      />
      <path
        d="M590.4 433.6c-12.8 12.8-32 12.8-44.8 0-12.8-12.8-12.8-32 0-44.8 12.8-12.8 32-12.8 44.8 0 12.8 11.2 12.8 32 0 44.8z m136 67.2c-12.8 12.8-32 12.8-44.8 0s-12.8-32 0-44.8 32-12.8 44.8 0 12.8 32 0 44.8z m17.6-176c-9.6 9.6-24 9.6-33.6 0-9.6-9.6-9.6-24 0-33.6 9.6-9.6 24-9.6 33.6 0s9.6 24 0 33.6z"
        fill="currentColor" className="text-primary"
      />
    </svg>
  )
}

// ─── Provider Icons ─────────────────────────────────────────────────────────────
function ClaudeIcon({ size = 24 }: { size?: number }) {
  const inner = Math.round(size * 0.64)
  return (
    <span className="provider-icon-claude" style={{
      display: 'inline-flex', alignItems: 'center', justifyContent: 'center',
      width: size, height: size, borderRadius: '25%', flexShrink: 0,
    }}>
      {/* eslint-disable-next-line @next/next/no-img-element */}
      <img src="/claude.svg" alt="Claude" width={inner} height={inner} />
    </span>
  )
}

function CodexIcon({ size = 24 }: { size?: number }) {
  const inner = Math.round(size * 0.64)
  return (
    <span className="provider-icon-codex" style={{
      display: 'inline-flex', alignItems: 'center', justifyContent: 'center',
      width: size, height: size, borderRadius: '25%', flexShrink: 0,
    }}>
      {/* eslint-disable-next-line @next/next/no-img-element */}
      <img src="/openai.svg" alt="Codex" width={inner} height={inner} />
    </span>
  )
}

function GeminiIcon({ size = 24 }: { size?: number }) {
  const inner = Math.round(size * 0.64)
  return (
    <span className="provider-icon-gemini" style={{
      display: 'inline-flex', alignItems: 'center', justifyContent: 'center',
      width: size, height: size, borderRadius: '25%', flexShrink: 0,
    }}>
      {/* eslint-disable-next-line @next/next/no-img-element */}
      <img src="/gemini.svg" alt="Gemini" width={inner} height={inner} />
    </span>
  )
}

// ─── Quota Bar ──────────────────────────────────────────────────────────────────
function QuotaBar({ label, pct, color }: { label: string; pct: number; color: string }) {
  return (
    <div className="space-y-1">
      <div className="flex justify-between text-xs opacity-70">
        <span>{label}</span>
        <span>{pct}%</span>
      </div>
      <div className="h-1.5 rounded-full bg-base-300 overflow-hidden">
        <div className="h-full rounded-full" style={{ width: `${pct}%`, background: color }} />
      </div>
    </div>
  )
}

// ─── Hero Provider Card ─────────────────────────────────────────────────────────
function ProviderCard({
  icon, name, account, quotas, active, className, floatClass,
}: {
  icon: React.ReactNode; name: string; account: string
  quotas: { label: string; pct: number; color: string }[]
  active?: boolean; className?: string; floatClass?: string
}) {
  const { t } = useI18n()
  return (
    <div className={`
      absolute rounded-2xl p-4 w-56 shadow-xl border bg-base-100 border-base-200
      ${active ? 'ring-2 ring-primary shadow-primary/20' : ''}
      ${floatClass ?? ''} ${className ?? ''}
    `}>
      <div className="flex items-center gap-2.5 mb-3">
        {icon}
        <div>
          <div className="font-display font-semibold text-sm leading-tight">{name}</div>
          <div className="text-xs opacity-50 truncate max-w-[120px]">{account}</div>
        </div>
        {active && (
          <div className="ml-auto flex items-center gap-1">
            <span className="w-2 h-2 rounded-full bg-success animate-pulse" />
            <span className="text-[10px] text-success font-medium">{t.hero.activeLabel}</span>
          </div>
        )}
      </div>
      <div className="space-y-2">
        {quotas.map((q) => <QuotaBar key={q.label} {...q} />)}
      </div>
    </div>
  )
}

// ─── Feature Card ───────────────────────────────────────────────────────────────
function FeatureCard({ icon, title, desc, large }: {
  icon: React.ReactNode; title: string; desc: string; large?: boolean
}) {
  return (
    <div className={`
      rounded-3xl p-7 flex flex-col gap-4 border border-base-200 bg-base-100
      transition-all duration-300 hover:border-primary/40 hover:shadow-lg hover:shadow-primary/10 group
      ${large ? 'md:col-span-2' : ''}
    `}>
      <div className="w-12 h-12 rounded-2xl flex items-center justify-center bg-base-200 group-hover:bg-primary/10 transition-colors duration-300">
        {icon}
      </div>
      <div>
        <h3 className="font-display font-bold text-xl mb-2">{title}</h3>
        <p className="text-sm leading-relaxed opacity-60">{desc}</p>
      </div>
    </div>
  )
}

// ─── Step ───────────────────────────────────────────────────────────────────────
function Step({ num, title, desc, last }: { num: string; title: string; desc: string; last?: boolean }) {
  return (
    <div className="flex gap-5">
      <div className="flex flex-col items-center">
        <div className="w-10 h-10 rounded-full bg-primary text-primary-content flex items-center justify-center font-display font-bold text-sm flex-shrink-0">
          {num}
        </div>
        {!last && <div className="w-px flex-1 bg-base-300 mt-2" />}
      </div>
      <div className="pb-8">
        <h4 className="font-display font-bold text-lg mb-1">{title}</h4>
        <p className="text-sm opacity-60 leading-relaxed">{desc}</p>
      </div>
    </div>
  )
}

// ─── Main Page ──────────────────────────────────────────────────────────────────
export default function HomePage() {
  const { t } = useI18n()
  const githubUrl = GITHUB_URL
  const releaseUrl = RELEASES_URL

  // Dynamic version from GitHub API
  const [version, setVersion] = useState('v0.3.5')
  useEffect(() => {
    fetch('https://api.github.com/repos/murongg/ai-accounts-hub/releases/latest', {
      headers: { Accept: 'application/vnd.github+json' },
    })
      .then((r) => r.json())
      .then((d) => { if (d.tag_name) setVersion(d.tag_name) })
      .catch(() => {})
  }, [])

  const featureIcons = [
    <svg key="users" viewBox="0 0 24 24" width="22" height="22" fill="none" stroke="currentColor" strokeWidth="2"><path d="M17 21v-2a4 4 0 00-4-4H5a4 4 0 00-4 4v2"/><circle cx="9" cy="7" r="4"/><path d="M23 21v-2a4 4 0 00-3-3.87M16 3.13a4 4 0 010 7.75"/></svg>,
    <svg key="switch" viewBox="0 0 24 24" width="22" height="22" fill="none" stroke="currentColor" strokeWidth="2"><polyline points="16 3 21 3 21 8"/><line x1="4" y1="20" x2="21" y2="3"/><polyline points="21 16 21 21 16 21"/><line x1="15" y1="15" x2="21" y2="21"/></svg>,
    <svg key="bar" viewBox="0 0 24 24" width="22" height="22" fill="none" stroke="currentColor" strokeWidth="2"><path d="M18 20V10M12 20V4M6 20v-6"/></svg>,
    <svg key="menu" viewBox="0 0 24 24" width="22" height="22" fill="none" stroke="currentColor" strokeWidth="2"><rect x="3" y="3" width="18" height="18" rx="2"/><path d="M3 9h18M9 21V9"/></svg>,
    <svg key="sync" viewBox="0 0 24 24" width="22" height="22" fill="none" stroke="currentColor" strokeWidth="2"><polyline points="23 4 23 10 17 10"/><path d="M20.49 15a9 9 0 11-2.12-9.36L23 10"/></svg>,
    <svg key="update" viewBox="0 0 24 24" width="22" height="22" fill="none" stroke="currentColor" strokeWidth="2"><path d="M21 2l-2 2m-7.61 7.61a5.5 5.5 0 11-7.778 7.778 5.5 5.5 0 017.777-7.777zm0 0L15.5 7.5m0 0l3 3L22 7l-3-3m-3.5 3.5L19 4"/></svg>,
  ]

  return (
    <div className="min-h-screen bg-base-100 text-base-content overflow-x-hidden">

      {/* ── Navbar ─────────────────────────────────────────────────────────── */}
      <header className="sticky top-0 z-50 border-b border-base-200 bg-base-100/80 backdrop-blur-xl">
        <div className="max-w-6xl mx-auto px-5 h-16 flex items-center gap-4">
          <a href="/" className="flex items-center gap-2.5 flex-shrink-0">
            <HubLogo size={28} />
            <span className="font-display font-bold text-base hidden sm:block">AI Accounts Hub</span>
          </a>

          <nav className="hidden md:flex items-center gap-1 ml-6 text-sm">
            {([
              ['#features', t.nav.features],
              ['#providers', t.nav.providers],
              ['#howto', t.nav.howto],
              ['#download', t.nav.download],
            ] as [string, string][]).map(([href, label]) => (
              <a key={href} href={href}
                className="px-3 py-1.5 rounded-lg opacity-60 hover:opacity-100 hover:bg-base-200 transition-all duration-200">
                {label}
              </a>
            ))}
          </nav>

          <div className="ml-auto flex items-center gap-1">
            <LanguageToggle />
            <ThemeToggle />
            <a href={githubUrl} target="_blank" rel="noopener noreferrer"
              className="btn btn-ghost btn-sm btn-circle" aria-label="GitHub">
              <svg viewBox="0 0 24 24" width="18" height="18" fill="currentColor" className="opacity-70">
                <path d="M12 2C6.477 2 2 6.484 2 12.017c0 4.425 2.865 8.18 6.839 9.504.5.092.682-.217.682-.483 0-.237-.008-.868-.013-1.703-2.782.605-3.369-1.343-3.369-1.343-.454-1.158-1.11-1.466-1.11-1.466-.908-.62.069-.608.069-.608 1.003.07 1.531 1.032 1.531 1.032.892 1.53 2.341 1.088 2.91.832.092-.647.35-1.088.636-1.338-2.22-.253-4.555-1.113-4.555-4.951 0-1.093.39-1.988 1.029-2.688-.103-.253-.446-1.272.098-2.65 0 0 .84-.27 2.75 1.026A9.564 9.564 0 0112 6.844c.85.004 1.705.115 2.504.337 1.909-1.296 2.747-1.027 2.747-1.027.546 1.379.202 2.398.1 2.651.64.7 1.028 1.595 1.028 2.688 0 3.848-2.339 4.695-4.566 4.943.359.309.678.92.678 1.855 0 1.338-.012 2.419-.012 2.747 0 .268.18.58.688.482A10.019 10.019 0 0022 12.017C22 6.484 17.522 2 12 2z" />
              </svg>
            </a>
          </div>
        </div>
      </header>

      {/* ── Hero ───────────────────────────────────────────────────────────── */}
      <section className="relative min-h-[calc(100vh-4rem)] flex items-center overflow-hidden">
        <div className="absolute inset-0 pointer-events-none">
          <div className="absolute top-1/4 right-0 w-96 h-96 rounded-full bg-primary/10 blur-3xl translate-x-1/2" />
          <div className="absolute bottom-0 left-1/4 w-64 h-64 rounded-full bg-primary/5 blur-2xl" />
        </div>

        <div className="max-w-6xl mx-auto px-5 py-20 grid md:grid-cols-2 gap-12 items-center w-full">
          <div className="animate-fade-in-up">
            <div className="badge badge-primary badge-outline mb-6 gap-1.5 text-xs font-medium py-1 px-3 rounded-full">
              <span className="w-1.5 h-1.5 rounded-full bg-primary" />
              {t.hero.badge.replace('v0.3.5', version)}
            </div>

            <h1 className="font-display font-extrabold leading-[1.05] mb-5"
              style={{ fontSize: 'clamp(2.8rem, 6vw, 4.5rem)' }}>
              {t.hero.titleLine1}<br />
              <span className="text-primary">{t.hero.titleLine2}</span>
            </h1>

            <p className="text-base leading-relaxed opacity-60 mb-8 max-w-md"
              style={{ fontSize: 'clamp(1rem, 2vw, 1.125rem)' }}>
              {t.hero.desc}
            </p>

            <div className="flex flex-wrap gap-3">
              <DownloadButton />
              <a href={githubUrl} target="_blank" rel="noopener noreferrer"
                className="btn btn-ghost rounded-2xl gap-2 px-6 border border-base-300">
                <svg viewBox="0 0 24 24" width="16" height="16" fill="currentColor">
                  <path d="M12 2C6.477 2 2 6.484 2 12.017c0 4.425 2.865 8.18 6.839 9.504.5.092.682-.217.682-.483 0-.237-.008-.868-.013-1.703-2.782.605-3.369-1.343-3.369-1.343-.454-1.158-1.11-1.466-1.11-1.466-.908-.62.069-.608.069-.608 1.003.07 1.531 1.032 1.531 1.032.892 1.53 2.341 1.088 2.91.832.092-.647.35-1.088.636-1.338-2.22-.253-4.555-1.113-4.555-4.951 0-1.093.39-1.988 1.029-2.688-.103-.253-.446-1.272.098-2.65 0 0 .84-.27 2.75 1.026A9.564 9.564 0 0112 6.844c.85.004 1.705.115 2.504.337 1.909-1.296 2.747-1.027 2.747-1.027.546 1.379.202 2.398.1 2.651.64.7 1.028 1.595 1.028 2.688 0 3.848-2.339 4.695-4.566 4.943.359.309.678.92.678 1.855 0 1.338-.012 2.419-.012 2.747 0 .268.18.58.688.482A10.019 10.019 0 0022 12.017C22 6.484 17.522 2 12 2z" />
                </svg>
                {t.hero.ctaGithub}
              </a>
            </div>

            <div className="flex gap-8 mt-10 pt-8 border-t border-base-200">
              {[
                { val: '3', label: t.hero.statProviders },
                { val: '∞', label: t.hero.statAccounts },
                { val: '1', label: t.hero.statSwitch },
              ].map(({ val, label }) => (
                <div key={label}>
                  <div className="font-display font-extrabold text-2xl text-primary">{val}</div>
                  <div className="text-xs opacity-50 mt-0.5">{label}</div>
                </div>
              ))}
            </div>
          </div>

          {/* Floating provider cards */}
          <div className="relative h-[420px] hidden md:block animate-fade-in-up animate-delay-200">
            <ProviderCard
              icon={<ClaudeIcon size={32} />} name="Claude" account="user@company.com" active
              quotas={[
                { label: t.providers.quotaLabels.claudeSession, pct: 68, color: '#D97757' },
                { label: t.providers.quotaLabels.claudeWeekly, pct: 42, color: '#D97757' },
              ]}
              className="top-0 left-0 z-30" floatClass="animate-float-0"
            />
            <ProviderCard
              icon={<CodexIcon size={32} />} name="Codex" account="work@example.io"
              quotas={[
                { label: t.providers.quotaLabels.codex5h, pct: 85, color: '#10A37F' },
                { label: t.providers.quotaLabels.codexWeekly, pct: 60, color: '#10A37F' },
              ]}
              className="top-24 right-0 z-20" floatClass="animate-float-1"
            />
            <ProviderCard
              icon={<GeminiIcon size={32} />} name="Gemini" account="personal@gmail.com"
              quotas={[
                { label: t.providers.quotaLabels.geminiPro, pct: 30, color: '#4285F4' },
                { label: t.providers.quotaLabels.geminiFlash, pct: 77, color: '#4285F4' },
              ]}
              className="bottom-0 left-10 z-10" floatClass="animate-float-2"
            />
            <div className="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 z-40">
              <div className="w-16 h-16 rounded-2xl bg-base-100 border-2 border-primary shadow-xl shadow-primary/30 flex items-center justify-center">
                <HubLogo size={36} />
              </div>
            </div>
            <svg className="absolute inset-0 w-full h-full pointer-events-none" style={{ zIndex: 5 }}>
              <line x1="112" y1="70" x2="50%" y2="50%" stroke="currentColor" strokeDasharray="4 4" opacity="0.2" strokeWidth="1.5" className="text-primary" />
              <line x1="88%" y1="130" x2="50%" y2="50%" stroke="currentColor" strokeDasharray="4 4" opacity="0.2" strokeWidth="1.5" className="text-primary" />
              <line x1="112" y1="88%" x2="50%" y2="50%" stroke="currentColor" strokeDasharray="4 4" opacity="0.2" strokeWidth="1.5" className="text-primary" />
            </svg>
          </div>
        </div>
      </section>

      {/* ── Features ───────────────────────────────────────────────────────── */}
      <section id="features" className="py-24 bg-base-200/40">
        <div className="max-w-6xl mx-auto px-5">
          <div className="mb-14">
            <div className="text-xs font-medium text-primary uppercase tracking-widest mb-3">{t.features.badge}</div>
            <h2 className="font-display font-extrabold" style={{ fontSize: 'clamp(2rem, 4vw, 3rem)' }}>
              {t.features.title}
            </h2>
            <p className="mt-3 opacity-60 max-w-xl">{t.features.desc}</p>
          </div>

          <div className="grid md:grid-cols-3 gap-4">
            <FeatureCard large icon={featureIcons[0]} title={t.features.items[0].title} desc={t.features.items[0].desc} />
            {t.features.items.slice(1).map((item, i) => (
              <FeatureCard key={i} icon={featureIcons[i + 1]} title={item.title} desc={item.desc} />
            ))}
          </div>
        </div>
      </section>

      {/* ── Providers ─────────────────────────────────────────────────────── */}
      <section id="providers" className="py-24">
        <div className="max-w-6xl mx-auto px-5">
          <div className="mb-14">
            <div className="text-xs font-medium text-primary uppercase tracking-widest mb-3">{t.providers.badge}</div>
            <h2 className="font-display font-extrabold" style={{ fontSize: 'clamp(2rem, 4vw, 3rem)' }}>
              {t.providers.title}
            </h2>
            <p className="mt-3 opacity-60 max-w-xl">{t.providers.desc}</p>
          </div>

          <div className="grid md:grid-cols-3 gap-5">
            {/* Claude */}
            <div className="rounded-3xl p-7 border border-base-200 bg-base-100 hover:border-[#D97757]/40 transition-all duration-300 hover:shadow-lg hover:shadow-[#D97757]/10">
              <div className="flex items-center gap-3 mb-6">
                <ClaudeIcon size={48} />
                <div>
                  <h3 className="font-display font-bold text-xl">Claude</h3>
                  <div className="text-xs opacity-50">by Anthropic</div>
                </div>
              </div>
              <div className="space-y-3 mb-6">
                {[
                  { label: t.providers.quotaLabels.claudeSession, pct: 68 },
                  { label: t.providers.quotaLabels.claudeWeekly, pct: 42 },
                ].map((q) => <QuotaBar key={q.label} label={q.label} pct={q.pct} color="#D97757" />)}
              </div>
              <div className="text-xs opacity-50 leading-relaxed border-t border-base-200 pt-4">
                {t.providers.claudeNote} <code className="bg-base-200 px-1 rounded">/usage</code>
              </div>
            </div>

            {/* Codex */}
            <div className="rounded-3xl p-7 border border-base-200 bg-base-100 hover:border-[#10A37F]/40 transition-all duration-300 hover:shadow-lg hover:shadow-[#10A37F]/10">
              <div className="flex items-center gap-3 mb-6">
                <CodexIcon size={48} />
                <div>
                  <h3 className="font-display font-bold text-xl">Codex</h3>
                  <div className="text-xs opacity-50">by OpenAI</div>
                </div>
              </div>
              <div className="space-y-3 mb-6">
                {[
                  { label: t.providers.quotaLabels.codex5h, pct: 85 },
                  { label: t.providers.quotaLabels.codexWeekly, pct: 60 },
                ].map((q) => <QuotaBar key={q.label} label={q.label} pct={q.pct} color="#10A37F" />)}
              </div>
              <div className="text-xs opacity-50 leading-relaxed border-t border-base-200 pt-4">
                {t.providers.codexNote}
              </div>
            </div>

            {/* Gemini */}
            <div className="rounded-3xl p-7 border border-base-200 bg-base-100 hover:border-[#4285F4]/40 transition-all duration-300 hover:shadow-lg hover:shadow-[#4285F4]/10">
              <div className="flex items-center gap-3 mb-6">
                <GeminiIcon size={48} />
                <div>
                  <h3 className="font-display font-bold text-xl">Gemini</h3>
                  <div className="text-xs opacity-50">by Google</div>
                </div>
              </div>
              <div className="space-y-3 mb-6">
                {[
                  { label: t.providers.quotaLabels.geminiPro, pct: 30 },
                  { label: t.providers.quotaLabels.geminiFlash, pct: 77 },
                  { label: t.providers.quotaLabels.geminiFlashLite, pct: 92 },
                ].map((q) => <QuotaBar key={q.label} label={q.label} pct={q.pct} color="#4285F4" />)}
              </div>
              <div className="text-xs opacity-50 leading-relaxed border-t border-base-200 pt-4">
                {t.providers.geminiNote}
              </div>
            </div>
          </div>

          {/* Support table */}
          <div className="mt-8 rounded-3xl border border-base-200 overflow-hidden">
            <table className="table table-sm w-full">
              <thead>
                <tr className="border-base-200">
                  {[t.providers.table.provider, t.providers.table.multiAccount, t.providers.table.switchLogin, t.providers.table.quota].map((h) => (
                    <th key={h} className="bg-base-200/60 font-display py-4 px-6">{h}</th>
                  ))}
                </tr>
              </thead>
              <tbody>
                {[
                  { name: 'Codex', icon: <CodexIcon size={20} /> },
                  { name: 'Claude', icon: <ClaudeIcon size={20} /> },
                  { name: 'Gemini', icon: <GeminiIcon size={20} /> },
                ].map(({ name, icon }) => (
                  <tr key={name} className="border-base-200">
                    <td className="py-4 px-6">
                      <div className="flex items-center gap-2.5 font-medium">{icon} {name}</div>
                    </td>
                    {[0, 1, 2].map((i) => (
                      <td key={i} className="py-4 px-6">
                        <span className="badge badge-success badge-sm gap-1">
                          <svg viewBox="0 0 24 24" width="10" height="10" fill="none" stroke="currentColor" strokeWidth="3"><polyline points="20 6 9 17 4 12"/></svg>
                          {t.providers.table.supported}
                        </span>
                      </td>
                    ))}
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </div>
      </section>

      {/* ── How It Works ──────────────────────────────────────────────────── */}
      <section id="howto" className="py-24 bg-base-200/40">
        <div className="max-w-6xl mx-auto px-5">
          <div className="grid md:grid-cols-2 gap-16 items-center">
            <div>
              <div className="text-xs font-medium text-primary uppercase tracking-widest mb-3">{t.howto.badge}</div>
              <h2 className="font-display font-extrabold mb-10" style={{ fontSize: 'clamp(2rem, 4vw, 3rem)' }}>
                {t.howto.title}
              </h2>
              {t.howto.steps.map((step, i) => (
                <Step key={i} num={String(i + 1)} title={step.title} desc={step.desc} last={i === t.howto.steps.length - 1} />
              ))}
            </div>

            {/* App window mockup */}
            <div className="relative">
              <div className="rounded-3xl border border-base-200 bg-base-100 shadow-2xl overflow-hidden">
                <div className="flex items-center gap-2 px-4 py-3 border-b border-base-200 bg-base-200/50">
                  <div className="flex gap-1.5">
                    <div className="w-3 h-3 rounded-full bg-error/70" />
                    <div className="w-3 h-3 rounded-full bg-warning/70" />
                    <div className="w-3 h-3 rounded-full bg-success/70" />
                  </div>
                  <div className="flex-1 flex justify-center">
                    <div className="flex gap-1">
                      {['Codex', 'Claude', 'Gemini'].map((tab, i) => (
                        <div key={tab} className={`text-xs px-3 py-1 rounded-lg ${i === 1 ? 'bg-primary text-primary-content font-medium' : 'opacity-40'}`}>
                          {tab}
                        </div>
                      ))}
                    </div>
                  </div>
                </div>
                <div className="p-4 space-y-2">
                  {[
                    { account: 'work@company.com', active: true, pct: 68 },
                    { account: 'personal@gmail.com', active: false, pct: 95 },
                    { account: 'dev@project.io', active: false, pct: 20 },
                  ].map(({ account, active, pct }, i) => (
                    <div key={i} className={`flex items-center gap-3 p-3 rounded-2xl border transition-all ${active ? 'border-primary bg-primary/10' : 'border-base-200 bg-base-100'}`}>
                      <ClaudeIcon size={32} />
                      <div className="flex-1 min-w-0">
                        <div className={`text-sm font-medium truncate ${active ? 'text-primary' : ''}`}>{account}</div>
                        <div className="flex items-center gap-1.5 mt-1.5">
                          <div className="h-1 w-16 bg-base-300 rounded-full overflow-hidden">
                            <div className="h-full bg-[#D97757] rounded-full" style={{ width: `${pct}%` }} />
                          </div>
                          <span className="text-[10px] opacity-50">{pct}%</span>
                        </div>
                      </div>
                      {active ? (
                        <div className="flex items-center gap-1.5 flex-shrink-0">
                          <span className="w-2 h-2 rounded-full bg-success" />
                          <span className="text-[10px] text-success font-medium">{t.howto.mockupActive}</span>
                        </div>
                      ) : (
                        <button className="btn btn-xs btn-ghost rounded-lg opacity-40 hover:opacity-100 flex-shrink-0">
                          {t.howto.mockupSwitch}
                        </button>
                      )}
                    </div>
                  ))}
                </div>
              </div>
              <div className="absolute -top-4 -right-4 bg-base-100 border border-base-200 rounded-2xl px-4 py-2.5 shadow-lg flex items-center gap-2 text-sm">
                <HubLogo size={20} />
                <span className="font-medium">{t.howto.menubarLabel}</span>
              </div>
            </div>
          </div>
        </div>
      </section>

      {/* ── Download CTA ──────────────────────────────────────────────────── */}
      <section id="download" className="py-24">
        <div className="max-w-6xl mx-auto px-5">
          <div className="download-cta-box rounded-3xl p-12 md:p-16 text-center relative overflow-hidden">
            <div className="absolute top-0 right-0 w-64 h-64 rounded-full opacity-10 blur-3xl translate-x-1/2 -translate-y-1/2 pointer-events-none bg-base-content" />
            <div className="absolute bottom-0 left-0 w-48 h-48 rounded-full opacity-10 blur-2xl -translate-x-1/2 translate-y-1/2 pointer-events-none bg-base-content" />

            <div className="relative z-10">
              <div className="flex justify-center mb-6">
                <div className="w-16 h-16 rounded-2xl cta-icon-bg flex items-center justify-center">
                  <HubLogo size={40} />
                </div>
              </div>
              <h2 className="font-display font-extrabold cta-title mb-4"
                style={{ fontSize: 'clamp(2rem, 5vw, 3.5rem)' }}>
                {t.cta.title}
              </h2>
              <p className="cta-subtitle mb-8 max-w-md mx-auto">{t.cta.desc}</p>

              <div className="flex flex-wrap justify-center gap-4">
                <DownloadButtonInverse />
                <a href={githubUrl} target="_blank" rel="noopener noreferrer"
                  className="btn btn-outline rounded-2xl gap-2 px-8 cta-outline-btn">
                  <svg viewBox="0 0 24 24" width="18" height="18" fill="currentColor">
                    <path d="M12 2C6.477 2 2 6.484 2 12.017c0 4.425 2.865 8.18 6.839 9.504.5.092.682-.217.682-.483 0-.237-.008-.868-.013-1.703-2.782.605-3.369-1.343-3.369-1.343-.454-1.158-1.11-1.466-1.11-1.466-.908-.62.069-.608.069-.608 1.003.07 1.531 1.032 1.531 1.032.892 1.53 2.341 1.088 2.91.832.092-.647.35-1.088.636-1.338-2.22-.253-4.555-1.113-4.555-4.951 0-1.093.39-1.988 1.029-2.688-.103-.253-.446-1.272.098-2.65 0 0 .84-.27 2.75 1.026A9.564 9.564 0 0112 6.844c.85.004 1.705.115 2.504.337 1.909-1.296 2.747-1.027 2.747-1.027.546 1.379.202 2.398.1 2.651.64.7 1.028 1.595 1.028 2.688 0 3.848-2.339 4.695-4.566 4.943.359.309.678.92.678 1.855 0 1.338-.012 2.419-.012 2.747 0 .268.18.58.688.482A10.019 10.019 0 0022 12.017C22 6.484 17.522 2 12 2z" />
                  </svg>
                  {t.cta.github}
                </a>
              </div>

              <div className="mt-8 flex justify-center gap-6 cta-bullets text-sm">
                {t.cta.bullets.map((b) => <span key={b}>✓ {b}</span>)}
              </div>
            </div>
          </div>
        </div>
      </section>

      {/* ── Footer ────────────────────────────────────────────────────────── */}
      <footer className="border-t border-base-200 py-10">
        <div className="max-w-6xl mx-auto px-5">
          <div className="flex flex-col md:flex-row items-center justify-between gap-4">
            <div className="flex items-center gap-2.5">
              <HubLogo size={22} />
              <span className="font-display font-semibold text-sm">AI Accounts Hub</span>
              <span className="text-xs opacity-40 ml-1">{version}</span>
            </div>
            <div className="flex items-center gap-6 text-sm opacity-50">
              <a href={`${githubUrl}/blob/main/LICENSE`} target="_blank" rel="noopener noreferrer" className="hover:opacity-100 transition-opacity">{t.footer.license}</a>
              <a href={`${githubUrl}/releases`} target="_blank" rel="noopener noreferrer" className="hover:opacity-100 transition-opacity">{t.footer.changelog}</a>
              <a href={`${githubUrl}/issues`} target="_blank" rel="noopener noreferrer" className="hover:opacity-100 transition-opacity">{t.footer.feedback}</a>
            </div>
            <div className="text-xs opacity-30">{t.footer.builtWith}</div>
          </div>
        </div>
      </footer>
    </div>
  )
}
