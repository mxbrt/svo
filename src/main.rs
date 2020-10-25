use svo::{render, simd, ui, voxel, voxel_csv, window};

use ui::ImguiContext;
use window::{RenderContext, WindowContext};
use winit::{
    event::{DeviceEvent, ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
};

fn main() {
    let event_loop = EventLoop::new();
    let mut window = WindowContext::new("SVO Renderer", &event_loop);
    let mut previous = std::time::Instant::now();
    let mut delta_time = previous.elapsed();

    let mut render_context = RenderContext::new(&window, 1280, 720);
    let mut imgui = ImguiContext::new(&mut window);
    // renderer initialization
    let mut width = render_context.swap_chain_descriptor.width;
    let mut height = render_context.swap_chain_descriptor.height;
    let mut img = vec![1.0f32; (4 * width * height) as usize];

    let mut renderer = render::Renderer::new(&mut window.device, width, height);

    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        println!("Need path to model csv.");
        std::process::exit(1);
    }
    let model_path = &args[1];
    let (bricks, points) = {
        let voxels = voxel_csv::read(model_path);
        let size = voxel_csv::parse_size(model_path);
        let grid = voxel::Grid::new(size, &voxels);
        let bricks = voxel::BrickPool::from(&grid);
        let points = simd::VectorList::from(&voxels);
        (bricks, points)
    };
    bricks._print_stats();
    points._print_stats();

    event_loop.run(move |event, _, control_flow| {
        imgui
            .platform
            .handle_event(imgui.context.io_mut(), &window.window, &event);
        match event {
            Event::NewEvents(_) => {
                delta_time = previous.elapsed();
                let delta =
                    delta_time.as_secs() as f32 * 1000.0 + delta_time.subsec_nanos() as f32 * 1e-6;
                if delta < 3.0 {
                    std::thread::sleep(std::time::Duration::from_millis(3));
                    delta_time = previous.elapsed();
                }
                previous = imgui.context.io_mut().update_delta_time(previous);
            }
            Event::DeviceEvent {
                event: DeviceEvent::MouseMotion { delta: (_, _) },
                ..
            } => {}
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
                let _ = match state {
                    ElementState::Pressed => 0.10,
                    ElementState::Released => 0.0,
                };
                match key {
                    VirtualKeyCode::Escape => *control_flow = ControlFlow::Exit,
                    _ => (),
                }
            }

            Event::WindowEvent {
                event: WindowEvent::Resized(size),
                ..
            } => {
                render_context = RenderContext::new(&window, size.width, size.height);
                renderer = render::Renderer::new(&mut window.device, width, height);
                img = vec![0.5f32; (4 * width * height) as usize];
                width = size.width;
                height = size.height;
            }
            Event::MainEventsCleared => window.window.request_redraw(),
            Event::RedrawRequested(_) => {
                let mut encoder =
                    window
                        .device
                        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                            label: Some("CommandEncoderDescriptor"),
                        });
                let frame = match render_context.swap_chain.get_current_frame() {
                    Ok(frame) => frame,
                    Err(e) => {
                        eprintln!("dropped frame: {:?}", e);
                        return;
                    }
                };

                {
                    let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                            attachment: &frame.output.view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(wgpu::Color::WHITE),
                                store: true,
                            },
                        }],
                        depth_stencil_attachment: None,
                    });

                    renderer.render(&window.queue, &mut render_pass, &img);
                    imgui.render(
                        &window.queue,
                        &window.device,
                        &window.window,
                        &mut render_pass,
                        &delta_time,
                    );
                }
                let x = Some(encoder.finish());
                window.queue.submit(x);
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,
            _ => {}
        }
    });
}
