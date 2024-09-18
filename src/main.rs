use project::*;

use colors_transform as colors;
use colors_transform::Color;

use opencv::core::*;
use opencv::imgproc::*;
use opencv::videoio::*;
use opencv::Result;

use rayon::iter::*;

fn main() -> Result<()> {
    let imheightx3 = 100;
    let img_size = Size::new(imheightx3 * 4, imheightx3 * 3);

    let mut camera = VideoCapture::new(0, CAP_ANY)?;
    let mut cam_img = image::Image::default();
    let mut proc_img = image::Image::default();

    let mut col: [f32; 3] = [0., 0., 1.];
    let mut tol = 10.;

    window::begin(|renderer, ui| {
        camera.read(&mut cam_img.mat)?;
        cvt_color_def(&cam_img.mat.clone(), &mut cam_img.mat, COLOR_BGR2RGB)?;
        resize_def(&cam_img.mat.clone(), &mut cam_img.mat, img_size)?;

        resize_def(&cam_img.mat, &mut proc_img.mat, img_size)?;
        proc_img
            .mat
            .data_typed_mut::<Vec3b>()?
            .par_iter_mut()
            .for_each(|data| {
                let mut rgb = data.map(|i| i as f32 / 255.);
                let mut hue = colors::Rgb::from_tuple(&rgb.into()).get_hue();
                rgb = col.map(|i| i * 255.);
                hue -= colors::Rgb::from_tuple(&rgb.into()).get_hue();
                if hue.abs() >= tol {
                    *data = Vec3b::from([0, 0, 0]);
                }
            });

        ui.window("Camera")
            .position([30., 30.], imgui::Condition::Once)
            .content_size(img_size.to_array())
            .resizable(false)
            .collapsible(false)
            .build(|| cam_img.make(renderer).build(ui));

        ui.window("Processed")
            .position([30., 100. + img_size.to_array()[1]], imgui::Condition::Once)
            .content_size(img_size.to_array())
            .resizable(false)
            .collapsible(false)
            .build(|| proc_img.make(renderer).build(ui));

        ui.window("Control Panel")
            .position([img_size.to_array()[0] + 100., 100.], imgui::Condition::Once)
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
