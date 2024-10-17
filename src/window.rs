use igr::glow;
use igr::glow::HasContext;
use imgui_glow_renderer as igr;
use imgui_sdl2_support as iss;

use imgui::{FontSource, Ui};
use opencv::Result;
use sdl2::{event::Event, video::GLProfile};

pub fn begin<T>(mut app: T)
where
    T: FnMut(&mut igr::AutoRenderer, &mut Ui) -> Result<()>,
{
    let sdl = sdl2::init().unwrap();
    let video_subsystem = sdl.video().unwrap();
    let window = video_subsystem
        .window("app", 1920 / 4 * 3, 1080 / 4 * 3)
        .allow_highdpi()
        .opengl()
        .position_centered()
        .build()
        .unwrap();

    let gl_context = window.gl_create_context().unwrap();
    window.gl_make_current(&gl_context).unwrap();
    window.subsystem().gl_set_swap_interval(1).unwrap();
    window.subsystem().gl_attr().set_context_version(3, 3);
    window
        .subsystem()
        .gl_attr()
        .set_context_profile(GLProfile::Core);

    let gl_context = unsafe {
        glow::Context::from_loader_function(|s| window.subsystem().gl_get_proc_address(s) as _)
    };

    let mut imgui = imgui::Context::create();
    //imgui.set_ini_filename(None);
    imgui.set_log_filename(None);
    imgui
        .fonts()
        .add_font(&[FontSource::DefaultFontData { config: None }]);
    imgui.style_mut().use_dark_colors();

    let mut event_pump = sdl.event_pump().unwrap();
    let mut platform = iss::SdlPlatform::new(&mut imgui);
    let mut renderer = igr::AutoRenderer::new(gl_context, &mut imgui).unwrap();

    'main: loop {
        for event in event_pump.poll_iter() {
            platform.handle_event(&mut imgui, &event);
            if let Event::Quit { .. } = event {
                break 'main;
            }
        }

        platform.prepare_frame(&mut imgui, &window, &event_pump);

        let mut ui = imgui.new_frame();
        let _ = app(&mut renderer, &mut ui);

        let draw_data = imgui.render();

        unsafe {
            renderer
                .gl_context()
                .clear(imgui_glow_renderer::glow::COLOR_BUFFER_BIT);
        }
        renderer.render(draw_data).unwrap();
        window.gl_swap_window();
    }
}
