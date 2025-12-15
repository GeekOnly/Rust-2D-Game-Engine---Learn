use std::path::PathBuf;
use std::fmt;

/// Error types for tilemap management operations
#[derive(Debug, Clone)]
pub enum TilemapError {
    /// File not found at the specified path
    FileNotFound(PathBuf),
    
    /// Invalid file format or corrupted data
    InvalidFormat(String),
    
    /// Validation error during loading
    ValidationError(String),
    
    /// Entity not found in the world
    EntityNotFound(ecs::Entity),
    
    /// Required component is missing from an entity
    ComponentMissing(String),
    
    /// Collider generation failed
    ColliderGenerationFailed(String),
    
    /// IO error occurred
    IoError(String),
    
    /// JSON parsing error
    JsonError(String),
}

impl TilemapError {
    /// Get a user-friendly error message
    pub fn display_message(&self) -> String {
        match self {
            TilemapError::FileNotFound(path) => {
                format!("File not found: {}", path.display())
            }
            TilemapError::InvalidFormat(msg) => {
                format!("Invalid file format: {}", msg)
            }
            TilemapError::ValidationError(msg) => {
                format!("Validation error: {}", msg)
            }
            TilemapError::EntityNotFound(entity) => {
                format!("Entity not found: {:?}", entity)
            }
            TilemapError::ComponentMissing(component) => {
                format!("Missing component: {}", component)
            }
            TilemapError::ColliderGenerationFailed(msg) => {
                format!("Collider generation failed: {}", msg)
            }
            TilemapError::IoError(msg) => {
                format!("IO error: {}", msg)
            }
            TilemapError::JsonError(msg) => {
                format!("JSON parsing error: {}", msg)
            }
        }
    }
    
    /// Log the error to the console with context
    pub fn log_error(&self) {
        let message = self.display_message();
        log::error!("[TilemapError] {}", message);
        
        // Log additional context based on error type
        match self {
            TilemapError::FileNotFound(path) => {
                log::error!("  Context: Attempted to load file at: {:?}", path);
                log::error!("  Suggestion: Check if the file exists and the path is correct");
                if let Some(parent) = path.parent() {
                    log::error!("  Parent directory: {:?}", parent);
                    log::error!("  Parent exists: {}", parent.exists());
                }
            }
            TilemapError::InvalidFormat(msg) => {
                log::error!("  Context: {}", msg);
                log::error!("  Suggestion: Verify the file is a valid LDtk file (version 1.5.3+)");
            }
            TilemapError::ValidationError(msg) => {
                log::error!("  Context: {}", msg);
                log::error!("  Suggestion: Check the LDtk file structure and required fields");
            }
            TilemapError::ColliderGenerationFailed(msg) => {
                log::error!("  Context: {}", msg);
                log::error!("  Suggestion: Check IntGrid layer data and collision values");
            }
            TilemapError::JsonError(msg) => {
                log::error!("  Context: {}", msg);
                log::error!("  Suggestion: Verify the file is valid JSON and not corrupted");
            }
            TilemapError::IoError(msg) => {
                log::error!("  Context: {}", msg);
                log::error!("  Suggestion: Check file permissions and disk space");
            }
            TilemapError::EntityNotFound(entity) => {
                log::error!("  Context: Entity {:?} not found in world", entity);
                log::error!("  Suggestion: Entity may have been despawned or never created");
            }
            TilemapError::ComponentMissing(component) => {
                log::error!("  Context: Component '{}' is missing", component);
                log::error!("  Suggestion: Ensure the entity has the required component");
            }
        }
        
        // Log stack trace information for debugging
        log::debug!("  Error occurred at: {}", std::panic::Location::caller());
    }
    
    /// Log the error with a custom context message
    pub fn log_error_with_context(&self, context: &str) {
        log::error!("[TilemapError] {} - {}", context, self.display_message());
        self.log_error();
    }
}

impl fmt::Display for TilemapError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.display_message())
    }
}

impl std::error::Error for TilemapError {}

// Conversion from std::io::Error
impl From<std::io::Error> for TilemapError {
    fn from(err: std::io::Error) -> Self {
        match err.kind() {
            std::io::ErrorKind::NotFound => {
                TilemapError::IoError(format!("File not found: {}", err))
            }
            _ => TilemapError::IoError(err.to_string())
        }
    }
}

// Conversion from serde_json::Error
impl From<serde_json::Error> for TilemapError {
    fn from(err: serde_json::Error) -> Self {
        TilemapError::JsonError(err.to_string())
    }
}

// Conversion from String (for backward compatibility)
impl From<String> for TilemapError {
    fn from(err: String) -> Self {
        // Try to infer the error type from the message
        if err.contains("not found") || err.contains("Not found") {
            TilemapError::ValidationError(err)
        } else if err.contains("parse") || err.contains("JSON") {
            TilemapError::JsonError(err)
        } else if err.contains("collider") {
            TilemapError::ColliderGenerationFailed(err)
        } else {
            TilemapError::ValidationError(err)
        }
    }
}
