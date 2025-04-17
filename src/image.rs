use std::rc::Rc;

use glium::glutin::surface::WindowSurface;
use glium::texture::RawImage2d;
use glium::uniforms::SamplerBehavior;
use glium::uniforms::{MagnifySamplerFilter, MinifySamplerFilter};
use glium::{Display, Texture2d};
use imgui::TextureId;
use imgui_glium_renderer::Texture;

use crate::window::WindowData;

#[derive(Default)]
pub struct Image(Option<TextureId>);

impl Image {
    pub fn get_texture(
        &mut self,
        data: &[u8],
        dim: (u32, u32),
        display: &Display<WindowSurface>,
    ) -> Texture {
        let image = RawImage2d::from_raw_rgb(data.to_vec(), dim);
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
        data: &[u8],
        dim: (u32, u32),
        size: (u32, u32),
        wdata: &mut WindowData,
    ) -> imgui::Image {
        let textures = wdata.renderer.textures();
        let id = {
            let texture = self.get_texture(data, dim, wdata.display);
            if self.0.is_none() {
                let id = textures.insert(texture);
                self.0 = Some(id);
            } else {
                textures.replace(self.0.unwrap(), texture);
            }
            self.0.unwrap()
        };
        let (w, h) = size;
        imgui::Image::new(id, [w as f32, h as f32])
    }
}
