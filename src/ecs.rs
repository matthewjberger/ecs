use std::{
	any::TypeId,
	cell::{Ref, RefCell, RefMut},
	collections::hash_map::HashMap,
	ops::{Deref, DerefMut},
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
		write!(f, "Entity '{:?}' does not exist.", self.entity)
	}
}

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
	pub fn add_to(&mut self, entity: Entity, value: Component) {
		while self.components.len() <= entity.index {
			self.components.push(None);
		}

		let previous_generation = match &self.components[entity.index] {
			Some(entry) => entry.generation,
			None => 0,
		};

		if previous_generation > entity.generation {
			// TODO: Add error
			return;
		}

		self.components[entity.index] = Some(Slot {
			value,
			generation: entity.generation,
		});
	}

	pub fn remove_from(&mut self, entity: Entity) {
		println!("Removing!");
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

	pub fn get_all_valid_indices(&self, allocator: &EntityAllocator) -> Vec<Entity> {
		let mut result = Vec::new();

		for index in 0..self.components.len() {
			if let Some(entry) = &self.components[index] {
				let entity = Entity {
					index,
					generation: entry.generation,
				};
				if allocator.in_use(entity) {
					result.push(entity);
				}
			}
		}

		result
	}

	pub fn get_first_valid_entry(&self, allocator: &EntityAllocator) -> Option<(Entity, &Component)> {
		for index in 0..self.components.len() {
			if let Some(entry) = &self.components[index] {
				let entity = Entity {
					index,
					generation: entry.generation,
				};
				if allocator.in_use(entity) {
					return Some((entity, &entry.value));
				}
			}
		}

		None
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

#[derive(Default, Debug, PartialEq, Eq, Copy, Clone)]
pub struct Entity {
	index: usize,
	generation: usize,
}

pub struct Allocation {
	in_use: bool,
	generation: usize,
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

#[derive(Default)]
pub struct EntityAllocator {
	allocations: Vec<Allocation>,
	available_handles: Vec<usize>,
}

impl EntityAllocator {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn allocate(&mut self) -> Entity {
		match self.available_handles.pop() {
			Some(index) => {
				self.allocations[index].generation += 1;
				self.allocations[index].in_use = true;
				Entity {
					index,
					generation: self.allocations[index].generation,
				}
			},
			None => {
				self.allocations.push(Allocation {
					in_use: true,
					generation: 0,
				});

				Entity {
					index: self.allocations.len() - 1,
					generation: 0,
				}
			},
		}
	}

	pub fn deallocate(&mut self, index: Entity) {
		if self.in_use(index) {
			// error
		}
		self.allocations[index.index].in_use = false;
		self.available_handles.push(index.index);
	}

	pub fn in_use(&self, entity: Entity) -> bool {
		self.entity_exists(entity)
			&& self.allocations[entity.index].generation == entity.generation
			&& self.allocations[entity.index].in_use
	}

	pub fn entity_exists(&self, entity: Entity) -> bool {
		entity.index < self.allocations.len()
	}
}

#[macro_export]
macro_rules! component_vec {
    ($($component:expr),*) => {
        {
            ComponentVec {
				components: vec![$(Some(Slot::new(Box::new($component))),)*],
			}
        }
    }
}

fn component_exists<T: 'static>(entity: Entity, components: &ComponentVecHandle) -> bool {
	components
		.borrow()
		.get(entity)
		.and_then(|c| c.downcast_ref::<T>())
		.is_some()
}

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
				components.add_to(entity, component);
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
	use std::ops::DerefMut;

	use super::*;

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
