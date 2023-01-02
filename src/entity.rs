#[derive(Default, Debug, PartialEq, Eq, Copy, Clone)]
pub struct Entity {
	pub index: usize,
	pub generation: usize,
}

pub struct Allocation {
	in_use: bool,
	generation: usize,
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
