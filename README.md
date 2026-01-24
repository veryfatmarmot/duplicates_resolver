# Duplicates Resolver

A high-performance Rust CLI tool to find and manage duplicate files by content hash (SHA-256). Designed for efficient scanning of large directories with parallel processing support.

## Features

- 🚀 **Parallel Processing**: Multi-threaded file scanning and hashing using Rayon
- 🔍 **Content-Based Detection**: Uses SHA-256 hashing to identify true duplicates, regardless of file names
- 📊 **JSON Reports**: Exports duplicate findings to JSON for inspection and automation
- 🗂️ **Smart Organization**: Automatically moves duplicates while preserving the original file
- ✅ **Safe Operations**: Validates paths to prevent overlapping source/target directories
- ⏱️ **Performance Tracking**: Built-in execution time logging

## Installation

### Prerequisites

- Rust 1.70+ (with Cargo)

### Build from Source

```bash
git clone <repository-url>
cd duplicates_resolver
cargo build --release
```

The compiled binary will be available at `target/release/duplicates_resolver.exe` (Windows) or `target/release/duplicates_resolver` (Linux/Mac).

## Usage

The tool provides two main commands: `scan` and `move`.

### 1. Scan for Duplicates

Scans a directory tree for duplicate files and generates a JSON report.

```bash
duplicates_resolver scan <scan_path> <json_output_path> --threads=<count>
```

**Arguments:**
- `<scan_path>`: Directory to scan for duplicates
- `<json_output_path>`: Path where the JSON report will be saved
- `--threads=<count>`: Number of threads to use (e.g., 4, 8, 16)

**Example:**
```bash
duplicates_resolver scan "C:/Photos" "reports/photo_duplicates.json" --threads=8
```

### 2. Move Duplicates

Moves duplicate files to a target directory based on a previously generated JSON report.

```bash
duplicates_resolver move <json_input_path> <target_path>
```

**Arguments:**
- `<json_input_path>`: Path to the JSON report from the scan command
- `<target_path>`: Destination directory for duplicate files (must not exist)

**Example:**
```bash
duplicates_resolver move "reports/photo_duplicates.json" "C:/DuplicateFiles"
```

**Important:** The first file in each duplicate group (sorted alphabetically by folder path) is considered the "original" and remains in place. All other duplicates are moved.

## How It Works

### Scanning Process

1. **File Collection**: Walks the directory tree and groups files by size
2. **Hash Calculation**: For files with matching sizes, calculates SHA-256 hash in parallel
3. **Duplicate Grouping**: Groups files with identical hashes
4. **Report Generation**: Exports findings to JSON with folder paths and file names

### Move Process

1. **Validation**: Ensures target directory doesn't exist and doesn't overlap with source
2. **Directory Creation**: Creates target directory structure as needed
3. **File Moving**: Relocates duplicate files while preserving folder structure
4. **Error Handling**: Skips missing files and continues processing

## JSON Report Format

```json
{
  "root_path": "C:/Photos",
  "duplicates": {
    "a3f5e9...": [
      {
        "folder_path": "2023/vacation",
        "file_name": "beach.jpg"
      },
      {
        "folder_path": "2023/summer",
        "file_name": "beach_copy.jpg"
      }
    ]
  }
}
```

## Safety Features

- ✅ Validates target directory doesn't exist to prevent accidental overwrites
- ✅ Checks for path overlaps (target inside source or vice versa)
- ✅ Creates parent directories automatically when needed
- ✅ Continues processing even if individual files are inaccessible
- ✅ Reports skipped files to stderr for troubleshooting

## Performance Tips

- **Thread Count**: Use a thread count matching your CPU cores for optimal performance
- **SSD vs HDD**: Performance significantly better on SSDs due to parallel I/O
- **Large Files**: Hash calculation time increases with file size
- **Network Drives**: May be slower; consider copying to local storage first

## Examples

### Example 1: Clean up a photo library
```bash
# Scan photos directory
duplicates_resolver scan "D:/Photos" "photo_report.json" --threads=12

# Review the JSON report to verify findings
# (use any JSON viewer or text editor)

# Move duplicates to a separate location
duplicates_resolver move "photo_report.json" "D:/Photo_Duplicates"
```

### Example 2: Find duplicates in documents
```bash
duplicates_resolver scan "C:/Documents" "doc_duplicates.json" --threads=8
duplicates_resolver move "doc_duplicates.json" "C:/Backup/Duplicates"
```

## Project Structure

```
duplicates_resolver/
├── src/
│   ├── main.rs          # CLI entry point
│   ├── lib.rs           # Module exports
│   ├── scanner.rs       # File scanning and hashing
│   ├── mover.rs         # Duplicate file moving
│   ├── json_helper.rs   # JSON serialization
│   └── types.rs         # Type definitions
├── libs/
│   └── utils/           # Utility library (scope timer, thread pool)
├── Cargo.toml           # Project dependencies
└── README.md
```

## Dependencies

- `rayon` - Parallel processing
- `walkdir` - Directory traversal
- `sha2` - SHA-256 hashing
- `serde` / `serde_json` - JSON serialization
- `anyhow` - Error handling

## Contributing

Contributions are welcome! Please ensure code follows Rust best practices and includes appropriate error handling.

## License

[Add your license information here]

## Notes

- The tool uses content-based comparison (SHA-256), so renamed files will still be detected as duplicates
- Empty files (0 bytes) are processed but may all be considered duplicates
- Symbolic links are not followed during scanning
- File permissions are preserved during move operations
