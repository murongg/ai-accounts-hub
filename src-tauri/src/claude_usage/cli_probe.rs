use std::fmt::{Display, Formatter};
use std::fs;
use std::io::{Read, Write};
use std::path::Path;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use portable_pty::{native_pty_system, CommandBuilder, PtySize};
use serde_json::{Map, Value};

use crate::claude_accounts::cli::resolve_claude_binary;
use crate::claude_accounts::live_credentials::ClaudeLiveCredentialSnapshot;
use crate::claude_accounts::paths::atomic_write;

use super::models::{ClaudeRateWindowSnapshot, FetchedClaudeUsage};

const PROBE_TIMEOUT: Duration = Duration::from_secs(20);

pub trait ClaudeCliUsageProbe: Send + Sync {
    fn probe_usage(
        &self,
        snapshot: &ClaudeLiveCredentialSnapshot,
    ) -> Result<FetchedClaudeUsage, ClaudeCliUsageProbeError>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClaudeCliUsageProbeError {
    ReloginRequired(String),
    ParseFailed(String),
    CommandFailed(String),
}

impl ClaudeCliUsageProbeError {
    pub fn needs_relogin(&self) -> bool {
        matches!(self, Self::ReloginRequired(_))
    }
}

impl Display for ClaudeCliUsageProbeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ReloginRequired(message)
            | Self::ParseFailed(message)
            | Self::CommandFailed(message) => f.write_str(message),
        }
    }
}

pub struct ProcessClaudeCliUsageProbe;

impl ClaudeCliUsageProbe for ProcessClaudeCliUsageProbe {
    fn probe_usage(
        &self,
        snapshot: &ClaudeLiveCredentialSnapshot,
    ) -> Result<FetchedClaudeUsage, ClaudeCliUsageProbeError> {
        let binary = resolve_claude_binary().ok_or_else(|| {
            ClaudeCliUsageProbeError::CommandFailed(
                "Claude CLI is unavailable, so CLI usage fallback cannot run.".to_string(),
            )
        })?;
        let temp_dir = temp_config_dir()?;
        let temp_path = temp_dir.path().to_path_buf();
        write_snapshot_to_temp_dir(&temp_path, snapshot)?;

        let pty_system = native_pty_system();
        let pair = pty_system
            .openpty(PtySize {
                rows: 40,
                cols: 140,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|error| ClaudeCliUsageProbeError::CommandFailed(error.to_string()))?;

        let mut command = CommandBuilder::new(binary);
        command.env("CLAUDE_CONFIG_DIR", &temp_path);
        command.cwd(&temp_path);

        let mut child = pair
            .slave
            .spawn_command(command)
            .map_err(|error| ClaudeCliUsageProbeError::CommandFailed(error.to_string()))?;
        drop(pair.slave);

        let mut reader = pair
            .master
            .try_clone_reader()
            .map_err(|error| ClaudeCliUsageProbeError::CommandFailed(error.to_string()))?;
        let mut writer = pair
            .master
            .take_writer()
            .map_err(|error| ClaudeCliUsageProbeError::CommandFailed(error.to_string()))?;

        let (tx, rx) = mpsc::channel();
        let reader_handle = thread::spawn(move || {
            let mut output = String::new();
            let _ = reader.read_to_string(&mut output);
            let _ = tx.send(output);
        });

        writer
            .write_all(b"/usage\r")
            .map_err(|error| ClaudeCliUsageProbeError::CommandFailed(error.to_string()))?;
        writer
            .flush()
            .map_err(|error| ClaudeCliUsageProbeError::CommandFailed(error.to_string()))?;
        thread::sleep(Duration::from_secs(2));
        writer
            .write_all(b"/status\r")
            .map_err(|error| ClaudeCliUsageProbeError::CommandFailed(error.to_string()))?;
        writer
            .flush()
            .map_err(|error| ClaudeCliUsageProbeError::CommandFailed(error.to_string()))?;
        thread::sleep(Duration::from_secs(2));
        writer
            .write_all(b"/exit\r")
            .map_err(|error| ClaudeCliUsageProbeError::CommandFailed(error.to_string()))?;
        writer
            .flush()
            .map_err(|error| ClaudeCliUsageProbeError::CommandFailed(error.to_string()))?;
        drop(writer);

        let output = rx.recv_timeout(PROBE_TIMEOUT).map_err(|_| {
            let _ = child.kill();
            ClaudeCliUsageProbeError::CommandFailed(
                "Claude CLI usage probe timed out before producing output.".to_string(),
            )
        })?;

        let _ = child.wait();
        let _ = reader_handle.join();

        parse_usage_output_at(&output, unix_timestamp_now())
    }
}

pub fn parse_usage_output(output: &str) -> Result<FetchedClaudeUsage, ClaudeCliUsageProbeError> {
    parse_usage_output_at(output, unix_timestamp_now())
}

fn parse_usage_output_at(
    output: &str,
    now_unix: i64,
) -> Result<FetchedClaudeUsage, ClaudeCliUsageProbeError> {
    let cleaned = strip_ansi(output);
    let lower = cleaned.to_ascii_lowercase();
    if lower.contains("authentication required")
        || lower.contains("run claude login again")
        || lower.contains("please login")
        || lower.contains("please log in")
    {
        return Err(ClaudeCliUsageProbeError::ReloginRequired(
            cleaned.trim().to_string(),
        ));
    }

    let session = extract_window(&cleaned, &["current session"], now_unix)?;
    let weekly = extract_window(&cleaned, &["current week", "weekly"], now_unix)?;
    let opus = extract_window(&cleaned, &["opus week", "opus weekly"], now_unix)?;
    let sonnet = extract_window(&cleaned, &["sonnet week", "sonnet weekly"], now_unix)?;

    let (model_weekly_label, model_weekly) = if let Some(window) = opus {
        (Some("Opus Weekly".to_string()), Some(window))
    } else if let Some(window) = sonnet {
        (Some("Sonnet Weekly".to_string()), Some(window))
    } else {
        (None, None)
    };

    if session.is_none() && weekly.is_none() && model_weekly.is_none() {
        return Err(ClaudeCliUsageProbeError::ParseFailed(
            "Claude CLI usage output did not contain any recognized quota windows.".to_string(),
        ));
    }

    Ok(FetchedClaudeUsage {
        session,
        weekly,
        model_weekly_label,
        model_weekly,
    })
}

fn extract_window(
    output: &str,
    labels: &[&str],
    now_unix: i64,
) -> Result<Option<ClaudeRateWindowSnapshot>, ClaudeCliUsageProbeError> {
    let lines = output.lines().map(str::trim).collect::<Vec<_>>();

    for (index, line) in lines.iter().enumerate() {
        let lower = line.to_ascii_lowercase();
        if !labels.iter().any(|label| lower.contains(label)) {
            continue;
        }

        let mut used_percent = None;
        let mut reset_at = None;

        for candidate in lines.iter().skip(index + 1).take(8) {
            if used_percent.is_none() {
                used_percent = parse_used_percent(candidate);
            }
            if reset_at.is_none() {
                reset_at = parse_reset_at(candidate, now_unix);
            }
            if used_percent.is_some() && reset_at.is_some() {
                break;
            }
        }

        if let (Some(used_percent), Some(reset_at)) = (used_percent, reset_at) {
            return Ok(Some(ClaudeRateWindowSnapshot {
                remaining_percent: 100_u8.saturating_sub(used_percent),
                used_percent,
                reset_at,
            }));
        }
    }

    Ok(None)
}

fn parse_used_percent(line: &str) -> Option<u8> {
    let percent_index = line.find('%')?;
    let digits = line[..percent_index]
        .chars()
        .rev()
        .take_while(|ch| ch.is_ascii_digit())
        .collect::<String>()
        .chars()
        .rev()
        .collect::<String>();
    digits.parse::<u8>().ok().map(|value| value.min(100))
}

fn parse_reset_at(line: &str, now_unix: i64) -> Option<String> {
    let lower = line.to_ascii_lowercase();
    let prefix_index = lower.find("resets in ")?;
    let raw = line[prefix_index + "resets in ".len()..].trim();
    let seconds = parse_relative_duration_seconds(raw)?;
    Some((now_unix + seconds).to_string())
}

fn parse_relative_duration_seconds(raw: &str) -> Option<i64> {
    let mut total = 0_i64;
    let normalized = raw.replace(',', " ");
    let tokens = normalized.split_whitespace().collect::<Vec<_>>();
    let mut index = 0;

    while index < tokens.len() {
        let token = tokens[index];
        if let Some(value) = token.strip_suffix('d').and_then(|value| value.parse::<i64>().ok()) {
            total += value * 24 * 60 * 60;
            index += 1;
            continue;
        }
        if let Some(value) = token.strip_suffix('h').and_then(|value| value.parse::<i64>().ok()) {
            total += value * 60 * 60;
            index += 1;
            continue;
        }
        if let Some(value) = token.strip_suffix('m').and_then(|value| value.parse::<i64>().ok()) {
            total += value * 60;
            index += 1;
            continue;
        }
        if token.chars().all(|ch| ch.is_ascii_digit()) && index + 1 < tokens.len() {
            let value = token.parse::<i64>().ok()?;
            let unit = tokens[index + 1].to_ascii_lowercase();
            match unit.as_str() {
                "day" | "days" => total += value * 24 * 60 * 60,
                "hour" | "hours" => total += value * 60 * 60,
                "minute" | "minutes" => total += value * 60,
                _ => {}
            }
            index += 2;
            continue;
        }
        index += 1;
    }

    (total > 0).then_some(total)
}

fn strip_ansi(raw: &str) -> String {
    let mut cleaned = String::with_capacity(raw.len());
    let mut chars = raw.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '\u{1b}' {
            if matches!(chars.peek(), Some('[')) {
                let _ = chars.next();
                while let Some(next) = chars.next() {
                    if ('@'..='~').contains(&next) {
                        break;
                    }
                }
                continue;
            }
        }
        cleaned.push(ch);
    }

    cleaned
}

fn write_snapshot_to_temp_dir(
    root: &Path,
    snapshot: &ClaudeLiveCredentialSnapshot,
) -> Result<(), ClaudeCliUsageProbeError> {
    fs::create_dir_all(root).map_err(|error| ClaudeCliUsageProbeError::CommandFailed(error.to_string()))?;
    atomic_write(&root.join(".credentials.json"), &snapshot.credentials_json)
        .map_err(ClaudeCliUsageProbeError::CommandFailed)?;

    if let Some(oauth_account_json) = snapshot.oauth_account_json.as_deref() {
        let oauth_account: Value = serde_json::from_slice(oauth_account_json)
            .map_err(|error| ClaudeCliUsageProbeError::ParseFailed(error.to_string()))?;
        let mut config = Map::new();
        config.insert("oauthAccount".to_string(), oauth_account);
        let bytes = serde_json::to_vec_pretty(&Value::Object(config))
            .map_err(|error| ClaudeCliUsageProbeError::ParseFailed(error.to_string()))?;
        atomic_write(&root.join(".claude.json"), &bytes)
            .map_err(ClaudeCliUsageProbeError::CommandFailed)?;
    }

    Ok(())
}

fn unix_timestamp_now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs() as i64)
        .unwrap_or(0)
}

struct TempConfigDir {
    path: std::path::PathBuf,
}

impl TempConfigDir {
    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TempConfigDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

fn temp_config_dir() -> Result<TempConfigDir, ClaudeCliUsageProbeError> {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| ClaudeCliUsageProbeError::CommandFailed(error.to_string()))?
        .as_nanos();
    let path = std::env::temp_dir().join(format!("aihub-claude-usage-{unique}"));
    fs::create_dir_all(&path).map_err(|error| ClaudeCliUsageProbeError::CommandFailed(error.to_string()))?;
    Ok(TempConfigDir { path })
}
