mod output;
mod tui;

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
    List {
        #[arg(long)]
        provider: Option<ProviderArg>,
    },
    Current {
        #[arg(long)]
        provider: Option<ProviderArg>,
    },
    Switch {
        #[arg(long)]
        provider: ProviderArg,
        selector: String,
    },
    Refresh {
        #[arg(long)]
        provider: Option<ProviderArg>,
    },
    /// Open the interactive TUI
    Tui {
        #[arg(long, hide = true)]
        snapshot: bool,
    },
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
    Status,
    Start,
    Stop,
    Enable {
        #[arg(long)]
        port: Option<u16>,
    },
    Disable,
    SetPort {
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
    let context = bootstrap_context(None, cli.data_dir.clone())?;
    let facade = CliFacade::new(context);

    match cli.command {
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
        Commands::Tui { snapshot } => tui::run_tui(&facade, snapshot),
        Commands::Relay { command } => {
            handle_relay_command(&facade, command, cli.json, cli.data_dir)
        }
        Commands::RelayHost { port } => run_relay_host(cli.data_dir, port),
    }
}

fn into_provider(provider: ProviderArg) -> Provider {
    match provider {
        ProviderArg::Codex => Provider::Codex,
        ProviderArg::Claude => Provider::Claude,
        ProviderArg::Gemini => Provider::Gemini,
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
        RelayCommands::Enable { port } => {
            let settings = facade
                .relay_enable(port)
                .map_err(|error| error.to_string())?;
            output::print_relay_settings("Relay enabled", settings);
            Ok(())
        }
        RelayCommands::Disable => {
            let settings = facade.relay_disable().map_err(|error| error.to_string())?;
            output::print_relay_settings("Relay disabled", settings);
            Ok(())
        }
        RelayCommands::SetPort { port } => {
            let settings = facade
                .relay_set_port(port)
                .map_err(|error| error.to_string())?;
            output::print_relay_settings("Relay port updated", settings);
            Ok(())
        }
        RelayCommands::Stop => {
            let stopped = facade.relay_stop().map_err(|error| error.to_string())?;
            output::print_relay_stopped(stopped);
            Ok(())
        }
        RelayCommands::Start => start_relay_host(facade, data_dir),
    }
}

fn start_relay_host(facade: &CliFacade, data_dir: Option<PathBuf>) -> Result<(), String> {
    let existing = facade.relay_status().map_err(|error| error.to_string())?;
    if existing.running {
        output::print_relay_started(existing);
        return Ok(());
    }

    let settings = facade.relay_settings().map_err(|error| error.to_string())?;
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

    while state.is_local_runtime_running() {
        thread::sleep(Duration::from_millis(250));
        if !facade_alive(&registry_paths, port) {
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
