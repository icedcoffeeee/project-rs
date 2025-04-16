use glium::glutin::surface::WindowSurface;
use glium::{Display, Surface};
use imgui::{Context, FontConfig, FontSource, Ui};
use imgui_glium_renderer::Renderer;
use imgui_winit_support::winit::dpi::LogicalSize;
use imgui_winit_support::winit::event::{Event, WindowEvent};
use imgui_winit_support::winit::event_loop::EventLoop;
use imgui_winit_support::winit::window::WindowAttributes;
use imgui_winit_support::{HiDpiMode, WinitPlatform};
use std::path::Path;
use std::time::Instant;

pub struct WindowData<'a> {
    pub display: &'a Display<WindowSurface>,
    pub renderer: &'a mut Renderer,
}

pub fn new<F: FnMut(&mut Ui, WindowData) + 'static>(title: &str, size: [i32; 2], run_ui: F) {
    new_with_startup(title, size, |_, _, _| {}, run_ui);
}

pub fn new_with_startup<FInit, FUi>(
    title: &str,
    size: [i32; 2],
    mut startup: FInit,
    mut run_ui: FUi,
) where
    FInit: FnMut(&mut Context, &mut Renderer, &Display<WindowSurface>) + 'static,
    FUi: FnMut(&mut Ui, WindowData) + 'static,
{
    let mut imgui = create_context();

    let title = match Path::new(&title).file_name() {
        Some(file_name) => file_name.to_str().unwrap(),
        None => title,
    };
    let event_loop = EventLoop::new().expect("Failed to create EventLoop");

    let window_attributes = WindowAttributes::default()
        .with_title(title)
        .with_inner_size(LogicalSize::<i32>::from(size));
    let (window, display) = glium::backend::glutin::SimpleWindowBuilder::new()
        .set_window_builder(window_attributes)
        .build(&event_loop);
    let mut renderer = Renderer::new(&mut imgui, &display).expect("Failed to initialize renderer");

    let mut platform = WinitPlatform::new(&mut imgui);
    {
        let dpi_mode = if let Ok(factor) = std::env::var("IMGUI_EXAMPLE_FORCE_DPI_FACTOR") {
            // Allow forcing of HiDPI factor for debugging purposes
            match factor.parse::<f64>() {
                Ok(f) => HiDpiMode::Locked(f),
                Err(e) => panic!("Invalid scaling factor: {}", e),
            }
        } else {
            HiDpiMode::Default
        };

        platform.attach_window(imgui.io_mut(), &window, dpi_mode);
    }

    let mut last_frame = Instant::now();

    startup(&mut imgui, &mut renderer, &display);
    let io = imgui.io_mut();
    io.config_flags |= imgui::ConfigFlags::DOCKING_ENABLE;

    #[allow(deprecated)]
    event_loop
        .run(move |event, window_target| match event {
            Event::NewEvents(_) => {
                let now = Instant::now();
                imgui.io_mut().update_delta_time(now - last_frame);
                last_frame = now;
            }
            Event::AboutToWait => {
                platform
                    .prepare_frame(imgui.io_mut(), &window)
                    .expect("Failed to prepare frame");
                window.request_redraw();
            }
            Event::WindowEvent {
                event: WindowEvent::RedrawRequested,
                ..
            } => {
                let ui = imgui.new_frame();
                ui.dockspace_over_main_viewport();

                let run = true;
                let window_data = WindowData {
                    display: &display,
                    renderer: &mut renderer,
                };
                run_ui(ui, window_data);
                (!run).then(|| window_target.exit());

                let mut target = display.draw();
                target.clear_color_srgb(1.0, 1.0, 1.0, 1.0);
                platform.prepare_render(ui, &window);
                let draw_data = imgui.render();
                renderer
                    .render(&mut target, draw_data)
                    .expect("Rendering failed");
                target.finish().expect("Failed to swap buffers");
            }
            Event::WindowEvent {
                event: WindowEvent::Resized(new_size),
                ..
            } => {
                (new_size.width > 0 && new_size.height > 0)
                    .then(|| display.resize((new_size.width, new_size.height)));

                platform.handle_event(imgui.io_mut(), &window, &event);
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => window_target.exit(),
            event => {
                platform.handle_event(imgui.io_mut(), &window, &event);
            }
        })
        .expect("EventLoop error");
}

/// Creates the imgui context
pub fn create_context() -> imgui::Context {
    let mut imgui = Context::create();
    imgui.fonts().add_font(&[FontSource::TtfData {
        data: include_bytes!("../assets/Karla.ttf"),
        size_pixels: 16.,
        config: Some(FontConfig {
            rasterizer_multiply: 1.5,
            oversample_h: 4,
            oversample_v: 4,
            ..FontConfig::default()
        }),
    }]);
    //imgui.set_ini_filename(None);

    imgui
}
