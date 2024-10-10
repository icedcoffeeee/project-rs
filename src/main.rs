use project::*;

fn main() -> Result<()> {
    let aspects = [[4, 3], [16, 9]];
    let mut base_px = 100;
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

    let mut offset_x = 0;
    let mut offset_y = 0;

    window::begin(|renderer, ui| {
        let aspect = aspects[aspect_idx];
        let img_size = Size::new(base_px * aspect[0], base_px * aspect[1]);

        for (n, camera) in cameras.iter_mut().enumerate() {
            camera.read(&mut feeds[n].mat)?;
        }

        if offset_x > 0 || offset_y > 0 {
            let size = feeds[1].mat.size()?;
            let mat = Mat::from_slice_2d(&[[1, 0, offset_x], [0, 1, -offset_y]])?;
            warp_affine_def(&feeds[1].mat.clone(), &mut feeds[1].mat, &mat, size)?;
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

                let size = img_size.to_array();
                ui.slider("Offset X", 0, size[0] as i32, &mut offset_x);
                ui.slider("Offset Y", 0, size[1] as i32, &mut offset_y);
            });
        Ok(())
    });

    Ok(())
}
