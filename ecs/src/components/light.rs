use serde::{Serialize, Deserialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum LightType {
    Directional,
    Point,
    Spot,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Light {
    pub light_type: LightType,
    pub color: [f32; 3],
    pub intensity: f32,
    pub range: f32, // For point/spot lights
    pub cast_shadows: bool,
}

impl Default for Light {
    fn default() -> Self {
        Self {
            light_type: LightType::Directional,
            color: [1.0, 1.0, 1.0],
            intensity: 1.0,
            range: 10.0,
            cast_shadows: false,
        }
    }
}
