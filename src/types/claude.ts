export interface ClaudeAccountSummary {
  id: string;
  email: string;
  display_name: string | null;
  plan: string | null;
  account_hint: string | null;
  is_active: boolean;
  last_authenticated_at: string;
  session_remaining_percent: number | null;
  session_refresh_at: string | null;
  weekly_remaining_percent: number | null;
  weekly_refresh_at: string | null;
  model_weekly_label: string | null;
  model_weekly_remaining_percent: number | null;
  model_weekly_refresh_at: string | null;
  last_synced_at: string | null;
  last_sync_error: string | null;
  needs_relogin: boolean | null;
}
