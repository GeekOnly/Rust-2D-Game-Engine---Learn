#[cfg(feature = "hecs")]
mod hecs_minimal_impl {
    use crate::{Transform, Sprite, Collider, Rigidbody2D, Mesh, Camera, Script, EntityTag};
    use crate::traits::{EcsWorld, ComponentAccess, EcsError, Serializable};
    use hecs::{self, Component};
    use std::collections::HashMap;

    /// Minimal wrapper around hecs::World
    pub struct HecsMinimal {
        inner: hecs::World,
    }

    impl HecsMinimal {
        pub fn new() -> Self {
            Self {
                inner: hecs::World::new(),
            }
        }
    }

    impl Default for HecsMinimal {
        fn default() -> Self {
            Self::new()
        }
    }

    impl EcsWorld for HecsMinimal {
        type Entity = hecs::Entity;
        type Error = EcsError;

        fn spawn(&mut self) -> Self::Entity {
            self.inner.spawn(())
        }

        fn despawn(&mut self, entity: Self::Entity) -> Result<(), Self::Error> {
            self.inner.despawn(entity).map_err(|_| EcsError::EntityNotFound)
        }

        fn is_alive(&self, entity: Self::Entity) -> bool {
            self.inner.contains(entity)
        }

        fn clear(&mut self) {
            self.inner.clear();
        }

        fn entity_count(&self) -> usize {
            self.inner.len() as usize
        }

        fn set_parent(&mut self, _child: Self::Entity, _parent: Option<Self::Entity>) -> Result<(), Self::Error> {
            // Minimal backend doesn't support hierarchy
            Ok(())
        }

        fn get_parent(&self, _entity: Self::Entity) -> Option<Self::Entity> {
            None
        }

        fn get_children(&self, _entity: Self::Entity) -> Vec<Self::Entity> {
            Vec::new()
        }
    }

    impl Serializable for HecsMinimal {
        fn save_to_json(&self) -> Result<String, Box<dyn std::error::Error>> {
            Ok("{}".to_string())
        }
        
        fn load_from_json(&mut self, _json: &str) -> Result<(), Box<dyn std::error::Error>> {
            self.clear();
            Ok(())
        }
    }

    macro_rules! impl_hecs_minimal_access {
        ($component:ty) => {
            impl ComponentAccess<$component> for HecsMinimal {
                type Entity = hecs::Entity;
                type Error = EcsError;
                
                type ReadGuard<'a> = hecs::Ref<'a, $component>;
                type WriteGuard<'a> = hecs::RefMut<'a, $component>;

                fn insert(&mut self, entity: Self::Entity, component: $component) -> Result<Option<$component>, Self::Error> {
                    if !self.inner.contains(entity) {
                        return Err(EcsError::EntityNotFound);
                    }
                    
                    let prev = self.inner.get::<&$component>(entity).ok().map(|c| (*c).clone());
                    self.inner.insert_one(entity, component).map_err(|_| EcsError::ComponentInsertFailed)?;
                    Ok(prev)
                }

                fn get<'a>(&'a self, entity: Self::Entity) -> Option<Self::ReadGuard<'a>> {
                    self.inner.get::<&$component>(entity).ok()
                }

                fn get_mut<'a>(&'a mut self, entity: Self::Entity) -> Option<Self::WriteGuard<'a>> {
                    self.inner.get::<&mut $component>(entity).ok()
                }

                fn remove(&mut self, entity: Self::Entity) -> Result<Option<$component>, Self::Error> {
                    if !self.inner.contains(entity) {
                        return Err(EcsError::EntityNotFound);
                    }
                    Ok(self.inner.remove_one::<$component>(entity).ok())
                }

                fn has(&self, entity: Self::Entity) -> bool {
                    self.inner.get::<&$component>(entity).is_ok()
                }
            }
        };
    }

    impl_hecs_minimal_access!(Transform);
    impl_hecs_minimal_access!(Sprite);
    impl_hecs_minimal_access!(Collider);
    impl_hecs_minimal_access!(Rigidbody2D);
    impl_hecs_minimal_access!(Mesh);
    impl_hecs_minimal_access!(Camera);
    impl_hecs_minimal_access!(Script);
    impl_hecs_minimal_access!(EntityTag);
    impl_hecs_minimal_access!((f32, f32));
    impl_hecs_minimal_access!(bool);
    impl_hecs_minimal_access!(u8);
    impl_hecs_minimal_access!(crate::Grid);
    impl_hecs_minimal_access!(crate::Tilemap);
    impl_hecs_minimal_access!(crate::TileSet);
    impl_hecs_minimal_access!(String);
}

#[cfg(feature = "hecs")]
pub use hecs_minimal_impl::HecsMinimal;
