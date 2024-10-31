use glow::HasContext;
use igr::glow;
use igr::TextureMap;
use imgui_glow_renderer as igr;

use opencv::core::*;
use opencv::imgproc;

use crate::SizeToArray;

/// usage:
/// ```
/// window::begin(|renderer, ui| {
///     ui.window(title).build(|| {
///             image.make(renderer).build(ui);
///     });
/// }
/// ```
#[derive(Default, Debug)]
pub struct Image {
    pub mat: Mat,
    texture: Option<glow::Texture>,
    texture_id: Option<imgui::TextureId>,
}

impl Image {
    pub fn make(&mut self, renderer: &mut igr::AutoRenderer, size: Size) -> imgui::Image {
        if self.texture_id.is_none() {
            self.init(renderer)
        }
        let gl = renderer.gl_context();

        let mut resized = Mat::default();
        imgproc::resize_def(&self.mat, &mut resized, size).unwrap();
        imgproc::cvt_color_def(&resized.clone(), &mut resized, imgproc::COLOR_BGR2RGB).unwrap();
        unsafe {
            gl.bind_texture(glow::TEXTURE_2D, self.texture);
            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_MIN_FILTER,
                glow::LINEAR as _,
            );
            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_MAG_FILTER,
                glow::LINEAR as _,
            );
            gl.tex_image_2d(
                glow::TEXTURE_2D,
                0,
                glow::RGB as _,
                size.width as _,
                size.height as _,
                0,
                glow::RGB,
                glow::UNSIGNED_BYTE,
                Some(resized.data_bytes().unwrap()),
            );
        };
        imgui::Image::new(self.texture_id.unwrap(), size.to_array())
    }

    fn init(&mut self, renderer: &mut igr::AutoRenderer) {
        self.texture = Some(unsafe { renderer.gl_context().create_texture() }.unwrap());
        self.texture_id = Some(
            renderer
                .texture_map_mut()
                .register(self.texture.unwrap())
                .unwrap(),
        );
    }
}
