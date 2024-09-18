use std::thread::sleep;
use std::time::Duration;

use imgui_glow_renderer::glow::HasContext;
use project::*;

use opencv::core::*;
use opencv::imgproc::*;
use opencv::videoio::*;
use opencv::Result;

fn main() -> Result<()> {
    let imheightx3 = 100;
    let img_size = Size::new(imheightx3 * 4, imheightx3 * 3);

    let mut camera = VideoCapture::new(0, CAP_ANY)?;
    let mut non_light_img = image::Image::default();
    let mut red_light_img = image::Image::default();
    let mut processed_img = image::Image::default();

    let mut col: [f32; 3] = [0., 0., 1.];
    let mut tol = 10.;
    let mut i = 0;

    window::begin(|renderer, ui| {
        sleep(Duration::new(0, 1e9 as u32 / 60));
        i = (i + 1) % 2;
        if i == 0 {
            unsafe {
                renderer.gl_context().clear_color(0., 0., 0., 1.);
            }
            camera.read(&mut non_light_img.mat)?;
            cvt_color_def(
                &non_light_img.mat.clone(),
                &mut non_light_img.mat,
                COLOR_BGR2RGB,
            )?;
            resize_def(&non_light_img.mat.clone(), &mut non_light_img.mat, img_size)?;
        } else {
            unsafe {
                renderer.gl_context().clear_color(1., 0., 0., 1.);
            }
            camera.read(&mut red_light_img.mat)?;
            cvt_color_def(
                &red_light_img.mat.clone(),
                &mut red_light_img.mat,
                COLOR_BGR2RGB,
            )?;
            resize_def(&red_light_img.mat.clone(), &mut red_light_img.mat, img_size)?;
            subtract_def(
                &red_light_img.mat,
                &non_light_img.mat,
                &mut processed_img.mat,
            )?;
        }
        if red_light_img.mat.empty() || processed_img.mat.empty() {
            return Ok(());
        };
        ui.window("Non Light")
            .position([30., 30.], imgui::Condition::Once)
            .content_size(img_size.to_array())
            .resizable(false)
            .collapsible(false)
            .build(|| non_light_img.make(renderer).build(ui));

        ui.window("Red Light")
            .position(
                [30. + 50. + img_size.to_array()[0], 30.],
                imgui::Condition::Once,
            )
            .content_size(img_size.to_array())
            .resizable(false)
            .collapsible(false)
            .build(|| red_light_img.make(renderer).build(ui));

        ui.window("Processed")
            .position(
                [30., 30. + 50. + img_size.to_array()[1]],
                imgui::Condition::Once,
            )
            .content_size(img_size.to_array())
            .resizable(false)
            .collapsible(false)
            .build(|| processed_img.make(renderer).build(ui));

        ui.window("Control Panel")
            .position(
                [30. + 2. * (50. + img_size.to_array()[0]), 30.],
                imgui::Condition::Once,
            )
            .content_size([500., 500.])
            .resizable(false)
            .collapsible(false)
            .build(|| {
                ui.color_edit3("Analyzed Color", &mut col);
                ui.slider("Color Tolerance (hue degrees)", 0., 50., &mut tol);
                ui.text(format!("FPS: {} fps", ui.io().framerate));
            });
        Ok(())
    });

    Ok(())
}

trait ToArray2 {
    fn to_array(self) -> [f32; 2];
}
impl ToArray2 for Size {
    fn to_array(self) -> [f32; 2] {
        [self.width as _, self.height as _]
    }
}
