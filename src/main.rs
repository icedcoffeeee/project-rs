#![allow(dead_code)] // throughout development
#![windows_subsystem = "windows"]

fn main() {
    let mut camera = refim::camera::Camera::new(0);
    //let mut camera = refim::camera::ImageCamera::new("assets/test.png");
    let mut images = [refim::image::Image::default(); 12];

    let (w, h) = camera.resolution();
    let [cw, ch] = [w as f32, h as f32];
    let grid = [4., 3.];

    let sub = |(u, v): (&u8, &u8)| (*u as f32 - *v as f32).abs() as u8;

    refim::window::new("ReFim", [1200, 700], move |ui, mut wdata| {
        let s = unsafe { ui.style().window_padding };

        ui.window("Controls").build(|| {});
        ui.window("Logs").build(|| {});
        ui.window("Feeds").build(|| {
            let [ww, wh] = ui.window_size();
            let max_w = (ww - (grid[0] + 1.) * s[0]) / grid[0] / cw;
            let max_h = (wh - (grid[1] + 1. + 3.) * s[1]) / grid[1] / ch;
            // extra 3 px for height due to docking tab bar
            let factor = max_w.min(max_h);

            let size = ((cw * factor) as u32, (ch * factor) as u32);
            let mut ims = Vec::new();
            ims.push(camera.get());
            ims.push(camera.get());

            ims.push(ims[0].iter().zip(&ims[1]).map(sub).collect::<Vec<_>>());

            for (im, j) in ims.iter().zip([0, 4, 8]) {
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
