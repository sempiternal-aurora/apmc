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
        #[arg(short, long)]
        path: Option<std::path::PathBuf>,

        /// Case-insensitive filter applied to event names and descriptions.
        filter: Option<String>,
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
        Commands::List { path, filter } => cmd_list(path.as_deref(), filter.as_deref()),
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
fn cmd_list(
    path: Option<&std::path::Path>,
    filter: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    let db = match path {
        Some(path) => KpepDatabase::load_from_path(path),
        #[cfg(target_os = "macos")]
        None => KpepDatabase::load_current_cpu(path),
        #[cfg(not(target_os = "macos"))]
        None => Err(apmc::kpep::KpepError::NoDefaultDatabase),
    }?;

    println!("CPU: {} ({})", db.cpu.marketing_name, db.cpu.architecture);
    println!(
        "Fixed counters: {}, Configurable counters: {}",
        db.cpu.fixed_counters, db.cpu.config_counters
    );

    if !db.cpu.aliases.is_empty() {
        println!("\nAliases:");
        for (alias, target) in &db.cpu.aliases {
            println!("  {alias} -> {target}");
        }
    }

    let fixed: Vec<_> = db.fixed_events().collect();
    if !fixed.is_empty() {
        println!("\nFixed counters:");
        for event in &fixed {
            println!(
                "  [fixed {}] {:<35} {}",
                event.fixed_counter.unwrap_or(0),
                event.name,
                event.description
            );
        }
    }

    println!("\nConfigurable events:");
    let mut count = 0;
    for event in db.configurable_events() {
        if let Some(pattern) = filter {
            let pattern_lower = pattern.to_lowercase();
            if !event.name.to_lowercase().contains(&pattern_lower)
                && !event.description.to_lowercase().contains(&pattern_lower)
            {
                continue;
            }
        }

        let mask_str = match event.counters_mask {
            Some(mask) => format!("mask=0x{mask:x}"),
            None => "any slot".to_string(),
        };

        println!(
            "  [{:#04x}] {:<35} ({}) {}",
            event.number.unwrap_or(0),
            event.name,
            mask_str,
            event.description,
        );
        count += 1;
    }
    println!("\n{count} events listed.");

    Ok(())
}
