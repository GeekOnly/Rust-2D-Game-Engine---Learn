//! HUD to UIPrefab Migration Tool
//! 
//! Recursively finds and converts .hud files to .uiprefab files

use std::path::{Path, PathBuf};
use std::fs;
use std::io::{self, Write};
use walkdir::WalkDir;
use clap::Parser;

/// Configuration for the migration process
#[derive(Debug, Clone)]
pub struct MigrationConfig {
    /// Root directories to search for .hud files
    pub search_paths: Vec<PathBuf>,
    
    /// Whether to perform a dry run (no files written)
    pub dry_run: bool,
    
    /// Whether to create backups of original files
    pub create_backups: bool,
    
    /// Output directory for converted files (None = same directory as source)
    pub output_dir: Option<PathBuf>,
    
    /// Whether to show verbose output
    pub verbose: bool,
}

impl Default for MigrationConfig {
    fn default() -> Self {
        Self {
            search_paths: vec![PathBuf::from(".")],
            dry_run: false,
            create_backups: true,
            output_dir: None,
            verbose: false,
        }
    }
}

/// Result of a single file migration
#[derive(Debug, Clone)]
pub struct MigrationResult {
    pub source_path: PathBuf,
    pub target_path: PathBuf,
    pub success: bool,
    pub error: Option<String>,
    pub backup_path: Option<PathBuf>,
}

/// Summary of the entire migration process
#[derive(Debug, Clone)]
pub struct MigrationReport {
    pub total_files_found: usize,
    pub successful_conversions: usize,
    pub failed_conversions: usize,
    pub results: Vec<MigrationResult>,
}

impl MigrationReport {
    pub fn new() -> Self {
        Self {
            total_files_found: 0,
            successful_conversions: 0,
            failed_conversions: 0,
            results: Vec::new(),
        }
    }
    
    pub fn add_result(&mut self, result: MigrationResult) {
        if result.success {
            self.successful_conversions += 1;
        } else {
            self.failed_conversions += 1;
        }
        self.results.push(result);
    }
    
    pub fn print_summary(&self) {
        println!("\n{}", "=".repeat(60));
        println!("Migration Summary");
        println!("{}", "=".repeat(60));
        println!("Total .hud files found: {}", self.total_files_found);
        println!("Successful conversions: {}", self.successful_conversions);
        println!("Failed conversions: {}", self.failed_conversions);
        
        if !self.results.is_empty() {
            println!("\nDetailed Results:");
            println!("{}", "-".repeat(60));
            
            for result in &self.results {
                let status = if result.success { "✓" } else { "✗" };
                println!("{} {}", status, result.source_path.display());
                
                if result.success {
                    println!("  → {}", result.target_path.display());
                    if let Some(backup) = &result.backup_path {
                        println!("  Backup: {}", backup.display());
                    }
                } else if let Some(error) = &result.error {
                    println!("  Error: {}", error);
                }
            }
        }
        
        println!("{}", "=".repeat(60));
    }
}

/// File discovery - recursively finds all .hud files
pub fn discover_hud_files(search_paths: &[PathBuf], verbose: bool) -> Vec<PathBuf> {
    let mut hud_files = Vec::new();
    
    for search_path in search_paths {
        if verbose {
            println!("Searching in: {}", search_path.display());
        }
        
        if !search_path.exists() {
            eprintln!("Warning: Path does not exist: {}", search_path.display());
            continue;
        }
        
        for entry in WalkDir::new(search_path)
            .follow_links(true)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            
            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("hud") {
                if verbose {
                    println!("  Found: {}", path.display());
                }
                hud_files.push(path.to_path_buf());
            }
        }
    }
    
    hud_files
}

/// Batch conversion - converts all discovered .hud files
pub fn batch_convert(
    hud_files: &[PathBuf],
    config: &MigrationConfig,
) -> MigrationReport {
    use ui::hud_converter::{HudAsset, HudToUIPrefabConverter};
    
    let mut report = MigrationReport::new();
    report.total_files_found = hud_files.len();
    
    for hud_path in hud_files {
        if config.verbose {
            println!("\nProcessing: {}", hud_path.display());
        }
        
        let result = convert_single_file(hud_path, config);
        
        if config.verbose {
            if result.success {
                println!("  ✓ Success: {}", result.target_path.display());
            } else if let Some(error) = &result.error {
                println!("  ✗ Failed: {}", error);
            }
        }
        
        report.add_result(result);
    }
    
    report
}

/// Convert a single .hud file to .uiprefab
fn convert_single_file(
    hud_path: &Path,
    config: &MigrationConfig,
) -> MigrationResult {
    use ui::hud_converter::{HudAsset, HudToUIPrefabConverter};
    
    // Determine output path
    let target_path = if let Some(output_dir) = &config.output_dir {
        let file_name = hud_path.file_name().unwrap();
        let prefab_name = Path::new(file_name).with_extension("uiprefab");
        output_dir.join(prefab_name)
    } else {
        hud_path.with_extension("uiprefab")
    };
    
    // Create backup if requested
    let backup_path = if config.create_backups && !config.dry_run {
        let backup = hud_path.with_extension("hud.backup");
        match fs::copy(hud_path, &backup) {
            Ok(_) => Some(backup),
            Err(e) => {
                return MigrationResult {
                    source_path: hud_path.to_path_buf(),
                    target_path,
                    success: false,
                    error: Some(format!("Failed to create backup: {}", e)),
                    backup_path: None,
                };
            }
        }
    } else {
        None
    };
    
    // Load HUD file
    let hud_content = match fs::read_to_string(hud_path) {
        Ok(content) => content,
        Err(e) => {
            return MigrationResult {
                source_path: hud_path.to_path_buf(),
                target_path,
                success: false,
                error: Some(format!("Failed to read file: {}", e)),
                backup_path,
            };
        }
    };
    
    // Parse HUD JSON
    let hud_asset: HudAsset = match serde_json::from_str(&hud_content) {
        Ok(asset) => asset,
        Err(e) => {
            return MigrationResult {
                source_path: hud_path.to_path_buf(),
                target_path,
                success: false,
                error: Some(format!("Failed to parse HUD JSON: {}", e)),
                backup_path,
            };
        }
    };
    
    // Convert to UIPrefab
    let prefab = HudToUIPrefabConverter::convert(&hud_asset);
    
    // Serialize to JSON
    let prefab_json = match serde_json::to_string_pretty(&prefab) {
        Ok(json) => json,
        Err(e) => {
            return MigrationResult {
                source_path: hud_path.to_path_buf(),
                target_path,
                success: false,
                error: Some(format!("Failed to serialize prefab: {}", e)),
                backup_path,
            };
        }
    };
    
    // Write output file (unless dry run)
    if !config.dry_run {
        // Create output directory if needed
        if let Some(parent) = target_path.parent() {
            if let Err(e) = fs::create_dir_all(parent) {
                return MigrationResult {
                    source_path: hud_path.to_path_buf(),
                    target_path,
                    success: false,
                    error: Some(format!("Failed to create output directory: {}", e)),
                    backup_path,
                };
            }
        }
        
        if let Err(e) = fs::write(&target_path, prefab_json) {
            return MigrationResult {
                source_path: hud_path.to_path_buf(),
                target_path,
                success: false,
                error: Some(format!("Failed to write output file: {}", e)),
                backup_path,
            };
        }
    }
    
    MigrationResult {
        source_path: hud_path.to_path_buf(),
        target_path,
        success: true,
        error: None,
        backup_path,
    }
}

/// HUD to UIPrefab Migration Tool
#[derive(Parser, Debug)]
#[command(name = "hud_migrator")]
#[command(about = "Converts legacy .hud files to .uiprefab format", long_about = None)]
struct Args {
    /// Directories to search for .hud files (defaults to current directory)
    #[arg(short, long, value_name = "DIR", num_args = 1..)]
    paths: Option<Vec<PathBuf>>,
    
    /// Perform a dry run without writing files
    #[arg(short = 'n', long)]
    dry_run: bool,
    
    /// Skip creating backup files
    #[arg(long)]
    no_backup: bool,
    
    /// Output directory for converted files (defaults to same directory as source)
    #[arg(short, long, value_name = "DIR")]
    output: Option<PathBuf>,
    
    /// Enable verbose output
    #[arg(short, long)]
    verbose: bool,
    
    /// Show progress during conversion
    #[arg(short = 'P', long)]
    progress: bool,
}

fn main() {
    let args = Args::parse();
    
    println!("HUD to UIPrefab Migration Tool");
    println!("{}", "=".repeat(60));
    
    // Build configuration from CLI arguments
    let config = MigrationConfig {
        search_paths: args.paths.unwrap_or_else(|| vec![PathBuf::from(".")]),
        dry_run: args.dry_run,
        create_backups: !args.no_backup,
        output_dir: args.output,
        verbose: args.verbose,
    };
    
    if config.dry_run {
        println!("\n⚠ DRY RUN MODE - No files will be modified");
    }
    
    if !config.create_backups && !config.dry_run {
        println!("\n⚠ Backups disabled - Original files will not be backed up");
    }
    
    // Discover .hud files
    println!("\nDiscovering .hud files in:");
    for path in &config.search_paths {
        println!("  - {}", path.display());
    }
    
    let hud_files = discover_hud_files(&config.search_paths, config.verbose);
    
    if hud_files.is_empty() {
        println!("\n❌ No .hud files found");
        std::process::exit(0);
    }
    
    println!("\n✓ Found {} .hud file(s)", hud_files.len());
    
    if !config.verbose && !args.progress {
        for file in &hud_files {
            println!("  - {}", file.display());
        }
    }
    
    // Convert files
    println!("\nConverting files...");
    
    let report = if args.progress {
        batch_convert_with_progress(&hud_files, &config)
    } else {
        batch_convert(&hud_files, &config)
    };
    
    // Print summary
    report.print_summary();
    
    // Exit with appropriate code
    if report.failed_conversions > 0 {
        std::process::exit(1);
    }
}

/// Batch conversion with progress reporting
fn batch_convert_with_progress(
    hud_files: &[PathBuf],
    config: &MigrationConfig,
) -> MigrationReport {
    use ui::hud_converter::{HudAsset, HudToUIPrefabConverter};
    
    let mut report = MigrationReport::new();
    report.total_files_found = hud_files.len();
    
    let total = hud_files.len();
    
    for (index, hud_path) in hud_files.iter().enumerate() {
        let progress = format!("[{}/{}]", index + 1, total);
        print!("{} Processing: {} ... ", progress, hud_path.display());
        io::stdout().flush().unwrap();
        
        let result = convert_single_file(hud_path, config);
        
        if result.success {
            println!("✓");
        } else if let Some(error) = &result.error {
            println!("✗");
            eprintln!("  Error: {}", error);
        }
        
        report.add_result(result);
    }
    
    report
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;
    
    #[test]
    fn test_discover_hud_files_empty_directory() {
        let temp_dir = TempDir::new().unwrap();
        let search_paths = vec![temp_dir.path().to_path_buf()];
        
        let files = discover_hud_files(&search_paths, false);
        assert_eq!(files.len(), 0);
    }
    
    #[test]
    fn test_discover_hud_files_single_file() {
        let temp_dir = TempDir::new().unwrap();
        let hud_path = temp_dir.path().join("test.hud");
        fs::write(&hud_path, "{}").unwrap();
        
        let search_paths = vec![temp_dir.path().to_path_buf()];
        let files = discover_hud_files(&search_paths, false);
        
        assert_eq!(files.len(), 1);
        assert_eq!(files[0], hud_path);
    }
    
    #[test]
    fn test_discover_hud_files_nested_directories() {
        let temp_dir = TempDir::new().unwrap();
        
        // Create nested structure
        let sub_dir1 = temp_dir.path().join("project1");
        let sub_dir2 = temp_dir.path().join("project2/assets/ui");
        fs::create_dir_all(&sub_dir1).unwrap();
        fs::create_dir_all(&sub_dir2).unwrap();
        
        // Create .hud files
        let hud1 = sub_dir1.join("hud1.hud");
        let hud2 = sub_dir2.join("hud2.hud");
        fs::write(&hud1, "{}").unwrap();
        fs::write(&hud2, "{}").unwrap();
        
        // Create non-.hud file
        fs::write(sub_dir1.join("other.txt"), "text").unwrap();
        
        let search_paths = vec![temp_dir.path().to_path_buf()];
        let files = discover_hud_files(&search_paths, false);
        
        assert_eq!(files.len(), 2);
        assert!(files.contains(&hud1));
        assert!(files.contains(&hud2));
    }
    
    #[test]
    fn test_discover_hud_files_multiple_search_paths() {
        let temp_dir1 = TempDir::new().unwrap();
        let temp_dir2 = TempDir::new().unwrap();
        
        let hud1 = temp_dir1.path().join("hud1.hud");
        let hud2 = temp_dir2.path().join("hud2.hud");
        fs::write(&hud1, "{}").unwrap();
        fs::write(&hud2, "{}").unwrap();
        
        let search_paths = vec![
            temp_dir1.path().to_path_buf(),
            temp_dir2.path().to_path_buf(),
        ];
        let files = discover_hud_files(&search_paths, false);
        
        assert_eq!(files.len(), 2);
        assert!(files.contains(&hud1));
        assert!(files.contains(&hud2));
    }
    
    #[test]
    fn test_discover_hud_files_nonexistent_path() {
        let search_paths = vec![PathBuf::from("/nonexistent/path")];
        let files = discover_hud_files(&search_paths, false);
        
        assert_eq!(files.len(), 0);
    }
    
    #[test]
    fn test_batch_convert_empty_list() {
        let config = MigrationConfig::default();
        let report = batch_convert(&[], &config);
        
        assert_eq!(report.total_files_found, 0);
        assert_eq!(report.successful_conversions, 0);
        assert_eq!(report.failed_conversions, 0);
    }
    
    #[test]
    fn test_batch_convert_single_file() {
        let temp_dir = TempDir::new().unwrap();
        
        // Create a simple HUD file
        let hud_path = temp_dir.path().join("test.hud");
        let hud_content = r#"{
            "name": "Test HUD",
            "elements": [
                {
                    "id": "label",
                    "element_type": {
                        "type": "Text",
                        "text": "Hello",
                        "font_size": 16.0,
                        "color": [1.0, 1.0, 1.0, 1.0]
                    },
                    "anchor": "Center",
                    "offset": [0.0, 0.0],
                    "size": [100.0, 30.0],
                    "visible": true
                }
            ]
        }"#;
        fs::write(&hud_path, hud_content).unwrap();
        
        let config = MigrationConfig {
            search_paths: vec![temp_dir.path().to_path_buf()],
            dry_run: false,
            create_backups: false,
            output_dir: None,
            verbose: false,
        };
        
        let hud_files = vec![hud_path.clone()];
        let report = batch_convert(&hud_files, &config);
        
        assert_eq!(report.total_files_found, 1);
        assert_eq!(report.successful_conversions, 1);
        assert_eq!(report.failed_conversions, 0);
        
        // Check that output file was created
        let prefab_path = hud_path.with_extension("uiprefab");
        assert!(prefab_path.exists());
        
        // Verify the content is valid JSON
        let prefab_content = fs::read_to_string(&prefab_path).unwrap();
        let _: serde_json::Value = serde_json::from_str(&prefab_content).unwrap();
    }
    
    #[test]
    fn test_batch_convert_with_backup() {
        let temp_dir = TempDir::new().unwrap();
        
        let hud_path = temp_dir.path().join("test.hud");
        let hud_content = r#"{
            "name": "Test HUD",
            "elements": []
        }"#;
        fs::write(&hud_path, hud_content).unwrap();
        
        let config = MigrationConfig {
            search_paths: vec![temp_dir.path().to_path_buf()],
            dry_run: false,
            create_backups: true,
            output_dir: None,
            verbose: false,
        };
        
        let hud_files = vec![hud_path.clone()];
        let report = batch_convert(&hud_files, &config);
        
        assert_eq!(report.successful_conversions, 1);
        
        // Check that backup was created
        let backup_path = hud_path.with_extension("hud.backup");
        assert!(backup_path.exists());
        
        // Verify backup content matches original
        let backup_content = fs::read_to_string(&backup_path).unwrap();
        assert_eq!(backup_content, hud_content);
    }
    
    #[test]
    fn test_batch_convert_dry_run() {
        let temp_dir = TempDir::new().unwrap();
        
        let hud_path = temp_dir.path().join("test.hud");
        let hud_content = r#"{
            "name": "Test HUD",
            "elements": []
        }"#;
        fs::write(&hud_path, hud_content).unwrap();
        
        let config = MigrationConfig {
            search_paths: vec![temp_dir.path().to_path_buf()],
            dry_run: true,
            create_backups: false,
            output_dir: None,
            verbose: false,
        };
        
        let hud_files = vec![hud_path.clone()];
        let report = batch_convert(&hud_files, &config);
        
        assert_eq!(report.successful_conversions, 1);
        
        // Check that output file was NOT created
        let prefab_path = hud_path.with_extension("uiprefab");
        assert!(!prefab_path.exists());
    }
    
    #[test]
    fn test_batch_convert_with_output_dir() {
        let temp_dir = TempDir::new().unwrap();
        let output_dir = temp_dir.path().join("output");
        
        let hud_path = temp_dir.path().join("test.hud");
        let hud_content = r#"{
            "name": "Test HUD",
            "elements": []
        }"#;
        fs::write(&hud_path, hud_content).unwrap();
        
        let config = MigrationConfig {
            search_paths: vec![temp_dir.path().to_path_buf()],
            dry_run: false,
            create_backups: false,
            output_dir: Some(output_dir.clone()),
            verbose: false,
        };
        
        let hud_files = vec![hud_path.clone()];
        let report = batch_convert(&hud_files, &config);
        
        assert_eq!(report.successful_conversions, 1);
        
        // Check that output file was created in output directory
        let prefab_path = output_dir.join("test.uiprefab");
        assert!(prefab_path.exists());
    }
    
    #[test]
    fn test_batch_convert_invalid_json() {
        let temp_dir = TempDir::new().unwrap();
        
        let hud_path = temp_dir.path().join("invalid.hud");
        fs::write(&hud_path, "not valid json").unwrap();
        
        let config = MigrationConfig::default();
        let hud_files = vec![hud_path];
        let report = batch_convert(&hud_files, &config);
        
        assert_eq!(report.total_files_found, 1);
        assert_eq!(report.successful_conversions, 0);
        assert_eq!(report.failed_conversions, 1);
        
        let result = &report.results[0];
        assert!(!result.success);
        assert!(result.error.is_some());
        assert!(result.error.as_ref().unwrap().contains("parse"));
    }
}
