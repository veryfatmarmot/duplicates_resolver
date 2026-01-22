use duplicates_resolver::resolve_duplicates;
use std::env;
use utils::ScopeTimeLogger;

// =============================================================================================

const USAGE: &str = "Usage: duplicates_resolver <path> <json_output_path> --threads=<count>";

// =============================================================================================

fn main() {
    let mut args = env::args();

    let path = args
        .nth(1)
        .expect(&format!("\nNo path provided.\n{}\n", USAGE));
    let json_path = args
        .next()
        .expect(&format!("\nNo JSON output path provided.\n{}\n", USAGE));
    let threads_arg = args
        .next()
        .expect(&format!("\nNo threads count provided.\n{}\n", USAGE));

    let threads_count: u8 = threads_arg
        .trim()
        .strip_prefix("--threads=")
        .expect(&format!("\nNo threads count provided\n{}\n", USAGE))
        .parse()
        .expect("Threads count must be a number");

    let _scoped_logger = ScopeTimeLogger::new("Total execution time");
    resolve_duplicates(&path, &json_path, threads_count);
}
