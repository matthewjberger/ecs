use crate::{
	entity::Entity,
	error::{EntityGenerationError, Result},
};
use std::{
	any::TypeId,
	cell::RefCell,
	collections::hash_map::HashMap,
	ops::{Deref, DerefMut},
	rc::Rc,
};

/*
   Entities:                    Entity 0                       Entity 1   Entity 2                         Entity 3
   Physics Components   -> Vec( Some(Physics { vel: 3 }),      None,      Some(Physics { vel: 10 }),       Some(Physics { vel: 04 }) )
   Position Components  -> Vec( Some(Position { x: 3, y: 3 }), None,      Some(Position { x: 10, y: -2 }), Some(Position { x: 100, y: -20 }) )
*/
pub type ComponentMap = HashMap<TypeId, ComponentVecHandle>;
pub type ComponentVecHandle = Rc<RefCell<ComponentVec>>;
pub type Component = Box<dyn std::any::Any + 'static>;
pub type SlotVec<T> = Vec<Option<Slot<T>>>;

#[derive(Default)]
pub struct ComponentVec {
	components: SlotVec<Component>,
}

impl ComponentVec {
	pub fn new(components: SlotVec<Component>) -> Self {
		Self { components }
	}

	pub fn add_to(&mut self, entity: Entity, value: Component) -> Result<()> {
		while self.components.len() <= entity.index {
			self.components.push(None);
		}

		let previous_generation = match self.components.get(entity.index) {
			Some(Some(entry)) => entry.generation,
			_ => 0,
		};

		if previous_generation > entity.generation {
			return Err(Box::new(EntityGenerationError { entity }));
		}

		self.components[entity.index] = Some(Slot {
			value,
			generation: entity.generation,
		});

		Ok(())
	}

	pub fn remove_from(&mut self, entity: Entity) {
		if entity.index < self.components.len() {
			self.components[entity.index] = None;
		}
	}

	pub fn get(&self, entity: Entity) -> Option<&Component> {
		if entity.index >= self.components.len() {
			return None;
		}

		match &self.components[entity.index] {
			Some(entry) => {
				if entry.generation == entity.generation {
					Some(&entry.value)
				} else {
					None
				}
			},
			None => None,
		}
	}

	pub fn get_mut(&mut self, entity: Entity) -> Option<&mut Component> {
		if entity.index >= self.components.len() {
			return None;
		}

		match &mut self.components[entity.index] {
			Some(entry) => {
				if entry.generation == entity.generation {
					Some(&mut entry.value)
				} else {
					None
				}
			},
			None => None,
		}
	}
}

impl Deref for ComponentVec {
	type Target = SlotVec<Component>;

	fn deref(&self) -> &Self::Target {
		&self.components
	}
}

impl DerefMut for ComponentVec {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.components
	}
}

pub struct Slot<T> {
	value: T,
	generation: usize,
}

impl<T> Slot<T> {
	pub fn new(value: T) -> Self {
		Self { value, generation: 0 }
	}
}

impl<T> Deref for Slot<T> {
	type Target = T;

	fn deref(&self) -> &Self::Target {
		&self.value
	}
}

impl<T> DerefMut for Slot<T> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.value
	}
}

#[macro_export]
macro_rules! component_vec {
    ($($component:expr),*) => {
        {
            ComponentVec::new(vec![$(Some(crate::component::Slot::new(Box::new($component))),)*])
        }
    }
}

pub fn component_exists<T: 'static>(entity: Entity, components: &ComponentVecHandle) -> bool {
	components
		.borrow()
		.get(entity)
		.and_then(|c| c.downcast_ref::<T>())
		.is_some()
}
