use project::*;

fn main() -> Result<()> {
    let mut base_px = 100;
    let aspects = [[4, 3], [16, 9]];
    let mut aspect_idx = 0;

    let mut cameras = [
        VideoCapture::new(0, CAP_ANY)?,
        VideoCapture::new(1, CAP_ANY)?,
    ];

    let mut feeds = [
        image::Image::default(),
        image::Image::default(),
        image::Image::default(),
    ];

    window::begin(|renderer, ui| {
        let aspect = aspects[aspect_idx];
        let img_size = Size::new(base_px * aspect[0], base_px * aspect[1]);

        for (n, camera) in cameras.iter_mut().enumerate() {
            camera.read(&mut feeds[n].mat)?;
        }

        add_def(
            &feeds[0].mat.clone(),
            &feeds[1].mat.clone(),
            &mut feeds[2].mat,
        )?;

        for (n, feed) in feeds.iter_mut().enumerate() {
            resize_def(&feed.mat.clone(), &mut feed.mat, img_size)?;
            ui.window(format!("Camera {}", n))
                .content_size(img_size.to_array())
                .build(|| {
                    feed.make(renderer).build(ui);
                });
        }

        ui.window("Control Panel")
            .size([500., 1000.], imgui::Condition::Once)
            .build(|| {
                ui.slider("Image base size", 1, 400, &mut base_px);

                if let Some(_) = ui.begin_combo("Aspect ratio", format!("{:#?}", aspect)) {
                    for (n, aspect) in aspects.iter().enumerate() {
                        if ui.selectable(format!("{:#?}", aspect)) {
                            aspect_idx = n;
                        };
                        if aspect_idx == n {
                            ui.set_item_default_focus();
                        }
                    }
                };
            });
        Ok(())
    });

    Ok(())
}
