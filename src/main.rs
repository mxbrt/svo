use std::convert::From;
use std::f32;

use cgmath::{prelude::*, Deg, Euler, Matrix4, Point3, Quaternion, Rad, Vector3};
use minifb::{Key, Window, WindowOptions};
use specs::{prelude::*, Read};

mod light;
mod raycast;
mod render;
mod svo;
mod transform;
mod voxel_grid;

use light::*;
use raycast::*;
use render::*;
use svo::*;
use transform::*;
use voxel_grid::*;

struct RotateSystem;

impl<'a> System<'a> for RotateSystem {
    type SystemData = (
        ReadStorage<'a, RaycastComponent>,
        WriteStorage<'a, TransformComponent>,
    );

    fn run(&mut self, (raycastable, mut transform): Self::SystemData) {
        let rotation = Quaternion::from_angle_y(Deg(1.0));
        for (_, transform) in (&raycastable, &mut transform).join() {
            transform.rotation = transform.rotation * rotation;
            transform.inv_model = Matrix4::from(transform.rotation)
                * Matrix4::from_translation(-transform.position.to_vec())
                * Matrix4::from_scale(1.0).invert().unwrap(); // TODO
        }
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        println!("Need path to model csv.");
        std::process::exit(1);
    }
    let model_path = &args[1];
    let size_start = model_path.find("_").unwrap() + 1;
    let size_end = model_path.find(".").unwrap();
    let model_size_str = &model_path[size_start..size_end];
    let model_size = model_size_str.parse::<usize>().unwrap();

    // load grid data
    let voxel_grid = VoxelGrid::from_csv(model_path.to_string(), model_size).unwrap();
    // build svo
    let svo = SparseVoxelOctree::from(&voxel_grid);

    println!(
        "Grid Size: {:?} mb SVO Size: {:?} mb",
        ((voxel_grid.size.pow(3) * std::mem::size_of::<bool>()) / 1024usize.pow(2)),
        (svo.node_pool.len() * std::mem::size_of::<svo::Node>()) / 1024usize.pow(2)
    );
    std::mem::drop(voxel_grid);

    let mut world = World::new();
    world.register::<RaycastComponent>();
    world.register::<TransformComponent>();
    world.register::<LightComponent>();

    let svo_arc = std::sync::Arc::new(svo);

    for x in 1..4 {
        for y in 1..4 {
            world
                .create_entity()
                .with(RaycastComponent(svo_arc.clone()))
                .with(TransformComponent {
                    position: Point3::new(x as f32 * 2.0, 0.0, y as f32 * 2.0),
                    rotation: Quaternion::from_angle_y(Deg(0.0)),
                    scale: Vector3::new(1.0, 1.0, 1.0),
                    inv_model: Matrix4::from_translation(Vector3::zero()),
                })
                .build();
        }
    }
    world
        .create_entity()
        .with(LightComponent {
            intensity: 3.0,
            color: Vector3::new(1.0, 1.0, 1.0),
            light_type: LightType::DirectionalLight,
        })
        .with(TransformComponent {
            position: Point3::origin(),
            rotation: Quaternion::from(Euler {
                x: Deg(30.0),
                y: Deg(90.0),
                z: Deg(-90.0),
            }),
            scale: Vector3::new(1.0, 1.0, 1.0),
            inv_model: Matrix4::identity(),
        })
        .build();
    world
        .create_entity()
        .with(LightComponent {
            intensity: 500.0,
            color: Vector3::new(0.2, 0.1, 0.8),
            light_type: LightType::SphericalLight,
        })
        .with(TransformComponent {
            position: Point3::new(2.0, 1.0, 1.0),
            rotation: Quaternion::zero(),
            scale: Vector3::new(1.0, 1.0, 1.0),
            inv_model: Matrix4::from_translation(Vector3::zero()),
        })
        .build();

    // render
    let fov = 60.0;
    // TODO use winit and OpenGL/Vulkan
    let mut window_options = WindowOptions::default();
    window_options.resize = true;
    let mut window = Window::new("SVO raytracer", 1920, 1080, window_options).unwrap();
    window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));
    let (mut width, mut height) = window.get_size();

    world.insert(ScreenBuffer(vec![0; width as usize * height as usize]));

    let mut dispatcher = DispatcherBuilder::new()
        .with(RotateSystem, "rotate_system", &[])
        .with(
            RenderSystem::new(width, height, fov),
            "render_system",
            &["rotate_system"],
        )
        .build();

    let mut previous = std::time::SystemTime::now();
    let mut view = ViewMatrix::default();
    let mut camera_position = Point3::new(0.0, 1.0, -3.0);
    let mut camera_rotation = Matrix4::<f32>::identity();
    while window.is_open() && !window.is_key_down(Key::Escape) {
        let elapsed = previous.elapsed().unwrap();
        let delta = elapsed.as_secs() as f32 + elapsed.subsec_nanos() as f32 * 1e-9;
        previous = std::time::SystemTime::now();

        let (cur_width, cur_height) = window.get_size();
        if cur_width != width || cur_height != height {
            width = cur_width;
            height = cur_height;
            world.insert(ScreenBuffer(vec![0; width as usize * height as usize]));
            dispatcher = DispatcherBuilder::new()
                .with(RotateSystem, "rotate_system", &[])
                .with(
                    RenderSystem::new(width, height, fov),
                    "render_system",
                    &["rotate_system"],
                )
                .build();
            println!("Resized: width {} height {}", width, height);
        }

        let mut camera_velocity = Vector3::zero();
        window.get_keys().map(|keys| {
            for k in keys {
                match k {
                    Key::A => camera_velocity.x = -1.0,
                    Key::D => camera_velocity.x = 1.0,
                    Key::W => camera_velocity.z = 1.0,
                    Key::S => camera_velocity.z = -1.0,
                    Key::Space => camera_velocity.y = 1.0,
                    Key::LeftCtrl => camera_velocity.y = -1.0,
                    Key::Left => {
                        camera_rotation = camera_rotation * Matrix4::from_angle_y(Rad(-delta))
                    }
                    Key::Right => {
                        camera_rotation = camera_rotation * Matrix4::from_angle_y(Rad(delta))
                    }
                    Key::Up => {
                        camera_rotation = camera_rotation * Matrix4::from_angle_x(Rad(delta))
                    }
                    Key::Down => {
                        camera_rotation = camera_rotation * Matrix4::from_angle_x(Rad(-delta))
                    }
                    _ => (),
                }
            }
        });
        camera_position += delta * camera_rotation.transform_vector(camera_velocity);
        view.0 = Matrix4::look_at_dir(
            camera_position,
            camera_rotation.transform_vector(Vector3::new(0.0, 0.0, 1.0)),
            Vector3::new(0.0, 1.0, 0.0),
        );
        world.insert(view.clone());
        dispatcher.dispatch(&mut world);

        let buffer: Read<ScreenBuffer> = world.system_data();
        window.update_with_buffer(&buffer.0, width, height).unwrap();
    }
}
