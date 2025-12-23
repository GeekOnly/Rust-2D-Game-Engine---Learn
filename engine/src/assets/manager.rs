use std::any::Any;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use crate::assets::core::AssetId;

/// A lightweight handle to an asset.
/// Takes ownership of the underlying Arc, keeping the asset alive.
#[derive(Debug, Clone)]
pub struct AssetHandle<T: ?Sized> {
    id: AssetId,
    inner: Arc<T>,
}

impl<T: ?Sized> AssetHandle<T> {
    pub fn new(id: AssetId, inner: Arc<T>) -> Self {
        Self { id, inner }
    }

    pub fn id(&self) -> AssetId {
        self.id
    }

    pub fn get(&self) -> &T {
        &self.inner
    }
}

impl<T: ?Sized> std::ops::Deref for AssetHandle<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

/// Type-erased container for any asset.
type AnyAsset = Arc<dyn Any + Send + Sync>;

/// Runtime Asset Manager.
/// Manages loading and caching of assets.
pub struct AssetManager {
    /// Cache of loaded assets, type-erased.
    cache: RwLock<HashMap<AssetId, AnyAsset>>,
    
    // In a real implementation, we'd need a way to map ID -> Path (or a loader trait).
    // For now, we'll keep it simple or assume we can register loaders.
}

impl AssetManager {
    pub fn new() -> Self {
        Self {
            cache: RwLock::new(HashMap::new()),
        }
    }

    /// Checks if an asset is already loaded.
    pub fn is_loaded(&self, id: AssetId) -> bool {
        self.cache.read().unwrap().contains_key(&id)
    }

    /// Manually registers an asset (useful for procedural assets or testing).
    pub fn add<T: Any + Send + Sync>(&self, id: AssetId, asset: T) -> AssetHandle<T> {
        let arc_asset = Arc::new(asset);
        // Store as Any
        self.cache.write().unwrap().insert(id, arc_asset.clone());
        
        AssetHandle::new(id, arc_asset)
    }

    /// Tries to get an existing handle for an asset.
    pub fn get_handle<T: Any + Send + Sync>(&self, id: AssetId) -> Option<AssetHandle<T>> {
        let cache = self.cache.read().unwrap();
        
        if let Some(any_asset) = cache.get(&id) {
            // Downcast
            if let Ok(typed_asset) = any_asset.clone().downcast::<T>() {
                return Some(AssetHandle::new(id, typed_asset));
            } else {
                log::warn!("Asset {} exists but is wrong type", id);
            }
        }
        None
    }
    
    // In Phase 4, we will add `load<T>(id)` which actually goes to disk.
    // That requires the AssetDatabase map (ID -> Path) to be available to the runtime,
    // or passed in.
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, PartialEq)]
    struct MockTexture {
        width: u32,
    }

    #[test]
    fn test_asset_caching() {
        let manager = AssetManager::new();
        let id = AssetId::new();
        let texture = MockTexture { width: 100 };

        // 1. Add asset
        let handle = manager.add(id, texture);
        assert_eq!(handle.width, 100);

        // 2. Retrieve handle
        let handle2: AssetHandle<MockTexture> = manager.get_handle(id).expect("Should come back");
        assert_eq!(handle2.width, 100);
        
        // 3. Verify IDs match
        assert_eq!(handle.id(), handle2.id());
    }
}
