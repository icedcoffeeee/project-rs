use project_rs::*;

use colors_transform as colors;
use colors_transform::Color;

use opencv::core::*;
use opencv::imgproc::*;
use opencv::videoio::*;
use opencv::Result;

use rayon::iter::*;

fn main() -> Result<()> {
    let imheightx3 = 100;
    let size = Size::new(imheightx3 * 4, imheightx3 * 3);

    let mut camera = VideoCapture::new(0, CAP_ANY)?;
    let mut cam_img = image::Image::default();
    let mut proc_img = image::Image::default();

    let mut col: [f32; 3] = [0., 0., 1.];
    let mut tol = 0.8;

    window::begin(|renderer, ui| {
        camera.read(&mut cam_img.mat)?;
        cvt_color_def(&cam_img.mat.clone(), &mut cam_img.mat, COLOR_BGR2RGB)?;
        resize_def(&cam_img.mat.clone(), &mut cam_img.mat, size)?;

        resize_def(&cam_img.mat, &mut proc_img.mat, size)?;
        proc_img
            .mat
            .data_typed_mut::<Vec3b>()?
            .par_iter_mut()
            .for_each(|i| {
                let pix: [f32; 3] = i
                    .to_vec()
                    .iter()
                    .map(|i| *i as f32 / 255.)
                    .collect::<Vec<f32>>()
                    .try_into()
                    .unwrap();
                let gate = rgb_to_hsl(pix)
                    .into_iter()
                    .zip(rgb_to_hsl(col))
                    .map(|(p, c)| (p - c).abs())
                    .all(|i| i < tol);
                if !gate {
                    *i = Vec3b::from([0, 0, 0]);
                };
            });

        ui.window("Camera")
            .position([30., 30.], imgui::Condition::Once)
            .content_size(size.to_slice())
            .resizable(false)
            .collapsible(false)
            .build(|| cam_img.make(renderer).build(ui));

        ui.window("Processed")
            .position([30., 100. + size.to_slice()[1]], imgui::Condition::Once)
            .content_size(size.to_slice())
            .resizable(false)
            .collapsible(false)
            .build(|| proc_img.make(renderer).build(ui));

        ui.window("Control Panel")
            .position([size.to_slice()[0] + 100., 100.], imgui::Condition::Once)
            .content_size([500., 500.])
            .resizable(false)
            .collapsible(false)
            .build(|| {
                ui.color_edit3("Analyzed Color", &mut col);
                let [h, s, l] = rgb_to_hsl(col);
                ui.text(format!("HSL: {} {} {}", h, s, l));

                ui.slider("Color Tolerance", 0., 10., &mut tol);
                ui.text(format!("FPS: {} fps", ui.io().framerate));
            });

        Ok(())
    });

    Ok(())
}

trait SizeToSlice {
    fn to_slice(self) -> [f32; 2];
}
impl SizeToSlice for Size {
    fn to_slice(self) -> [f32; 2] {
        [self.width as _, self.height as _]
    }
}

fn rgb_to_hsl(rgb: [f32; 3]) -> [f32; 3] {
    let rgb: (f32, f32, f32) = rgb.map(|i| i as f32 * 255.).try_into().unwrap();
    let hsl = colors::Rgb::from_tuple(&rgb).to_hsl();
    [
        hsl.get_hue() / 360. * 255.,
        hsl.get_saturation() / 100. * 255.,
        hsl.get_lightness() / 100. * 255.,
    ]
}
