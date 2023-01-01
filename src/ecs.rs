use std::{
	any::{Any, TypeId},
	cell::{Ref, RefCell, RefMut},
	collections::hash_map::HashMap,
	ops::Deref,
	rc::Rc,
};

pub type Result<T, E = Box<dyn std::error::Error>> = std::result::Result<T, E>;

#[derive(Debug)]
pub struct EntityNotFoundError {
	entity: Entity,
}

impl std::error::Error for EntityNotFoundError {}

impl std::fmt::Display for EntityNotFoundError {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(f, "Entity '{}' does not exist.", self.entity)
	}
}

// Entities:                    Entity 0                       Entity 1   Entity 2
// Entity 3 Physics Components   -> Vec( Some(Physics { vel: 3 }),      None,      Some(Physics {
// vel: 10 }),       Some(Physics { vel: 04 }) ) Position Components  -> Vec( Some(Position { x: 3,
// y: 3 }), None,      Some(Position { x: 10, y: -2 }), Some(Position { x: 100, y: -20 }) )
pub type ComponentMap = HashMap<TypeId, ComponentVecHandle>;
pub type ComponentVecHandle = Rc<RefCell<ComponentVec>>;
pub type ComponentVec = Vec<Option<Box<dyn std::any::Any + 'static>>>;
pub type Entity = usize;

#[macro_export]
macro_rules! component_vec {
    ($($component:expr),*) => {
        {
            let components: Vec<Option<Box<dyn std::any::Any + 'static>>> = vec![$(Some(Box::new($component)),)*];
            components
        }
    }
}

fn component_exists<T: 'static>(entity: Entity, components: &ComponentVecHandle) -> bool {
	components
		.borrow()
		.get(entity)
		.and_then(|c| c.as_ref().and_then(|c| c.downcast_ref::<T>()))
		.is_some()
}

#[derive(Default, Debug)]
pub struct World {
	number_of_entities: usize,
	components: ComponentMap,
}

impl World {
	pub fn create_entity(&mut self) -> Entity {
		let entity = self.number_of_entities;
		self.number_of_entities += 1;
		self.grow_vectors(self.number_of_entities);
		entity
	}

	pub fn add_component<T: 'static>(&mut self, entity: Entity, component: T) -> Result<()> {
		if entity >= self.number_of_entities {
			return Err(Box::new(EntityNotFoundError { entity }) as Box<dyn std::error::Error>);
		}
		let mut components = self
			.components
			.entry(TypeId::of::<T>())
			.or_insert_with(|| Rc::new(RefCell::new(Vec::new())))
			.borrow_mut();

		match components.get_mut(entity) {
			Some(c) => {
				*c = Some(Box::new(component));
			},
			None => {
				components.insert(entity, Some(Box::new(component)));
			},
		};
		Ok(())
	}

	#[must_use]
	pub fn get_component<T: 'static>(&self, entity: Entity) -> Option<Ref<T>> {
		self.components.get(&TypeId::of::<T>()).and_then(|c| {
			if !component_exists::<T>(entity, c) {
				return None;
			}
			Some(Ref::map(c.borrow(), |t| {
				t.get(entity)
					.and_then(|c| c.as_ref().and_then(|c| c.downcast_ref::<T>()))
					.unwrap()
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
				t.get_mut(entity)
					.and_then(|c| c.as_mut().and_then(|c| c.downcast_mut::<T>()))
					.unwrap()
			}))
		})
	}

	#[allow(dead_code)]
	fn get_component_vec<T: 'static>(&self) -> RefMut<Vec<Option<Box<dyn Any>>>> {
		self.components.get(&TypeId::of::<T>()).unwrap().deref().borrow_mut()
	}

	fn grow_vectors(&mut self, capacity: usize) {
		self.components.values_mut().for_each(|c| {
			let mut components = c.borrow_mut();
			while components.len() < capacity {
				components.push(None);
			}
		});
	}
}

#[cfg(test)]
mod tests {
	use std::ops::DerefMut;

	use super::*;

	#[derive(Default, Debug, PartialEq)]
	pub struct Position {
		x: f32,
		y: f32,
	}

	#[derive(Default, Debug, PartialEq, Eq)]
	pub struct Health {
		value: u8,
	}

	fn create_test_world() -> World {
		World {
			number_of_entities: 2,
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
		}
	}

	#[test]
	fn add_component() -> Result<()> {
		let mut world = World::default();
		let entity = world.create_entity();
		world.add_component(entity, Position::default())?;
		world.add_component(entity, Health { value: 10 })?;
		world.get_component_mut::<Health>(entity).unwrap().value = 0;
		assert_eq!(*world.get_component::<Position>(entity).unwrap(), Position::default());
		assert_eq!(*world.get_component::<Health>(entity).unwrap(), Health { value: 0 });
		Ok(())
	}

	#[test]
	fn get_component() {
		assert_eq!(
			*create_test_world().get_component::<Position>(0).unwrap(),
			Position::default()
		);
	}

	#[test]
	fn get_component_mut() {
		let world = create_test_world();
		world.get_component_mut::<Position>(0).unwrap().deref_mut().x = 10.0;
		let actual = world.get_component::<Position>(0).unwrap();
		assert_eq!(*actual, Position { x: 10.0, y: 0.0 });
	}

	#[test]
	fn system() -> Result<()> {
		let mut world = World::default();
		let entity = world.create_entity();
		world.add_component(entity, Position::default())?;
		world.add_component(entity, Health { value: 10 })?;

		{
			let mut p = world.get_component_vec::<Position>();
			let mut h = world.get_component_vec::<Health>();

			for (position, health) in p.iter_mut().zip(h.iter_mut()).filter_map(|(position, health)| {
				let position = position.as_mut().and_then(|p| p.downcast_mut::<Position>());
				let health = health.as_mut().and_then(|h| h.downcast_mut::<Health>());
				match (position, health) {
					(Some(position), Some(health)) => Some((position, health)),
					_ => None,
				}
			}) {
				position.x = 10.0;
				position.y = 10.0;
				health.value = 0;
			}
		}

		assert_eq!(
			*world.get_component::<Position>(entity).unwrap(),
			Position { x: 10.0, y: 10.0 }
		);

		Ok(())
	}
}
