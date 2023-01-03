use crate::{
	error::Result,
	resource::ResourceMap,
	vec::{error::HandleNotFoundError, GenerationalVec, Handle, HandleAllocator, SlotVec},
};
use std::{
	any::TypeId,
	cell::{Ref, RefCell, RefMut},
	collections::HashMap,
	ops::Deref,
	rc::Rc,
};

/*
   Entities:                    Entity 0                       Entity 1   Entity 2                         Entity 3
   Physics Components   -> Vec( Some(Physics { vel: 3 }),      None,      None,                            Some(Physics { vel: 04 }) )
   Position Components  -> Vec( Some(Position { x: 3, y: 3 }), None,      Some(Position { x: 10, y: -2 }), Some(Position { x: 100, y: -20 }) )
*/
pub type Entity = Handle;
pub type EntityHash = u16;
pub type ComponentMap = HashMap<TypeId, ComponentVecHandle>;
pub type ComponentVecHandle = Rc<RefCell<ComponentVec>>;
pub type Component = Box<dyn std::any::Any + 'static>;
pub type ComponentVec = GenerationalVec<Component>;

impl Default for ComponentVec {
	fn default() -> Self {
		GenerationalVec::new(SlotVec::<Component>::default())
	}
}

#[macro_export]
macro_rules! component_vec {
    ($($component:expr),*) => {
        {
			use std::{rc::Rc, cell::RefCell};
			use $crate::world::ComponentVec;
            Rc::new(RefCell::new(ComponentVec::new(vec![$(Some($crate::vec::Slot::new(Box::new($component), 0)),)*])))
        }
    }
}

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

	pub const fn resources(&self) -> &ResourceMap {
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

	pub fn has_component<T: 'static>(&mut self, entity: Entity) -> bool {
		self.get_component::<T>(entity).is_some()
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
		self.components.get(&TypeId::of::<T>()).and_then(|component_vec| {
			if !entity_has_component(entity, component_vec) {
				return None;
			}
			Some(Ref::map(component_vec.borrow(), |t| {
				t.get(entity)
					.and_then(|component| component.downcast_ref::<T>())
					.unwrap()
			}))
		})
	}

	#[must_use]
	pub fn get_component_mut<T: 'static>(&self, entity: Entity) -> Option<RefMut<T>> {
		if !self.entity_exists(entity) {
			return None;
		}
		self.components.get(&TypeId::of::<T>()).and_then(|component_vec| {
			if !entity_has_component(entity, component_vec) {
				return None;
			}
			Some(RefMut::map(component_vec.borrow_mut(), |t| {
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

	pub fn hash_entity(&self, entity: Entity) -> EntityHash {
		self.components
			.values()
			.enumerate()
			.fold(0, |mut hash, (offset, components)| {
				let value = EntityHash::from(entity_has_component(entity, components));
				hash |= value << offset;
				hash
			})
	}
}

pub fn entity_has_component(entity: Entity, components: &ComponentVecHandle) -> bool {
	components.borrow().get(entity).is_some()
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

	struct Name(String);

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
		assert!(world.has_component::<Position>(entity));
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

	#[test]
	fn entity_hashes() -> Result<()> {
		let mut world = World::default();
		let entity = world.create_entity();

		world.add_component(entity, Position::default())?;
		assert_eq!(0b1, world.hash_entity(entity));

		world.add_component(entity, Health { value: 10 })?;
		assert_eq!(0b11, world.hash_entity(entity));

		Ok(())
	}

	#[test]
	fn component_exists() -> Result<()> {
		let mut entity_allocator = HandleAllocator::new();
		let entity = entity_allocator.allocate();

		let components = component_vec!();
		components
			.borrow_mut()
			.insert(entity, Box::new(Name("Elliot Alderson".to_string())))?;

		assert!(entity_has_component(entity, &components));

		Ok(())
	}
}
