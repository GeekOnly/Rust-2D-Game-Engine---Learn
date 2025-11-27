/// Component Management System (Unity-like)
/// 
/// ระบบจัดการ Component แบบ Unity ที่สามารถ:
/// - AddComponent<T>() - เพิ่ม Component
/// - RemoveComponent<T>() - ลบ Component
/// - GetComponent<T>() - ดึงข้อมูล Component
/// - HasComponent<T>() - ตรวจสอบว่ามี Component หรือไม่

use crate::{World, Entity, Transform, Sprite, Collider, Mesh, Camera, Script, EntityTag};
use std::collections::HashMap;

/// Component Type Enum สำหรับระบุประเภท Component
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ComponentType {
    Transform,
    Sprite,
    BoxCollider,
    Rigidbody,
    Mesh,
    Camera,
    Script,
    Tag,
}

impl ComponentType {
    /// รายการ Component ทั้งหมดที่สามารถเพิ่มได้
    pub fn all() -> Vec<ComponentType> {
        vec![
            ComponentType::Transform,
            ComponentType::Sprite,
            ComponentType::BoxCollider,
            ComponentType::Rigidbody,
            ComponentType::Mesh,
            ComponentType::Camera,
            ComponentType::Script,
            ComponentType::Tag,
        ]
    }

    /// ชื่อ Component สำหรับแสดงใน UI
    pub fn display_name(&self) -> &'static str {
        match self {
            ComponentType::Transform => "Transform",
            ComponentType::Sprite => "Sprite Renderer",
            ComponentType::BoxCollider => "Box Collider",
            ComponentType::Rigidbody => "Rigidbody 2D",
            ComponentType::Mesh => "Mesh Renderer",
            ComponentType::Camera => "Camera",
            ComponentType::Script => "Script",
            ComponentType::Tag => "Tag",
        }
    }

    /// ตรวจสอบว่า Component นี้จำเป็นต้องมีหรือไม่ (ไม่สามารถลบได้)
    pub fn is_required(&self) -> bool {
        matches!(self, ComponentType::Transform)
    }
}

/// Component Manager Extension สำหรับ World
pub trait ComponentManager {
    /// เพิ่ม Component ให้กับ Entity (Unity-like AddComponent)
    fn add_component(&mut self, entity: Entity, component_type: ComponentType) -> Result<(), String>;
    
    /// ลบ Component จาก Entity (Unity-like RemoveComponent)
    fn remove_component(&mut self, entity: Entity, component_type: ComponentType) -> Result<(), String>;
    
    /// ตรวจสอบว่า Entity มี Component หรือไม่ (Unity-like HasComponent)
    fn has_component(&self, entity: Entity, component_type: ComponentType) -> bool;
    
    /// ดึงรายการ Component ทั้งหมดของ Entity
    fn get_components(&self, entity: Entity) -> Vec<ComponentType>;
    
    /// ดึงรายการ Component ที่สามารถเพิ่มได้ (ยังไม่มีใน Entity)
    fn get_addable_components(&self, entity: Entity) -> Vec<ComponentType>;
}

impl ComponentManager for World {
    fn add_component(&mut self, entity: Entity, component_type: ComponentType) -> Result<(), String> {
        // ตรวจสอบว่า Entity มีอยู่จริง
        if !self.active.contains_key(&entity) {
            return Err(format!("Entity {} does not exist", entity));
        }

        // ตรวจสอบว่ามี Component อยู่แล้วหรือไม่
        if self.has_component(entity, component_type) {
            return Err(format!("Entity {} already has {:?}", entity, component_type));
        }

        // เพิ่ม Component ตามประเภท
        match component_type {
            ComponentType::Transform => {
                self.transforms.insert(entity, Transform::default());
            }
            ComponentType::Sprite => {
                self.sprites.insert(entity, Sprite {
                    texture_id: "default".to_string(),
                    width: 32.0,
                    height: 32.0,
                    color: [1.0, 1.0, 1.0, 1.0],
                    billboard: false,
                });
            }
            ComponentType::BoxCollider => {
                self.colliders.insert(entity, Collider {
                    width: 32.0,
                    height: 32.0,
                });
            }
            ComponentType::Rigidbody => {
                self.velocities.insert(entity, (0.0, 0.0));
            }
            ComponentType::Mesh => {
                self.meshes.insert(entity, Mesh {
                    mesh_type: crate::MeshType::Cube,
                    color: [1.0, 1.0, 1.0, 1.0],
                });
            }
            ComponentType::Camera => {
                self.cameras.insert(entity, Camera::default());
            }
            ComponentType::Script => {
                self.scripts.insert(entity, Script {
                    script_name: "NewScript".to_string(),
                    enabled: true,
                    parameters: HashMap::new(),
                });
            }
            ComponentType::Tag => {
                self.tags.insert(entity, EntityTag::Player);
            }
        }

        Ok(())
    }

    fn remove_component(&mut self, entity: Entity, component_type: ComponentType) -> Result<(), String> {
        // ตรวจสอบว่า Component นี้จำเป็นหรือไม่
        if component_type.is_required() {
            return Err(format!("{} is required and cannot be removed", component_type.display_name()));
        }

        // ตรวจสอบว่ามี Component อยู่หรือไม่
        if !self.has_component(entity, component_type) {
            return Err(format!("Entity {} does not have {:?}", entity, component_type));
        }

        // ลบ Component ตามประเภท
        match component_type {
            ComponentType::Transform => {
                // Transform ไม่สามารถลบได้ (ตรวจสอบไว้ด้านบนแล้ว)
                unreachable!()
            }
            ComponentType::Sprite => {
                self.sprites.remove(&entity);
            }
            ComponentType::BoxCollider => {
                self.colliders.remove(&entity);
            }
            ComponentType::Rigidbody => {
                self.velocities.remove(&entity);
            }
            ComponentType::Mesh => {
                self.meshes.remove(&entity);
            }
            ComponentType::Camera => {
                self.cameras.remove(&entity);
            }
            ComponentType::Script => {
                self.scripts.remove(&entity);
            }
            ComponentType::Tag => {
                self.tags.remove(&entity);
            }
        }

        Ok(())
    }

    fn has_component(&self, entity: Entity, component_type: ComponentType) -> bool {
        match component_type {
            ComponentType::Transform => self.transforms.contains_key(&entity),
            ComponentType::Sprite => self.sprites.contains_key(&entity),
            ComponentType::BoxCollider => self.colliders.contains_key(&entity),
            ComponentType::Rigidbody => self.velocities.contains_key(&entity),
            ComponentType::Mesh => self.meshes.contains_key(&entity),
            ComponentType::Camera => self.cameras.contains_key(&entity),
            ComponentType::Script => self.scripts.contains_key(&entity),
            ComponentType::Tag => self.tags.contains_key(&entity),
        }
    }

    fn get_components(&self, entity: Entity) -> Vec<ComponentType> {
        let mut components = Vec::new();

        for component_type in ComponentType::all() {
            if self.has_component(entity, component_type) {
                components.push(component_type);
            }
        }

        components
    }

    fn get_addable_components(&self, entity: Entity) -> Vec<ComponentType> {
        ComponentType::all()
            .into_iter()
            .filter(|ct| !self.has_component(entity, *ct))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_sprite_component() {
        let mut world = World::new();
        let entity = world.spawn();

        // เพิ่ม Transform (จำเป็น)
        world.add_component(entity, ComponentType::Transform).unwrap();
        
        // เพิ่ม Sprite
        assert!(world.add_component(entity, ComponentType::Sprite).is_ok());
        assert!(world.has_component(entity, ComponentType::Sprite));
        assert!(world.sprites.contains_key(&entity));
    }

    #[test]
    fn test_remove_sprite_component() {
        let mut world = World::new();
        let entity = world.spawn();

        world.add_component(entity, ComponentType::Transform).unwrap();
        world.add_component(entity, ComponentType::Sprite).unwrap();

        // ลบ Sprite
        assert!(world.remove_component(entity, ComponentType::Sprite).is_ok());
        assert!(!world.has_component(entity, ComponentType::Sprite));
        assert!(!world.sprites.contains_key(&entity));
    }

    #[test]
    fn test_cannot_remove_transform() {
        let mut world = World::new();
        let entity = world.spawn();
        world.add_component(entity, ComponentType::Transform).unwrap();

        // ไม่สามารถลบ Transform ได้
        assert!(world.remove_component(entity, ComponentType::Transform).is_err());
    }

    #[test]
    fn test_get_components() {
        let mut world = World::new();
        let entity = world.spawn();

        world.add_component(entity, ComponentType::Transform).unwrap();
        world.add_component(entity, ComponentType::Sprite).unwrap();
        world.add_component(entity, ComponentType::BoxCollider).unwrap();

        let components = world.get_components(entity);
        assert_eq!(components.len(), 3);
        assert!(components.contains(&ComponentType::Transform));
        assert!(components.contains(&ComponentType::Sprite));
        assert!(components.contains(&ComponentType::BoxCollider));
    }

    #[test]
    fn test_get_addable_components() {
        let mut world = World::new();
        let entity = world.spawn();

        world.add_component(entity, ComponentType::Transform).unwrap();
        world.add_component(entity, ComponentType::Sprite).unwrap();

        let addable = world.get_addable_components(entity);
        
        // ต้องไม่มี Transform และ Sprite
        assert!(!addable.contains(&ComponentType::Transform));
        assert!(!addable.contains(&ComponentType::Sprite));
        
        // ต้องมี Component อื่นๆ
        assert!(addable.contains(&ComponentType::BoxCollider));
        assert!(addable.contains(&ComponentType::Rigidbody));
    }
}
