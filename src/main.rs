//! CLI for reading Apple Silicon hardware performance counters.
//!
//! Two subcommands:
//! - **`list`**: Discover available PMC events for the current CPU by reading
//!   the kpep database at `/usr/share/kpep/`.
//! - **`stat`**: Measure hardware performance counters while running a command.
//!
//! ## Counting modes
//!
//! **Per-process** (default): A dylib is injected via `DYLD_INSERT_LIBRARIES`
//! that uses `pthread_introspection_hook` to track thread lifecycle and
//! accumulates per-thread counter deltas. Results include the target process
//! and all descendant processes (via `pthread_atfork` and environment inheritance).
//!
//! **System-wide** (`-S`): Reads global counters summed across all CPUs before
//! and after the command. Includes background system activity.
//!
//! Both modes require root privileges (`sudo`) for counter access.

#[cfg(target_os = "macos")]
mod stat;

#[cfg(target_os = "macos")]
use crate::stat::cmd_stat;
use apmc::kpep::KpepDatabase;
use clap::{Parser, Subcommand};

/// Apple Silicon hardware performance counters.
#[derive(Parser)]
#[command(
    name = "apmc",
    version,
    about = "Apple Silicon hardware performance counters"
)]
struct Cli {
    /// Disable colored output.
    #[arg(long = "no-color", global = true)]
    no_color: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// List available PMC events for the current CPU.
    List {
        /// Path to the kpep database plist
        #[cfg(target_os = "macos")]
        path: Option<std::path::PathBuf>,
        #[cfg(not(target_os = "macos"))]
        path: std::path::PathBuf,
    },

    /// Measure hardware counters for a command (requires sudo).
    #[command(
        trailing_var_arg = true,
        after_help = concat!(
            "Default events: L1D_CACHE_MISS_LD, L1D_CACHE_MISS_ST, ATOMIC_OR_EXCLUSIVE_FAIL,\n",
            "  MAP_STALL, LDST_X64_UOP, BRANCH_MISPRED_NONSPEC, MAP_SIMD_UOP, SCHEDULE_EMPTY\n\n",
            "Run `apmc list` to see all available events.",
        )
    )]
    #[cfg(target_os = "macos")]
    Stat {
        /// Comma-separated list of events to monitor.
        #[arg(short = 'e', long = "events", value_delimiter = ',')]
        events: Option<Vec<String>>,

        /// Use system-wide counting instead of per-process.
        #[arg(short = 's', long = "system-wide")]
        system_wide: bool,

        /// Region mode: only measure code between apmc_start()/apmc_stop() calls.
        #[arg(short = 'r', long = "region", conflicts_with = "system_wide")]
        region: bool,

        /// Command and arguments to run.
        #[arg(required = true, allow_hyphen_values = true)]
        command: Vec<String>,
    },
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        #[cfg(target_os = "macos")]
        Commands::List { path } => cmd_list(path.as_deref()),
        #[cfg(not(target_os = "macos"))]
        Commands::List { path } => cmd_list(path),
        #[cfg(target_os = "macos")]
        Commands::Stat {
            events,
            system_wide,
            region,
            command,
        } => cmd_stat(
            events.unwrap_or_default(),
            system_wide,
            region,
            &command,
            cli.no_color,
        ),
    }
}

/// List all PMC events available on the current CPU.
///
/// Reads the kpep database from `/usr/share/kpep/` and prints fixed counters,
/// configurable events, aliases, and counter slot masks. When `filter` is
/// provided, only events whose name or description matches (case-insensitive)
/// are shown.
#[cfg(target_os = "macos")]
fn cmd_list(path: Option<&std::path::Path>) -> Result<(), Box<dyn std::error::Error>> {
    let db = match path {
        Some(path) => KpepDatabase::load_from_path(path),
        None => KpepDatabase::load_current_cpu(),
    }?;
    println!("{db}");
    Ok(())
}
#[cfg(not(target_os = "macos"))]
fn cmd_list(path: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
    let db = KpepDatabase::load_from_path(path)?;
    println!("{db}");
    Ok(())
}
