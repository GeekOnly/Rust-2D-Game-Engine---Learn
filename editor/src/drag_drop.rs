/// Drag & Drop system for assets
use std::path::PathBuf;
use crate::asset_manager::AssetType;

#[derive(Debug, Clone)]
pub struct DraggedAsset {
    pub path: PathBuf,
    pub name: String,
    pub asset_type: AssetType,
}

pub struct DragDropState {
    pub dragging: Option<DraggedAsset>,
    pub drop_position: Option<egui::Pos2>,
}

impl DragDropState {
    pub fn new() -> Self {
        Self {
            dragging: None,
            drop_position: None,
        }
    }
    
    pub fn start_drag(&mut self, asset: DraggedAsset) {
        self.dragging = Some(asset);
    }
    
    pub fn stop_drag(&mut self) {
        self.dragging = None;
        self.drop_position = None;
    }
    
    pub fn is_dragging(&self) -> bool {
        self.dragging.is_some()
    }
    
    pub fn set_drop_position(&mut self, pos: egui::Pos2) {
        self.drop_position = Some(pos);
    }
    
    pub fn get_dragged_asset(&self) -> Option<&DraggedAsset> {
        self.dragging.as_ref()
    }
}

impl Default for DragDropState {
    fn default() -> Self {
        Self::new()
    }
}
