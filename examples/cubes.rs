use anyhow::Result;
use ecs::{zip, World};
use kiss3d::{camera::ArcBall, light::Light, scene::SceneNode, window::Window};
use nalgebra::{Point3, UnitQuaternion, Vector3};
use rand::Rng;

pub struct Render(pub SceneNode);

fn main() -> Result<()> {
	let mut window = Window::new("ECS Demo");
	window.set_light(Light::StickToCamera);

	let mut world = World::new();

	let mut rng = rand::thread_rng();
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

	let eye = Point3::new(10.0, 10.0, 10.0);
	let at = Point3::origin();
	let mut arc_ball = ArcBall::new(eye, at);

	while window.render_with_camera(&mut arc_ball) {
		let rotation = UnitQuaternion::from_axis_angle(&Vector3::y_axis(), 0.014);

		zip!(world.get_component_vec_mut::<Render>().iter_mut())
			.enumerate()
			.filter_map(|(entity, render)| {
				let render = render.as_mut().and_then(|p| p.downcast_mut::<Render>());
				match render {
					Some(render) => Some((entity, render)),
					_ => None,
				}
			})
			.into_iter()
			.for_each(|(_entity, render)| render.0.prepend_to_local_rotation(&rotation));
	}

	Ok(())
}
