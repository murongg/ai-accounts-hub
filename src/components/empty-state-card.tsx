export function EmptyStateCard({
  title,
  description,
}: {
  title: string;
  description: string;
}) {
  return (
    <div className="flex min-h-[420px] items-center justify-center">
      <div className="card w-full max-w-[560px] rounded-[28px] bg-transparent shadow-none">
        <div className="card-body items-center px-8 py-10 text-center md:px-10 md:py-12">
          <EmptyAccountsIllustration />
          <p className="text-lg font-semibold text-base-content">{title}</p>
          <p className="mt-2 text-sm leading-6 text-base-content/60">{description}</p>
        </div>
      </div>
    </div>
  );
}

function EmptyAccountsIllustration() {
  return (
    <svg
      viewBox="0 0 320 220"
      fill="none"
      aria-hidden="true"
      className="mx-auto mb-5 w-full max-w-[260px]"
    >
      <rect x="8" y="8" width="304" height="204" rx="32" fill="var(--color-base-200)" opacity="0.72" />
      <ellipse cx="160" cy="188" rx="84" ry="16" fill="var(--color-base-300)" opacity="0.6" />

      <rect
        x="62"
        y="54"
        width="114"
        height="104"
        rx="24"
        fill="var(--color-base-100)"
        stroke="var(--color-base-content)"
        strokeOpacity="0.16"
        strokeWidth="2"
      />
      <rect
        x="144"
        y="36"
        width="114"
        height="120"
        rx="28"
        fill="var(--color-base-100)"
        stroke="var(--color-primary)"
        strokeOpacity="0.28"
        strokeWidth="2"
      />

      <rect x="166" y="58" width="70" height="12" rx="6" fill="var(--color-primary)" opacity="0.18" />
      <rect x="166" y="80" width="42" height="10" rx="5" fill="var(--color-base-content)" opacity="0.12" />
      <rect x="166" y="128" width="70" height="10" rx="5" fill="var(--color-base-content)" opacity="0.12" />

      <circle cx="201" cy="108" r="24" fill="var(--color-base-200)" />
      <circle
        cx="201"
        cy="108"
        r="16"
        stroke="var(--color-primary)"
        strokeWidth="8"
        strokeDasharray="72 28"
        strokeLinecap="round"
      />

      <circle cx="107" cy="92" r="17" fill="var(--color-base-200)" />
      <path
        d="M107 84C110.866 84 114 87.134 114 91C114 94.866 110.866 98 107 98C103.134 98 100 94.866 100 91C100 87.134 103.134 84 107 84Z"
        fill="var(--color-base-content)"
        opacity="0.36"
      />
      <path
        d="M92 119C92 111.82 97.82 106 105 106H109C116.18 106 122 111.82 122 119V121H92V119Z"
        fill="var(--color-base-content)"
        opacity="0.18"
      />
      <path
        d="M247 82L258 93L247 104"
        stroke="var(--color-secondary)"
        strokeWidth="6"
        strokeLinecap="round"
        strokeLinejoin="round"
      />
    </svg>
  );
}
