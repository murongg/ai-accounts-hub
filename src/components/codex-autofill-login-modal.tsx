import { X } from "lucide-react";

import type { I18nMessages } from "../lib/i18n";

interface CodexAutofillLoginModalProps {
  copy: I18nMessages["accounts"]["autofillModal"];
  pendingLabel: string;
  email: string;
  password: string;
  isSubmitting: boolean;
  onEmailChange: (value: string) => void;
  onPasswordChange: (value: string) => void;
  onCancel: () => void;
  onSubmit: () => void;
}

export function CodexAutofillLoginModal({
  copy,
  pendingLabel,
  email,
  password,
  isSubmitting,
  onEmailChange,
  onPasswordChange,
  onCancel,
  onSubmit,
}: CodexAutofillLoginModalProps) {
  const canSubmit = email.trim().length > 0 && password.length > 0 && !isSubmitting;

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/35 px-4">
      <div className="w-full max-w-md rounded-lg border border-base-300 bg-base-100 p-5 shadow-xl">
        <div className="mb-4 flex items-start justify-between gap-4">
          <div>
            <h2 className="text-lg font-semibold text-base-content">{copy.title}</h2>
            <p className="mt-1 text-sm leading-6 text-base-content/60">{copy.description}</p>
          </div>
          <button
            type="button"
            aria-label={copy.cancel}
            className="btn btn-ghost btn-sm h-8 min-h-0 w-8 rounded-lg p-0"
            disabled={isSubmitting}
            onClick={onCancel}
          >
            <X size={16} aria-hidden="true" />
          </button>
        </div>

        <form
          className="grid gap-4"
          onSubmit={(event) => {
            event.preventDefault();
            if (canSubmit) {
              onSubmit();
            }
          }}
        >
          <div className="grid gap-2">
            <label
              htmlFor="codex-autofill-email"
              className="text-sm font-semibold text-base-content/70"
            >
              {copy.emailLabel}
            </label>
            <input
              id="codex-autofill-email"
              type="email"
              value={email}
              placeholder={copy.emailPlaceholder}
              className="input input-bordered h-11 w-full rounded-lg"
              autoComplete="username"
              disabled={isSubmitting}
              onChange={(event) => onEmailChange(event.target.value)}
            />
          </div>

          <div className="grid gap-2">
            <label
              htmlFor="codex-autofill-password"
              className="text-sm font-semibold text-base-content/70"
            >
              {copy.passwordLabel}
            </label>
            <input
              id="codex-autofill-password"
              type="password"
              value={password}
              placeholder={copy.passwordPlaceholder}
              className="input input-bordered h-11 w-full rounded-lg"
              autoComplete="current-password"
              disabled={isSubmitting}
              onChange={(event) => onPasswordChange(event.target.value)}
            />
          </div>

          <div className="rounded-lg border border-info/20 bg-info/10 p-3 text-xs leading-5 text-base-content/70">
            <p>{copy.privacyNote}</p>
            <p className="mt-1">{copy.verificationNote}</p>
          </div>

          <div className="mt-1 flex justify-end gap-2">
            <button
              type="button"
              className="btn btn-ghost btn-sm h-10 rounded-lg px-4"
              disabled={isSubmitting}
              onClick={onCancel}
            >
              {copy.cancel}
            </button>
            <button
              type="submit"
              className="btn btn-primary btn-sm h-10 rounded-lg px-4"
              disabled={!canSubmit}
            >
              {isSubmitting ? pendingLabel : copy.submit}
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}
