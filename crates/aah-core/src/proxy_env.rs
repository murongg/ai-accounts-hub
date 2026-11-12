use std::collections::BTreeMap;
use std::path::PathBuf;
use std::process::Command;

use reqwest::blocking::{Client, ClientBuilder};
use reqwest::{NoProxy, Proxy};

#[cfg(windows)]
static STARTUP_PROXY_ENV: std::sync::OnceLock<BTreeMap<String, String>> =
    std::sync::OnceLock::new();

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
    let updates = resolve_proxy_env_updates(&current, &shell);
    #[cfg(windows)]
    let updates = {
        // Keep the original environment so quota clients can use live OS settings
        // even after exporting the initial settings for child processes.
        let _ = STARTUP_PROXY_ENV.set(current.clone());
        let mut updates = updates;
        if family_value(&current, "ALL_PROXY", "all_proxy").is_none() {
            updates.extend(resolve_proxy_env_updates(
                &current,
                &read_windows_proxy_env(),
            ));
        }
        updates
    };

    for (key, value) in updates {
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

pub(crate) fn quota_http_client_builder() -> Result<ClientBuilder, String> {
    // Preserve reqwest's guard against CGI request headers being read as proxy settings.
    if std::env::var_os("REQUEST_METHOD").is_some() {
        return Ok(Client::builder().no_proxy());
    }

    let env = current_proxy_env();
    #[cfg(windows)]
    let env = STARTUP_PROXY_ENV.get().unwrap_or(&env);
    #[cfg(not(windows))]
    let env = &env;
    quota_client_with_proxies(env, &read_system_proxy_env())
}

fn quota_client_with_proxies(
    env: &BTreeMap<String, String>,
    system: &BTreeMap<String, String>,
) -> Result<ClientBuilder, String> {
    let mut client = Client::builder().no_proxy();
    for (upper, lower) in [("HTTP_PROXY", "http_proxy"), ("HTTPS_PROXY", "https_proxy")] {
        let explicit =
            family_value(env, upper, lower).or_else(|| family_value(env, "ALL_PROXY", "all_proxy"));
        let proxy = explicit.or_else(|| {
            family_value(system, upper, lower)
                .or_else(|| family_value(system, "ALL_PROXY", "all_proxy"))
        });
        if let Some(address) = proxy {
            let exclusions = family_value(env, "NO_PROXY", "no_proxy").or_else(|| {
                if explicit.is_some() {
                    None
                } else {
                    family_value(system, "NO_PROXY", "no_proxy")
                }
            });
            let bypass = format!(
                "localhost,127.0.0.0/8,::1,{}",
                exclusions.unwrap_or_default()
            );
            let url = if address.contains("://") {
                reqwest::Url::parse(address)
            } else {
                reqwest::Url::parse(&format!("http://{address}"))
            }
            .ok()
            .filter(|url| {
                url.has_host()
                    && matches!(
                        url.scheme(),
                        "http" | "https" | "socks4" | "socks4a" | "socks5" | "socks5h"
                    )
            })
            .ok_or_else(|| format!("Invalid {upper} setting for quota refresh"))?;
            let proxy = if upper == "HTTP_PROXY" {
                Proxy::http(url)
            } else {
                Proxy::https(url)
            }
            .map_err(|_| format!("Invalid {upper} setting for quota refresh"))?;
            client = client.proxy(proxy.no_proxy(NoProxy::from_string(&bypass)));
        }
    }
    Ok(client)
}

#[cfg(windows)]
fn read_system_proxy_env() -> BTreeMap<String, String> {
    read_windows_proxy_env()
}

#[cfg(not(any(windows, target_os = "macos")))]
fn read_system_proxy_env() -> BTreeMap<String, String> {
    BTreeMap::new()
}

#[cfg(target_os = "macos")]
fn read_system_proxy_env() -> BTreeMap<String, String> {
    use system_configuration::dynamic_store::SCDynamicStoreBuilder;

    SCDynamicStoreBuilder::new("ai-accounts-hub")
        .build()
        .and_then(|store| store.get_proxies())
        .map_or_else(BTreeMap::new, |settings| {
            parse_macos_proxy_settings(&settings)
        })
}

#[cfg(target_os = "macos")]
fn parse_macos_proxy_settings(
    settings: &system_configuration::core_foundation::dictionary::CFDictionary<
        system_configuration::core_foundation::string::CFString,
        system_configuration::core_foundation::base::CFType,
    >,
) -> BTreeMap<String, String> {
    use system_configuration::core_foundation::{
        array::CFArray,
        base::{CFType, TCFType},
        number::CFNumber,
        string::CFString,
    };

    let mut proxies = BTreeMap::new();
    for (prefix, upper, lower, scheme) in [
        ("HTTP", "HTTP_PROXY", "http_proxy", "http"),
        ("HTTPS", "HTTPS_PROXY", "https_proxy", "http"),
        ("SOCKS", "ALL_PROXY", "all_proxy", "socks5h"),
    ] {
        let number = |suffix| {
            settings
                .find(CFString::new(&format!("{prefix}{suffix}")).as_concrete_TypeRef())
                .and_then(|value| value.downcast::<CFNumber>())
                .and_then(|value| value.to_i32())
        };
        if number("Enable") != Some(1) {
            continue;
        }
        let host = settings
            .find(CFString::new(&format!("{prefix}Proxy")).as_concrete_TypeRef())
            .and_then(|value| value.downcast::<CFString>())
            .map(|value| value.to_string());
        if let (Some(host), Some(port)) = (host, number("Port")) {
            let host = match host.parse::<std::net::Ipv6Addr>() {
                Ok(address) => format!("[{address}]"),
                Err(_) => host,
            };
            insert_proxy_family(
                &mut proxies,
                upper,
                lower,
                &format!("{scheme}://{host}:{port}"),
            );
        }
    }
    if let Some(exceptions) = settings
        .find(CFString::new("ExceptionsList").as_concrete_TypeRef())
        .and_then(|value| value.downcast::<CFArray>())
    {
        // SystemConfiguration returns a CFArray of CFStrings.
        let exclusions = exceptions
            .iter()
            .filter_map(|item| unsafe { CFType::wrap_under_get_rule(*item) }.downcast::<CFString>())
            .map(|value| value.to_string())
            .collect::<Vec<_>>();
        let normalized = normalize_system_proxy_exclusions(exclusions.iter().map(String::as_str));
        insert_proxy_family(&mut proxies, "NO_PROXY", "no_proxy", &normalized);
    }
    proxies
}

#[cfg(any(windows, target_os = "macos", test))]
fn normalize_system_proxy_exclusions<'a>(values: impl Iterator<Item = &'a str>) -> String {
    values
        .map(str::trim)
        .filter_map(|value| {
            if value == "<local>" || value.is_empty() {
                return None;
            }
            if let Some(prefix) = value.strip_suffix(".*") {
                // Windows uses IPv4 prefix wildcards. Reqwest takes CIDR ranges.
                let parts = prefix.split('.').count();
                if parts <= 3 && prefix.split('.').all(|part| part.parse::<u8>().is_ok()) {
                    return Some(format!(
                        "{prefix}{}/{bits}",
                        ".0".repeat(4 - parts),
                        bits = 8 * parts
                    ));
                }
            }
            Some(value.strip_prefix("*.").unwrap_or(value).to_string())
        })
        .collect::<Vec<_>>()
        .join(",")
}

#[cfg(windows)]
fn read_windows_proxy_env() -> BTreeMap<String, String> {
    let enabled = read_windows_internet_setting_dword("ProxyEnable").unwrap_or_default() != 0;
    let server = read_windows_internet_setting_string("ProxyServer");
    let override_list = read_windows_internet_setting_string("ProxyOverride");
    parse_windows_proxy_settings(enabled, server.as_deref(), override_list.as_deref())
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

#[cfg(any(windows, test))]
fn parse_windows_proxy_settings(
    enabled: bool,
    proxy_server: Option<&str>,
    proxy_override: Option<&str>,
) -> BTreeMap<String, String> {
    let mut proxy_env = BTreeMap::new();
    if !enabled {
        return proxy_env;
    }

    let Some(proxy_server) = proxy_server
        .map(str::trim)
        .filter(|value| !value.is_empty())
    else {
        return proxy_env;
    };

    if proxy_server.contains('=') {
        for entry in proxy_server
            .split(';')
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
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
                "https" => {
                    insert_proxy_family(&mut proxy_env, "HTTPS_PROXY", "https_proxy", &normalized)
                }
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

#[cfg(any(windows, target_os = "macos", test))]
fn insert_proxy_family(
    proxy_env: &mut BTreeMap<String, String>,
    upper: &str,
    lower: &str,
    value: &str,
) {
    proxy_env.insert(upper.to_string(), value.to_string());
    proxy_env.insert(lower.to_string(), value.to_string());
}

#[cfg(any(windows, test))]
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

#[cfg(any(windows, test))]
fn normalize_windows_proxy_override(proxy_override: Option<&str>) -> Option<String> {
    let normalized = normalize_system_proxy_exclusions(proxy_override?.split(';'));
    (!normalized.is_empty()).then_some(normalized)
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
                ("NO_PROXY", "localhost,127.0.0.0/8,10.0.0.0/8"),
                ("no_proxy", "localhost,127.0.0.0/8,10.0.0.0/8"),
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

    #[cfg(target_os = "macos")]
    #[test]
    fn parses_macos_https_socks_and_exclusions() {
        use system_configuration::core_foundation::{
            array::CFArray, base::TCFType, dictionary::CFDictionary, number::CFNumber,
            string::CFString,
        };

        let entries = [
            ("HTTPEnable", CFNumber::from(0).as_CFType()),
            ("HTTPProxy", CFString::new("disabled.example").as_CFType()),
            ("HTTPPort", CFNumber::from(8888).as_CFType()),
            ("HTTPSEnable", CFNumber::from(1).as_CFType()),
            ("HTTPSProxy", CFString::new("127.0.0.1").as_CFType()),
            ("HTTPSPort", CFNumber::from(7890).as_CFType()),
            ("SOCKSEnable", CFNumber::from(1).as_CFType()),
            ("SOCKSProxy", CFString::new("::1").as_CFType()),
            ("SOCKSPort", CFNumber::from(1080).as_CFType()),
            (
                "ExceptionsList",
                CFArray::from_CFTypes(&[CFString::new("*.example.test"), CFString::new("10.*")])
                    .as_CFType(),
            ),
        ];
        let pairs = entries.map(|(key, value)| (CFString::new(key), value));
        let settings = CFDictionary::from_CFType_pairs(&pairs);

        let proxies = parse_macos_proxy_settings(&settings);

        assert!(!proxies.contains_key("HTTP_PROXY"));
        assert_eq!(proxies["HTTPS_PROXY"], "http://127.0.0.1:7890");
        assert_eq!(proxies["ALL_PROXY"], "socks5h://[::1]:1080");
        assert_eq!(proxies["NO_PROXY"], "example.test,10.0.0.0/8");
    }

    #[test]
    fn quota_refresh_uses_current_windows_proxy_and_explicit_override() {
        use std::io::{BufRead, BufReader, Write};
        use std::net::TcpListener;
        use std::thread;
        use std::time::Duration;

        let empty = BTreeMap::new();
        let mut stale_settings = None;
        for explicit_override in [false, false, true] {
            let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
            let address = listener.local_addr().unwrap();
            let server = format!("https={address}");
            let settings = parse_windows_proxy_settings(true, Some(&server), None);
            let explicit = map(&[("all_proxy", &format!("http://{address}"))]);
            let (env, system) = if explicit_override {
                (&explicit, stale_settings.as_ref().unwrap())
            } else {
                (&empty, &settings)
            };
            let client = quota_client_with_proxies(env, system)
                .unwrap()
                .timeout(Duration::from_secs(2))
                .build()
                .unwrap();
            let request = thread::spawn(move || client.get("https://quota.example.test/").send());
            listener.set_nonblocking(true).unwrap();
            let deadline = std::time::Instant::now() + Duration::from_secs(3);
            let mut connection = loop {
                match listener.accept() {
                    Ok((stream, _)) => break stream,
                    Err(error)
                        if error.kind() == std::io::ErrorKind::WouldBlock
                            && std::time::Instant::now() < deadline =>
                    {
                        thread::sleep(Duration::from_millis(10));
                    }
                    Err(error) => panic!("quota request did not reach the expected proxy: {error}"),
                }
            };
            connection
                .set_read_timeout(Some(Duration::from_secs(2)))
                .unwrap();
            let mut line = String::new();
            BufReader::new(&connection).read_line(&mut line).unwrap();
            assert_eq!(line.trim_end(), "CONNECT quota.example.test:443 HTTP/1.1");
            connection
                .write_all(b"HTTP/1.1 502 Bad Gateway\r\nContent-Length: 0\r\n\r\n")
                .unwrap();
            assert!(request.join().unwrap().is_err());
            stale_settings = Some(settings);
        }
    }

    #[test]
    fn disabled_windows_proxy_yields_no_routes() {
        assert!(parse_windows_proxy_settings(false, Some("127.0.0.1:7890"), None).is_empty());
    }

    #[test]
    fn invalid_quota_proxy_does_not_expose_its_value() {
        for address in [
            "http://name:secret@[invalid",
            "ftp://name:secret@proxy.example",
        ] {
            let env = map(&[("HTTPS_PROXY", address)]);
            assert_eq!(
                quota_client_with_proxies(&env, &BTreeMap::new())
                    .err()
                    .as_deref(),
                Some("Invalid HTTPS_PROXY setting for quota refresh")
            );
        }
    }

    #[test]
    fn skips_login_shell_proxy_probe_on_windows() {
        assert!(!should_probe_login_shell_for_os(true));
        assert!(should_probe_login_shell_for_os(false));
    }
}
