use futures::executor::block_on;
use wgpu::{Device, Queue, Surface, SwapChain, SwapChainDescriptor};
use winit::{
    dpi::LogicalSize,
    event_loop::EventLoop,
    window::{Window, WindowBuilder},
};

pub struct WindowContext {
    pub window: Window,
    pub device: Device,
    pub queue: Queue,
    pub surface: Surface,
}
impl WindowContext {
    pub fn new(title: &str, event_loop: &EventLoop<()>) -> WindowContext {
        let window = WindowBuilder::new()
            .with_title(title.to_owned())
            .with_inner_size(LogicalSize::new(1280.0, 720.0))
            .with_resizable(true)
            .build(event_loop)
            .unwrap();
        window.set_cursor_grab(true).unwrap();
        window.set_cursor_visible(false);
        let surface = wgpu::Surface::create(&window);
        let adapter = block_on(wgpu::Adapter::request(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
            },
            wgpu::BackendBit::PRIMARY,
        ))
        .unwrap();

        let (device, queue) = block_on(adapter.request_device(&wgpu::DeviceDescriptor {
            extensions: wgpu::Extensions {
                anisotropic_filtering: false,
            },
            limits: wgpu::Limits::default(),
        }));
        WindowContext {
            window,
            device,
            queue,
            surface,
        }
    }
}

pub struct RenderContext {
    pub swap_chain: SwapChain,
    pub swap_chain_descriptor: SwapChainDescriptor,
}

impl RenderContext {
    pub fn new(window: &WindowContext, width: u32, height: u32) -> RenderContext {
        let swap_chain_descriptor = SwapChainDescriptor {
            usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
            format: wgpu::TextureFormat::Bgra8Unorm,
            width,
            height,
            present_mode: wgpu::PresentMode::Immediate,
        };
        let swap_chain = window
            .device
            .create_swap_chain(&window.surface, &swap_chain_descriptor);

        RenderContext {
            swap_chain,
            swap_chain_descriptor,
        }
    }
}
