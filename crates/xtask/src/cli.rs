use camino::Utf8PathBuf;
use clap::Parser;
use clap::Subcommand;

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
#[clap(disable_help_subcommand = true)]
pub(crate) struct Cli {
    /// Operation to perform
    #[command(subcommand)]
    pub(crate) command: Commands,
}

#[derive(Debug, Subcommand)]
pub(crate) enum Commands {
    /// Generate man page
    Man {
        /// Output directory
        #[arg(short, long)]
        output: Utf8PathBuf,
    },
    /// Generate shell completions
    Completions {
        /// Output directory
        #[arg(short, long)]
        output: Utf8PathBuf,
    },
}
