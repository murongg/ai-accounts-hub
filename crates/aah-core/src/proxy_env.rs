use std::collections::BTreeMap;
use std::path::PathBuf;
use std::process::Command;

const START_MARKER: &str = "__AIHUB_PROXY_ENV_START__";
const END_MARKER: &str = "__AIHUB_PROXY_ENV_END__";

pub(crate) const PROXY_ENV_FAMILIES: [(&str, &str); 4] = [
    ("HTTP_PROXY", "http_proxy"),
    ("HTTPS_PROXY", "https_proxy"),
    ("ALL_PROXY", "all_proxy"),
    ("NO_PROXY", "no_proxy"),
];

pub fn import_shell_proxy_env_if_missing() {
    let current = current_proxy_env();
    let shell = if should_probe_login_shell() && needs_shell_probe(&current) {
        match read_login_shell_proxy_env() {
            Ok(proxy_env) => proxy_env,
            Err(error) => {
                eprintln!("failed to import shell proxy environment: {error}");
                BTreeMap::new()
            }
        }
    } else {
        BTreeMap::new()
    };
    let windows = if needs_shell_probe(&current) {
        read_windows_proxy_env()
    } else {
        BTreeMap::new()
    };

    let shell_updates = resolve_proxy_env_updates(&current, &shell);
    let merged = merge_proxy_env(&shell_updates, &windows);

    for (key, value) in merged {
        std::env::set_var(key, value);
    }
}

#[cfg(target_os = "macos")]
pub(crate) fn build_proxy_export_block_from_current_env() -> String {
    build_proxy_export_block(&current_proxy_env())
}

pub(crate) fn parse_shell_proxy_env_output(output: &str) -> BTreeMap<String, String> {
    let mut proxy_env = BTreeMap::new();
    let mut inside_markers = false;

    for line in output.lines().map(str::trim) {
        if line == START_MARKER {
            inside_markers = true;
            continue;
        }
        if line == END_MARKER {
            break;
        }
        if !inside_markers || line.is_empty() {
            continue;
        }

        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        if !is_supported_proxy_key(key) || value.is_empty() {
            continue;
        }

        proxy_env.insert(key.to_string(), value.to_string());
    }

    proxy_env
}

pub(crate) fn resolve_proxy_env_updates(
    current: &BTreeMap<String, String>,
    shell: &BTreeMap<String, String>,
) -> BTreeMap<String, String> {
    let mut updates = BTreeMap::new();

    for (upper, lower) in PROXY_ENV_FAMILIES {
        let current_value = family_value(current, upper, lower);
        let resolved_value = current_value
            .or_else(|| family_value(shell, upper, lower))
            .map(str::to_string);

        let Some(resolved_value) = resolved_value else {
            continue;
        };

        if current_value.is_none()
            || current
                .get(upper)
                .is_none_or(|value| value.trim().is_empty())
        {
            updates.insert(upper.to_string(), resolved_value.clone());
        }
        if current_value.is_none()
            || current
                .get(lower)
                .is_none_or(|value| value.trim().is_empty())
        {
            updates.insert(lower.to_string(), resolved_value);
        }
    }

    updates
}

fn merge_proxy_env(
    primary: &BTreeMap<String, String>,
    secondary: &BTreeMap<String, String>,
) -> BTreeMap<String, String> {
    let mut merged = secondary.clone();
    for (key, value) in primary {
        merged.insert(key.clone(), value.clone());
    }
    merged
}

#[cfg(windows)]
fn read_windows_proxy_env() -> BTreeMap<String, String> {
    let enabled = read_windows_internet_setting_dword("ProxyEnable").unwrap_or_default() != 0;
    let server = read_windows_internet_setting_string("ProxyServer");
    let override_list = read_windows_internet_setting_string("ProxyOverride");
    parse_windows_proxy_settings(enabled, server.as_deref(), override_list.as_deref())
}

#[cfg(not(windows))]
fn read_windows_proxy_env() -> BTreeMap<String, String> {
    BTreeMap::new()
}

#[cfg(windows)]
fn read_windows_internet_setting_string(name: &str) -> Option<String> {
    use winreg::enums::HKEY_CURRENT_USER;
    use winreg::RegKey;

    let key = RegKey::predef(HKEY_CURRENT_USER)
        .open_subkey("Software\\Microsoft\\Windows\\CurrentVersion\\Internet Settings")
        .ok()?;
    let value = key.get_value::<String, _>(name).ok()?;
    let trimmed = value.trim();
    (!trimmed.is_empty()).then(|| trimmed.to_string())
}

#[cfg(windows)]
fn read_windows_internet_setting_dword(name: &str) -> Option<u32> {
    use winreg::enums::HKEY_CURRENT_USER;
    use winreg::RegKey;

    let key = RegKey::predef(HKEY_CURRENT_USER)
        .open_subkey("Software\\Microsoft\\Windows\\CurrentVersion\\Internet Settings")
        .ok()?;
    key.get_value::<u32, _>(name).ok()
}

fn parse_windows_proxy_settings(
    enabled: bool,
    proxy_server: Option<&str>,
    proxy_override: Option<&str>,
) -> BTreeMap<String, String> {
    let mut proxy_env = BTreeMap::new();
    if !enabled {
        return proxy_env;
    }

    let Some(proxy_server) = proxy_server.map(str::trim).filter(|value| !value.is_empty()) else {
        return proxy_env;
    };

    if proxy_server.contains('=') {
        for entry in proxy_server.split(';').map(str::trim).filter(|value| !value.is_empty()) {
            let Some((protocol, value)) = entry.split_once('=') else {
                continue;
            };
            let normalized = normalize_windows_proxy_url(protocol.trim(), value.trim());
            let Some(normalized) = normalized else {
                continue;
            };

            match protocol.trim().to_ascii_lowercase().as_str() {
                "http" => {
                    insert_proxy_family(&mut proxy_env, "HTTP_PROXY", "http_proxy", &normalized)
                }
                "https" => insert_proxy_family(
                    &mut proxy_env,
                    "HTTPS_PROXY",
                    "https_proxy",
                    &normalized,
                ),
                "socks" | "socks5" => {
                    insert_proxy_family(&mut proxy_env, "ALL_PROXY", "all_proxy", &normalized)
                }
                _ => {}
            }
        }
    } else if let Some(normalized) = normalize_windows_proxy_url("http", proxy_server) {
        insert_proxy_family(&mut proxy_env, "HTTP_PROXY", "http_proxy", &normalized);
        insert_proxy_family(&mut proxy_env, "HTTPS_PROXY", "https_proxy", &normalized);
        insert_proxy_family(&mut proxy_env, "ALL_PROXY", "all_proxy", &normalized);
    }

    if let Some(no_proxy) = normalize_windows_proxy_override(proxy_override) {
        insert_proxy_family(&mut proxy_env, "NO_PROXY", "no_proxy", &no_proxy);
    }

    proxy_env
}

fn insert_proxy_family(
    proxy_env: &mut BTreeMap<String, String>,
    upper: &str,
    lower: &str,
    value: &str,
) {
    proxy_env.insert(upper.to_string(), value.to_string());
    proxy_env.insert(lower.to_string(), value.to_string());
}

fn normalize_windows_proxy_url(protocol: &str, raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }
    if trimmed.contains("://") {
        return Some(trimmed.to_string());
    }

    let scheme = match protocol.to_ascii_lowercase().as_str() {
        "socks" | "socks5" => "socks5",
        "https" => "http",
        _ => "http",
    };

    Some(format!("{scheme}://{trimmed}"))
}

fn normalize_windows_proxy_override(proxy_override: Option<&str>) -> Option<String> {
    let values = proxy_override?
        .split(';')
        .map(str::trim)
        .filter(|value| !value.is_empty() && *value != "<local>")
        .collect::<Vec<_>>();

    (!values.is_empty()).then(|| values.join(","))
}

#[cfg(any(target_os = "macos", test))]
pub(crate) fn build_proxy_export_block(proxy_env: &BTreeMap<String, String>) -> String {
    let normalized = resolve_proxy_env_updates(&BTreeMap::new(), proxy_env);
    let mut block = String::new();

    for (key, value) in normalized {
        block.push_str("export ");
        block.push_str(&key);
        block.push('=');
        block.push_str(&shell_escape_value(&value));
        block.push('\n');
    }

    block
}

#[cfg(test)]
pub(crate) fn proxy_env_markers() -> (&'static str, &'static str) {
    (START_MARKER, END_MARKER)
}

fn current_proxy_env() -> BTreeMap<String, String> {
    let mut proxy_env = BTreeMap::new();

    for (upper, lower) in PROXY_ENV_FAMILIES {
        for key in [upper, lower] {
            if let Ok(value) = std::env::var(key) {
                let trimmed = value.trim();
                if !trimmed.is_empty() {
                    proxy_env.insert(key.to_string(), trimmed.to_string());
                }
            }
        }
    }

    proxy_env
}

fn needs_shell_probe(current: &BTreeMap<String, String>) -> bool {
    PROXY_ENV_FAMILIES
        .iter()
        .any(|(upper, lower)| family_value(current, upper, lower).is_none())
}

fn should_probe_login_shell() -> bool {
    should_probe_login_shell_for_os(cfg!(windows))
}

fn should_probe_login_shell_for_os(is_windows: bool) -> bool {
    !is_windows
}

fn read_login_shell_proxy_env() -> Result<BTreeMap<String, String>, String> {
    let shell = resolve_login_shell();
    let command = build_proxy_probe_command();
    let output = Command::new(&shell)
        .args(["-ilc", &command])
        .output()
        .map_err(|error| format!("failed to run {} -ilc: {error}", shell.display()))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed = parse_shell_proxy_env_output(&stdout);
    if !parsed.is_empty() || stdout.contains(START_MARKER) {
        return Ok(parsed);
    }

    if output.status.success() {
        Ok(BTreeMap::new())
    } else {
        Err(format!(
            "{} -ilc exited with status {}",
            shell.display(),
            output.status
        ))
    }
}

fn resolve_login_shell() -> PathBuf {
    std::env::var_os("SHELL")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/bin/zsh"))
}

fn build_proxy_probe_command() -> String {
    format!(
        "printf '%s\\n' '{START_MARKER}'; \
printf 'HTTP_PROXY=%s\\n' \"${{HTTP_PROXY-}}\"; \
printf 'HTTPS_PROXY=%s\\n' \"${{HTTPS_PROXY-}}\"; \
printf 'ALL_PROXY=%s\\n' \"${{ALL_PROXY-}}\"; \
printf 'NO_PROXY=%s\\n' \"${{NO_PROXY-}}\"; \
printf 'http_proxy=%s\\n' \"${{http_proxy-}}\"; \
printf 'https_proxy=%s\\n' \"${{https_proxy-}}\"; \
printf 'all_proxy=%s\\n' \"${{all_proxy-}}\"; \
printf 'no_proxy=%s\\n' \"${{no_proxy-}}\"; \
printf '%s\\n' '{END_MARKER}'"
    )
}

fn family_value<'a>(
    proxy_env: &'a BTreeMap<String, String>,
    upper: &str,
    lower: &str,
) -> Option<&'a str> {
    proxy_env
        .get(upper)
        .map(String::as_str)
        .filter(|value| !value.trim().is_empty())
        .or_else(|| {
            proxy_env
                .get(lower)
                .map(String::as_str)
                .filter(|value| !value.trim().is_empty())
        })
}

fn is_supported_proxy_key(key: &str) -> bool {
    PROXY_ENV_FAMILIES
        .iter()
        .any(|(upper, lower)| key == *upper || key == *lower)
}

#[cfg(any(target_os = "macos", test))]
fn shell_escape_value(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn map(entries: &[(&str, &str)]) -> BTreeMap<String, String> {
        entries
            .iter()
            .map(|(key, value)| (key.to_string(), value.to_string()))
            .collect()
    }

    #[test]
    fn parses_shell_proxy_output_between_markers_and_ignores_noise() {
        let (start, end) = proxy_env_markers();
        let output = format!(
            "oh-my-zsh noisy banner\n{start}\nHTTP_PROXY=http://127.0.0.1:7890\nHTTPS_PROXY=http://127.0.0.1:7890\nNO_PROXY=localhost,127.0.0.1\n{end}\nrandom footer\n"
        );

        let parsed = parse_shell_proxy_env_output(&output);

        assert_eq!(
            parsed,
            map(&[
                ("HTTP_PROXY", "http://127.0.0.1:7890"),
                ("HTTPS_PROXY", "http://127.0.0.1:7890"),
                ("NO_PROXY", "localhost,127.0.0.1"),
            ])
        );
    }

    #[test]
    fn resolves_missing_proxy_variants_without_overwriting_existing_values() {
        let current = map(&[("HTTPS_PROXY", "http://existing:9000")]);
        let shell = map(&[
            ("http_proxy", "http://shell:7890"),
            ("HTTPS_PROXY", "http://shell:7890"),
            ("NO_PROXY", "localhost,127.0.0.1"),
        ]);

        let updates = resolve_proxy_env_updates(&current, &shell);

        assert_eq!(
            updates,
            map(&[
                ("HTTP_PROXY", "http://shell:7890"),
                ("http_proxy", "http://shell:7890"),
                ("https_proxy", "http://existing:9000"),
                ("NO_PROXY", "localhost,127.0.0.1"),
                ("no_proxy", "localhost,127.0.0.1"),
            ])
        );
    }

    #[test]
    fn builds_proxy_export_block_for_available_proxy_values() {
        let block = build_proxy_export_block(&map(&[
            ("HTTP_PROXY", "http://127.0.0.1:7890"),
            ("HTTPS_PROXY", "http://127.0.0.1:7890"),
            ("NO_PROXY", "localhost,127.0.0.1"),
        ]));

        assert!(block.contains("export HTTP_PROXY='http://127.0.0.1:7890'"));
        assert!(block.contains("export http_proxy='http://127.0.0.1:7890'"));
        assert!(block.contains("export HTTPS_PROXY='http://127.0.0.1:7890'"));
        assert!(block.contains("export https_proxy='http://127.0.0.1:7890'"));
        assert!(block.contains("export NO_PROXY='localhost,127.0.0.1'"));
        assert!(block.contains("export no_proxy='localhost,127.0.0.1'"));
    }

    #[test]
    fn parses_windows_proxy_server_with_single_endpoint() {
        let parsed = parse_windows_proxy_settings(
            true,
            Some("127.0.0.1:7897"),
            Some("localhost;127.*;10.*"),
        );

        assert_eq!(
            parsed,
            map(&[
                ("HTTP_PROXY", "http://127.0.0.1:7897"),
                ("http_proxy", "http://127.0.0.1:7897"),
                ("HTTPS_PROXY", "http://127.0.0.1:7897"),
                ("https_proxy", "http://127.0.0.1:7897"),
                ("ALL_PROXY", "http://127.0.0.1:7897"),
                ("all_proxy", "http://127.0.0.1:7897"),
                ("NO_PROXY", "localhost,127.*,10.*"),
                ("no_proxy", "localhost,127.*,10.*"),
            ])
        );
    }

    #[test]
    fn parses_windows_proxy_server_with_protocol_specific_entries() {
        let parsed = parse_windows_proxy_settings(
            true,
            Some("http=127.0.0.1:8080;https=127.0.0.1:8443;socks=127.0.0.1:1080"),
            None,
        );

        assert_eq!(
            parsed,
            map(&[
                ("HTTP_PROXY", "http://127.0.0.1:8080"),
                ("http_proxy", "http://127.0.0.1:8080"),
                ("HTTPS_PROXY", "http://127.0.0.1:8443"),
                ("https_proxy", "http://127.0.0.1:8443"),
                ("ALL_PROXY", "socks5://127.0.0.1:1080"),
                ("all_proxy", "socks5://127.0.0.1:1080"),
            ])
        );
    }

    #[test]
    fn skips_login_shell_proxy_probe_on_windows() {
        assert!(!should_probe_login_shell_for_os(true));
        assert!(should_probe_login_shell_for_os(false));
    }
}
