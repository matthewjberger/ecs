use crate::{
	entity::Entity,
	vec::{GenerationalVec, SlotVec},
};
use std::{any::TypeId, cell::RefCell, collections::hash_map::HashMap, rc::Rc};

/*
   Entities:                    Entity 0                       Entity 1   Entity 2                         Entity 3
   Physics Components   -> Vec( Some(Physics { vel: 3 }),      None,      Some(Physics { vel: 10 }),       Some(Physics { vel: 04 }) )
   Position Components  -> Vec( Some(Position { x: 3, y: 3 }), None,      Some(Position { x: 10, y: -2 }), Some(Position { x: 100, y: -20 }) )
*/
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
            ComponentVec::new(vec![$(Some($crate::vec::Slot::new(Box::new($component), 0)),)*])
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

// #[cfg(test)]
// mod tests {
// 	use super::*;
// 	use crate::component_vec;
// 	use std::{collections::HashMap, ops::DerefMut};

// 	#[test]
// 	fn asdf() {}
// }
