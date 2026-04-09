export interface ClaudeAccountSummary {
  id: string;
  email: string;
  display_name: string | null;
  plan: string | null;
  account_hint: string | null;
  is_active: boolean;
  last_authenticated_at: string;
  last_synced_at: string | null;
  last_sync_error: string | null;
  needs_relogin: boolean | null;
}
