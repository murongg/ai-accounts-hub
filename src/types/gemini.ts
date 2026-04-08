export interface GeminiAccountSummary {
  id: string;
  email: string;
  subject: string | null;
  auth_type: string | null;
  plan: string | null;
  is_active: boolean;
  last_authenticated_at: string;
  pro_remaining_percent: number | null;
  flash_remaining_percent: number | null;
  flash_lite_remaining_percent: number | null;
  pro_refresh_at: string | null;
  flash_refresh_at: string | null;
  flash_lite_refresh_at: string | null;
  last_synced_at: string | null;
  last_sync_error: string | null;
  needs_relogin: boolean | null;
}
