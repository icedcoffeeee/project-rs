use std::rc::Rc;

use glium::glutin::surface::WindowSurface;
use glium::texture::RawImage2d;
use glium::uniforms::SamplerBehavior;
use glium::uniforms::{MagnifySamplerFilter, MinifySamplerFilter};
use glium::{Display, Texture2d};
use imgui::{TextureId, Textures};
use imgui_glium_renderer::Texture;

#[derive(Default)]
pub struct Image {
    pub data: Vec<u8>,
    id: Option<TextureId>,
}

impl Image {
    pub fn get_texture(
        &mut self,
        data: Vec<u8>,
        dim: (u32, u32),
        display: &Display<WindowSurface>,
    ) -> Texture {
        let image = RawImage2d::from_raw_rgb(data, dim);
        let texture = Texture2d::new(display, image).unwrap();
        Texture {
            texture: Rc::new(texture),
            sampler: SamplerBehavior {
                magnify_filter: MagnifySamplerFilter::Linear,
                minify_filter: MinifySamplerFilter::Linear,
                ..Default::default()
            },
        }
    }

    pub fn make(
        &mut self,
        data: Vec<u8>,
        dim: (u32, u32),
        size: Option<(u32, u32)>,
        ui: &imgui::Ui,
        display: &Display<WindowSurface>,
        textures: &mut Textures<Texture>,
    ) {
        self.data = data;
        let id = {
            let texture = self.get_texture(self.data.clone(), dim, display);
            if self.id.is_none() {
                let id = textures.insert(texture);
                self.id = Some(id);
            } else {
                textures.replace(self.id.unwrap(), texture);
            }
            self.id.unwrap()
        };
        let (w, h) = size.unwrap_or(dim);
        imgui::Image::new(id, [w as f32, h as f32]).build(ui);
    }
}
