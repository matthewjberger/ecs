use anyhow::Result;
use kiss3d::{camera::ArcBall, light::Light, scene::SceneNode, window::Window};
use nalgebra::{Point3, UnitQuaternion, Vector3};
use parsecs::{system, world::World};
use rand::Rng;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

fn main() -> Result<()> {
	let mut window = Window::new("Entity-Component-System Architecture Demo");
	window.set_light(Light::StickToCamera);

	let mut world = create_world(&mut window);

	let mut arc_ball = {
		let eye = Point3::new(10.0, 10.0, 10.0);
		let at = Point3::origin();
		ArcBall::new(eye, at)
	};

	let start = Instant::now();
	let color_system = ColorSystem::new();
	while window.render_with_camera(&mut arc_ball) {
		rotation_system(0.014, &mut world)?;
		let now = Instant::now();
		let elapsed = now.duration_since(start).as_secs_f32();
		scaling_system(elapsed, &mut world)?;
		color_system.run(&mut world)?;
	}

	Ok(())
}

fn create_world(window: &mut Window) -> World {
	let mut rng = rand::thread_rng();
	let mut world = World::new();
	let entities = world.create_entities(10);
	for entity in entities {
		let mut node = window.add_cube(1.0, 1.0, 1.0);
		node.set_color(0.0, 1.0, 0.0);
		node.set_visible(true);
		node.set_local_translation([rng.gen_range(-5.0..5.0), rng.gen_range(-5.0..5.0), rng.gen_range(-5.0..5.0)].into());
		world.add_component(entity, node).unwrap();
	}
	world
}

// Using the `system!` macro
system!(rotation_system, [_resources, _entity], (value: f32), (node: SceneNode) -> Result<()> {
	node.prepend_to_local_rotation(&UnitQuaternion::from_axis_angle(&Vector3::y_axis(), value));
	Ok(())
});

// Using a plain function
pub fn scaling_system(value: f32, world: &mut World) -> Result<()> {
	world
		.get_component_vec_mut::<SceneNode>()
		.unwrap_or_else(|| panic!("System accessed an unregistered component type: {:?}", stringify!(SceneNode)))
		.iter_mut()
		.enumerate()
		.filter_map(|(entity, node)| match node {
			Some(node) => {
				let node = node.downcast_mut::<SceneNode>().unwrap();
				Some((world.resources().clone(), entity, node))
			},
			_ => None,
		})
		.try_for_each(|(_resources, _entity, node)| {
			let factor = value.sin().max(0.2);
			node.set_local_scale(factor, factor, factor);
			Ok(())
		})
}

// Encapsulating the system in a struct
// to persist system state across calls
struct ColorSystem {
	start_time: Duration,
}

impl ColorSystem {
	pub fn new() -> Self {
		Self {
			start_time: SystemTime::now().duration_since(UNIX_EPOCH).unwrap(),
		}
	}

	system!(run, [_resources, _entity], (self: &Self), (node: SceneNode) -> Result<()> {
		let time = (SystemTime::now().duration_since(UNIX_EPOCH).unwrap() - self.start_time).as_secs_f32();
		node.set_color(time.sin(), time.cos(), 0.5);
		Ok(())
	});
}
