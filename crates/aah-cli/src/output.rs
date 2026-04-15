use std::path::Path;

use aah_core::app_settings::models::RelaySettings;
use aah_core::cli_facade::{
    AccountMetadataExport, AddOutcome, CliFacade, Provider, SwitchSelection,
};
use aah_core::relay::RelayRuntimeStatus;

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
            "{:<8}  {:<6}  {:<36}  {}",
            "PROVIDER", "ACTIVE", "ACCOUNT", "SUMMARY"
        );
        for row in rows {
            println!(
                "{:<8}  {:<6}  {:<36}  {}",
                provider_label(row.provider),
                if row.is_active { "yes" } else { "no" },
                account_display(row.label.as_deref(), &row.email),
                row.summary
            );
        }
    }
    Ok(())
}

pub fn print_add(facade: &CliFacade, provider: Provider, json: bool) -> Result<(), String> {
    let outcome = facade.add(provider).map_err(|error| error.to_string())?;
    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&outcome).map_err(|error| error.to_string())?
        );
    } else {
        println!("{}", add_message(&outcome));
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
        println!("{:<8}  {:<36}  {}", "PROVIDER", "ACTIVE ACCOUNT", "SUMMARY");
        for row in rows {
            println!(
                "{:<8}  {:<36}  {}",
                provider_label(row.provider),
                match row.active_email {
                    Some(email) => account_display(row.active_label.as_deref(), &email),
                    None => "-".to_string(),
                },
                row.summary
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
        "{} {} account to {} ({}).",
        if outcome.already_active {
            "Re-synced"
        } else {
            "Switched"
        },
        provider_title(outcome.provider),
        outcome.email,
        outcome.id
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
        "Removed {} account {} ({}).",
        provider_title(outcome.provider),
        outcome.email,
        outcome.id
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
            "Labelled {} account {} ({}) as \"{}\".",
            provider_title(outcome.provider),
            outcome.email,
            outcome.id,
            label
        ),
        None => println!(
            "Cleared label for {} account {} ({}).",
            provider_title(outcome.provider),
            outcome.email,
            outcome.id
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
        "Exported metadata for {} account(s) to {}. Credentials were not exported.",
        metadata.accounts.len(),
        output.display()
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
        "Imported metadata for {} account(s); skipped {}.",
        outcome.imported_count,
        outcome.skipped.len()
    );
    for skipped in outcome.skipped {
        println!(
            "Skipped {} account {} ({}): {}",
            skipped.provider, skipped.email, skipped.id, skipped.reason
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
        println!("{:<8}  {:<4}  MESSAGE", "PROVIDER", "OK");
        for row in rows {
            println!(
                "{:<8}  {:<4}  {}",
                provider_label(row.provider),
                if row.ok { "yes" } else { "no" },
                row.message.unwrap_or_else(|| "-".to_string())
            );
        }
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
        println!("running: {}", if status.running { "yes" } else { "no" });
        println!("host: {}", status.bind_host);
        println!("port: {}", status.port);
        println!("codex: {}", status.codex_base_url);
        if let Some(error) = status.last_error {
            println!("error: {error}");
        }
    }
    Ok(())
}

pub fn print_relay_settings(action: &str, settings: RelaySettings) {
    println!(
        "{action}: enabled={}, port={}",
        if settings.enabled { "yes" } else { "no" },
        settings.port
    );
}

pub fn print_relay_started(status: RelayRuntimeStatus) {
    println!("Relay running at {}", status.codex_base_url);
}

pub fn print_relay_stopped(stopped: bool) {
    if stopped {
        println!("Relay stopped.");
    } else {
        println!("Relay is not running.");
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
