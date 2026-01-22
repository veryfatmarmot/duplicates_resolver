use std::{
    env,
};
use duplicates_resolver::resolve_duplicates;

// =============================================================================================

const USAGE: &str = "Usage: duplicates_resolver <path> --threads=<count>";

// =============================================================================================

fn main() {
    let mut args = env::args();

    let path = args.nth(1).expect(&format!("\nNo path provided.\n{}\n", USAGE));
    let threads_arg = args.next()
        .expect(&format!("\nNo threads count provided.\n{}\n", USAGE));
    
    let threads_count: u8 = threads_arg
        .trim()
        .strip_prefix("--threads=")
        .expect(&format!("\nNo threads count provided\n{}\n", USAGE))
        .parse()
        .expect("Threads count must be a number");

    resolve_duplicates(&path, threads_count);
}