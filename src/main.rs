use duplicates_resolver::{
    json_helper::{read_from_json, save_to_json},
    mover::move_duplicates,
    scanner::collect_duplicates,
    types::*,
};
use std::env;
use utils::ScopeTimeLogger;

// =============================================================================================

const USAGE: &str = "
Usage:
1. scan <scan_path> <json_output_path> --threads=<count>
    scans the path for the duplicates and builds the JSON report
    the duplicate are sourted in the lexicographical order by their folder paths    
2. move <json_input_path> <dst_path>
    moves the duplicates found in the JSON report to the destination path
    the first path in the duplicates list is considered the original and is not moved";

// =============================================================================================

fn main() {
    let mut args = env::args();
    args.next(); // skip exe name

    let command = args
        .next()
        .expect(&format!("\nNo command provided.\n{USAGE}\n"));

    let _scoped_time_logger = ScopeTimeLogger::new("Total execution time");

    match command.as_str() {
        "scan" => run_scan_command(&mut args),
        "move" => run_move_command(&mut args),
        _ => panic!("\nUnknown command: {command}\n{USAGE}\n"),
    }
}

fn run_scan_command(args: &mut env::Args) {
    let path = args
        .next()
        .expect(&format!("\nNo source path provided.\n{USAGE}\n"));
    let json_path = args
        .next()
        .expect(&format!("\nNo JSON output path provided.\n{USAGE}\n"));
    let threads_count = parse_threads_count(args);

    let duplicates: DuplicatesRegistry = collect_duplicates(&path, threads_count);

    save_to_json(duplicates, &json_path);
}

fn run_move_command(args: &mut env::Args) {
    let json_path = args
        .next()
        .expect(&format!("\nNo JSON output path provided.\n{USAGE}\n"));
    let path = args
        .next()
        .expect(&format!("\nNo source path provided.\n{USAGE}\n"));

    let duplicates: DuplicatesRegistry = read_from_json(&json_path);

    move_duplicates(duplicates, &path);
}

fn parse_threads_count(args: &mut env::Args) -> u8 {
    let threads_arg = args
        .next()
        .expect(&format!("\nNo threads count provided.\n{USAGE}\n"));

    let threads_count: u8 = threads_arg
        .trim()
        .strip_prefix("--threads=")
        .expect(&format!("\nNo threads count provided\n{USAGE}\n"))
        .parse()
        .expect("Threads count must be a number");

    threads_count
}
