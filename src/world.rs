use crate::{
	component::{component_exists, Component, ComponentMap, ComponentVec},
	entity::{Entity, EntityAllocator, EntityNotFoundError},
	error::Result,
};
use std::{
	any::TypeId,
	cell::{Ref, RefCell, RefMut},
	ops::Deref,
	rc::Rc,
};

#[macro_export]
macro_rules! zip{
    ($x: expr) => ($x);
    ($x: expr, $($y: expr), +) => ($x.zip(zip!($($y), +)))
}

#[derive(Default)]
pub struct World {
	components: ComponentMap,
	entity_allocator: EntityAllocator,
}

impl World {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn create_entity(&mut self) -> Entity {
		self.create_entities(1)[0]
	}

	pub fn create_entities(&mut self, count: usize) -> Vec<Entity> {
		(0..count)
			.into_iter()
			.map(|_index| self.entity_allocator.allocate())
			.collect()
	}

	pub fn add_component<T: 'static>(&mut self, entity: Entity, component: T) -> Result<()> {
		self.assign_component::<T>(entity, Some(Box::new(component)))
	}

	pub fn remove_component<T: 'static>(&mut self, entity: Entity) -> Result<()> {
		self.assign_component::<T>(entity, None)
	}

	fn assign_component<T: 'static>(&mut self, entity: Entity, value: Option<Component>) -> Result<()> {
		if !self.entity_allocator.entity_exists(entity) {
			return Err(Box::new(EntityNotFoundError { entity }) as Box<dyn std::error::Error>);
		}

		let mut components = self
			.components
			.entry(TypeId::of::<T>())
			.or_insert_with(|| Rc::new(RefCell::new(ComponentVec::default())))
			.borrow_mut();

		match value {
			Some(component) => {
				components.add_to(entity, component)?;
			},
			None => {
				components.remove_from(entity);
			},
		}

		Ok(())
	}

	#[must_use]
	pub fn get_component<T: 'static>(&self, entity: Entity) -> Option<Ref<T>> {
		self.components.get(&TypeId::of::<T>()).and_then(|c| {
			if !component_exists::<T>(entity, c) {
				return None;
			}
			Some(Ref::map(c.borrow(), |t| {
				t.get(entity).and_then(|c| c.downcast_ref::<T>()).unwrap()
			}))
		})
	}

	#[must_use]
	pub fn get_component_mut<T: 'static>(&self, entity: Entity) -> Option<RefMut<T>> {
		self.components.get(&TypeId::of::<T>()).and_then(|c| {
			if !component_exists::<T>(entity, c) {
				return None;
			}
			Some(RefMut::map(c.borrow_mut(), |t| {
				t.get_mut(entity).and_then(|c| c.downcast_mut::<T>()).unwrap()
			}))
		})
	}

	pub fn get_component_vec<T: 'static>(&self) -> Ref<ComponentVec> {
		self.components.get(&TypeId::of::<T>()).unwrap().deref().borrow()
	}

	pub fn get_component_vec_mut<T: 'static>(&self) -> RefMut<ComponentVec> {
		self.components.get(&TypeId::of::<T>()).unwrap().deref().borrow_mut()
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::component_vec;
	use std::{collections::HashMap, ops::DerefMut};

	#[derive(Debug, Default, PartialEq, Copy, Clone)]
	pub struct Position {
		x: f32,
		y: f32,
	}

	#[derive(Debug, Default, PartialEq, Eq, Copy, Clone)]
	pub struct Health {
		value: u8,
	}

	fn create_test_world() -> World {
		World {
			components: HashMap::from([
				(
					TypeId::of::<Position>(),
					Rc::new(RefCell::new(component_vec!(Position::default()))),
				),
				(
					TypeId::of::<Health>(),
					Rc::new(RefCell::new(component_vec!(Health::default()))),
				),
			]),
			..Default::default()
		}
	}

	#[test]
	fn add_component() -> Result<()> {
		let mut world = World::default();
		let entity = world.create_entity();
		assert_eq!(world.get_component::<Position>(entity).as_deref(), None);
		assert_eq!(world.get_component::<Health>(entity).as_deref(), None);
		world.add_component(entity, Position::default())?;
		world.add_component(entity, Health { value: 10 })?;
		world.get_component_mut::<Health>(entity).unwrap().value = 0;
		assert_eq!(*world.get_component::<Position>(entity).unwrap(), Position::default());
		assert_eq!(*world.get_component::<Health>(entity).unwrap(), Health { value: 0 });
		Ok(())
	}

	#[test]
	fn remove_component() -> Result<()> {
		let mut world = World::new();
		let entity = world.create_entity();
		let position = Position { x: 10.0, y: 10.0 };
		world.add_component(entity, position)?;
		assert_eq!(world.get_component::<Position>(entity).as_deref(), Some(&position));
		world.remove_component::<Position>(entity)?;
		assert!(world.get_component::<Position>(entity).is_none());
		Ok(())
	}

	#[test]
	fn get_component() {
		let entity = Entity::default();
		assert_eq!(
			*create_test_world().get_component::<Position>(entity).unwrap(),
			Position::default()
		);
	}

	#[test]
	fn get_component_mut() {
		let world = create_test_world();
		let entity = Entity::default();
		world.get_component_mut::<Position>(entity).unwrap().deref_mut().x = 10.0;
		let actual = world.get_component::<Position>(entity).unwrap();
		assert_eq!(*actual, Position { x: 10.0, y: 0.0 });
	}

	#[test]
	fn system() -> Result<()> {
		let mut world = World::default();
		let entity = world.create_entity();
		world.add_component(entity, Position::default())?;
		world.add_component(entity, Health { value: 10 })?;

		// TODO: Abstract system creation with macros/generics
		zip!(
			world.get_component_vec_mut::<Position>().iter_mut(),
			world.get_component_vec::<Health>().iter()
		)
		.enumerate()
		.filter_map(|(entity, (position, health))| {
			let position = position.as_mut().and_then(|p| p.downcast_mut::<Position>());
			let health = health.as_ref().and_then(|h| h.downcast_ref::<Health>());
			match (position, health) {
				(Some(position), Some(health)) => Some((entity, (position, health))),
				_ => None,
			}
		})
		.into_iter()
		.for_each(|(_entity, (position, _health))| {
			position.x = 10.0;
			position.y = 10.0;
		});

		assert_eq!(
			*world.get_component::<Position>(entity).unwrap(),
			Position { x: 10.0, y: 10.0 }
		);

		Ok(())
	}
}
