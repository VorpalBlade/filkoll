use anstyle::Effects;
use anstyle::Reset;
use clap::Parser;
use filkoll::cli::Cli;
use filkoll::lookup::LookupError;
use std::fmt::Write as _;
use std::io::Write as _;
use std::process::ExitCode;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

#[cfg(target_env = "musl")]
mod _musl {
    use mimalloc::MiMalloc;
    #[global_allocator]
    static GLOBAL: MiMalloc = MiMalloc;
}

fn main() -> eyre::Result<ExitCode> {
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
            cmd_not_found_handler,
            no_fuzzy_if_exact,
        } => {
            let candidates =
                filkoll::lookup::lookup(edit_distance, no_fuzzy_if_exact, &search_term);
            let mut candidates = match candidates {
                Ok(candidates) => candidates,
                Err(LookupError::Other(e)) => {
                    return Err(e);
                }
                Err(err) => {
                    let mut stderr = anstream::stderr().lock();
                    writeln!(
                        stderr,
                        "{}Error{}: {err}",
                        Effects::BOLD.render(),
                        Reset.render()
                    )?;
                    let suggestion = err
                        .suggestion()
                        .unwrap_or_else(|| "No suggestion available.".to_string());
                    writeln!(
                        stderr,
                        "\n{}Suggestion for fix{}:\n{suggestion}",
                        Effects::BOLD.render(),
                        Reset.render(),
                    )?;
                    return Ok(ExitCode::from(1u8));
                }
            };
            if cmd_not_found_handler && candidates.is_empty() {
                // If we are in command-not-found handler mode and there are no candidates,
                // we should exit with code 2 to signal this to the shell script function.
                //
                // This is used to tell apart the case where we failed with an error (1).
                return Ok(ExitCode::from(2u8));
            }
            candidates.sort();
            let mut stdout = anstream::stdout().lock();
            let mut maxwidth1 = 0;
            for candidate in &candidates {
                maxwidth1 = maxwidth1.max(candidate.repo.len() + candidate.package.len() + 1);
            }
            if cmd_not_found_handler {
                // Since the vec is sorted, and we have at least one candidate, this will be
                // true:
                let first = candidates.first().expect("candidates is not empty");
                if first.distance == 0 {
                    writeln!(
                        stdout,
                        "{search_term} may be found in the following packages:"
                    )?;
                } else {
                    writeln!(
                        stdout,
                        "{search_term} not found, but the following are similar:"
                    )?;
                }
            }
            let indent = if cmd_not_found_handler { "  " } else { "" };
            let mut buf1 = String::new();
            for candidate in &candidates {
                buf1.clear();
                write!(&mut buf1, "{}/{}", candidate.repo, candidate.package)?;
                if candidate.command == search_term {
                    writeln!(
                        stdout,
                        "{indent}{}{buf1: <maxwidth1$}        /{}/{}{}    (exact match)",
                        Effects::BOLD.render(),
                        candidate.directory,
                        candidate.command,
                        Reset.render()
                    )?;
                } else {
                    writeln!(
                        stdout,
                        "{indent}{buf1: <maxwidth1$}        /{}/{}",
                        candidate.directory, candidate.command
                    )?;
                }
            }
        }
    }

    Ok(ExitCode::SUCCESS)
}
