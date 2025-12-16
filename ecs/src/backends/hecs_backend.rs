#[cfg(feature = "hecs")]
mod hecs_impl {
    use crate::traits::{EcsWorld, ComponentAccess, EcsError};
    use hecs;
    use std::any::Any;
    use std::collections::HashMap;

    /// Wrapper around hecs::World to implement our EcsWorld trait
    pub struct HecsWorld {
    inner: hecs::World,
    // We need to track hierarchy manually or using components since hecs doesn't force it
    // For this implementation, we rely on hecs components if we want persistence, 
    // but here we might need aux storage for parents/children if they aren't standard components.
    // However, to keep it simple and trait-compliant, we can use hecs components for hierarchy too.
}

impl HecsWorld {
    pub fn new() -> Self {
        Self {
            inner: hecs::World::new(),
        }
    }
}

// Hierarchy Components
#[derive(Clone, Copy, Debug, PartialEq)]
struct Parent(hecs::Entity);

#[derive(Clone, Debug, PartialEq)]
struct Children(Vec<hecs::Entity>);

impl EcsWorld for HecsWorld {
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

    fn set_parent(&mut self, child: Self::Entity, parent: Option<Self::Entity>) -> Result<(), Self::Error> {
        // Validation: verify child exists
        if !self.is_alive(child) {
            return Err(EcsError::EntityNotFound);
        }

        // Remove from old parent
        if let Ok(old_parent) = self.inner.get::<&Parent>(child).map(|p| p.0) {
            if let Ok(mut children) = self.inner.get::<&mut Children>(old_parent) {
                children.0.retain(|&x| x != child);
            }
            self.inner.remove_one::<Parent>(child).unwrap(); // Remove Parent component
        }

        // Add to new parent
        if let Some(parent_entity) = parent {
            if !self.is_alive(parent_entity) {
                return Err(EcsError::EntityNotFound);
            }
            
            // Circular dependency check omitted for brevity in this MVP, but should be here

            self.inner.insert_one(child, Parent(parent_entity)).map_err(|_| EcsError::EntityNotFound)?;
            
            // Add to parent's children list
            let mut children_entry = self.inner.entry(parent_entity).ok_or(EcsError::EntityNotFound)?;
            if let Ok(mut children) = children_entry.get::<&mut Children>() {
               children.0.push(child); 
            } else {
                children_entry.insert(Children(vec![child])).unwrap();
            }
        }

        Ok(())
    }

    fn get_parent(&self, entity: Self::Entity) -> Option<Self::Entity> {
        self.inner.get::<&Parent>(entity).ok().map(|p| p.0)
    }

    fn get_children(&self, entity: Self::Entity) -> Vec<Self::Entity> {
        self.inner.get::<&Children>(entity).ok().map(|c| c.0.clone()).unwrap_or_default()
    }
}

// Implement ComponentAccess for any Type that is 'static + Send + Sync (hecs requirements)
// Note: We need a concrete implementation for the macro or generic impl. 
// Rust orphan rules might block generic impl for foreign trait + foreign type, 
// but ComponentAccess is local, so generic impl is allowed!

impl<T: hecs::Component + Clone> ComponentAccess<T> for HecsWorld {
    type Entity = hecs::Entity;
    type Error = EcsError;
    
    // Use hecs guards
    type ReadGuard<'a> = hecs::Ref<'a, T>;
    type WriteGuard<'a> = hecs::RefMut<'a, T>;

    fn insert(&mut self, entity: Self::Entity, component: T) -> Result<Option<T>, Self::Error> {
        if !self.inner.contains(entity) {
             return Err(EcsError::EntityNotFound);
        }
        
        let prev = self.inner.get::<&T>(entity).ok().map(|c| c.clone());
        self.inner.insert_one(entity, component).map_err(|_| EcsError::EntityNotFound)?;
        Ok(prev)
    }

    fn get<'a>(&'a self, entity: Self::Entity) -> Option<Self::ReadGuard<'a>> {
        self.inner.get::<&T>(entity).ok()
    }

    fn get_mut<'a>(&'a mut self, entity: Self::Entity) -> Option<Self::WriteGuard<'a>> {
        self.inner.get::<&mut T>(entity).ok()
    }

    fn remove(&mut self, entity: Self::Entity) -> Result<Option<T>, Self::Error> {
        if !self.inner.contains(entity) {
             return Err(EcsError::EntityNotFound);
        }
        let val = self.inner.remove_one::<T>(entity).ok();
        Ok(val)
    }

    fn has(&self, entity: Self::Entity) -> bool {
        self.inner.get::<&T>(entity).is_ok()
    }
}
}

#[cfg(feature = "hecs")]
pub use hecs_impl::HecsWorld;