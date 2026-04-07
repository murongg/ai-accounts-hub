export interface CodexAccountSummary {
  id: string;
  email: string;
  plan: string | null;
  account_id: string | null;
  is_active: boolean;
  last_authenticated_at: string;
  five_hour_remaining_percent: number | null;
  weekly_remaining_percent: number | null;
  five_hour_refresh_at: string | null;
  weekly_refresh_at: string | null;
  last_synced_at: string | null;
  last_sync_error: string | null;
  credits_balance: number | null;
  needs_relogin: boolean | null;
}

export interface CodexRefreshSettings {
  enabled: boolean;
  interval_seconds: number;
}
