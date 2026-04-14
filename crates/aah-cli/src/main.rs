mod output;

use std::path::PathBuf;

use aah_core::bootstrap::bootstrap_context;
use aah_core::cli_facade::{CliFacade, Provider};
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
    }
}

fn into_provider(provider: ProviderArg) -> Provider {
    match provider {
        ProviderArg::Codex => Provider::Codex,
        ProviderArg::Claude => Provider::Claude,
        ProviderArg::Gemini => Provider::Gemini,
    }
}
