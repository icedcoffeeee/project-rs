use igr::glow::HasContext;
use igr::{glow, TextureMap};
use imgui_glow_renderer as igr;
use opencv::core::{self as cv, MatTraitConst, MatTraitConstManual};

#[derive(Default, Debug)]
pub struct Image {
    pub mat: cv::Mat,
    texture: Option<glow::Texture>,
    texture_id: Option<imgui::TextureId>,
}

impl Image {
    pub fn make(&mut self, renderer: &mut igr::AutoRenderer) -> imgui::Image {
        if self.texture_id.is_none() {
            self.init(renderer)
        }
        let gl = renderer.gl_context();
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
                self.size()[0] as _,
                self.size()[1] as _,
                0,
                glow::RGB,
                glow::UNSIGNED_BYTE,
                Some(self.mat.data_bytes().unwrap()),
            );
        };
        imgui::Image::new(self.texture_id.unwrap(), self.size())
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

    fn size(&self) -> [f32; 2] {
        [
            self.mat.size().unwrap().width as _,
            self.mat.size().unwrap().height as _,
        ]
    }
}
