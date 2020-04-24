use std::convert::From;
use std::f32;

use cgmath::{prelude::*, Deg, Matrix4, Vector3};

mod camera;
mod raycast;
mod raytracer;
mod shader;
mod svo;
mod ui;
mod util;
mod voxel_grid;
mod window;

use camera::Camera;
use raytracer::Raytracer;
use svo::SparseVoxelOctree;
use ui::ImguiContext;
use util::clamp;
use voxel_grid::VoxelGrid;
use window::{RenderContext, WindowContext};

use winit::{
    event::{DeviceEvent, ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        println!("Need path to model csv.");
        std::process::exit(1);
    }
    let model_path = &args[1];

    // build svo
    let svo = {
        let voxel_grid = VoxelGrid::from_csv(model_path.to_string()).unwrap();
        SparseVoxelOctree::from(&voxel_grid)
    };

    // render
    let mut camera = Camera::new();

    let event_loop = EventLoop::new();
    let mut window = WindowContext::new("SVO Renderer", &event_loop);
    let mut previous = std::time::Instant::now();

    let mut render_context = RenderContext::new(&window, 1280, 720);
    let mut imgui = ImguiContext::new(&mut window);
    // renderer initialization
    let mut width = render_context.swap_chain_descriptor.width;
    let mut height = render_context.swap_chain_descriptor.height;
    let raytracer = Raytracer::new(&mut window.device, &svo);

    // main event loop
    let mut camera_velocity = Vector3::zero();
    let mut pitch = 0.0;
    let mut yaw = 0.0;
    let mut delta = 0.0;
    let mut delta_time = previous.elapsed();
    event_loop.run(move |event, _, control_flow| {
        imgui
            .platform
            .handle_event(imgui.context.io_mut(), &window.window, &event);
        match event {
            Event::NewEvents(_) => {
                delta_time = previous.elapsed();
                delta =
                    delta_time.as_secs() as f32 * 1000.0 + delta_time.subsec_nanos() as f32 * 1e-6;
                if delta < 3.0 {
                    std::thread::sleep(std::time::Duration::from_millis(3));
                    delta_time = previous.elapsed();
                    delta = delta_time.as_secs() as f32 * 1000.0
                        + delta_time.subsec_nanos() as f32 * 1e-6;
                }
                delta = delta / (1000.0 / 60.0);
                previous = imgui.context.io_mut().update_delta_time(previous);
            }
            Event::DeviceEvent {
                event: DeviceEvent::MouseMotion { delta: (x, y) },
                ..
            } => {
                pitch = clamp(pitch + y as f32 * delta, -89.0, 89.0);
                yaw = (yaw + x as f32 * delta) % 360.0;
            }
            Event::WindowEvent {
                event:
                    WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                state,
                                virtual_keycode: Some(key),
                                ..
                            },
                        ..
                    },
                ..
            } => {
                let pressed = match state {
                    ElementState::Pressed => 0.10,
                    ElementState::Released => 0.0,
                };
                match key {
                    VirtualKeyCode::A => camera_velocity.x = -pressed,
                    VirtualKeyCode::D => camera_velocity.x = pressed,
                    VirtualKeyCode::W => camera_velocity.z = pressed,
                    VirtualKeyCode::S => camera_velocity.z = -pressed,
                    VirtualKeyCode::Space => camera_velocity.y = pressed,
                    VirtualKeyCode::LControl => camera_velocity.y = -pressed,
                    _ => (),
                }
            }

            Event::WindowEvent {
                event: WindowEvent::Resized(size),
                ..
            } => {
                render_context = RenderContext::new(&window, size.width, size.height);
                width = size.width;
                height = size.height;
            }
            Event::MainEventsCleared => window.window.request_redraw(),
            Event::RedrawRequested(_) => {
                camera.rotation =
                    Matrix4::from_angle_y(Deg(yaw)) * Matrix4::from_angle_x(Deg(pitch));
                camera.position += camera.rotation.transform_vector(camera_velocity * delta);
                let frame = match render_context.swap_chain.get_next_texture() {
                    Ok(frame) => frame,
                    Err(e) => {
                        eprintln!("dropped frame: {:?}", e);
                        return;
                    }
                };

                let mut encoder: wgpu::CommandEncoder = window
                    .device
                    .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
                raytracer.render(
                    &mut window.device,
                    &mut encoder,
                    &frame.view,
                    width,
                    height,
                    &camera,
                );
                imgui.render(
                    &mut window.device,
                    &mut encoder,
                    &window.window,
                    &frame.view,
                    &delta_time,
                );
                window.queue.submit(&[encoder.finish()]);
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,
            _ => {}
        }
    });
}
