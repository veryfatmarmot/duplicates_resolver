use anyhow::{Context, Result, anyhow};
use duplicates_resolver::{
    json_helper::{read_from_json, save_to_json},
    mover::move_duplicates,
    scanner::{collect_duplicates, prescan_duplicates},
    types::*,
};
use std::env;
use utils::ScopeTimeLogger;

// =============================================================================================

const USAGE: &str = "
Usage:
1. scan <scan_path> <json_output_path> [--threads=<count>]
    scans the path for the duplicates and builds the JSON report
    the duplicate are sourted in the lexicographical order by their folder paths
    --threads is optional (defaults to CPU core count)
2. move <json_input_path> <dst_path> [--no-path-len-check]
    moves the duplicates found in the JSON report to the destination path
    the first path in the duplicates list is considered the original and is not moved
    --no-path-len-check flag disables the max path length check (may lead to errors on older Windows)
3. prescan <scan_path>
    prescans the path and shows estimated number of duplicates without building the JSON report + some additional info";

// =============================================================================================

fn main() -> Result<()> {
    let mut args = env::args();
    args.next(); // skip exe name

    let command = args
        .next()
        .context(format!("\nNo command provided.\n{USAGE}\n"))?;

    let _scoped_time_logger = ScopeTimeLogger::new("Total execution time");

    match command.as_str() {
        "prescan" => run_prescan_command(&mut args).context("Prescan command failed")?,
        "scan" => run_scan_command(&mut args).context("Scan command failed")?,
        "move" => run_move_command(&mut args).context("Move command failed")?,
        _ => return Err(anyhow!("\nUnknown command: {command}\n{USAGE}\n")),
    }

    Ok(())
}

fn run_scan_command(args: &mut env::Args) -> Result<()> {
    let path = args
        .next()
        .context(format!("\nNo source path provided.\n{USAGE}\n"))?;
    let json_path = args
        .next()
        .context(format!("\nNo JSON output path provided.\n{USAGE}\n"))?;
    let threads_count = parse_threads_count(args)?;

    let duplicates: DuplicatesRegistry =
        collect_duplicates(&path, threads_count).context("Failed to collect duplicates")?;

    save_to_json(duplicates, &json_path).context("Failed to save to JSON")?;

    Ok(())
}

fn run_move_command(args: &mut env::Args) -> Result<()> {
    let json_path = args
        .next()
        .context(format!("\nNo JSON output path provided.\n{USAGE}\n"))?;
    let path = args
        .next()
        .context(format!("\nNo source path provided.\n{USAGE}\n"))?;

    let mut check_max_path_len = true;
    if let Some(flag) = args.next() {
        if flag.trim() == "--no-path-len-check" {
            check_max_path_len = false;
            println!(
                "Warning: Skipping max path length check. This may lead to errors on Windows systems."
            );
        }
    }

    let duplicates: DuplicatesRegistry =
        read_from_json(&json_path).context("Failed to read from JSON")?;

    move_duplicates(duplicates, &path, check_max_path_len).context("Failed to move duplicates")?;

    Ok(())
}

fn run_prescan_command(args: &mut env::Args) -> Result<()> {
    let path = args
        .next()
        .context(format!("\nNo source path provided.\n{USAGE}\n"))?;

    prescan_duplicates(&path).context("Failed to prescan duplicates")?;

    Ok(())
}

fn parse_threads_count(args: &mut env::Args) -> Result<u8> {
    let threads_count = if let Some(threads_arg) = args.next() {
        let count: u8 = threads_arg
            .trim()
            .strip_prefix("--threads=")
            .context(format!(
                "\nInvalid threads argument format. Use --threads=<count>\n{USAGE}\n"
            ))?
            .parse()
            .context("Threads count must be a number")?;

        if count == 0 {
            return Err(anyhow!("Threads count must be at least 1"));
        }

        if count > 128 {
            return Err(anyhow!("Threads count cannot exceed 128"));
        }

        count
    } else {
        // Default to number of CPU cores
        let cpu_count = std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(4)
            .min(128) as u8;

        println!("Using {} threads (CPU core count)", cpu_count);
        cpu_count
    };

    Ok(threads_count)
}
