#![allow(dead_code)] // throughout development
#![windows_subsystem = "windows"]

fn main() {
    let mut camera = refim::camera::Camera::new(0);
    //let mut camera = refim::camera::ImageCamera(Some(image::open("assets/test.png").unwrap()));
    let mut image1 = refim::image::Image::default();
    let mut image2 = refim::image::Image::default();
    let mut image3 = refim::image::Image::default();

    refim::window::new("ReFim", [1200, 700], move |ui, mut wdata| {
        let (w, h) = camera.resolution();
        let [wh, hf] = [w as f32, h as f32];
        let s = unsafe { ui.style().window_padding };

        ui.window("Controls").build(|| {});
        ui.window("Logs").build(|| {});
        ui.window("Feeds").build(|| {
            let [w_, h_] = ui.window_size();
            let factor = ((w_ - 4. * s[0]) / 3. / wh).min((h_ - 5. * s[1]) / hf);

            let size = ((wh * factor) as u32, (hf * factor) as u32);
            let im1 = camera.get();
            let im2 = camera.get();

            image1.make(&im1, (w, h), size, &mut wdata).build(ui);
            ui.same_line();
            image2.make(&im2, (w, h), size, &mut wdata).build(ui);

            let sub = |(u, v): (&u8, &u8)| (*u as f32 - *v as f32).abs() as u8;
            let im3 = im1.iter().zip(&im2).map(sub).collect::<Vec<_>>();
            ui.same_line();
            image3.make(&im3, (w, h), size, &mut wdata).build(ui);
        });
    });
}
