mod output;
mod tui;
mod upgrade;

use std::io::{self, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use aah_core::bootstrap::bootstrap_context;
use aah_core::claude_accounts::paths::ClaudeAccountPaths;
use aah_core::cli_facade::{CliFacade, Provider};
use aah_core::codex_accounts::paths::CodexAccountPaths;
use aah_core::gemini_accounts::paths::GeminiAccountPaths;
use aah_core::relay::credentials::LiveRelayCredentialSource;
use aah_core::relay::registry::RelayRegistryPaths;
use aah_core::relay::{RelayOwnerKind, RelayServerState};
use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser)]
#[command(name = "aah", version, about = "AI Accounts Hub CLI")]
struct Cli {
    #[arg(long, global = true)]
    json: bool,
    #[arg(long, global = true)]
    data_dir: Option<PathBuf>,
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start a provider login flow and add the account to the managed pool
    Add {
        #[arg(long, help = "Provider to add")]
        provider: ProviderArg,
    },
    /// List managed accounts in the shared account pool
    List {
        #[arg(long, help = "Filter accounts by provider")]
        provider: Option<ProviderArg>,
    },
    /// Show the currently active account for each provider
    Current {
        #[arg(long, help = "Filter active accounts by provider")]
        provider: Option<ProviderArg>,
    },
    /// Switch the active account for a provider
    Switch {
        #[arg(long, help = "Provider whose active account should change")]
        provider: ProviderArg,
        #[arg(help = "Account email or managed account ID to activate")]
        selector: String,
    },
    /// Refresh usage and account status for managed accounts
    Refresh {
        #[arg(long, help = "Refresh only one provider")]
        provider: Option<ProviderArg>,
    },
    /// Remove a managed account from the shared account pool
    Remove {
        #[arg(long, help = "Provider whose account should be removed")]
        provider: ProviderArg,
        #[arg(help = "Account email or managed account ID to remove")]
        selector: String,
        #[arg(long, help = "Skip the interactive confirmation prompt")]
        yes: bool,
    },
    /// Set or clear a managed account display label
    Label {
        #[arg(long, help = "Provider whose account label should change")]
        provider: ProviderArg,
        #[arg(help = "Account email or managed account ID to label")]
        selector: String,
        #[arg(
            help = "Display label to show in list and TUI output",
            required_unless_present = "clear",
            conflicts_with = "clear"
        )]
        label: Option<String>,
        #[arg(long, help = "Clear the account display label")]
        clear: bool,
    },
    /// Upgrade the installed CLI
    Upgrade,
    /// Open the interactive TUI
    Tui {
        #[arg(long, hide = true)]
        snapshot: bool,
    },
    /// Manage the local relay server
    Relay {
        #[command(subcommand)]
        command: RelayCommands,
    },
    #[command(hide = true, name = "relay-host")]
    RelayHost {
        #[arg(long)]
        port: u16,
    },
}

#[derive(Subcommand)]
enum RelayCommands {
    /// Show local relay runtime status
    Status,
    /// Start the local relay server
    Start {
        #[arg(long, help = "Port to bind the relay server to")]
        port: Option<u16>,
    },
    /// Stop the local relay server
    Stop,
    /// Persist the relay port setting
    SetPort {
        #[arg(help = "Port to persist for future relay starts")]
        port: u16,
    },
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, ValueEnum)]
enum ProviderArg {
    Codex,
    Claude,
    Gemini,
}

fn main() -> Result<(), String> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Upgrade => upgrade::run(cli.json),
        command => {
            let context = bootstrap_context(None, cli.data_dir.clone())?;
            let facade = CliFacade::new(context);

            match command {
                Commands::Add { provider } => {
                    output::print_add(&facade, into_provider(provider), cli.json)
                }
                Commands::List { provider } => {
                    output::print_list(&facade, provider.map(into_provider), cli.json)
                }
                Commands::Current { provider } => {
                    output::print_current(&facade, provider.map(into_provider), cli.json)
                }
                Commands::Switch { provider, selector } => {
                    output::print_switch(&facade, into_provider(provider), selector)
                }
                Commands::Refresh { provider } => {
                    output::print_refresh(&facade, provider.map(into_provider), cli.json)
                }
                Commands::Remove {
                    provider,
                    selector,
                    yes,
                } => {
                    ensure_text_only("remove", cli.json)?;
                    let provider = into_provider(provider);
                    if !yes && !confirm_remove(provider, &selector)? {
                        println!("Remove cancelled.");
                        return Ok(());
                    }
                    output::print_remove(&facade, provider, selector)
                }
                Commands::Label {
                    provider,
                    selector,
                    label,
                    clear,
                } => {
                    ensure_text_only("label", cli.json)?;
                    let label = if clear { None } else { label };
                    output::print_label(&facade, into_provider(provider), selector, label)
                }
                Commands::Tui { snapshot } => tui::run_tui(&facade, snapshot),
                Commands::Relay { command } => {
                    handle_relay_command(&facade, command, cli.json, cli.data_dir)
                }
                Commands::RelayHost { port } => run_relay_host(cli.data_dir, port),
                Commands::Upgrade => unreachable!("upgrade handled before bootstrap"),
            }
        }
    }
}

fn ensure_text_only(command: &str, json: bool) -> Result<(), String> {
    if json {
        Err(format!("--json is not supported for {command}"))
    } else {
        Ok(())
    }
}

fn confirm_remove(provider: Provider, selector: &str) -> Result<bool, String> {
    print!(
        "Remove {} account {} from the account pool? [y/N] ",
        provider_title(provider),
        selector
    );
    io::stdout()
        .flush()
        .map_err(|error| format!("failed to write prompt: {error}"))?;

    let mut answer = String::new();
    io::stdin()
        .read_line(&mut answer)
        .map_err(|error| format!("failed to read confirmation: {error}"))?;
    Ok(matches!(
        answer.trim().to_ascii_lowercase().as_str(),
        "y" | "yes"
    ))
}

fn into_provider(provider: ProviderArg) -> Provider {
    match provider {
        ProviderArg::Codex => Provider::Codex,
        ProviderArg::Claude => Provider::Claude,
        ProviderArg::Gemini => Provider::Gemini,
    }
}

fn provider_title(provider: Provider) -> &'static str {
    match provider {
        Provider::Codex => "Codex",
        Provider::Claude => "Claude",
        Provider::Gemini => "Gemini",
    }
}

fn handle_relay_command(
    facade: &CliFacade,
    command: RelayCommands,
    json: bool,
    data_dir: Option<PathBuf>,
) -> Result<(), String> {
    match command {
        RelayCommands::Status => output::print_relay_status(facade, json),
        RelayCommands::Start { port } => start_relay_host(facade, data_dir, port),
        RelayCommands::SetPort { port } => {
            let settings = facade
                .relay_set_port(port)
                .map_err(|error| error.to_string())?;
            output::print_relay_settings("Relay port updated", settings);
            Ok(())
        }
        RelayCommands::Stop => {
            let _ = facade.relay_disable().map_err(|error| error.to_string())?;
            let stopped = facade.relay_stop().map_err(|error| error.to_string())?;
            output::print_relay_stopped(stopped);
            Ok(())
        }
    }
}

fn start_relay_host(
    facade: &CliFacade,
    data_dir: Option<PathBuf>,
    port_override: Option<u16>,
) -> Result<(), String> {
    let settings = facade
        .relay_enable(port_override)
        .map_err(|error| error.to_string())?;
    let existing = facade.relay_status().map_err(|error| error.to_string())?;
    if existing.running {
        output::print_relay_started(existing);
        return Ok(());
    }

    let current_exe = std::env::current_exe()
        .map_err(|error| format!("failed to resolve current exe: {error}"))?;
    let mut command = Command::new(current_exe);
    if let Some(data_dir) = data_dir {
        command.arg("--data-dir").arg(data_dir);
    }
    command
        .arg("relay-host")
        .arg("--port")
        .arg(settings.port.to_string())
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    command
        .spawn()
        .map_err(|error| format!("failed to start relay host: {error}"))?;

    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        let status = facade.relay_status().map_err(|error| error.to_string())?;
        if status.running {
            output::print_relay_started(status);
            return Ok(());
        }
        if Instant::now() >= deadline {
            return Err("relay host did not become ready within 5 seconds".to_string());
        }
        thread::sleep(Duration::from_millis(100));
    }
}

fn run_relay_host(data_dir: Option<PathBuf>, port: u16) -> Result<(), String> {
    let context = bootstrap_context(None, data_dir)?;
    let codex_paths =
        CodexAccountPaths::from_roots(context.managed_root.clone(), context.user_home.clone());
    let claude_paths =
        ClaudeAccountPaths::from_roots(context.managed_root.clone(), context.user_home.clone());
    let gemini_paths =
        GeminiAccountPaths::from_roots(context.managed_root.clone(), context.user_home);
    let registry_paths = RelayRegistryPaths::from_managed_root(&context.managed_root);
    let credential_source = std::sync::Arc::new(LiveRelayCredentialSource::new(
        codex_paths,
        claude_paths,
        gemini_paths,
    ));
    let state = RelayServerState::default();
    let runtime = tokio::runtime::Runtime::new()
        .map_err(|error| format!("failed to create relay runtime: {error}"))?;
    let status = runtime.block_on(state.apply_settings(
        aah_core::app_settings::models::RelaySettings {
            enabled: true,
            port,
        },
        credential_source,
        &registry_paths,
        RelayOwnerKind::Cli,
    ));
    if !status.running {
        return Err(status
            .last_error
            .unwrap_or_else(|| "relay host failed to start".to_string()));
    }

    let mut missed_health_checks = 0u8;
    while state.is_local_runtime_running() {
        thread::sleep(Duration::from_millis(250));
        if facade_alive(&registry_paths, port) {
            missed_health_checks = 0;
            continue;
        }
        missed_health_checks = missed_health_checks.saturating_add(1);
        if missed_health_checks >= 8 {
            break;
        }
    }
    Ok(())
}

fn facade_alive(registry_paths: &RelayRegistryPaths, port: u16) -> bool {
    let status = aah_core::relay::registry::shared_runtime_status(
        &aah_core::app_settings::models::RelaySettings {
            enabled: true,
            port,
        },
        registry_paths,
        None,
    );
    status.running
}
