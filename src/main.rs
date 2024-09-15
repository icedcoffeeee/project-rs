use project_rs::*;

use opencv::core::*;
use opencv::imgproc::*;
use opencv::videoio::*;
use opencv::Result;

use rayon::iter::*;

fn main() -> Result<()> {
    let imheightx3 = 100;
    let size = Size::new(imheightx3 * 4, imheightx3 * 3);

    let mut camera = VideoCapture::new(0, CAP_ANY)?;
    let mut cam_img = image::Image::new(size.to_slice());
    let mut proc_img = image::Image::new(size.to_slice());

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
                let pix: [u8; 3] = i.to_vec().try_into().unwrap();
                let col: [u8; 3] = col.map(|c| (c * 255.) as u8);
                let gate = rgb_to_hsl(pix)
                    .into_iter()
                    .zip(rgb_to_hsl(col))
                    .map(|(p, c)| (p - c).abs())
                    .all(|i| i < tol);
                if gate {
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
                ui.slider("Color Tolerance", 0., 1., &mut tol);
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

const TOL: f32 = 0.1;
fn rgb_to_hsl(rgb: [u8; 3]) -> [f32; 3] {
    let [r, g, b] = rgb.map(|i| i as f32 / 255.);
    let (max, min) = (r.max(g).max(b), r.min(g).min(b));
    let lum = (max + min) / 2.;
    let (mut hue, mut sat) = (0., 0.);
    let chroma = (max - min).abs();
    if chroma > TOL {
        sat += chroma / (1. + (2. * lum - 1.).abs());
        if max - g < TOL {
            hue += (b - r) / chroma + 2.;
        } else if max - b < TOL {
            hue += (r - g) / chroma + 4.;
        }
    }
    return [hue, sat, lum];
}
