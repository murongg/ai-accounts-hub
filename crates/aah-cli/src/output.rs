use aah_core::cli_facade::{CliFacade, Provider, SwitchSelection};

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
