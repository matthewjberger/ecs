use std::any::{Any, TypeId};
use std::collections::HashMap;

#[derive(Default)]
pub struct ResourceMap {
	data: HashMap<TypeId, Box<dyn Any + 'static>>,
}

impl ResourceMap {
	pub fn new() -> Self {
		Self::default()
	}
}

impl ResourceMap {
	/// Retrieve the value stored in the map for the type `T`, if it exists.
	pub fn get<T: 'static>(&self) -> Option<&T> {
		self.data.get(&TypeId::of::<T>()).and_then(|any| any.downcast_ref())
	}

	/// Retrieve a mutable reference to the value stored in the map for the type `T`, if it exists.
	pub fn get_mut<T: 'static>(&mut self) -> Option<&mut T> {
		self.data.get_mut(&TypeId::of::<T>()).and_then(|any| any.downcast_mut())
	}

	/// Set the value contained in the map for the type `T`.
	/// This will override any previous value stored.
	pub fn add<T: 'static>(&mut self, value: T) {
		self.data
			.insert(TypeId::of::<T>(), Box::new(value) as Box<dyn Any + 'static>);
	}

	/// Remove the value for the type `T` if it existed.
	pub fn remove<T: 'static>(&mut self) {
		self.data.remove(&TypeId::of::<T>());
	}
}

#[cfg(test)]
mod tests {
	use super::ResourceMap;

	struct DeltaTime(f64);

	struct Input {
		esc_key_down: bool,
	}

	#[test]
	fn anymap() {
		let delta_time = 0.01;
		let mut anymap = ResourceMap::new();
		anymap.add(DeltaTime(delta_time));
		assert_eq!(anymap.get::<DeltaTime>().unwrap().0, delta_time);

		let delta_time = 0.02;
		if let Some(entry) = anymap.get_mut::<DeltaTime>() {
			entry.0 = delta_time;
		}
		assert_eq!(anymap.get::<DeltaTime>().unwrap().0, delta_time);

		anymap.add(Input { esc_key_down: false });
		assert_eq!(anymap.get::<Input>().unwrap().esc_key_down, false);

		anymap.remove::<DeltaTime>();
		assert!(anymap.get::<DeltaTime>().is_none());
	}
}
