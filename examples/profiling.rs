use anyhow::Result;
use parsecs::{izip, system, world::World};
use std::ops::DerefMut;

fn main() -> Result<()> {
	println!("* Insertion");
	profile_insertion();

	println!("* Removal");
	profile_removal();

	println!("* Reinsertion");
	profile_reinsertion();

	println!("* Component Insertion");
	profile_component_insertion();

	println!("* Component Removal");
	profile_component_removal();

	println!("* Component Mutation");
	profile_component_mutation();

	println!("* Complex Entities (3+ components)");
	profile_complex_entities();

	println!("* Complex Entity System");
	profile_complex_entity_system();

	Ok(())
}

fn profile_insertion() {
	let (results, duration) = profile!({
		let mut world = World::new();
		let results = profile_n!(100, world.create_entities(1_000_000));
		results
	});
	let average_duration = results
		.iter()
		.map(|(_entities, duration)| duration.as_millis() as f32)
		.sum::<f32>()
		/ results.len() as f32;
	println!(
		"  - Creating 1 million entities: {}ms (average of {} runs).",
		average_duration,
		results.len()
	);
	println!("  - Creating 100 million entities: {}ms.", duration.as_millis());
}

fn profile_removal() {
	let mut world = World::new();
	let number_of_entities = 1_000_000;
	let entities = world.create_entities(number_of_entities);
	let (_, duration) = profile!(world.remove_entities(&entities));
	println!("  - Removing 1 million entities: {:?}.", duration);
}

fn profile_reinsertion() {
	let mut world = World::new();
	let number_of_entities = 1_000_000;
	let entities = world.create_entities(number_of_entities);
	world.remove_entities(&entities);
	let (_, duration) = profile!(world.create_entities(number_of_entities));
	println!("  - Reinserting 1 million entities: {:?}.", duration);
}

#[derive(Default)]
struct Position(f32, f32);

#[derive(Default)]
struct Health(u8);

#[derive(Default)]
struct Name(String);

fn profile_component_insertion() {
	let mut world = World::new();
	let number_of_entities = 1_000_000;
	let entities = world.create_entities(number_of_entities);
	let (_, duration) = profile!({
		for entity in entities.iter() {
			world.add_component(*entity, Position::default()).unwrap();
		}
	});
	println!("  - Inserting 1 million components: {:?}.", duration);
}

fn profile_component_removal() {
	let mut world = World::new();
	let number_of_entities = 1_000_000;
	let entities = world.create_entities(number_of_entities);
	for entity in entities.iter() {
		world.add_component(*entity, Position::default()).unwrap();
	}
	let (_, duration) = profile!({
		for entity in entities.iter() {
			world.remove_component::<Position>(*entity).unwrap();
		}
	});
	println!("  - Removing 1 million components: {:?}.", duration);
}

fn profile_component_mutation() {
	let mut world = World::new();
	let number_of_entities = 1_000_000;
	let entities = world.create_entities(number_of_entities);
	for entity in entities.iter() {
		world.add_component(*entity, Position::default()).unwrap();
	}
	let (_, duration) = profile!({
		for entity in entities.iter() {
			world.get_component_mut::<Position>(*entity).unwrap().deref_mut().0 = 10.0;
		}
	});
	println!("  - Updating 1 million components: {:?}.", duration);
}

fn profile_complex_entities() {
	let mut world = World::new();
	let number_of_entities = 1_000_000;
	let entities = world.create_entities(number_of_entities);
	for entity in entities.iter() {
		world.add_component(*entity, Position::default()).unwrap();
		world.add_component(*entity, Health::default()).unwrap();
		world
			.add_component(*entity, Name("Test Component".to_string()))
			.unwrap();
	}
	let (_, duration) = profile!({
		for entity in entities.iter() {
			world.get_component_mut::<Position>(*entity).unwrap().deref_mut().0 = 10.0;
			world.get_component_mut::<Health>(*entity).unwrap().deref_mut().0 = 4;
			world.get_component_mut::<Name>(*entity).unwrap().deref_mut().0 = "Renamed".to_string();
		}
	});
	println!("  - Updating 1 million complex entities: {:?}.", duration);
}

fn profile_complex_entity_system() {
	let mut world = World::new();
	let number_of_entities = 1_000_000;
	let entities = world.create_entities(number_of_entities);
	for entity in entities.iter() {
		world.add_component(*entity, Position::default()).unwrap();
		world.add_component(*entity, Health::default()).unwrap();
		world
			.add_component(*entity, Name("Test Component".to_string()))
			.unwrap();
	}
	let (_, duration) = profile!(translation_system(&mut world));
	println!("  - Updating 1 million complex entities with a system: {:?}.", duration);
}

// Translate only named entities
system!(translation_system, (), (position: Position, name: Name, health: Health) {
	position.0 = 10.0;
	health.0 = 4;
	name.0 = "Renamed".to_string();
});

#[macro_export]
macro_rules! profile {
    ($($actions:tt)*) => {
        {
            let start = std::time::Instant::now();
            let result = $($actions)*;
            let duration = start.elapsed();
            (result, duration)
        }
    }
}

#[macro_export]
macro_rules! profile_n {
	($n:tt, $($actions:tt)*) => {
		(0..$n).map(|_| {
			profile!($($actions)*)
		})
        .collect::<Vec<_>>()
	}
}
