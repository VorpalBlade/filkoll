use anstyle::Effects;
use anstyle::Reset;
use clap::Parser;
use filkoll::cli::Cli;
use std::fmt::Write as _;
use std::io::Write as _;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

#[cfg(target_env = "musl")]
mod _musl {
    use mimalloc::MiMalloc;
    #[global_allocator]
    static GLOBAL: MiMalloc = MiMalloc;
}

fn main() -> eyre::Result<()> {
    color_eyre::install()?;
    // Set up logging with tracing
    let filter = tracing_subscriber::EnvFilter::builder()
        .with_default_directive(tracing::level_filters::LevelFilter::INFO.into())
        .from_env()?;
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(filter)
        .with(tracing_error::ErrorLayer::default())
        .init();
    let cli = Cli::parse();

    match cli.command {
        filkoll::cli::Commands::Update { no_download } => {
            filkoll::update::update(!no_download)?;
        }
        filkoll::cli::Commands::Binary {
            edit_distance,
            search_term,
            no_fuzzy_if_exact,
        } => {
            let mut candidates =
                filkoll::lookup::lookup(edit_distance, no_fuzzy_if_exact, &search_term)?;
            candidates.sort();
            let mut stdout = anstream::stdout().lock();
            let mut maxwidth1 = 0;
            for candidate in &candidates {
                maxwidth1 = maxwidth1.max(candidate.repo.len() + candidate.package.len() + 1);
            }
            let mut buf1 = String::new();
            for candidate in &candidates {
                buf1.clear();
                write!(&mut buf1, "{}/{}", candidate.repo, candidate.package)?;
                if candidate.command == search_term {
                    writeln!(
                        stdout,
                        "{}{buf1: <maxwidth1$}        /{}/{}{}    (exact match)",
                        Effects::BOLD.render(),
                        candidate.directory,
                        candidate.command,
                        Reset.render()
                    )?;
                } else {
                    writeln!(
                        stdout,
                        "{buf1: <maxwidth1$}        /{}/{}",
                        candidate.directory, candidate.command
                    )?;
                }
            }
        }
    }

    Ok(())
}
