use crate::entity::Entity;

pub type Result<T, E = Box<dyn std::error::Error>> = std::result::Result<T, E>;

#[derive(Debug)]
pub struct EntityGenerationError {
	pub entity: Entity,
}

impl std::error::Error for EntityGenerationError {}

impl std::fmt::Display for EntityGenerationError {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(f, "Entity '{:?}' generation i.", self.entity)
	}
}

#[derive(Debug)]
pub struct EntityNotFoundError {
	pub entity: Entity,
}

impl std::error::Error for EntityNotFoundError {}

impl std::fmt::Display for EntityNotFoundError {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(f, "Entity '{:?}' does not exist.", self.entity)
	}
}
