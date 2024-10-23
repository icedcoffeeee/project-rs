use project::*;

fn main() -> Result<()> {
    let aspects = [[4, 3], [16, 9]];
    let mut base_px = 100;
    let mut aspect_idx = 0;

    let mut cameras = [
        videoio::VideoCapture::new(0, videoio::CAP_ANY)?,
        //videoio::VideoCapture::new(1, videoio::CAP_ANY)?,
    ];
    for cam in &mut cameras {
        cam.set(videoio::CAP_PROP_FPS, 30.)?;
    }

    let mut feeds = [
        image::Image::default(),
        image::Image::default(),
        image::Image::default(),
    ];

    let mut homography = Mat::default();

    window::begin(|renderer, ui| {
        let aspect = aspects[aspect_idx];
        let img_size = Size::new(base_px * aspect[0], base_px * aspect[1]);

        //for (n, camera) in cameras.iter_mut().enumerate() {
        //    camera.read(&mut feeds[n].mat)?;
        //}
        cameras[0].read(&mut feeds[0].mat)?;
        cameras[0].read(&mut feeds[1].mat)?;

        if homography.rows() > 0 {
            let clone = feeds[1].mat.clone();
            imgproc::warp_perspective_def(&clone, &mut feeds[1].mat, &homography, clone.size()?)?;
        }

        subtract_def(
            &feeds[0].mat.clone(),
            &feeds[1].mat.clone(),
            &mut feeds[2].mat,
        )?;

        for (n, feed) in feeds.iter_mut().enumerate() {
            imgproc::resize_def(&feed.mat.clone(), &mut feed.mat, img_size)?;
            ui.window(format!("Camera {}", n + 1))
                .content_size(img_size.to_array())
                .build(|| {
                    feed.make(renderer).build(ui);
                });
        }

        ui.window("Control Panel")
            .content_size([500., 500.])
            .build(|| {
                ui.slider("Image base size", 1, 400, &mut base_px);

                if let Some(_) = ui.begin_combo("Aspect ratio", format!("{:?}", aspect)) {
                    for (n, aspect) in aspects.iter().enumerate() {
                        if ui.selectable(format!("{:?}", aspect)) {
                            aspect_idx = n;
                        };
                        if aspect_idx == n {
                            ui.set_item_default_focus();
                        }
                    }
                };

                if ui.button("Calibrate") {
                    calibrate(&feeds[0].mat, &feeds[1].mat, &mut homography).unwrap();
                };
                ui.same_line();
                if ui.button("Image") {
                    save_pic(&feeds[0].mat);
                };
                ui.same_line();
                if ui.button("Subtract Captured") {
                    let m1 = imgcodecs::imread_def("output/pol-0.png").unwrap();
                    let m2 = imgcodecs::imread_def("output/pol-1.png").unwrap();
                    let mut m3 = Mat::default();
                    subtract_def(&m1, &m2, &mut m3).unwrap();
                    save_pic(&m3);
                };
            });
        Ok(())
    });

    Ok(())
}

fn calibrate(camera1: &Mat, camera2: &Mat, homo: &mut Mat) -> Result<()> {
    let mut orb = features2d::ORB::create_def()?;
    let mut mask = Mat::default();

    let (mut keypoints1, mut descriptors1) = (Vector::new(), Mat::default());
    let (mut keypoints2, mut descriptors2) = (Vector::new(), Mat::default());
    orb.detect_and_compute_def(camera1, &mask, &mut keypoints1, &mut descriptors1)?;
    orb.detect_and_compute_def(camera2, &mask, &mut keypoints2, &mut descriptors2)?;

    let mut matcher = features2d::DescriptorMatcher::create("BruteForce-Hamming")?;
    let mut matches = Vector::new();
    matcher.add(&descriptors1)?;
    matcher.match__def(&descriptors2, &mut matches)?;

    let mut matches = matches.to_vec();
    matches.sort_by(|x, y| x.distance.total_cmp(&y.distance));
    if matches.len() < 4 {
        println!("Not enough matches");
        return Ok(());
    } else {
        // cut the last 10%
        for _ in 0..(matches.len() / 10) {
            matches.pop();
        }
    }

    let (mut points1, mut points2) = (Vec::new(), Vec::new());
    for match_ in matches {
        points1.push(keypoints1.get(match_.train_idx as usize)?.pt());
        points2.push(keypoints2.get(match_.query_idx as usize)?.pt());
    }
    *homo = calib3d::find_homography(
        &Mat::from_slice(points1.as_slice())?,
        &Mat::from_slice(points2.as_slice())?,
        &mut mask,
        calib3d::RANSAC,
        0.5,
    )?;
    println!("{:?}", homo.to_vec_2d::<VecN<f64, 1>>()?);
    Ok(())
}

fn save_pic(mat: &Mat) {
    let mut i = 0;
    if !fs::metadata("output/").is_ok() {
        fs::create_dir("output").unwrap();
    }
    let mut filename;
    while {
        filename = format!("output/pol-{}.png", i);
        fs::metadata(&filename).is_ok()
    } {
        i += 1;
    }
    imgcodecs::imwrite_def(&filename, mat).unwrap();
}
