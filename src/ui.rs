use clipboard::{ClipboardContext, ClipboardProvider};
use imgui::*;
use imgui_wgpu::Renderer;
use imgui_winit_support::{HiDpiMode, WinitPlatform};

use crate::window::WindowContext;

struct Clipboard(ClipboardContext);
impl ClipboardBackend for Clipboard {
    fn get(&mut self) -> Option<ImString> {
        self.0.get_contents().ok().map(|text| text.into())
    }
    fn set(&mut self, text: &ImStr) {
        let _ = self.0.set_contents(text.to_str().to_owned());
    }
}

pub struct ImguiContext {
    pub context: Context,
    pub platform: WinitPlatform,
    pub renderer: Renderer,
    pub font_size: f32,
}

impl ImguiContext {
    pub fn new(window_context: &mut WindowContext) -> ImguiContext {
        let mut imgui = Context::create();
        imgui.set_ini_filename(None);

        if let Ok(clipboard) = ClipboardContext::new() {
            imgui.set_clipboard_backend(Box::new(Clipboard(clipboard)));
        } else {
            eprintln!("Failed to initialize clipboard");
        }

        let mut platform = WinitPlatform::init(&mut imgui);
        platform.attach_window(imgui.io_mut(), &window_context.window, HiDpiMode::Default);

        let hidpi_factor = platform.hidpi_factor();
        let font_size = (13.0 * hidpi_factor) as f32;
        imgui.fonts().add_font(&[FontSource::DefaultFontData {
            config: Some(imgui::FontConfig {
                oversample_h: 1,
                pixel_snap_h: true,
                size_pixels: font_size,
                ..Default::default()
            }),
        }]);

        imgui.io_mut().font_global_scale = (1.0 / hidpi_factor) as f32;
        let renderer = Renderer::new(
            &mut imgui,
            &window_context.device,
            &mut window_context.queue,
            wgpu::TextureFormat::Bgra8Unorm,
            None,
        );

        ImguiContext {
            context: imgui,
            platform,
            renderer,
            font_size,
        }
    }

    pub fn render(
        &mut self,
        device: &mut wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        window: &winit::window::Window,
        frame_view: &wgpu::TextureView,
        delta_time: &std::time::Duration,
    ) {
        self.platform
            .prepare_frame(self.context.io_mut(), &window)
            .expect("Failed to prepare frame");
        let ui = self.context.frame();
        compose(&ui, delta_time);
        self.platform.prepare_render(&ui, &window);
        self.renderer
            .render(ui.render(), device, encoder, frame_view)
            .expect("Rendering failed");
    }
}

fn compose(ui: &imgui::Ui, delta_time: &std::time::Duration) {
    let window = imgui::Window::new(im_str!("SVO Renderer"));
    window
        .size([300.0, 100.0], Condition::FirstUseEver)
        .build(&ui, || {
            ui.text(im_str!("Frametime: {:?}", delta_time));
        });
}
