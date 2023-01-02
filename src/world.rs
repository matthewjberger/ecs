use crate::{
	component::{component_exists, Component, ComponentMap, ComponentVec, Entity},
	error::Result,
	resource::ResourceMap,
	vec::{error::HandleNotFoundError, HandleAllocator},
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
	resources: ResourceMap,
	components: ComponentMap,
	allocator: HandleAllocator,
}

impl World {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn resources(&self) -> &ResourceMap {
		&self.resources
	}

	pub fn resources_mut(&mut self) -> &mut ResourceMap {
		&mut self.resources
	}

	pub fn create_entity(&mut self) -> Entity {
		self.create_entities(1)[0]
	}

	pub fn create_entities(&mut self, count: usize) -> Vec<Entity> {
		(0..count).into_iter().map(|_index| self.allocator.allocate()).collect()
	}

	pub fn remove_entity(&mut self, entity: Entity) {
		self.remove_entities(&[entity]);
	}

	pub fn remove_entities(&mut self, entities: &[Entity]) {
		entities.iter().for_each(|entity| self.allocator.deallocate(entity))
	}
	pub fn add_component<T: 'static>(&mut self, entity: Entity, component: T) -> Result<()> {
		self.assign_component::<T>(entity, Some(Box::new(component)))
	}

	pub fn remove_component<T: 'static>(&mut self, entity: Entity) -> Result<()> {
		self.assign_component::<T>(entity, None)
	}

	fn assign_component<T: 'static>(&mut self, entity: Entity, value: Option<Component>) -> Result<()> {
		if !self.allocator.handle_exists(&entity) {
			return Err(Box::new(HandleNotFoundError { handle: entity }) as Box<dyn std::error::Error>);
		}

		let mut components = self
			.components
			.entry(TypeId::of::<T>())
			.or_insert_with(|| Rc::new(RefCell::new(ComponentVec::default())))
			.borrow_mut();

		match value {
			Some(component) => {
				components.insert(entity, component)?;
			},
			None => {
				components.remove(entity);
			},
		}

		Ok(())
	}

	#[must_use]
	pub fn get_component<T: 'static>(&self, entity: Entity) -> Option<Ref<T>> {
		if !self.entity_exists(entity) {
			return None;
		}
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
		if !self.entity_exists(entity) {
			return None;
		}
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

	pub fn entity_exists(&self, entity: Entity) -> bool {
		self.allocator.is_allocated(&entity)
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	use std::ops::DerefMut;

	#[derive(Debug, Default, PartialEq, Copy, Clone)]
	pub struct Position {
		x: f32,
		y: f32,
	}

	#[derive(Debug, Default, PartialEq, Eq, Copy, Clone)]
	pub struct Health {
		value: u8,
	}

	#[test]
	fn entity() -> Result<()> {
		let mut world = World::default();
		let entity = world.create_entity();
		world.add_component(entity, Position::default())?;
		assert!(world.get_component::<Position>(entity).as_deref().is_some());
		world.remove_entity(entity);
		assert_eq!(world.get_component::<Position>(entity).as_deref(), None);
		Ok(())
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
	fn get_component() -> Result<()> {
		let mut world = World::default();
		let entity = world.create_entity();
		world.add_component(entity, Position::default())?;
		assert_eq!(
			world.get_component::<Position>(entity).as_deref(),
			Some(&Position::default())
		);
		Ok(())
	}

	#[test]
	fn get_component_mut() -> Result<()> {
		let mut world = World::default();
		let entity = world.create_entity();
		world.add_component(entity, Position::default())?;
		world.get_component_mut::<Position>(entity).unwrap().deref_mut().x = 10.0;
		assert_eq!(
			world.get_component::<Position>(entity).as_deref(),
			Some(&Position { x: 10.0, y: 0.0 })
		);
		Ok(())
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
