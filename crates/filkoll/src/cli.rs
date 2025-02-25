use clap::Parser;
use clap::Subcommand;

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
#[clap(disable_help_subcommand = true)]
pub struct Cli {
    /// Operation to perform
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Update index
    Update {
        /// Skip downloading the latest package data with pacman -Fy
        #[arg(short, long)]
        no_download: bool,
    },
    /// Check package files and search for unexpected files
    Binary {
        /// How fuzzy the search should be (max edit distance)
        #[arg(short, long, default_value = "1")]
        edit_distance: u8,
        /// Skip fuzzy search if there is an exact match
        #[arg(short, long)]
        no_fuzzy_if_exact: bool,
        /// File name to search for in PATH (PATH from the update command is
        /// used)
        #[arg()]
        search_term: String,
    },
}
