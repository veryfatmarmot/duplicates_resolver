# Duplicates Resolver

Fast parallel duplicate file finder and remover using SHA-256 content hashing.

## Build

```bash
cargo build --release
```

## Commands

### prescan - Quick estimate
```bash
duplicates_resolver prescan <path>
```
Preview duplicates without generating full report. Shows count and statistics.

### scan - Find duplicates
```bash
duplicates_resolver scan <path> <output.json> [--threads=<n>]
```
Scan directory for duplicates and save results to JSON.
- `<path>` - Directory to scan
- `<output.json>` - Report output file
- `[--threads=<n>]` - Thread count (1-128, default: CPU cores)

Files sorted lexicographically by folder path in results.

### move - Relocate duplicates
```bash
duplicates_resolver move <input.json> <target> [--no-path-len-check]
```
Move duplicate files to target directory based on scan report.
- `<input.json>` - Report from scan command
- `<target>` - Destination folder (will be created)
- `--no-path-len-check` - Skip Windows path length validation (optional)

First file in each duplicate group is preserved; others are moved.

## How It Works

1. Walks directory tree and groups files by size
2. Calculates SHA-256 hash for same-sized files in parallel
3. Identifies duplicates by matching hashes
4. Exports findings to JSON (prescan shows stats only)
5. Optionally moves duplicates while maintaining folder structure

## JSON Report Format

```json
{
  "root_path": "/scan/path",
  "duplicates": {
    "hash_value": [
      {"folder_path": "dir1", "file_name": "file.txt"},
      {"folder_path": "dir2", "file_name": "file.txt"}
    ]
  }
}
```

## Features

- Parallel multi-threaded hashing (Rayon)
- Content-based detection (finds renamed duplicates)
- Safe operations (validates paths, preserves originals)
- Prescan mode for quick preview
- Automatic error recovery
- Windows path length checking
- Performance tracking

## Safety

- Target directory must not exist (prevents overwrites)
- Detects and blocks overlapping paths
- Continues if individual files fail
- Preserves file permissions

## Performance

- Use threads ≤ CPU core count for best performance
- SSD recommended (much faster than HDD)
- Local storage faster than network drives

## Stack

rayon, walkdir, sha2, serde_json, anyhow
