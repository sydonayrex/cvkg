//! App-owned entities with stable handles and explicit notification.
//!
//! `State<T>` remains the lightweight reactive value. An `EntityArena<T>` adds
//! identity and observation for state that outlives a single view.

use std::marker::PhantomData;
use std::sync::Arc;

/// A stable, typed handle into an [`EntityArena`].
#[derive(Debug, Eq, PartialEq)]
pub struct Entity<T> {
    index: usize,
    generation: u64,
    _marker: PhantomData<fn() -> T>,
}

impl<T> Clone for Entity<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for Entity<T> {}

impl<T> Entity<T> {
    fn new(index: usize, generation: u64) -> Self {
        Self {
            index,
            generation,
            _marker: PhantomData,
        }
    }
}

/// Callback registered by [`EntityArena::observe`].
pub type Observer<T> = Arc<dyn Fn(&T) + Send + Sync>;

struct Slot<T> {
    generation: u64,
    value: Option<T>,
    observers: Vec<Observer<T>>,
}

/// An arena of typed entities.
pub struct EntityArena<T: Clone + Send + Sync + 'static> {
    slots: Vec<Slot<T>>,
}

impl<T: Clone + Send + Sync + 'static> Default for EntityArena<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Clone + Send + Sync + 'static> EntityArena<T> {
    pub fn new() -> Self {
        Self { slots: Vec::new() }
    }

    pub fn insert(&mut self, value: T) -> Entity<T> {
        let index = self.slots.len();
        self.slots.push(Slot {
            generation: 0,
            value: Some(value),
            observers: Vec::new(),
        });
        Entity::new(index, 0)
    }

    pub fn get(&self, entity: Entity<T>) -> Option<T> {
        self.slot(entity).and_then(|slot| slot.value.clone())
    }

    pub fn update<F: FnOnce(&mut T)>(&mut self, entity: Entity<T>, update: F) {
        if let Some(slot) = self.slots.get_mut(entity.index) {
            if slot.generation == entity.generation {
                if let Some(value) = slot.value.as_mut() {
                    update(value);
                }
            }
        }
    }

    /// Register an observer. It runs only when `notify` is called.
    pub fn observe<F>(&mut self, entity: Entity<T>, observer: F)
    where
        F: Fn(&T) + Send + Sync + 'static,
    {
        if let Some(slot) = self.slots.get_mut(entity.index) {
            if slot.generation == entity.generation {
                slot.observers.push(Arc::new(observer));
            }
        }
    }

    /// Notify every observer once with the current value.
    pub fn notify(&self, entity: Entity<T>) {
        let observers = match self.slots.get(entity.index) {
            Some(slot) if slot.generation == entity.generation => slot.observers.clone(),
            _ => return,
        };
        if let Some(value) = self.get(entity) {
            for observer in observers {
                observer(&value);
            }
        }
    }

    /// Invalidate the handle and drop the value, keeping observers intact.
    pub fn remove(&mut self, entity: Entity<T>) {
        if let Some(slot) = self.slots.get_mut(entity.index) {
            if slot.generation == entity.generation {
                slot.value = None;
                slot.generation += 1;
            }
        }
    }

    fn slot(&self, entity: Entity<T>) -> Option<&Slot<T>> {
        let slot = self.slots.get(entity.index)?;
        (slot.generation == entity.generation).then_some(slot)
    }
}
