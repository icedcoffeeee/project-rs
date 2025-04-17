#![allow(dead_code)] // throughout development
#![windows_subsystem = "windows"]

fn main() {
    let mut camera = refim::camera::Camera::new(0);
    let mut image1 = refim::image::Image::default();
    let mut image2 = refim::image::Image::default();
    let mut image3 = refim::image::Image::default();

    refim::window::new("ReFim", [1200, 700], move |ui, wdata| {
        let (w, h) = camera.resolution();

        ui.window("Controls").build(|| {});
        ui.window("Logs").build(|| {});
        ui.window("Feeds").build(|| {
            let [w, h] = [w as f32, h as f32];
            let [w_, h_] = ui.window_size();
            let s = unsafe { ui.style().window_padding };
            let factor = ((w_ - 4. * s[0]) / 3. / w).min((h_ - 5. * s[1]) / h);

            let dim = (w as u32, h as u32);
            let size = Some(((w * factor) as u32, (h * factor) as u32));
            let im1 = camera.get();
            let im2 = camera.get();

            image1.make(im1, dim, size, ui, wdata.display, wdata.renderer.textures());
            ui.same_line();
            image2.make(im2, dim, size, ui, wdata.display, wdata.renderer.textures());

            let sub = |(u, v): (&u8, &u8)| (*u as f32 - *v as f32).abs() as u8;
            let im3 = image1.data.iter().zip(&image2.data).map(sub).collect();
            ui.same_line();
            image3.make(im3, dim, size, ui, wdata.display, wdata.renderer.textures());
        });
    });
}
