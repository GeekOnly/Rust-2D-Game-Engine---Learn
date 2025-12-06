//! UI prefab system for reusable UI templates

use serde::{Deserialize, Serialize};
use crate::{
    RectTransform, UIElement, UIImage, UIText, UIButton, UIPanel,
    UISlider, UIToggle, UIDropdown, UIInputField, UIScrollView,
    UIMask, HorizontalLayoutGroup, VerticalLayoutGroup, GridLayoutGroup,
};

/// UI Prefab for reusable UI templates
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UIPrefab {
    /// Prefab name
    pub name: String,
    
    /// Root element data
    pub root: UIPrefabElement,
}

/// UI Prefab element (recursive structure)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UIPrefabElement {
    /// Element name
    pub name: String,
    
    /// Components
    pub rect_transform: RectTransform,
    pub ui_element: UIElement,
    pub image: Option<UIImage>,
    pub text: Option<UIText>,
    pub button: Option<UIButton>,
    pub panel: Option<UIPanel>,
    pub slider: Option<UISlider>,
    pub toggle: Option<UIToggle>,
    pub dropdown: Option<UIDropdown>,
    pub input_field: Option<UIInputField>,
    pub scroll_view: Option<UIScrollView>,
    pub mask: Option<UIMask>,
    pub horizontal_layout: Option<HorizontalLayoutGroup>,
    pub vertical_layout: Option<VerticalLayoutGroup>,
    pub grid_layout: Option<GridLayoutGroup>,
    
    /// Children
    pub children: Vec<UIPrefabElement>,
}
