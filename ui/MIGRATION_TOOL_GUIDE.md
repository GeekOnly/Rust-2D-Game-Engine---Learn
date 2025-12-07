# HUD to UIPrefab Migration Tool

## Overview

The HUD migration tool (`hud_migrator`) is a command-line utility that converts legacy `.hud` files to the new `.uiprefab` format. It provides batch conversion capabilities with safety features like backups and dry-run mode.

> **📖 For detailed CLI documentation, see [CLI_MIGRATION_TOOL.md](CLI_MIGRATION_TOOL.md)**

## Installation

The tool is built as part of the `ui` crate:

```bash
cd ui
cargo build --release --bin hud_migrator
```

The binary will be located at `target/release/hud_migrator` (or `hud_migrator.exe` on Windows).

## Usage

### Basic Usage

Convert all .hud files in the current directory:

```bash
hud_migrator
```

### Specify Search Paths

Convert files in specific directories:

```bash
hud_migrator --paths "projects/MyGame" "projects/OtherGame"
```

### Dry Run

Preview what would be converted without making changes:

```bash
hud_migrator --dry-run
```

### Progress Reporting

Show progress during conversion:

```bash
hud_migrator --progress
```

### Custom Output Directory

Save converted files to a different location:

```bash
hud_migrator --output "converted_prefabs"
```

### Skip Backups

Don't create backup files (use with caution):

```bash
hud_migrator --no-backup
```

### Verbose Output

Enable detailed logging:

```bash
hud_migrator --verbose
```

## Command-Line Options

```
Options:
  -p, --paths <DIR>...  Directories to search for .hud files (defaults to current directory)
  -n, --dry-run         Perform a dry run without writing files
      --no-backup       Skip creating backup files
  -o, --output <DIR>    Output directory for converted files (defaults to same directory as source)
  -v, --verbose         Enable verbose output
  -P, --progress        Show progress during conversion
  -h, --help            Print help
```

## Conversion Process

The tool performs the following steps for each .hud file:

1. **Discovery**: Recursively searches specified directories for .hud files
2. **Backup**: Creates a `.hud.backup` file (unless `--no-backup` is specified)
3. **Parse**: Loads and parses the HUD JSON format
4. **Convert**: Transforms HUD elements to UIPrefab elements using the converter
5. **Serialize**: Generates pretty-printed JSON for the UIPrefab
6. **Write**: Saves the .uiprefab file (unless `--dry-run` is specified)

## Conversion Notes

### Element Type Mapping

- **Text** → UIText component
- **DynamicText** → UIText with notes for Lua binding
- **Image** → UIImage component
- **HealthBar** → UIPanel with notes for adding fill image
- **ProgressBar** → UIPanel with notes for adding fill image
- **Container** → UIPanel with children
- **Minimap** → UIPanel with notes for custom implementation

### Anchor Conversion

All 9 anchor positions are correctly mapped:
- TopLeft, TopCenter, TopRight
- CenterLeft, Center, CenterRight
- BottomLeft, BottomCenter, BottomRight

### Special Cases

- **DynamicText**: Includes notes in the element name about Lua binding requirements
- **HealthBar/ProgressBar**: Includes notes about adding UIImage children with fill_method
- **Minimap**: Includes notes about custom component implementation

## Example

Convert all HUD files in a game project with progress reporting:

```bash
hud_migrator --paths "projects/Celeste Demo" --progress
```

Output:
```
HUD to UIPrefab Migration Tool
============================================================

Discovering .hud files in:
  - projects/Celeste Demo

✓ Found 2 .hud file(s)

Converting files...
[1/2] Processing: projects/Celeste Demo\assets\ui\celeste_hud.hud ... ✓
[2/2] Processing: projects/Celeste Demo\assets\ui\test_hud.hud ... ✓

============================================================
Migration Summary
============================================================
Total .hud files found: 2
Successful conversions: 2
Failed conversions: 0

Detailed Results:
------------------------------------------------------------
✓ projects/Celeste Demo\assets\ui\celeste_hud.hud
  → projects/Celeste Demo\assets\ui\celeste_hud.uiprefab
  Backup: projects/Celeste Demo\assets\ui\celeste_hud.hud.backup
✓ projects/Celeste Demo\assets\ui\test_hud.hud
  → projects/Celeste Demo\assets\ui\test_hud.uiprefab
  Backup: projects/Celeste Demo\assets\ui\test_hud.hud.backup
============================================================
```

## Error Handling

The tool provides clear error messages for common issues:

- **File not found**: Warns about non-existent search paths
- **Invalid JSON**: Reports parsing errors with file location
- **Serialization errors**: Reports issues converting to UIPrefab format
- **I/O errors**: Reports file system errors (permissions, disk space, etc.)

## Testing

The migration tool includes comprehensive tests:

```bash
cd ui
cargo test --bin hud_migrator
```

Test coverage includes:
- File discovery in various directory structures
- Batch conversion with different configurations
- Dry-run mode
- Backup creation
- Output directory handling
- Error handling for invalid files

## Implementation Details

The migration tool is implemented in `ui/src/bin/hud_migrator.rs` and uses:

- **walkdir**: For recursive directory traversal
- **clap**: For command-line argument parsing
- **serde_json**: For JSON serialization/deserialization
- **ui::hud_converter**: For the actual HUD → UIPrefab conversion logic

The converter logic is in `ui/src/hud_converter.rs` and includes unit tests for all element types and anchor positions.
