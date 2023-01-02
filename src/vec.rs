use crate::error::Result;
use std::ops::{Deref, DerefMut};

pub type SlotVec<T> = Vec<Option<Slot<T>>>;

#[derive(Debug)]
pub struct GenerationError {
	pub handle: Handle,
}

impl std::error::Error for GenerationError {}

impl std::fmt::Display for GenerationError {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(f, "Entity '{:?}' generation i.", self.handle)
	}
}

#[derive(Default, Debug, PartialEq, Eq, Copy, Clone)]
pub struct Handle {
	pub index: usize,
	pub generation: usize,
}

pub struct GenerationalVec<T> {
	elements: SlotVec<T>,
}

impl<T> GenerationalVec<T> {
	pub fn new(elements: SlotVec<T>) -> Self {
		Self { elements }
	}

	pub fn add_to(&mut self, handle: Handle, value: T) -> Result<()> {
		while self.elements.len() <= handle.index {
			self.elements.push(None);
		}

		let previous_generation = match self.elements.get(handle.index) {
			Some(Some(entry)) => entry.generation,
			_ => 0,
		};

		if previous_generation > handle.generation {
			return Err(Box::new(GenerationError { handle }));
		}

		self.elements[handle.index] = Some(Slot {
			value,
			generation: handle.generation,
		});

		Ok(())
	}

	pub fn remove_from(&mut self, handle: Handle) {
		if handle.index < self.elements.len() {
			self.elements[handle.index] = None;
		}
	}

	pub fn get(&self, handle: Handle) -> Option<&T> {
		if handle.index >= self.elements.len() {
			return None;
		}
		self.elements[handle.index].as_ref().and_then(|entry| {
			if entry.generation == handle.generation {
				Some(&entry.value)
			} else {
				None
			}
		})
	}

	pub fn get_mut(&mut self, handle: Handle) -> Option<&mut T> {
		if handle.index >= self.elements.len() {
			return None;
		}

		match &mut self.elements[handle.index] {
			Some(entry) => {
				if entry.generation == handle.generation {
					Some(&mut entry.value)
				} else {
					None
				}
			},
			None => None,
		}
	}
}

impl<T> Deref for GenerationalVec<T> {
	type Target = SlotVec<T>;

	fn deref(&self) -> &Self::Target {
		&self.elements
	}
}

impl<T> DerefMut for GenerationalVec<T> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.elements
	}
}

pub struct Slot<T> {
	value: T,
	generation: usize,
}

impl<T> Slot<T> {
	pub const fn new(value: T, generation: usize) -> Self {
		Self { value, generation }
	}

	pub const fn generation(&self) -> usize {
		self.generation
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
