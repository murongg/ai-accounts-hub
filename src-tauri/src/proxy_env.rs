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
    let shell = if needs_shell_probe(&current) {
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

    for (key, value) in resolve_proxy_env_updates(&current, &shell) {
        std::env::set_var(key, value);
    }
}

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

        if current_value.is_none() || current.get(upper).is_none_or(|value| value.trim().is_empty()) {
            updates.insert(upper.to_string(), resolved_value.clone());
        }
        if current_value.is_none() || current.get(lower).is_none_or(|value| value.trim().is_empty()) {
            updates.insert(lower.to_string(), resolved_value);
        }
    }

    updates
}

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

fn family_value<'a>(proxy_env: &'a BTreeMap<String, String>, upper: &str, lower: &str) -> Option<&'a str> {
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
}
