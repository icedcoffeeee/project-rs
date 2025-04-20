#![allow(dead_code)] // throughout development
#![windows_subsystem = "windows"]

fn main() {
    //let mut camera1 = refim::camera::Camera::new(0);
    let mut camera1 = refim::camera::ImageCamera::new("assets/test1.png");
    let mut camera2 = refim::camera::ImageCamera::new("assets/test2.png");
    let mut images = [refim::image::Image::default(); 12];

    let (w, h) = camera1.resolution();
    let [cw, ch] = [w as f32, h as f32];
    let grid = [4., 3.];

    let to_arr = |x: &[u8]| -> [f32; 3] {
        let x: Vec<_> = x.iter().map(|i| *i as f32).collect();
        x.try_into().unwrap()
    };
    let sub = move |(u, v): (&[u8], &[u8])| -> [u8; 3] {
        let rgb1 = to_arr(u);
        let rgb2 = to_arr(v);
        let rgb: Vec<_> = (0..3)
            .map(|i| {
                let diff1 = (rgb2[(i + 0) % 3] - rgb1[(i + 0) % 3]).clamp(0., 255.);
                let diff2 = (rgb1[(i + 1) % 3] - rgb2[(i + 1) % 3]).clamp(0., 255.);
                let diff3 = (rgb1[(i + 2) % 3] - rgb2[(i + 2) % 3]).clamp(0., 255.);
                (diff1 + diff2 + diff3) as u8
            })
            .collect();
        rgb.try_into().unwrap()
    };

    refim::window::new("RefIm", [1200, 700], move |ui, mut wdata| {
        let s = unsafe { ui.style().window_padding };

        ui.window("Controls").build(|| {});
        ui.window("Logs").build(|| {
            ui.text_wrapped(format!("FPS: {}", ui.io().framerate));
        });
        ui.window("Feeds").build(|| {
            let [ww, wh] = ui.window_size();
            let max_w = (ww - (grid[0] + 1.) * s[0]) / grid[0] / cw;
            let max_h = (wh - (grid[1] + 1. + 3.) * s[1]) / grid[1] / ch;
            // ^^^ extra 3 px for height due to docking tab bar
            let factor = max_w.min(max_h);

            let size = ((cw * factor) as u32, (ch * factor) as u32);
            let mut ims = Vec::new();
            ims.push(camera1.get());
            ims.push(camera2.get());

            let im = ims[0].chunks(3).zip(ims[1].chunks(3));
            let im: Vec<_> = im.map(sub).flatten().collect();
            ims.push(im);

            for (im, j) in ims.iter().zip((0..12).step_by(4)) {
                images[j].make(im, (w, h), size, &mut wdata).build(ui);
                for i in 1..4 {
                    let channel = refim::image::get_channel(im, i - 1);
                    ui.same_line();
                    images[j + i]
                        .make(&channel, (w, h), size, &mut wdata)
                        .build(ui);
                }
            }
        });
    });
}
