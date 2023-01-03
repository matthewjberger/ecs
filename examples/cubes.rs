use std::ops::DerefMut;

use anyhow::Result;
use kiss3d::{camera::ArcBall, light::Light, scene::SceneNode, window::Window};
use nalgebra::{Point3, UnitQuaternion, Vector3};
use parsecs::{system, world::World, zip};
use rand::Rng;

pub struct Render(pub SceneNode);

fn main() -> Result<()> {
	let mut window = Window::new("Entity-Component-System Architecture Demo");
	window.set_light(Light::StickToCamera);

	let mut world = create_world(&mut window);

	let mut arc_ball = create_camera();

	while window.render_with_camera(&mut arc_ball) {
		rotation_system(&mut world, 0.014);
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
		node.set_local_translation(
			[
				rng.gen_range(-5.0..5.0),
				rng.gen_range(-5.0..5.0),
				rng.gen_range(-5.0..5.0),
			]
			.into(),
		);
		world.add_component(entity, Render(node)).unwrap();
	}
	world
}

system!(rotation_system, (value: f32), (render: Render) {
	render.0.deref_mut().0.prepend_to_local_rotation(&UnitQuaternion::from_axis_angle(&Vector3::y_axis(), value))
});

fn create_camera() -> ArcBall {
	let eye = Point3::new(10.0, 10.0, 10.0);
	let at = Point3::origin();
	ArcBall::new(eye, at)
}
