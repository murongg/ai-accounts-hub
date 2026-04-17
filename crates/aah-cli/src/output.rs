use std::{env, io::IsTerminal, path::Path};

use aah_core::app_settings::models::RelaySettings;
use aah_core::cli_facade::{
    AccountMetadataExport, AddOutcome, CliFacade, CodexAutofillLoginInput, Provider,
    SwitchSelection,
};
use aah_core::relay::RelayRuntimeStatus;
use crossterm::style::Stylize;

pub fn print_list(
    facade: &CliFacade,
    provider: Option<Provider>,
    json: bool,
) -> Result<(), String> {
    let rows = facade.list(provider).map_err(|error| error.to_string())?;
    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&rows).map_err(|error| error.to_string())?
        );
    } else {
        println!(
            "{}  {}  {}  {}",
            header_cell("PROVIDER", 8),
            header_cell("ACTIVE", 6),
            header_cell("ACCOUNT", 36),
            paint("SUMMARY", Tone::Header)
        );
        for row in rows {
            println!(
                "{}  {}  {:<36}  {}",
                provider_cell(row.provider, 8),
                active_cell(row.is_active, 6),
                account_display(row.label.as_deref(), &row.email),
                summary_text(&row.summary)
            );
        }
    }
    Ok(())
}

pub fn print_add(facade: &CliFacade, provider: Provider, json: bool) -> Result<(), String> {
    let outcome = facade.add(provider).map_err(|error| error.to_string())?;
    print_add_outcome(&outcome, json)
}

pub fn print_add_codex_autofill(
    facade: &CliFacade,
    input: CodexAutofillLoginInput,
    json: bool,
) -> Result<(), String> {
    let outcome = facade
        .add_codex_autofill(input)
        .map_err(|error| error.to_string())?;
    print_add_outcome(&outcome, json)
}

fn print_add_outcome(outcome: &AddOutcome, json: bool) -> Result<(), String> {
    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(outcome).map_err(|error| error.to_string())?
        );
    } else {
        println!("{}", paint(add_message(outcome), Tone::Success));
    }
    Ok(())
}

pub fn print_current(
    facade: &CliFacade,
    provider: Option<Provider>,
    json: bool,
) -> Result<(), String> {
    let rows = facade
        .current(provider)
        .map_err(|error| error.to_string())?;
    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&rows).map_err(|error| error.to_string())?
        );
    } else {
        println!(
            "{}  {}  {}",
            header_cell("PROVIDER", 8),
            header_cell("ACTIVE ACCOUNT", 36),
            paint("SUMMARY", Tone::Header)
        );
        for row in rows {
            println!(
                "{}  {:<36}  {}",
                provider_cell(row.provider, 8),
                match row.active_email {
                    Some(email) => account_display(row.active_label.as_deref(), &email),
                    None => muted("-"),
                },
                summary_text(&row.summary)
            );
        }
    }
    Ok(())
}

pub fn print_switch(
    facade: &CliFacade,
    provider: Provider,
    selector: String,
) -> Result<(), String> {
    let selection = if selector.contains('@') {
        SwitchSelection::Email(selector.clone())
    } else {
        SwitchSelection::Id(selector.clone())
    };
    let outcome = facade
        .switch(provider, selection)
        .map_err(|error| error.to_string())?;
    println!(
        "{}",
        paint(
            format!(
                "{} {} account to {} ({}).",
                if outcome.already_active {
                    "Re-synced"
                } else {
                    "Switched"
                },
                provider_title(outcome.provider),
                outcome.email,
                outcome.id
            ),
            Tone::Success,
        )
    );
    Ok(())
}

pub fn print_remove(
    facade: &CliFacade,
    provider: Provider,
    selector: String,
) -> Result<(), String> {
    let selection = selection_from_selector(&selector);
    let outcome = facade
        .remove(provider, selection)
        .map_err(|error| error.to_string())?;
    println!(
        "{}",
        paint(
            format!(
                "Removed {} account {} ({}).",
                provider_title(outcome.provider),
                outcome.email,
                outcome.id
            ),
            Tone::Success,
        )
    );
    Ok(())
}

pub fn print_label(
    facade: &CliFacade,
    provider: Provider,
    selector: String,
    label: Option<String>,
) -> Result<(), String> {
    let selection = selection_from_selector(&selector);
    let outcome = facade
        .label(provider, selection, label)
        .map_err(|error| error.to_string())?;
    match outcome.label {
        Some(label) => println!(
            "{}",
            paint(
                format!(
                    "Labelled {} account {} ({}) as \"{}\".",
                    provider_title(outcome.provider),
                    outcome.email,
                    outcome.id,
                    label
                ),
                Tone::Success,
            )
        ),
        None => println!(
            "{}",
            paint(
                format!(
                    "Cleared label for {} account {} ({}).",
                    provider_title(outcome.provider),
                    outcome.email,
                    outcome.id
                ),
                Tone::Warning,
            )
        ),
    }
    Ok(())
}

pub fn print_export(facade: &CliFacade, output: &Path) -> Result<(), String> {
    let metadata = facade
        .export_metadata()
        .map_err(|error| error.to_string())?;
    let bytes = serde_json::to_vec_pretty(&metadata).map_err(|error| error.to_string())?;
    std::fs::write(output, bytes)
        .map_err(|error| format!("failed to write {}: {error}", output.display()))?;
    println!(
        "{}",
        paint(
            format!(
                "Exported metadata for {} account(s) to {}. Credentials were not exported.",
                metadata.accounts.len(),
                output.display()
            ),
            Tone::Success,
        )
    );
    Ok(())
}

pub fn print_import(facade: &CliFacade, input: &Path) -> Result<(), String> {
    let text = std::fs::read_to_string(input)
        .map_err(|error| format!("failed to read {}: {error}", input.display()))?;
    let metadata: AccountMetadataExport = serde_json::from_str(&text)
        .map_err(|error| format!("failed to parse metadata: {error}"))?;
    let outcome = facade
        .import_metadata(metadata)
        .map_err(|error| error.to_string())?;
    println!(
        "{}",
        paint(
            format!(
                "Imported metadata for {} account(s); skipped {}.",
                outcome.imported_count,
                outcome.skipped.len()
            ),
            Tone::Success,
        )
    );
    for skipped in outcome.skipped {
        println!(
            "{}",
            paint(
                format!(
                    "Skipped {} account {} ({}): {}",
                    skipped.provider, skipped.email, skipped.id, skipped.reason
                ),
                Tone::Warning,
            )
        );
    }
    Ok(())
}

pub fn print_refresh(
    facade: &CliFacade,
    provider: Option<Provider>,
    json: bool,
) -> Result<(), String> {
    let rows = facade
        .refresh(provider)
        .map_err(|error| error.to_string())?;
    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&rows).map_err(|error| error.to_string())?
        );
    } else {
        println!(
            "{}  {}  {}",
            header_cell("PROVIDER", 8),
            header_cell("OK", 4),
            paint("MESSAGE", Tone::Header)
        );
        for row in rows {
            println!(
                "{}  {}  {}",
                provider_cell(row.provider, 8),
                refresh_ok_cell(row.ok, 4),
                row.message
                    .as_deref()
                    .map(summary_text)
                    .unwrap_or_else(|| muted("-"))
            );
        }
    }
    Ok(())
}

pub fn print_paths(facade: &CliFacade) {
    let paths = facade.paths();
    println!("managed root: {}", paths.managed_root);
    println!("user home: {}", paths.user_home);
    for row in paths.rows {
        println!("{} {}: {}", row.scope, row.name, row.path);
    }
}

pub fn print_doctor(facade: &CliFacade) {
    let report = facade.doctor();
    println!("{}", paint("AAH doctor", Tone::Header));
    println!("managed root: {}", report.managed_root);
    println!("user home: {}", report.user_home);
    match (report.relay, report.relay_error) {
        (Some(status), _) => {
            println!(
                "{}",
                paint(
                    format!(
                        "relay: {} ({})",
                        if status.running { "running" } else { "stopped" },
                        status.codex_base_url
                    ),
                    if status.running {
                        Tone::Success
                    } else {
                        Tone::Warning
                    },
                )
            );
        }
        (None, Some(error)) => {
            println!("{}", paint(format!("relay: warn ({error})"), Tone::Warning))
        }
        (None, None) => println!(
            "{}",
            paint("relay: warn (status unavailable)", Tone::Warning)
        ),
    }

    println!("{}", paint("providers:", Tone::Header));
    for provider in report.providers {
        let state = if provider.issues.is_empty() {
            "ok"
        } else {
            "warn"
        };
        let line = format!(
            "{}: {} accounts={}, active={}, cli={}",
            provider_label(provider.provider),
            state,
            provider.account_count,
            provider.active_email.unwrap_or_else(|| "-".to_string()),
            provider.cli_path.unwrap_or_else(|| "not found".to_string())
        );
        println!(
            "{}",
            paint(
                line,
                if provider.issues.is_empty() {
                    Tone::Success
                } else {
                    Tone::Warning
                }
            )
        );
        for issue in provider.issues {
            println!("{}", paint(format!("  - {issue}"), Tone::Warning));
        }
    }

    for warning in report.import_warnings {
        println!(
            "{}",
            paint(format!("import warning: {warning}"), Tone::Warning)
        );
    }
}

pub fn print_doctor_fix(facade: &CliFacade) -> Result<(), String> {
    let report = facade.doctor_fix().map_err(|error| error.to_string())?;
    println!("{}", paint("AAH doctor --fix", Tone::Header));
    println!("managed root: {}", report.managed_root);
    println!("user home: {}", report.user_home);
    for fix in report.fixes {
        println!(
            "{}",
            paint(
                format!("{}: {} - {}", fix.status, fix.name, fix.message),
                doctor_fix_tone(&fix.status)
            )
        );
    }
    Ok(())
}

pub fn print_relay_status(facade: &CliFacade, json: bool) -> Result<(), String> {
    let status = facade.relay_status().map_err(|error| error.to_string())?;
    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&status).map_err(|error| error.to_string())?
        );
    } else {
        println!(
            "{}",
            paint(
                format!("running: {}", if status.running { "yes" } else { "no" }),
                if status.running {
                    Tone::Success
                } else {
                    Tone::Warning
                }
            )
        );
        println!("host: {}", status.bind_host);
        println!("port: {}", status.port);
        println!("codex: {}", status.codex_base_url);
        if let Some(error) = status.last_error {
            println!("{}", paint(format!("error: {error}"), Tone::Danger));
        }
    }
    Ok(())
}

pub fn print_relay_settings(action: &str, settings: RelaySettings) {
    println!(
        "{}",
        paint(
            format!(
                "{action}: enabled={}, port={}",
                if settings.enabled { "yes" } else { "no" },
                settings.port
            ),
            Tone::Success,
        )
    );
}

pub fn print_relay_started(status: RelayRuntimeStatus) {
    println!(
        "{}",
        paint(
            format!("Relay running at {}", status.codex_base_url),
            Tone::Success
        )
    );
}

pub fn print_relay_stopped(stopped: bool) {
    if stopped {
        println!("{}", paint("Relay stopped.", Tone::Warning));
    } else {
        println!("{}", paint("Relay is not running.", Tone::Muted));
    }
}

fn provider_label(provider: Provider) -> &'static str {
    match provider {
        Provider::Codex => "codex",
        Provider::Claude => "claude",
        Provider::Gemini => "gemini",
    }
}

fn provider_title(provider: Provider) -> &'static str {
    match provider {
        Provider::Codex => "Codex",
        Provider::Claude => "Claude",
        Provider::Gemini => "Gemini",
    }
}

fn account_display(label: Option<&str>, email: &str) -> String {
    match label {
        Some(label) => format!("{label} <{email}>"),
        None => email.to_string(),
    }
}

fn selection_from_selector(selector: &str) -> SwitchSelection {
    if selector.contains('@') {
        SwitchSelection::Email(selector.to_string())
    } else {
        SwitchSelection::Id(selector.to_string())
    }
}

fn add_message(outcome: &AddOutcome) -> String {
    format!(
        "Added {} account {} ({}) to the account pool. Current active account was not changed.",
        provider_title(outcome.provider),
        outcome.email,
        outcome.id
    )
}

#[derive(Copy, Clone)]
enum Tone {
    Header,
    Success,
    Warning,
    Danger,
    Muted,
    Provider(Provider),
}

fn colors_enabled() -> bool {
    if env::var_os("NO_COLOR").is_some() {
        return false;
    }

    match env::var("CLICOLOR_FORCE") {
        Ok(value) if value != "0" => true,
        _ => std::io::stdout().is_terminal(),
    }
}

fn paint(text: impl Into<String>, tone: Tone) -> String {
    let text = text.into();
    if !colors_enabled() {
        return text;
    }

    match tone {
        Tone::Header => text.as_str().cyan().bold().to_string(),
        Tone::Success => text.as_str().green().bold().to_string(),
        Tone::Warning => text.as_str().yellow().bold().to_string(),
        Tone::Danger => text.as_str().red().bold().to_string(),
        Tone::Muted => text.as_str().dark_grey().to_string(),
        Tone::Provider(Provider::Codex) => text.as_str().green().bold().to_string(),
        Tone::Provider(Provider::Claude) => text.as_str().yellow().bold().to_string(),
        Tone::Provider(Provider::Gemini) => text.as_str().blue().bold().to_string(),
    }
}

fn header_cell(text: &str, width: usize) -> String {
    paint(format!("{text:<width$}"), Tone::Header)
}

fn provider_cell(provider: Provider, width: usize) -> String {
    paint(
        format!("{:<width$}", provider_label(provider)),
        Tone::Provider(provider),
    )
}

fn active_cell(active: bool, width: usize) -> String {
    paint(
        format!("{:<width$}", if active { "yes" } else { "no" }),
        if active { Tone::Success } else { Tone::Muted },
    )
}

fn refresh_ok_cell(ok: bool, width: usize) -> String {
    paint(
        format!("{:<width$}", if ok { "yes" } else { "no" }),
        if ok { Tone::Success } else { Tone::Danger },
    )
}

fn summary_text(summary: &str) -> String {
    if summary == "-" {
        muted(summary)
    } else {
        summary.to_string()
    }
}

fn muted(text: &str) -> String {
    paint(text, Tone::Muted)
}

fn doctor_fix_tone(status: &str) -> Tone {
    match status {
        "fixed" => Tone::Success,
        "warn" => Tone::Warning,
        "error" => Tone::Danger,
        _ => Tone::Header,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_message_mentions_account_pool_and_no_activation() {
        let message = add_message(&AddOutcome {
            provider: Provider::Codex,
            id: "codex-123".to_string(),
            email: "user@example.com".to_string(),
            activated: false,
        });

        assert!(message.contains("Added Codex account user@example.com (codex-123)"));
        assert!(message.contains("account pool"));
        assert!(message.contains("not changed"));
    }
}
