use std::time::Instant;

use imgui::Ui;
use imgui_glow_renderer::AutoRenderer;
use winit::application::ApplicationHandler;
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::WindowId;

use glutin::surface::GlSurface;
use imgui_glow_renderer::glow::{self, HasContext};

use crate::app::App;

impl<Loop: FnMut(&mut Ui, &mut AutoRenderer)> ApplicationHandler for App<Loop> {
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
pub fn create<Loop: FnMut(&mut Ui, &mut AutoRenderer)>(main_loop: Loop) {
    let event_loop = EventLoop::new().unwrap();
    let mut app = App::new(main_loop);
    event_loop.run_app(&mut app).expect("could not run app");
}
