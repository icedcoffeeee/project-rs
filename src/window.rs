use opencv::Result;
use std::rc::Rc;
use std::time::Instant;

pub use imgui::Ui;
use imgui_glow_renderer::glow::{self, HasContext};
pub use imgui_glow_renderer::AutoRenderer;

use glutin::surface::GlSurface;
use winit::application::ApplicationHandler;
use winit::event::{ElementState, Event, KeyEvent, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::WindowId;

use crate::app::{App, MainLoop};

impl<Loop: MainLoop> ApplicationHandler for App<Loop> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.setup(event_loop);
    }

    fn new_events(&mut self, _event_loop: &ActiveEventLoop, _cause: winit::event::StartCause) {
        let now = Instant::now();
        self.imgui.io_mut().update_delta_time(now - self.last_frame);
        self.last_frame = now;
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        self.platform
            .prepare_frame(self.imgui.io_mut(), self.window.as_ref().unwrap())
            .expect("could not prepare frame");
        self.window.as_ref().unwrap().request_redraw();
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::RedrawRequested => {
                let renderer = self.renderer.as_mut().unwrap();
                let ui = self.imgui.new_frame();
                self.platform
                    .prepare_render(ui, self.window.as_ref().unwrap());

                (self.main_loop)(ui, renderer);

                unsafe {
                    renderer.gl_context().clear(glow::COLOR_BUFFER_BIT);
                }

                let draw_data = self.imgui.render();
                if draw_data.draw_lists_count() != 0 {
                    renderer.render(draw_data).unwrap();
                }

                let (surface, context) = self.surface.as_ref().unwrap();
                surface.swap_buffers(&context).unwrap();
            }
            WindowEvent::KeyboardInput { event, .. } => match event {
                KeyEvent {
                    physical_key: PhysicalKey::Code(keycode),
                    state: ElementState::Pressed,
                    ..
                } => match keycode {
                    KeyCode::Enter => {
                        let renderer = self.renderer.as_mut().unwrap();
                        let gl_ctx = renderer.gl_context();
                        let physical_size: (i32, i32) =
                            self.window.as_ref().unwrap().inner_size().into();
                        screenshot_window(gl_ctx, physical_size).unwrap();
                    }
                    _ => {}
                },
                _ => {}
            },
            WindowEvent::CloseRequested => event_loop.exit(),
            _ => {
                self.platform.handle_event(
                    self.imgui.io_mut(),
                    self.window.as_ref().unwrap(),
                    &Event::<()>::WindowEvent { window_id, event },
                );
            }
        }
    }
}

fn screenshot_window(gl_ctx: &Rc<glow::Context>, (width, height): (i32, i32)) -> Result<()> {
    use crate::utils;
    use opencv::core::*;
    use opencv::imgcodecs;

    let (format, gltype) = (glow::BGR, glow::UNSIGNED_BYTE);
    let mut rgb = [0; 3 * 1440 * 810];
    let px = glow::PixelPackData::Slice(&mut rgb);
    unsafe {
        gl_ctx.pixel_store_i32(glow::PACK_ALIGNMENT, 4);
        gl_ctx.read_buffer(glow::FRONT);
        gl_ctx.read_pixels(0, 0, width, height, format, gltype, px);
    }

    let mut mat = Mat::default();
    Mat::from_bytes_mut::<VecN<u8, 3>>(&mut rgb)?
        .reshape(3, 810)?
        .assign_to_def(&mut mat)?;
    flip(&mat.clone(), &mut mat, 0)?;

    imgcodecs::imwrite_def(&utils::get_save_filepath("screenshot.png"), &mat)?;
    Ok(())
}

pub fn create(main_loop: impl MainLoop) {
    let event_loop = EventLoop::new().unwrap();
    let mut app = App::new(main_loop);
    event_loop.run_app(&mut app).expect("could not run app");
}
