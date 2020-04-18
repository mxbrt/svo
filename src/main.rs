use std::convert::From;
use std::f32;

use cgmath::{Matrix4, Point3, Rad, Vector3};
use minifb::{Key, Window, WindowOptions};

mod render;
mod svo;
mod voxel_grid;

use render::{Renderable, Renderer};
use svo::SparseVoxelOctree;
use voxel_grid::VoxelGrid;

fn main() {
    // load grid data
    let voxel_grid = VoxelGrid::from_csv("data/dragon512.csv".to_string(), 512).unwrap();
    // build svo
    let mut svo = SparseVoxelOctree::from(&voxel_grid);
    let mut svo2 = svo.clone();

    svo2.translate(Vector3::new(2.0, 0.0, 0.0));

    println!(
        "Grid Size: {:?} mb SVO Size: {:?} mb",
        ((voxel_grid.size.pow(3) * std::mem::size_of::<bool>()) / 1024usize.pow(2)),
        (svo.node_pool.len() * std::mem::size_of::<svo::Node>()) / 1024usize.pow(2)
    );
    std::mem::drop(voxel_grid);

    // render
    let fov = 60.0;
    // TODO use winit and OpenGL/Vulkan
    let mut window_options = WindowOptions::default();
    window_options.resize = true;
    let mut window = Window::new("SVO raytracer", 1920, 1080, window_options).unwrap();
    window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));
    let (mut width, mut height) = window.get_size();
    let mut renderer = Renderer::new(width, height, 60.0);
    let mut buffer = vec![0; width as usize * height as usize];
    let mut previous = std::time::SystemTime::now();
    let mut view = Matrix4::look_at_dir(
        Point3::new(0.0, 0.0, -svo.size.x),
        Vector3::new(0.0, 0.0, 1.0),
        Vector3::new(0.0, 1.0, 0.0),
    );

    while window.is_open() && !window.is_key_down(Key::Escape) {
        let elapsed = previous.elapsed().unwrap();
        let delta = elapsed.as_secs() as f32 + elapsed.subsec_nanos() as f32 * 1e-9;
        previous = std::time::SystemTime::now();

        let (cur_width, cur_height) = window.get_size();
        if cur_width != width || cur_height != height {
            width = cur_width;
            height = cur_height;
            buffer = vec![0; width as usize * height as usize];
            renderer.resize(width, height, fov);
            println!("Resized: width {} height {}", width, height);
        }

        let mut camera_velocity = Vector3::new(0.0, 0.0, 0.0);
        window.get_keys().map(|keys| {
            for k in keys {
                match k {
                    Key::A => camera_velocity.x = -1.0,
                    Key::D => camera_velocity.x = 1.0,
                    Key::W => camera_velocity.z = -1.0,
                    Key::S => camera_velocity.z = 1.0,
                    Key::Space => camera_velocity.y = 1.0,
                    Key::LeftCtrl => camera_velocity.y = -1.0,
                    Key::Left => view = view * Matrix4::from_angle_y(Rad(delta)),
                    Key::Right => view = view * Matrix4::from_angle_y(Rad(-delta)),
                    Key::Up => view = view * Matrix4::from_angle_x(Rad(delta)),
                    Key::Down => view = view * Matrix4::from_angle_x(Rad(-delta)),
                    _ => (),
                }
            }
        });

        // camera changes
        camera_velocity *= delta;
        view = view * Matrix4::from_translation(camera_velocity);

        // model changes
        svo.model = svo.model * Matrix4::from_angle_y(Rad(delta));

        let svos = vec![
            &svo as &(dyn Renderable + Sync),
            &svo2 as &(dyn Renderable + Sync),
        ];
        renderer.render(&svos, &mut buffer, &view);
        window.update_with_buffer(&buffer, width, height).unwrap();
    }
}
