use std::time::Instant;

use glutin::config::{Config, ConfigTemplateBuilder, GlConfig};
use glutin::context::{ContextApi, ContextAttributesBuilder, PossiblyCurrentContext};
use glutin::display::GetGlDisplay;
use glutin::prelude::{GlDisplay, NotCurrentGlContext};
use glutin::surface::{Surface, WindowSurface};
use glutin_winit::GlWindow;

use imgui::{Context, FontSource, Ui};
use imgui_glow_renderer::glow::{self, HasContext};
use imgui_glow_renderer::AutoRenderer;
use imgui_winit_support::{HiDpiMode, WinitPlatform};

use winit::dpi::PhysicalSize;
use winit::event_loop::ActiveEventLoop;
use winit::raw_window_handle::HasWindowHandle;
use winit::window::{Window, WindowAttributes};

pub trait MainLoop = FnMut(&mut Ui, &mut AutoRenderer);

pub struct App<Loop: MainLoop> {
    pub imgui: Context,
    pub platform: WinitPlatform,
    pub main_loop: Loop,
    pub last_frame: Instant,

    pub window: Option<Window>,
    pub renderer: Option<AutoRenderer>,
    pub surface: Option<(Surface<WindowSurface>, PossiblyCurrentContext)>,
}

impl<Loop: MainLoop> App<Loop> {
    pub fn new(main_loop: Loop) -> Self {
        let mut imgui = Context::create();
        let font_atlas = imgui.fonts();
        font_atlas.add_font(&[FontSource::TtfData {
            data: include_bytes!("../assets/JetBrainsMonoNerdFont-Regular.ttf"),
            size_pixels: 16.,
            config: None,
        }]);
        font_atlas.build_rgba32_texture();

        let platform = WinitPlatform::new(&mut imgui);

        Self {
            imgui,
            platform,
            main_loop,
            last_frame: Instant::now(),

            window: None,
            renderer: None,
            surface: None,
        }
    }

    pub fn setup(&mut self, event_loop: &ActiveEventLoop) {
        let size = PhysicalSize::new(1920 * 3 / 4, 1080 * 3 / 4);
        let wind_attr = WindowAttributes::default()
            .with_title("project")
            .with_inner_size(size);
        let template = ConfigTemplateBuilder::new();
        let compare_conf = |a: &Config, b: &Config| a.num_samples().cmp(&b.num_samples());

        let (window, config) = glutin_winit::DisplayBuilder::new()
            .with_window_attributes(Some(wind_attr))
            .build(event_loop, template, |config| {
                config.max_by(compare_conf).unwrap()
            })
            .unwrap();
        let window_ref = window.as_ref().unwrap();
        let display = config.display();

        let surf_attr = window_ref
            .build_surface_attributes(Default::default())
            .unwrap();
        let surface = unsafe { display.create_window_surface(&config, &surf_attr).unwrap() };

        let gl_version = glutin::context::Version { major: 4, minor: 1 };
        let ctx_api = ContextApi::OpenGl(Some(gl_version));
        let ctx_attr = ContextAttributesBuilder::new()
            .with_context_api(ctx_api)
            .build(Some(window_ref.window_handle().unwrap().into()));
        let context = unsafe { display.create_context(&config, &ctx_attr).unwrap() };
        let context = context.make_current(&surface).unwrap();

        self.platform
            .attach_window(self.imgui.io_mut(), window_ref, HiDpiMode::Default);

        self.window = window;
        self.renderer = unsafe {
            let gl = glow::Context::from_loader_function_cstr(|s| display.get_proc_address(s));
            gl.clear_color(0.3, 0.3, 0.3, 1.);
            Some(AutoRenderer::new(gl, &mut self.imgui).unwrap())
        };
        self.surface = Some((surface, context));
    }
}
