use clap::CommandFactory;
use clap::Parser;
use clap::ValueEnum;
use clap_complete::Shell;
use cli::Commands;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

mod cli;

fn main() -> eyre::Result<()> {
    color_eyre::install()?;
    let filter = tracing_subscriber::EnvFilter::builder()
        .with_default_directive(tracing::level_filters::LevelFilter::INFO.into())
        .from_env()?;
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(filter)
        .init();
    let cli = cli::Cli::parse();

    match cli.command {
        Commands::Man { output } => {
            let cmd = filkoll::cli::Cli::command();
            std::fs::create_dir_all(&output)?;
            clap_mangen::generate_to(cmd, &output)?;
        }
        Commands::Completions { output } => {
            let mut cmd = filkoll::cli::Cli::command();
            std::fs::create_dir_all(&output)?;
            for &shell in Shell::value_variants() {
                clap_complete::generate_to(shell, &mut cmd, "filkoll", &output)?;
            }
        }
    }
    Ok(())
}
