# 🎮 UGC Game Engine Design: สร้าง Engine ระดับ Roblox

**วันที่**: 26 ธันวาคม 2025
**เป้าหมาย**: ออกแบบ Rust 2D Game Engine ให้รองรับ **User-Generated Content (UGC)** แบบ Roblox
**ขนาดตลาด**: Multi-billion dollar industry (2025)
**แรงบันดาลใจ**: Roblox, Fortnite Creative, The Sandbox

---

## 📊 Executive Summary

### UGC คืออะไร?

**User-Generated Content (UGC)** = ผู้เล่นสร้างเนื้อหาเอง (assets, levels, games, scripts)

### ทำไมต้องทำ UGC?

| ข้อดี | ผลกระทบ |
|-------|----------|
| **Infinite Content** | ผู้เล่นสร้างเนื้อหาไม่รู้จบ |
| **Community-Driven** | ชุมชนแข็งแรง, อายุยืน |
| **Monetization** | ผู้สร้างขายผลงาน, platform รับ % |
| **Lower Dev Cost** | ผู้เล่นทำคอนเทนต์ให้ฟรี |
| **Viral Growth** | ผู้สร้างดึงผู้เล่นใหม่เข้ามา |

### ตัวอย่างที่ประสบความสำเร็จ

| Platform | Revenue 2025 | Key Feature |
|----------|--------------|-------------|
| **Roblox** | $3B+ | Luau scripting, Marketplace |
| **Fortnite Creative** | $1B+ | UEFN (Unreal Editor) |
| **Minecraft** | $800M+ | Marketplace, Mods |
| **The Sandbox** | $500M+ | Voxel editor, Blockchain |

---

## 🏗️ Core Architecture: 7 เสาหลัก

```
┌─────────────────────────────────────────────────────────┐
│                    UGC Game Engine                       │
├─────────────────────────────────────────────────────────┤
│  1. Asset System        (Upload, Storage, Versioning)   │
│  2. Scripting Sandbox   (Luau/Rhai, Safe Execution)     │
│  3. Editor Tools        (Visual Studio, Game Maker)     │
│  4. Marketplace         (Buy/Sell, Monetization)        │
│  5. Moderation System   (AI, Community, Manual)         │
│  6. Multiplayer Sync    (Realtime Collaboration)        │
│  7. Analytics & Economy (Track Usage, Balance Economy)  │
└─────────────────────────────────────────────────────────┘
```

---

## 🗂️ Pillar 1: Asset System (Upload, Validation, Storage)

### 1.1 Architecture

```rust
// ไฟล์: engine/src/ugc/asset_system.rs

use serde::{Serialize, Deserialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AssetType {
    /// 3D Models (meshes, animations)
    Model3D {
        format: ModelFormat,
        poly_count: u32,
        has_rigging: bool,
    },

    /// 2D Sprites/Textures
    Texture2D {
        format: TextureFormat,
        width: u32,
        height: u32,
    },

    /// Audio files
    Audio {
        format: AudioFormat,
        duration_seconds: f32,
    },

    /// Scripts (Luau/Rhai)
    Script {
        language: ScriptLanguage,
        line_count: u32,
    },

    /// Scenes/Levels
    Scene {
        asset_count: u32,
        script_count: u32,
    },

    /// Game Templates
    GameTemplate {
        genre: String,
        includes_scripts: bool,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ModelFormat {
    GLTF,
    FBX,
    OBJ,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TextureFormat {
    PNG,
    JPEG,
    ASTC, // Compressed for mobile
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AudioFormat {
    OGG,
    MP3,
    WAV,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScriptLanguage {
    Luau,
    Rhai,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UgcAsset {
    pub id: String,              // Unique ID (UUID)
    pub creator_id: String,      // User who uploaded
    pub asset_type: AssetType,
    pub name: String,
    pub description: String,
    pub tags: Vec<String>,
    pub price: Option<f32>,      // None = free
    pub version: u32,
    pub upload_date: i64,        // Unix timestamp
    pub downloads: u64,
    pub rating: f32,             // 0.0 - 5.0
    pub moderation_status: ModerationStatus,

    // Storage
    pub file_path: PathBuf,
    pub file_size_bytes: u64,
    pub thumbnail_path: Option<PathBuf>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ModerationStatus {
    Pending,       // Waiting for review
    Approved,      // Safe to use
    Rejected,      // Violates policy
    Flagged,       // Needs manual review
}

impl UgcAsset {
    pub fn new(
        creator_id: String,
        asset_type: AssetType,
        name: String,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            creator_id,
            asset_type,
            name,
            description: String::new(),
            tags: Vec::new(),
            price: None,
            version: 1,
            upload_date: chrono::Utc::now().timestamp(),
            downloads: 0,
            rating: 0.0,
            moderation_status: ModerationStatus::Pending,
            file_path: PathBuf::new(),
            file_size_bytes: 0,
            thumbnail_path: None,
        }
    }

    pub fn is_usable(&self) -> bool {
        self.moderation_status == ModerationStatus::Approved
    }
}
```

### 1.2 Asset Upload Pipeline

```rust
// ไฟล์: engine/src/ugc/upload_pipeline.rs

pub struct AssetUploadPipeline {
    validation_rules: ValidationRules,
    storage: AssetStorage,
    moderation_queue: ModerationQueue,
}

pub struct ValidationRules {
    pub max_file_size_mb: f32,
    pub max_poly_count: u32,
    pub max_texture_size: (u32, u32),
    pub allowed_formats: Vec<String>,
    pub banned_keywords: Vec<String>,
}

impl Default for ValidationRules {
    fn default() -> Self {
        Self {
            max_file_size_mb: 50.0,        // 50 MB max
            max_poly_count: 100_000,       // 100k triangles
            max_texture_size: (4096, 4096), // 4K max
            allowed_formats: vec![
                "gltf".to_string(),
                "png".to_string(),
                "ogg".to_string(),
            ],
            banned_keywords: vec![
                "exploit".to_string(),
                "hack".to_string(),
                // ... more keywords
            ],
        }
    }
}

#[derive(Debug)]
pub enum UploadError {
    FileTooLarge { actual_mb: f32, max_mb: f32 },
    InvalidFormat { format: String },
    TooManyPolygons { actual: u32, max: u32 },
    TextureTooLarge { width: u32, height: u32 },
    ContainsBannedContent { reason: String },
    MalwareDetected,
    VirusDetected,
}

impl AssetUploadPipeline {
    pub async fn upload(
        &mut self,
        file_path: PathBuf,
        creator_id: String,
        asset_type: AssetType,
    ) -> Result<UgcAsset, UploadError> {

        // Step 1: File validation
        self.validate_file(&file_path, &asset_type)?;

        // Step 2: Security scan
        self.security_scan(&file_path).await?;

        // Step 3: Asset-specific validation
        match &asset_type {
            AssetType::Model3D { .. } => self.validate_model(&file_path)?,
            AssetType::Texture2D { .. } => self.validate_texture(&file_path)?,
            AssetType::Script { .. } => self.validate_script(&file_path)?,
            _ => {}
        }

        // Step 4: Generate thumbnail
        let thumbnail_path = self.generate_thumbnail(&file_path, &asset_type).await?;

        // Step 5: Store asset
        let stored_path = self.storage.store(&file_path).await?;

        // Step 6: Create asset record
        let mut asset = UgcAsset::new(creator_id, asset_type, "New Asset".to_string());
        asset.file_path = stored_path;
        asset.thumbnail_path = Some(thumbnail_path);
        asset.file_size_bytes = std::fs::metadata(&file_path)?.len();

        // Step 7: Queue for moderation
        self.moderation_queue.enqueue(asset.clone()).await;

        Ok(asset)
    }

    fn validate_file(
        &self,
        file_path: &PathBuf,
        asset_type: &AssetType,
    ) -> Result<(), UploadError> {
        // Check file size
        let metadata = std::fs::metadata(file_path)
            .map_err(|_| UploadError::InvalidFormat { format: "unknown".to_string() })?;

        let size_mb = metadata.len() as f32 / 1_048_576.0;
        if size_mb > self.validation_rules.max_file_size_mb {
            return Err(UploadError::FileTooLarge {
                actual_mb: size_mb,
                max_mb: self.validation_rules.max_file_size_mb,
            });
        }

        // Check file extension
        let extension = file_path.extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");

        if !self.validation_rules.allowed_formats.contains(&extension.to_string()) {
            return Err(UploadError::InvalidFormat {
                format: extension.to_string(),
            });
        }

        Ok(())
    }

    async fn security_scan(&self, file_path: &PathBuf) -> Result<(), UploadError> {
        // Step 1: Virus scan (integrate with ClamAV or similar)
        if self.scan_for_virus(file_path).await? {
            return Err(UploadError::VirusDetected);
        }

        // Step 2: Malware detection (static analysis)
        if self.scan_for_malware(file_path).await? {
            return Err(UploadError::MalwareDetected);
        }

        Ok(())
    }

    async fn scan_for_virus(&self, _file_path: &PathBuf) -> Result<bool, UploadError> {
        // TODO: Integrate with ClamAV or cloud antivirus API
        Ok(false) // No virus detected
    }

    async fn scan_for_malware(&self, _file_path: &PathBuf) -> Result<bool, UploadError> {
        // TODO: Static code analysis for scripts
        Ok(false) // No malware detected
    }

    fn validate_model(&self, file_path: &PathBuf) -> Result<(), UploadError> {
        // Load model and check poly count
        // let model = load_gltf(file_path)?;
        // let poly_count = count_triangles(&model);

        let poly_count = 50_000; // Example

        if poly_count > self.validation_rules.max_poly_count {
            return Err(UploadError::TooManyPolygons {
                actual: poly_count,
                max: self.validation_rules.max_poly_count,
            });
        }

        Ok(())
    }

    fn validate_texture(&self, file_path: &PathBuf) -> Result<(), UploadError> {
        // Load image and check dimensions
        let img = image::open(file_path)
            .map_err(|_| UploadError::InvalidFormat { format: "image".to_string() })?;

        let (width, height) = img.dimensions();
        let (max_width, max_height) = self.validation_rules.max_texture_size;

        if width > max_width || height > max_height {
            return Err(UploadError::TextureTooLarge { width, height });
        }

        Ok(())
    }

    fn validate_script(&self, file_path: &PathBuf) -> Result<(), UploadError> {
        // Read script content
        let content = std::fs::read_to_string(file_path)
            .map_err(|_| UploadError::InvalidFormat { format: "script".to_string() })?;

        // Check for banned keywords
        for keyword in &self.validation_rules.banned_keywords {
            if content.to_lowercase().contains(keyword) {
                return Err(UploadError::ContainsBannedContent {
                    reason: format!("Contains banned keyword: {}", keyword),
                });
            }
        }

        // TODO: Luau/Rhai syntax validation

        Ok(())
    }

    async fn generate_thumbnail(
        &self,
        _file_path: &PathBuf,
        _asset_type: &AssetType,
    ) -> Result<PathBuf, UploadError> {
        // TODO: Generate thumbnail based on asset type
        // - Model: Render preview image
        // - Texture: Resize to 256x256
        // - Audio: Waveform visualization
        // - Script: Code icon

        Ok(PathBuf::from("thumbnails/placeholder.png"))
    }
}

pub struct AssetStorage {
    root_path: PathBuf,
}

impl AssetStorage {
    pub async fn store(&self, file_path: &PathBuf) -> Result<PathBuf, UploadError> {
        // Copy file to storage location
        // In production: Use cloud storage (S3, Azure Blob, etc.)
        let uuid = uuid::Uuid::new_v4();
        let dest_path = self.root_path.join(format!("{}", uuid));

        std::fs::copy(file_path, &dest_path)
            .map_err(|_| UploadError::InvalidFormat { format: "copy failed".to_string() })?;

        Ok(dest_path)
    }
}

pub struct ModerationQueue {
    queue: Vec<UgcAsset>,
}

impl ModerationQueue {
    pub async fn enqueue(&mut self, asset: UgcAsset) {
        self.queue.push(asset);
        println!("Asset queued for moderation");
    }
}
```

### 1.3 Asset Database Schema

```sql
-- assets table
CREATE TABLE ugc_assets (
    id VARCHAR(36) PRIMARY KEY,
    creator_id VARCHAR(36) NOT NULL,
    asset_type VARCHAR(50) NOT NULL,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    price DECIMAL(10, 2),
    version INT NOT NULL DEFAULT 1,
    upload_date BIGINT NOT NULL,
    downloads BIGINT DEFAULT 0,
    rating FLOAT DEFAULT 0.0,
    moderation_status VARCHAR(20) NOT NULL,
    file_path VARCHAR(512) NOT NULL,
    file_size_bytes BIGINT NOT NULL,
    thumbnail_path VARCHAR(512),

    FOREIGN KEY (creator_id) REFERENCES users(id),
    INDEX idx_creator (creator_id),
    INDEX idx_status (moderation_status),
    INDEX idx_type (asset_type)
);

-- asset tags (many-to-many)
CREATE TABLE asset_tags (
    asset_id VARCHAR(36) NOT NULL,
    tag VARCHAR(50) NOT NULL,

    PRIMARY KEY (asset_id, tag),
    FOREIGN KEY (asset_id) REFERENCES ugc_assets(id) ON DELETE CASCADE
);

-- asset ratings
CREATE TABLE asset_ratings (
    user_id VARCHAR(36) NOT NULL,
    asset_id VARCHAR(36) NOT NULL,
    rating INT NOT NULL CHECK (rating >= 1 AND rating <= 5),
    review TEXT,
    created_at BIGINT NOT NULL,

    PRIMARY KEY (user_id, asset_id),
    FOREIGN KEY (asset_id) REFERENCES ugc_assets(id) ON DELETE CASCADE
);
```

---

## 🔒 Pillar 2: Scripting Sandbox (Safe User Code Execution)

### 2.1 Why Luau? (Roblox's Language)

**Luau** = Lua-derived language optimized for games
- ✅ **Sandboxed** by design
- ✅ **Fast** (JIT compilation)
- ✅ **Type-safe** (gradual typing)
- ✅ **Memory-safe** (no buffer overflows)
- ✅ **Battle-tested** (Roblox uses it)

### 2.2 Alternative: Rhai (Pure Rust)

**Rhai** = Rust-based scripting language
- ✅ **Native Rust integration**
- ✅ **Sandboxed**
- ✅ **Simpler than Luau**
- ⚠️ **Smaller ecosystem**

**Decision**: Start with **Rhai** (easier Rust integration), add Luau later

### 2.3 Scripting Architecture

```rust
// ไฟล์: engine/src/scripting/sandbox.rs

use rhai::{Engine, Scope, EvalAltResult, Dynamic, AST};
use std::time::Duration;

pub struct ScriptSandbox {
    engine: Engine,
    max_execution_time: Duration,
    memory_limit_mb: usize,
}

impl ScriptSandbox {
    pub fn new() -> Self {
        let mut engine = Engine::new();

        // Disable dangerous features
        engine.set_max_expr_depths(100, 50); // Prevent stack overflow
        engine.set_max_operations(1_000_000); // Prevent infinite loops
        engine.set_max_modules(10);
        engine.set_max_string_size(10_000);
        engine.set_max_array_size(1_000);

        // Register safe API
        Self::register_safe_api(&mut engine);

        Self {
            engine,
            max_execution_time: Duration::from_secs(5),
            memory_limit_mb: 50,
        }
    }

    fn register_safe_api(engine: &mut Engine) {
        // Game API: Safe functions only
        engine.register_fn("print", |msg: &str| {
            println!("[Script] {}", msg);
        });

        engine.register_fn("spawn_entity", |name: &str, x: f32, y: f32| {
            println!("Spawning entity '{}' at ({}, {})", name, x, y);
            // TODO: Actual entity spawning
        });

        engine.register_fn("get_player_position", || -> rhai::Map {
            let mut map = rhai::Map::new();
            map.insert("x".into(), Dynamic::from(100.0));
            map.insert("y".into(), Dynamic::from(200.0));
            map
        });

        // Blocked: File I/O, Network, System access
        // engine.register_fn("read_file", ...) // BLOCKED
        // engine.register_fn("http_request", ...) // BLOCKED
    }

    pub fn execute(
        &self,
        script: &str,
        context: ScriptContext,
    ) -> Result<Dynamic, Box<EvalAltResult>> {

        // Compile script first
        let ast = self.engine.compile(script)?;

        // Create scope with context variables
        let mut scope = Scope::new();
        scope.push("delta_time", context.delta_time);
        scope.push("player_id", context.player_id.clone());

        // Execute with timeout
        let result = std::thread::spawn(move || {
            self.engine.eval_ast_with_scope::<Dynamic>(&mut scope, &ast)
        });

        // Wait with timeout
        match result.join() {
            Ok(res) => res,
            Err(_) => Err("Script execution timeout".into()),
        }
    }

    pub fn validate_syntax(&self, script: &str) -> Result<(), Box<EvalAltResult>> {
        // Just compile, don't execute
        self.engine.compile(script)?;
        Ok(())
    }
}

pub struct ScriptContext {
    pub delta_time: f32,
    pub player_id: String,
    // Add more context as needed
}
```

### 2.4 Script Validation Before Upload

```rust
// ไฟล์: engine/src/scripting/validator.rs

pub struct ScriptValidator {
    sandbox: ScriptSandbox,
    banned_patterns: Vec<String>,
}

impl ScriptValidator {
    pub fn new() -> Self {
        Self {
            sandbox: ScriptSandbox::new(),
            banned_patterns: vec![
                r"while\s*true".to_string(),  // Infinite loops
                r"eval\(".to_string(),         // Dynamic eval
                r"__".to_string(),             // Private APIs
            ],
        }
    }

    pub fn validate(&self, script: &str) -> Result<(), String> {
        // Step 1: Check for banned patterns
        for pattern in &self.banned_patterns {
            if script.contains(pattern) {
                return Err(format!("Script contains banned pattern: {}", pattern));
            }
        }

        // Step 2: Syntax validation
        self.sandbox.validate_syntax(script)
            .map_err(|e| format!("Syntax error: {}", e))?;

        // Step 3: Static analysis (optional)
        // TODO: Detect suspicious code patterns

        Ok(())
    }
}
```

### 2.5 Luau Integration (Future)

```rust
// ไฟล์: engine/src/scripting/luau.rs (future implementation)

// Using mlua crate with Luau backend
use mlua::{Lua, Result, Table};

pub struct LuauSandbox {
    lua: Lua,
}

impl LuauSandbox {
    pub fn new() -> Result<Self> {
        let lua = Lua::new();

        // Disable dangerous globals
        let globals = lua.globals();
        globals.set("io", mlua::Nil)?;      // No file I/O
        globals.set("os", mlua::Nil)?;      // No OS access
        globals.set("debug", mlua::Nil)?;   // No debug hooks
        globals.set("package", mlua::Nil)?; // No module loading

        // Register safe API
        Self::register_api(&lua)?;

        Ok(Self { lua })
    }

    fn register_api(lua: &Lua) -> Result<()> {
        let game = lua.create_table()?;

        // game.spawn(name, x, y)
        game.set("spawn", lua.create_function(|_, (name, x, y): (String, f32, f32)| {
            println!("Spawning {} at ({}, {})", name, x, y);
            Ok(())
        })?)?;

        lua.globals().set("game", game)?;

        Ok(())
    }

    pub fn execute(&self, script: &str) -> Result<()> {
        self.lua.load(script).exec()
    }
}
```

---

## 🎨 Pillar 3: Visual Editor Tools

### 3.1 Editor Architecture

```
┌─────────────────────────────────────────────┐
│         In-Game Editor (Runtime)            │
├─────────────────────────────────────────────┤
│  - Scene Editor (Place objects)             │
│  - Terrain Editor (Paint terrain)           │
│  - Script Editor (Write Rhai/Luau)          │
│  - UI Editor (Design interfaces)            │
│  - Animation Editor (Create animations)     │
└─────────────────────────────────────────────┘
```

### 3.2 Scene Editor

```rust
// ไฟล์: editor/src/scene_editor.rs

pub struct SceneEditor {
    pub selected_entity: Option<Entity>,
    pub gizmo_mode: GizmoMode,
    pub asset_browser: AssetBrowser,
}

pub enum GizmoMode {
    Translate,  // Move
    Rotate,     // Rotate
    Scale,      // Scale
}

impl SceneEditor {
    pub fn handle_input(&mut self, input: &InputState) {
        // Click to select entity
        if input.mouse_button_pressed(MouseButton::Left) {
            self.selected_entity = self.raycast_entity(input.mouse_position);
        }

        // Drag to move (if gizmo active)
        if let Some(entity) = self.selected_entity {
            if input.mouse_button_held(MouseButton::Left) {
                self.update_gizmo(entity, input.mouse_delta);
            }
        }

        // Hotkeys
        if input.key_pressed(KeyCode::Delete) {
            if let Some(entity) = self.selected_entity {
                self.delete_entity(entity);
            }
        }

        if input.key_pressed(KeyCode::G) {
            self.gizmo_mode = GizmoMode::Translate;
        }
    }

    pub fn spawn_from_asset(&mut self, asset_id: &str, position: Vec3) {
        // Load asset and instantiate in scene
        println!("Spawning asset {} at {:?}", asset_id, position);
    }
}

pub struct AssetBrowser {
    pub categories: Vec<AssetCategory>,
    pub search_query: String,
    pub selected_asset: Option<String>,
}

pub struct AssetCategory {
    pub name: String,
    pub assets: Vec<UgcAsset>,
}
```

### 3.3 Visual Scripting (Future)

```
┌───────────────────────────────────────┐
│     Node-Based Visual Scripting      │
├───────────────────────────────────────┤
│  ┌──────┐      ┌──────┐              │
│  │Start │─────▶│ If  │──┐            │
│  └──────┘      └──────┘  │            │
│                 ▲    │    │            │
│                 │    └────▼────────┐   │
│              ┌──┴────┐  ┌────────┐ │   │
│              │Health<│  │ Damage │ │   │
│              │  50   │  │ Player │ │   │
│              └───────┘  └────────┘ │   │
│                           │        │   │
│                           └────────┘   │
└───────────────────────────────────────┘
```

---

## 💰 Pillar 4: Marketplace & Monetization

### 4.1 Economy Model (Roblox-style)

```rust
// ไฟล์: engine/src/marketplace/mod.rs

pub struct Marketplace {
    pub listings: Vec<MarketplaceListing>,
    pub transactions: Vec<Transaction>,
    pub currency: VirtualCurrency,
}

pub struct VirtualCurrency {
    pub name: String,         // e.g., "Robux", "V-Bucks"
    pub usd_conversion: f32,  // e.g., 100 currency = $1 USD
}

pub struct MarketplaceListing {
    pub asset_id: String,
    pub seller_id: String,
    pub price_currency: u64,
    pub stock: Option<u32>,   // None = unlimited
    pub sales_count: u64,
}

pub struct Transaction {
    pub id: String,
    pub buyer_id: String,
    pub seller_id: String,
    pub asset_id: String,
    pub price_paid: u64,
    pub platform_fee: u64,    // Platform takes %
    pub seller_revenue: u64,
    pub timestamp: i64,
}

impl Marketplace {
    pub fn purchase_asset(
        &mut self,
        buyer_id: String,
        asset_id: String,
    ) -> Result<Transaction, PurchaseError> {

        // Find listing
        let listing = self.listings.iter()
            .find(|l| l.asset_id == asset_id)
            .ok_or(PurchaseError::AssetNotFound)?;

        // Check buyer balance
        let buyer_balance = self.get_user_balance(&buyer_id);
        if buyer_balance < listing.price_currency {
            return Err(PurchaseError::InsufficientFunds);
        }

        // Calculate fees (platform takes 30%, like Roblox)
        let platform_fee = (listing.price_currency as f32 * 0.30) as u64;
        let seller_revenue = listing.price_currency - platform_fee;

        // Create transaction
        let transaction = Transaction {
            id: uuid::Uuid::new_v4().to_string(),
            buyer_id: buyer_id.clone(),
            seller_id: listing.seller_id.clone(),
            asset_id: asset_id.clone(),
            price_paid: listing.price_currency,
            platform_fee,
            seller_revenue,
            timestamp: chrono::Utc::now().timestamp(),
        };

        // Process payment
        self.deduct_balance(&buyer_id, listing.price_currency);
        self.add_balance(&listing.seller_id, seller_revenue);
        self.add_balance("platform", platform_fee);

        // Grant asset to buyer
        self.grant_asset(&buyer_id, &asset_id);

        // Record transaction
        self.transactions.push(transaction.clone());

        Ok(transaction)
    }

    fn get_user_balance(&self, _user_id: &str) -> u64 {
        1000 // Example balance
    }

    fn deduct_balance(&mut self, _user_id: &str, _amount: u64) {
        // TODO: Update user balance in database
    }

    fn add_balance(&mut self, _user_id: &str, _amount: u64) {
        // TODO: Update user balance in database
    }

    fn grant_asset(&mut self, _user_id: &str, _asset_id: &str) {
        // TODO: Add asset to user's inventory
    }
}

#[derive(Debug)]
pub enum PurchaseError {
    AssetNotFound,
    InsufficientFunds,
    AssetNotApproved,
}
```

### 4.2 Revenue Sharing Model

| Stakeholder | Cut | Example ($100 sale) |
|-------------|-----|---------------------|
| **Creator** | 70% | $70 |
| **Platform** | 30% | $30 |

Compare to:
- Roblox: 30% creator, 70% platform
- Unity Asset Store: 70% creator, 30% platform
- Unreal Marketplace: 88% creator, 12% platform

**Recommendation**: Start with **70/30** (creator-friendly)

---

## 🛡️ Pillar 5: Moderation & Safety

### 5.1 Three-Layer Moderation

```
Layer 1: Automated (AI/ML)     ─┐
  - Image recognition           │
  - Text filtering              ├─▶ 95% of content
  - Code analysis               │
Layer 2: Community              │
  - User reports               ─┤
  - Trusted moderators          │
Layer 3: Manual Review         ─┘
  - Human moderators            ├─▶ 5% flagged content
  - Appeals process            ─┘
```

### 5.2 Automated Moderation

```rust
// ไฟล์: engine/src/moderation/auto_moderator.rs

pub struct AutoModerator {
    text_filter: TextFilter,
    image_scanner: ImageScanner,
    code_analyzer: CodeAnalyzer,
}

pub struct TextFilter {
    banned_words: Vec<String>,
    profanity_patterns: Vec<String>,
}

impl TextFilter {
    pub fn check(&self, text: &str) -> ModerationResult {
        for word in &self.banned_words {
            if text.to_lowercase().contains(word) {
                return ModerationResult::Rejected {
                    reason: format!("Contains banned word: {}", word),
                };
            }
        }

        // Check for profanity patterns (regex)
        // TODO: Use ML model for better detection

        ModerationResult::Approved
    }
}

pub struct ImageScanner {
    // Use cloud ML APIs (Google Vision, AWS Rekognition)
}

impl ImageScanner {
    pub async fn scan(&self, image_path: &PathBuf) -> ModerationResult {
        // TODO: Call cloud API
        // Check for:
        // - Inappropriate content
        // - Violence
        // - Copyrighted material

        ModerationResult::Approved
    }
}

pub struct CodeAnalyzer {
    // Static code analysis
}

impl CodeAnalyzer {
    pub fn analyze(&self, script: &str) -> ModerationResult {
        // Check for malicious patterns
        let malicious_patterns = vec![
            "while true",      // Infinite loop
            "os.execute",      // System command
            "__index",         // Metatable manipulation
        ];

        for pattern in malicious_patterns {
            if script.contains(pattern) {
                return ModerationResult::Flagged {
                    reason: format!("Suspicious code pattern: {}", pattern),
                };
            }
        }

        ModerationResult::Approved
    }
}

pub enum ModerationResult {
    Approved,
    Rejected { reason: String },
    Flagged { reason: String },  // Needs manual review
}
```

### 5.3 User Report System

```rust
// ไฟล์: engine/src/moderation/reports.rs

pub struct ReportSystem {
    pub reports: Vec<UserReport>,
}

pub struct UserReport {
    pub id: String,
    pub reporter_id: String,
    pub asset_id: String,
    pub reason: ReportReason,
    pub description: String,
    pub timestamp: i64,
    pub status: ReportStatus,
}

pub enum ReportReason {
    InappropriateContent,
    Copyright,
    Spam,
    Scam,
    Malware,
    Other,
}

pub enum ReportStatus {
    Pending,
    UnderReview,
    Resolved,
    Dismissed,
}

impl ReportSystem {
    pub fn submit_report(
        &mut self,
        reporter_id: String,
        asset_id: String,
        reason: ReportReason,
        description: String,
    ) {
        let report = UserReport {
            id: uuid::Uuid::new_v4().to_string(),
            reporter_id,
            asset_id,
            reason,
            description,
            timestamp: chrono::Utc::now().timestamp(),
            status: ReportStatus::Pending,
        };

        self.reports.push(report);

        // Auto-escalate if many reports for same asset
        self.check_auto_escalation();
    }

    fn check_auto_escalation(&mut self) {
        // Count reports per asset
        let mut asset_report_counts = std::collections::HashMap::new();

        for report in &self.reports {
            if report.status == ReportStatus::Pending {
                *asset_report_counts.entry(report.asset_id.clone()).or_insert(0) += 1;
            }
        }

        // Auto-flag if > 10 reports
        for (asset_id, count) in asset_report_counts {
            if count > 10 {
                println!("Auto-flagging asset {} ({}  reports)", asset_id, count);
                // TODO: Auto-disable asset pending review
            }
        }
    }
}
```

---

## 🌐 Pillar 6: Multiplayer & Realtime Collaboration

### 6.1 Architecture (Like Roblox Studio Collaboration)

```
┌────────────────────────────────────────────┐
│          Collaborative Editing             │
├────────────────────────────────────────────┤
│  User A ──┐                                │
│           ├─▶ Websocket Server ◀─┬─ User B │
│  User C ──┘         │             │        │
│                     ▼             │        │
│              Conflict Resolution  │        │
│                     │             │        │
│                     ▼             │        │
│              Broadcast Changes ───┘        │
└────────────────────────────────────────────┘
```

### 6.2 Websocket Server (Rust)

```rust
// ไฟล์: server/src/collaboration_server.rs

use tokio::net::TcpListener;
use tokio_tungstenite::accept_async;
use futures_util::{StreamExt, SinkExt};
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CollaborationMessage {
    EntitySpawned { id: String, position: [f32; 3] },
    EntityMoved { id: String, position: [f32; 3] },
    EntityDeleted { id: String },
    ScriptEdited { entity_id: String, new_code: String },
    UserJoined { user_id: String, username: String },
    UserLeft { user_id: String },
}

pub struct CollaborationServer {
    sessions: Vec<CollaborationSession>,
}

pub struct CollaborationSession {
    pub scene_id: String,
    pub users: Vec<String>,
}

impl CollaborationServer {
    pub async fn run() {
        let listener = TcpListener::bind("127.0.0.1:9001").await.unwrap();

        while let Ok((stream, _)) = listener.accept().await {
            tokio::spawn(async move {
                let ws_stream = accept_async(stream).await.unwrap();
                Self::handle_connection(ws_stream).await;
            });
        }
    }

    async fn handle_connection(ws_stream: tokio_tungstenite::WebSocketStream<tokio::net::TcpStream>) {
        let (mut write, mut read) = ws_stream.split();

        while let Some(msg) = read.next().await {
            if let Ok(msg) = msg {
                if let Ok(text) = msg.to_text() {
                    // Parse message
                    if let Ok(collab_msg) = serde_json::from_str::<CollaborationMessage>(text) {
                        // Broadcast to other users
                        Self::broadcast(collab_msg).await;
                    }
                }
            }
        }
    }

    async fn broadcast(_msg: CollaborationMessage) {
        // TODO: Send to all connected clients in same session
    }
}
```

---

## 📊 Pillar 7: Analytics & Economy Balance

### 7.1 Track Everything

```rust
// ไฟล์: engine/src/analytics/tracker.rs

pub struct AnalyticsTracker {
    events: Vec<AnalyticsEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsEvent {
    pub event_type: EventType,
    pub user_id: String,
    pub timestamp: i64,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventType {
    AssetUploaded,
    AssetDownloaded,
    AssetPurchased,
    GamePlayed,
    ScriptExecuted,
    UserReported,
}

impl AnalyticsTracker {
    pub fn track(&mut self, event_type: EventType, user_id: String, metadata: serde_json::Value) {
        let event = AnalyticsEvent {
            event_type,
            user_id,
            timestamp: chrono::Utc::now().timestamp(),
            metadata,
        };

        self.events.push(event);

        // Send to analytics backend (Google Analytics, Mixpanel, etc.)
    }
}
```

### 7.2 Economy Dashboard

```
┌───────────────────────────────────────────┐
│         Economy Health Dashboard          │
├───────────────────────────────────────────┤
│  Total Currency in Circulation: 10M       │
│  New Users Today: 1,234                   │
│  Assets Sold Today: 567                   │
│  Platform Revenue Today: $12,345          │
│  Average Asset Price: 150 currency        │
│  Top Earning Creator: User123 ($5,000)    │
└───────────────────────────────────────────┘
```

---

## 🚀 Implementation Roadmap

### Phase 1: Foundation (3 months)
- [ ] Asset upload system (models, textures, audio)
- [ ] Basic validation (file size, format)
- [ ] Asset database
- [ ] Simple marketplace (browse, download)

### Phase 2: Scripting (2 months)
- [ ] Rhai sandbox integration
- [ ] Script validation
- [ ] Safe API design
- [ ] Script editor UI

### Phase 3: Editor (3 months)
- [ ] In-game scene editor
- [ ] Asset browser
- [ ] Gizmo tools (translate, rotate, scale)
- [ ] Terrain editor

### Phase 4: Monetization (2 months)
- [ ] Virtual currency system
- [ ] Marketplace purchasing
- [ ] Revenue tracking
- [ ] Payout system

### Phase 5: Moderation (2 months)
- [ ] Automated text/image filtering
- [ ] User report system
- [ ] Manual review queue
- [ ] Appeals process

### Phase 6: Multiplayer (3 months)
- [ ] Websocket server
- [ ] Realtime collaboration
- [ ] Conflict resolution
- [ ] Session management

### Phase 7: Analytics & Polish (1 month)
- [ ] Analytics tracking
- [ ] Economy dashboard
- [ ] Performance optimization
- [ ] Bug fixes

**Total**: ~16 months to full UGC platform

---

## 📚 Best Practices from Roblox

### 1. Safety First
- ✅ Sandbox ALL user code
- ✅ Validate ALL uploads
- ✅ Monitor for abuse
- ✅ Quick response to reports

### 2. Creator-Friendly
- ✅ Easy-to-use tools
- ✅ Fair revenue share (70/30)
- ✅ Fast payout (weekly/monthly)
- ✅ Creator support & docs

### 3. Scalability
- ✅ Cloud storage for assets (S3/Azure)
- ✅ CDN for fast downloads
- ✅ Database sharding
- ✅ Horizontal scaling

### 4. Community
- ✅ Forums & Discord
- ✅ Showcase top creators
- ✅ Contests & events
- ✅ Creator education (tutorials)

---

## 🎯 Success Metrics

| Metric | Target (Year 1) |
|--------|-----------------|
| **Active Creators** | 10,000+ |
| **Assets Uploaded** | 100,000+ |
| **Monthly Sales** | $100,000+ |
| **Daily Active Users** | 50,000+ |
| **Creator Earnings** | $50,000+/month total |
| **Platform Revenue** | $30,000+/month (30% cut) |

---

## 🔗 References & Sources

### Roblox UGC
- [Roblox UGC Creator Guide](https://reelmind.ai/blog/roblox-ugc-creator-user-generated-content)
- [Roblox Moments: User-Generated Discovery](https://corp.roblox.com/newsroom/2025/09/roblox-moments-user-generated-discovery)
- [Roblox System Design Interview](https://www.systemdesignhandbook.com/guides/roblox-system-design-interview/)
- [How Roblox Made UGC Work](https://patch3000.medium.com/how-roblox-made-user-generated-content-ugc-work-2a06434349c4)

### Game Engine UGC
- [How UGC Can Make Your Game a Hit](https://lootlocker.com/blog/how-ugc-make-game-hit)
- [UGC in Unity Game Development](https://medium.com/@william.miller5612/the-power-of-user-generated-content-ugc-empowering-players-in-unity-game-development-46d3fae9415f)
- [Safe UGC Management - CurseForge](https://docs.curseforge.com/docs/content-management/moderation/overview/)

### Luau Security
- [Luau Sandboxing](https://luau.org/sandbox)
- [Luau Security Guide](https://github.com/Roblox/luau/blob/master/SECURITY.md)
- [Luau Sandboxer Community Resource](https://devforum.roblox.com/t/v133-luau-sandboxer/3858342)

---

## ✅ สรุป

**UGC Engine Design สำเร็จ = 7 Pillars:**

1. ✅ **Asset System** - Upload, validate, store
2. ✅ **Scripting Sandbox** - Rhai/Luau, safe execution
3. ✅ **Editor Tools** - Visual scene/script editor
4. ✅ **Marketplace** - Buy/sell, 70/30 revenue split
5. ✅ **Moderation** - AI + Community + Manual
6. ✅ **Multiplayer** - Realtime collaboration
7. ✅ **Analytics** - Track economy health

**Timeline**: 16 เดือน → Full UGC Platform
**Investment**: High, but **massive potential** (multi-billion $ industry)

**Next Step**: เริ่มจาก **Phase 1** (Asset System) และสร้าง MVP! 🚀
