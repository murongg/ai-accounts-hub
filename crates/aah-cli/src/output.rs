use aah_core::app_settings::models::RelaySettings;
use aah_core::cli_facade::{CliFacade, Provider, SwitchSelection};
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
            "{:<8}  {:<6}  {:<28}  {}",
            "PROVIDER", "ACTIVE", "EMAIL", "SUMMARY"
        );
        for row in rows {
            println!(
                "{:<8}  {:<6}  {:<28}  {}",
                provider_label(row.provider),
                if row.is_active { "yes" } else { "no" },
                row.email,
                row.summary
            );
        }
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
        println!("{:<8}  {:<28}  {}", "PROVIDER", "ACTIVE EMAIL", "SUMMARY");
        for row in rows {
            println!(
                "{:<8}  {:<28}  {}",
                provider_label(row.provider),
                row.active_email.unwrap_or_else(|| "-".to_string()),
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
