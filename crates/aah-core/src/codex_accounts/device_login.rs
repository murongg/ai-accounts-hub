use std::ffi::OsString;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::{mpsc, Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use serde_json::{json, Value};
use tungstenite::{connect, Message, WebSocket};

use super::auth::read_account_identity_from_path;
use super::cli::resolve_codex_binary;
use super::store::auth_path_for_home;

const DEVICE_AUTH_TIMEOUT: Duration = Duration::from_secs(15 * 60);
const DEVICE_PROMPT_TIMEOUT: Duration = Duration::from_secs(30);
const CDP_READY_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CodexDeviceAutofillLoginRequest {
    pub email: String,
    pub password: String,
}

pub trait CodexDeviceAutofillLoginRunner: Send + Sync {
    fn run_login(
        &self,
        managed_home: &Path,
        request: CodexDeviceAutofillLoginRequest,
    ) -> Result<(), String>;
}

pub struct ProcessCodexDeviceAutofillLoginRunner;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CodexDeviceAuthPrompt {
    pub url: String,
    pub code: Option<String>,
}

struct CapturedCodexLogin {
    child: Child,
    stdout: StreamCapture,
    stderr: StreamCapture,
    prompt_rx: mpsc::Receiver<CodexDeviceAuthPrompt>,
}

struct StreamCapture {
    output: Arc<Mutex<String>>,
    join: Option<JoinHandle<()>>,
}

struct CollectedOutput {
    stdout: String,
    stderr: String,
}

impl StreamCapture {
    fn snapshot(&self) -> String {
        self.output.lock().expect("stream capture lock").clone()
    }

    fn finish(mut self) -> String {
        if let Some(join) = self.join.take() {
            let _ = join.join();
        }
        self.output.lock().expect("stream capture lock").clone()
    }
}

impl CapturedCodexLogin {
    fn combined_output_snapshot(&self) -> String {
        format!("{}\n{}", self.stderr.snapshot(), self.stdout.snapshot())
    }

    fn collect_output(self) -> CollectedOutput {
        CollectedOutput {
            stdout: self.stdout.finish(),
            stderr: self.stderr.finish(),
        }
    }

    fn kill_and_collect(mut self) -> CollectedOutput {
        let _ = self.child.kill();
        let _ = self.child.wait();
        self.collect_output()
    }
}

impl CodexDeviceAutofillLoginRunner for ProcessCodexDeviceAutofillLoginRunner {
    fn run_login(
        &self,
        managed_home: &Path,
        request: CodexDeviceAutofillLoginRequest,
    ) -> Result<(), String> {
        std::fs::create_dir_all(managed_home)
            .map_err(|error| format!("failed to create managed home: {error}"))?;

        let codex = resolve_codex_binary()
            .ok_or_else(|| "未检测到 codex 命令，请先安装 Codex CLI".to_string())?;
        let child = spawn_codex_login(&codex, managed_home)?;
        let prompt = match read_codex_login_prompt(&child) {
            Ok(prompt) => prompt,
            Err(error) => {
                let _ = child.kill_and_collect();
                return Err(error);
            }
        };
        let browser = match run_browser_autofill(&prompt, &request) {
            Ok(browser) => browser,
            Err(error) => {
                let _ = child.kill_and_collect();
                return Err(error);
            }
        };

        let wait_result = wait_for_codex_login(child, DEVICE_AUTH_TIMEOUT);
        let _ = browser.close();
        wait_result?;

        let auth_path = auth_path_for_home(managed_home);
        read_account_identity_from_path(&auth_path).map(|_| ())
    }
}

fn spawn_codex_login(codex: &Path, managed_home: &Path) -> Result<CapturedCodexLogin, String> {
    let child = Command::new(codex)
        .arg("login")
        .env("CODEX_HOME", managed_home)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| format!("failed to launch codex login: {error}"))?;
    capture_login_process(child)
}

fn capture_login_process(mut child: Child) -> Result<CapturedCodexLogin, String> {
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| "failed to capture codex login stdout".to_string())?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| "failed to capture codex login stderr".to_string())?;
    let (prompt_tx, prompt_rx) = mpsc::channel();

    Ok(CapturedCodexLogin {
        child,
        stdout: spawn_stream_capture(stdout, prompt_tx.clone()),
        stderr: spawn_stream_capture(stderr, prompt_tx),
        prompt_rx,
    })
}

fn spawn_stream_capture<R: Read + Send + 'static>(
    stream: R,
    prompt_tx: mpsc::Sender<CodexDeviceAuthPrompt>,
) -> StreamCapture {
    let output = Arc::new(Mutex::new(String::new()));
    let output_clone = Arc::clone(&output);

    let join = thread::spawn(move || {
        let mut reader = BufReader::new(stream);
        let mut prompt_sent = false;

        loop {
            let mut line = String::new();
            match reader.read_line(&mut line) {
                Ok(0) => break,
                Ok(_) => {
                    let prompt = {
                        let mut captured = output_clone.lock().expect("stream capture lock");
                        captured.push_str(&line);
                        if prompt_sent {
                            None
                        } else {
                            parse_codex_login_prompt(&captured).ok()
                        }
                    };
                    if let Some(prompt) = prompt {
                        let _ = prompt_tx.send(prompt);
                        prompt_sent = true;
                    }
                }
                Err(_) => break,
            }
        }
    });

    StreamCapture {
        output,
        join: Some(join),
    }
}

fn read_codex_login_prompt(child: &CapturedCodexLogin) -> Result<CodexDeviceAuthPrompt, String> {
    let started = Instant::now();

    while started.elapsed() < DEVICE_PROMPT_TIMEOUT {
        let remaining = DEVICE_PROMPT_TIMEOUT.saturating_sub(started.elapsed());
        let wait_for = remaining.min(Duration::from_millis(250));
        match child.prompt_rx.recv_timeout(wait_for) {
            Ok(prompt) => return Ok(prompt),
            Err(mpsc::RecvTimeoutError::Timeout) => continue,
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
        }
    }

    parse_codex_login_prompt(&child.combined_output_snapshot())
}

#[cfg(test)]
fn read_login_prompt_from_stream<R: Read>(stream: R) -> Result<CodexDeviceAuthPrompt, String> {
    let started = Instant::now();
    let mut reader = BufReader::new(stream);
    let mut output = String::new();

    while started.elapsed() < DEVICE_PROMPT_TIMEOUT {
        let mut line = String::new();
        match reader.read_line(&mut line) {
            Ok(0) => break,
            Ok(_) => {
                output.push_str(&line);
                if let Ok(prompt) = parse_codex_login_prompt(&output) {
                    return Ok(prompt);
                }
            }
            Err(error) => {
                return Err(format!("failed to read codex login output: {error}"));
            }
        }
    }

    parse_codex_login_prompt(&output)
}

fn wait_for_codex_login(mut child: CapturedCodexLogin, timeout: Duration) -> Result<(), String> {
    let started = Instant::now();
    loop {
        match child.child.try_wait() {
            Ok(Some(status)) if status.success() => {
                let _ = child.collect_output();
                return Ok(());
            }
            Ok(Some(status)) => {
                let output = child.collect_output();
                return Err(format_codex_login_exit_error(status, &output));
            }
            Ok(None) => {
                if started.elapsed() >= timeout {
                    let output = child.kill_and_collect();
                    let detail = summarize_captured_output(&output)
                        .map(|message| format!(": {message}"))
                        .unwrap_or_default();
                    return Err(format!(
                        "Codex device login timed out before authorization completed{detail}"
                    ));
                }
                thread::sleep(Duration::from_millis(500));
            }
            Err(error) => {
                let output = child.kill_and_collect();
                let detail = summarize_captured_output(&output)
                    .map(|message| format!(": {message}"))
                    .unwrap_or_default();
                return Err(format!("failed to wait for codex device login: {error}{detail}"));
            }
        }
    }
}

fn format_codex_login_exit_error(status: std::process::ExitStatus, output: &CollectedOutput) -> String {
    let detail = summarize_captured_output(output)
        .map(|message| format!(": {message}"))
        .unwrap_or_default();
    format!(
        "codex device login exited with {}{detail}",
        describe_exit_status(status)
    )
}

fn describe_exit_status(status: std::process::ExitStatus) -> String {
    status
        .code()
        .map(|code| format!("code {code}"))
        .unwrap_or_else(|| status.to_string())
}

fn summarize_captured_output(output: &CollectedOutput) -> Option<String> {
    summarize_output_text(&output.stderr).or_else(|| summarize_output_text(&output.stdout))
}

fn summarize_output_text(output: &str) -> Option<String> {
    let lines = output
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>();
    if lines.is_empty() {
        return None;
    }

    Some(
        lines
            .into_iter()
            .rev()
            .take(4)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect::<Vec<_>>()
            .join(" | "),
    )
}

struct BrowserSession {
    child: Child,
    profile_dir: PathBuf,
}

impl BrowserSession {
    fn close(mut self) -> Result<(), String> {
        let _ = self.child.kill();
        let _ = self.child.wait();
        let _ = std::fs::remove_dir_all(&self.profile_dir);
        Ok(())
    }
}

fn run_browser_autofill(
    prompt: &CodexDeviceAuthPrompt,
    request: &CodexDeviceAutofillLoginRequest,
) -> Result<BrowserSession, String> {
    let chrome = resolve_chromium_binary()
        .ok_or_else(|| "未检测到 Chrome 或 Chromium，无法自动填充 Codex 登录".to_string())?;
    let port = reserve_local_port()?;
    let profile_dir =
        std::env::temp_dir().join(format!("aah-codex-device-login-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&profile_dir)
        .map_err(|error| format!("failed to create temporary Chrome profile: {error}"))?;

    let mut child = Command::new(chrome)
        .args(chrome_args(&profile_dir, port, &prompt.url))
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|error| format!("failed to launch Chrome for Codex login: {error}"))?;

    let result = drive_cdp_autofill(port, request, prompt.code.as_deref());
    if let Err(error) = result {
        let _ = child.kill();
        let _ = child.wait();
        let _ = std::fs::remove_dir_all(&profile_dir);
        return Err(error);
    }

    Ok(BrowserSession { child, profile_dir })
}

fn reserve_local_port() -> Result<u16, String> {
    let listener = TcpListener::bind(("127.0.0.1", 0))
        .map_err(|error| format!("failed to reserve local browser debugging port: {error}"))?;
    let port = listener
        .local_addr()
        .map_err(|error| format!("failed to read local browser debugging port: {error}"))?
        .port();
    drop(listener);
    Ok(port)
}

fn chrome_args(profile_dir: &Path, port: u16, url: &str) -> Vec<String> {
    vec![
        format!("--remote-debugging-port={port}"),
        format!("--user-data-dir={}", profile_dir.display()),
        "--no-first-run".to_string(),
        "--no-default-browser-check".to_string(),
        "--disable-extensions".to_string(),
        "--new-window".to_string(),
        url.to_string(),
    ]
}

fn drive_cdp_autofill(
    port: u16,
    request: &CodexDeviceAutofillLoginRequest,
    code: Option<&str>,
) -> Result<(), String> {
    let ws_url = wait_for_cdp_websocket_url(port)?;
    let (mut socket, _) = connect(ws_url.as_str())
        .map_err(|error| format!("failed to connect to Chrome DevTools Protocol: {error}"))?;
    send_cdp_command(&mut socket, 1, "Page.enable", json!({}))?;
    send_cdp_command(&mut socket, 2, "Runtime.enable", json!({}))?;

    let script = build_autofill_script(&request.email, &request.password, code);
    for index in 0..90 {
        let response = send_cdp_command(
            &mut socket,
            100 + index,
            "Runtime.evaluate",
            json!({
                "expression": script,
                "awaitPromise": false,
                "returnByValue": true
            }),
        )?;
        if response_contains_success(&response) {
            return Ok(());
        }
        thread::sleep(Duration::from_secs(1));
    }

    Ok(())
}

fn wait_for_cdp_websocket_url(port: u16) -> Result<String, String> {
    let client = reqwest::blocking::Client::new();
    let started = Instant::now();
    let endpoint = format!("http://127.0.0.1:{port}/json");

    while started.elapsed() < CDP_READY_TIMEOUT {
        match client.get(&endpoint).send() {
            Ok(response) => {
                let pages = response
                    .json::<Vec<Value>>()
                    .map_err(|error| format!("failed to parse Chrome page list: {error}"))?;
                if let Some(url) = select_cdp_page_websocket_url(&pages) {
                    return Ok(url.to_string());
                }
            }
            Err(_) => thread::sleep(Duration::from_millis(250)),
        }
    }

    Err("Chrome DevTools Protocol did not become ready".to_string())
}

fn select_cdp_page_websocket_url(pages: &[Value]) -> Option<&str> {
    pages.iter().find_map(|page| {
        let is_page = page.get("type").and_then(Value::as_str) == Some("page");
        let url = page.get("url").and_then(Value::as_str).unwrap_or_default();
        if !is_page || !is_openai_auth_page_url(url) {
            return None;
        }

        page.get("webSocketDebuggerUrl").and_then(Value::as_str)
    })
}

fn is_openai_auth_page_url(url: &str) -> bool {
    url == "https://auth.openai.com/codex/device"
        || url.starts_with("https://auth.openai.com/")
}

fn send_cdp_command<S>(
    socket: &mut WebSocket<S>,
    id: u64,
    method: &str,
    params: Value,
) -> Result<Value, String>
where
    S: Read + Write,
{
    let payload = json!({
        "id": id,
        "method": method,
        "params": params
    });
    socket
        .send(Message::Text(payload.to_string()))
        .map_err(|error| format!("failed to send Chrome DevTools command: {error}"))?;

    loop {
        let message = socket
            .read()
            .map_err(|error| format!("failed to read Chrome DevTools response: {error}"))?;
        let Message::Text(text) = message else {
            continue;
        };
        let response: Value = serde_json::from_str(&text)
            .map_err(|error| format!("failed to parse Chrome DevTools response: {error}"))?;
        if response.get("id").and_then(Value::as_u64) == Some(id) {
            return Ok(response);
        }
    }
}

fn response_contains_success(response: &Value) -> bool {
    response
        .pointer("/result/result/value")
        .and_then(Value::as_object)
        .is_some_and(|value| {
            value
                .get("consentSubmitted")
                .and_then(Value::as_bool)
                .unwrap_or(false)
        })
}

fn build_autofill_script(email: &str, password: &str, code: Option<&str>) -> String {
    let email = serde_json::to_string(email).expect("email json string");
    let password = serde_json::to_string(password).expect("password json string");
    let code = serde_json::to_string(code.unwrap_or("")).expect("code json string");

    format!(
        r#"
(() => {{
  const email = {email};
  const passwordValue = {password};
  const code = {code};
  const visible = (el) => {{
    const rect = el.getBoundingClientRect();
    const style = window.getComputedStyle(el);
    return rect.width > 0 && rect.height > 0 && style.visibility !== "hidden" && style.display !== "none";
  }};
  const inputs = Array.from(document.querySelectorAll("input")).filter(visible);
  const typeableInputs = inputs.filter((input) => {{
    if (input.disabled || input.readOnly) {{
      return false;
    }}
    return !["hidden", "submit", "button", "checkbox", "radio", "file"].includes((input.type || "").toLowerCase());
  }});
  const labelFor = (input) => `${{input.name || ""}} ${{input.id || ""}} ${{input.placeholder || ""}} ${{input.autocomplete || ""}}`.toLowerCase();
  const setValue = (input, value) => {{
    input.focus();
    const prototype = input instanceof HTMLTextAreaElement ? HTMLTextAreaElement.prototype : HTMLInputElement.prototype;
    const setter = Object.getOwnPropertyDescriptor(prototype, "value")?.set;
    if (setter) {{
      setter.call(input, value);
    }} else {{
      input.value = value;
    }}
    input.dispatchEvent(new InputEvent("input", {{ bubbles: true, inputType: "insertText", data: value }}));
    input.dispatchEvent(new Event("change", {{ bubbles: true }}));
  }};
  const clickSubmit = () => {{
    const buttons = Array.from(document.querySelectorAll("button,input[type=submit]")).filter(visible);
    const button = buttons.find((item) => !item.disabled && /continue|next|sign in|log in|submit|authorize|allow|confirm|继续|下一步|登录|登入|提交|授权|允许|确认/i.test(item.innerText || item.value || ""));
    if (button) {{
      button.click();
      return true;
    }}
    const fallback = buttons.find((item) => !item.disabled);
    if (fallback) {{
      fallback.click();
      return true;
    }}
    return false;
  }};
  const codeInput = inputs.find((input) => /code|device|otp/.test(labelFor(input)));
  if (code && codeInput && codeInput.value !== code) {{
    setValue(codeInput, code);
    clickSubmit();
    return {{ codeSubmitted: true, emailSubmitted: false, passwordSubmitted: false, consentSubmitted: false }};
  }}
  const emailInput = inputs.find((input) => input.type === "email" || /email|username|identifier/.test(labelFor(input)));
  if (emailInput && emailInput.value !== email) {{
    setValue(emailInput, email);
    clickSubmit();
    return {{ codeSubmitted: false, emailSubmitted: true, passwordSubmitted: false, consentSubmitted: false }};
  }}
  const passwordInput = inputs.find((input) => input.type === "password" || /password/.test(labelFor(input)));
  if (passwordInput && passwordInput.value !== passwordValue) {{
    setValue(passwordInput, passwordValue);
    clickSubmit();
    return {{ codeSubmitted: false, emailSubmitted: false, passwordSubmitted: true, consentSubmitted: false }};
  }}
  if (typeableInputs.length === 0 && clickSubmit()) {{
    return {{ codeSubmitted: false, emailSubmitted: false, passwordSubmitted: false, consentSubmitted: true }};
  }}
  return {{ codeSubmitted: false, emailSubmitted: false, passwordSubmitted: false, consentSubmitted: false }};
}})();
"#
    )
}

pub fn parse_codex_login_prompt(raw: &str) -> Result<CodexDeviceAuthPrompt, String> {
    let plain = strip_ansi(raw);
    let url = plain
        .split_whitespace()
        .find(|part| {
            let value = part.trim();
            value.starts_with("https://auth.openai.com/oauth/authorize?")
                || value == "https://auth.openai.com/codex/device"
        })
        .map(str::to_string)
        .ok_or_else(|| "Codex login URL was not found".to_string())?;

    let code = plain
        .split_whitespace()
        .find(|part| is_device_code(part))
        .map(|part| part.trim().to_string());

    Ok(CodexDeviceAuthPrompt { url, code })
}

pub fn parse_device_auth_prompt(raw: &str) -> Result<CodexDeviceAuthPrompt, String> {
    let prompt = parse_codex_login_prompt(raw).map_err(|_| "Codex device auth URL was not found".to_string())?;
    if prompt.url != "https://auth.openai.com/codex/device" {
        return Err("Codex device auth URL was not found".to_string());
    }
    if prompt.code.is_none() {
        return Err("Codex device auth code was not found".to_string());
    }
    Ok(prompt)
}

fn strip_ansi(raw: &str) -> String {
    let mut output = String::with_capacity(raw.len());
    let mut chars = raw.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '\u{1b}' && chars.peek() == Some(&'[') {
            let _ = chars.next();
            for next in chars.by_ref() {
                if next.is_ascii_alphabetic() {
                    break;
                }
            }
            continue;
        }
        output.push(ch);
    }

    output
}

fn is_device_code(value: &str) -> bool {
    let trimmed = value.trim();
    let Some((left, right)) = trimmed.split_once('-') else {
        return false;
    };

    left.len() == 4
        && right.len() == 5
        && left.chars().all(|ch| ch.is_ascii_alphanumeric())
        && right.chars().all(|ch| ch.is_ascii_alphanumeric())
}

pub fn resolve_chromium_binary() -> Option<PathBuf> {
    let app_roots = default_chromium_app_roots();
    resolve_chromium_binary_from(&app_roots, std::env::var_os("PATH"))
}

fn default_chromium_app_roots() -> Vec<PathBuf> {
    vec![
        PathBuf::from("/Applications"),
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("Applications"),
    ]
}

pub(crate) fn resolve_chromium_binary_from(
    app_roots: &[PathBuf],
    path_var: Option<OsString>,
) -> Option<PathBuf> {
    for root in app_roots {
        for relative in [
            "Google Chrome.app/Contents/MacOS/Google Chrome",
            "Chromium.app/Contents/MacOS/Chromium",
        ] {
            let candidate = root.join(relative);
            if candidate.exists() {
                return Some(candidate);
            }
        }
    }

    let path_var = path_var?;
    for dir in std::env::split_paths(&path_var) {
        for name in [
            "google-chrome",
            "google-chrome-stable",
            "chromium",
            "chromium-browser",
        ] {
            let candidate = dir.join(name);
            if candidate.exists() {
                return Some(candidate);
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::process::{Command, Stdio};
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn parses_device_auth_prompt_with_ansi_colors() {
        let prompt = "\
Welcome to Codex [v\u{1b}[90m0.121.0\u{1b}[0m]

Follow these steps to sign in with ChatGPT using device code authorization:

1. Open this link in your browser and sign in to your account
   \u{1b}[94mhttps://auth.openai.com/codex/device\u{1b}[0m

2. Enter this one-time code \u{1b}[90m(expires in 15 minutes)\u{1b}[0m
   \u{1b}[94mK88F-TC9HS\u{1b}[0m
";

        let parsed = parse_device_auth_prompt(prompt).expect("prompt");

        assert_eq!(parsed.url, "https://auth.openai.com/codex/device");
        assert_eq!(parsed.code, Some("K88F-TC9HS".to_string()));
    }

    #[test]
    fn rejects_prompt_without_device_url() {
        let error = parse_device_auth_prompt("code ABCD-EFGH").expect_err("missing url");

        assert_eq!(error, "Codex device auth URL was not found");
    }

    #[test]
    fn rejects_prompt_without_device_code() {
        let error =
            parse_device_auth_prompt("https://auth.openai.com/codex/device").expect_err("missing code");

        assert_eq!(error, "Codex device auth code was not found");
    }

    #[test]
    fn resolves_chrome_from_macos_app_bundle() {
        let root = temp_test_dir("chrome-resolver");
        let chrome = root.join("Google Chrome.app/Contents/MacOS/Google Chrome");
        fs::create_dir_all(chrome.parent().expect("chrome parent")).expect("chrome dir");
        fs::write(&chrome, "").expect("chrome binary");

        let resolved = resolve_chromium_binary_from(&[root.clone()], None);

        assert_eq!(resolved, Some(chrome));
    }

    #[test]
    fn resolves_chromium_from_path() {
        let root = temp_test_dir("chromium-path");
        let bin = root.join("bin");
        let chromium = bin.join("chromium");
        fs::create_dir_all(&bin).expect("bin dir");
        fs::write(&chromium, "").expect("chromium binary");

        let resolved = resolve_chromium_binary_from(
            &[],
            Some(std::ffi::OsString::from(bin.to_string_lossy().to_string())),
        );

        assert_eq!(resolved, Some(chromium));
    }

    #[test]
    fn autofill_script_contains_values_but_does_not_return_password() {
        let script =
            build_autofill_script("user@example.com", "secret-password", Some("K88F-TC9HS"));

        assert!(script.contains("user@example.com"));
        assert!(script.contains("secret-password"));
        assert!(script.contains("K88F-TC9HS"));
        assert!(script.contains("passwordSubmitted"));
        assert!(script.contains("HTMLInputElement.prototype"));
        assert!(script.contains("InputEvent"));
        assert!(!script.contains("return password"));
    }

    #[test]
    fn consent_click_requires_no_typeable_inputs() {
        let script =
            build_autofill_script("user@example.com", "secret-password", Some("K88F-TC9HS"));

        assert!(script.contains("typeableInputs.length === 0"));
        assert!(script.contains("consentSubmitted"));
    }

    #[test]
    fn command_args_use_temporary_profile_and_debug_port() {
        let profile = std::path::Path::new("/tmp/aah-profile");
        let args = chrome_args(profile, 9333, "https://auth.openai.com/codex/device");

        assert!(args.contains(&"--remote-debugging-port=9333".to_string()));
        assert!(args.contains(&"--user-data-dir=/tmp/aah-profile".to_string()));
        assert!(args.contains(&"https://auth.openai.com/codex/device".to_string()));
    }

    #[test]
    fn selects_openai_auth_page_instead_of_background_targets() {
        let targets = serde_json::json!([
            {
                "type": "background_page",
                "url": "chrome-extension://nkeimhogjdpnpccoofpliimaahmaaome/background.html",
                "webSocketDebuggerUrl": "ws://127.0.0.1:9229/devtools/page/background"
            },
            {
                "type": "page",
                "url": "https://auth.openai.com/log-in",
                "webSocketDebuggerUrl": "ws://127.0.0.1:9229/devtools/page/openai"
            },
            {
                "type": "service_worker",
                "url": "chrome-extension://fignfifoniblkonapihmkfakmlgkbkcf/service_worker.js",
                "webSocketDebuggerUrl": "ws://127.0.0.1:9229/devtools/page/worker"
            }
        ]);
        let pages = targets.as_array().expect("targets");

        let selected = select_cdp_page_websocket_url(pages).expect("openai page");

        assert_eq!(selected, "ws://127.0.0.1:9229/devtools/page/openai");
    }

    #[test]
    fn parses_local_oauth_login_prompt_without_device_code() {
        let prompt = "\
Starting local login server on http://localhost:1455.
If your browser did not open, navigate to this URL to authenticate:

https://auth.openai.com/oauth/authorize?response_type=code&client_id=app_example&redirect_uri=http%3A%2F%2Flocalhost%3A1455%2Fauth%2Fcallback

On a remote or headless machine? Use `codex login --device-auth` instead.
";

        let parsed = parse_codex_login_prompt(prompt).expect("oauth prompt");

        assert_eq!(parsed.url, "https://auth.openai.com/oauth/authorize?response_type=code&client_id=app_example&redirect_uri=http%3A%2F%2Flocalhost%3A1455%2Fauth%2Fcallback");
        assert_eq!(parsed.code, None);
    }

    #[test]
    fn password_submission_does_not_end_autofill_loop() {
        let response = serde_json::json!({
            "result": {
                "result": {
                    "value": {
                        "codeSubmitted": false,
                        "emailSubmitted": false,
                        "passwordSubmitted": true,
                        "consentSubmitted": false
                    }
                }
            }
        });

        assert!(!response_contains_success(&response));
    }

    #[test]
    fn consent_submission_ends_autofill_loop() {
        let response = serde_json::json!({
            "result": {
                "result": {
                    "value": {
                        "codeSubmitted": false,
                        "emailSubmitted": false,
                        "passwordSubmitted": false,
                        "consentSubmitted": true
                    }
                }
            }
        });

        assert!(response_contains_success(&response));
    }

    #[test]
    fn includes_codex_stderr_when_login_process_fails() {
        let prompt = "\
Starting local login server on http://localhost:1455.\n\
If your browser did not open, navigate to this URL to authenticate:\n\n\
https://auth.openai.com/oauth/authorize?response_type=code&client_id=app_example&redirect_uri=http%3A%2F%2Flocalhost%3A1455%2Fauth%2Fcallback\n";
        let child = Command::new("python3")
            .args([
                "-c",
                &format!(
                    "import sys,time; sys.stderr.write({prompt:?}); sys.stderr.flush(); time.sleep(0.5); sys.stderr.write('token exchange failed\\n'); sys.stderr.flush(); sys.exit(101)"
                ),
            ])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("spawn");

        let child = capture_login_process(child).expect("capture");
        let _ = read_codex_login_prompt(&child).expect("prompt");
        let error = wait_for_codex_login(child, Duration::from_secs(5)).expect_err("failure");

        assert!(error.contains("token exchange failed"));
    }

    #[test]
    fn reads_login_prompt_before_stream_eof() {
        let prompt = "\
Starting local login server on http://localhost:1455.\n\
If your browser did not open, navigate to this URL to authenticate:\n\n\
https://auth.openai.com/oauth/authorize?response_type=code&client_id=app_example&redirect_uri=http%3A%2F%2Flocalhost%3A1455%2Fauth%2Fcallback\n";
        let mut child = Command::new("python3")
            .args([
                "-c",
                &format!(
                    "import sys,time; sys.stderr.write({prompt:?}); sys.stderr.flush(); time.sleep(3)"
                ),
            ])
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .spawn()
            .expect("spawn");

        let stderr = child.stderr.take().expect("stderr");
        let started = Instant::now();
        let parsed = read_login_prompt_from_stream(stderr).expect("prompt");
        let elapsed = started.elapsed();
        let _ = child.kill();
        let _ = child.wait();

        assert!(elapsed < Duration::from_secs(2), "elapsed={elapsed:?}");
        assert!(parsed.url.starts_with("https://auth.openai.com/oauth/authorize?"));
    }

    fn temp_test_dir(prefix: &str) -> std::path::PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        let path = std::env::temp_dir().join(format!("aihub-{prefix}-{unique}"));
        fs::create_dir_all(&path).expect("temp dir");
        path
    }
}
