use project::*;

const OUTPUT_FOLDER: &str = "output";

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
    let mut writer: Option<videoio::VideoWriter> = None;

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

        let mut f1 = Mat::default();
        let mut f2 = Mat::default();
        feeds[0].mat.convert_to_def(&mut f1, CV_32FC3)?;
        feeds[1].mat.convert_to_def(&mut f2, CV_32FC3)?;
        subtract_def(&f1, &f2, &mut feeds[2].mat)?;
        abs(&feeds[2].mat.clone())?
            .to_mat()?
            .convert_to_def(&mut feeds[2].mat, CV_8UC3)?;

        for (n, feed) in feeds.iter_mut().enumerate() {
            imgproc::resize_def(&feed.mat.clone(), &mut feed.mat, img_size)?;
            ui.window(format!("Camera {}", n + 1))
                .content_size(img_size.to_array())
                .build(|| {
                    feed.make(renderer).build(ui);
                });
        }
        if let Some(writer) = writer.as_mut() {
            writer.write(&feeds[2].mat)?;
        }

        ui.window("Control Panel")
            .content_size([500., 500.])
            .build(|| {
                ui.slider("image base size", 1, 400, &mut base_px);

                if ui
                    .begin_combo("aspect ratio", format!("{:?}", aspect))
                    .is_some()
                {
                    for (n, aspect) in aspects.iter().enumerate() {
                        if ui.selectable(format!("{:?}", aspect)) {
                            aspect_idx = n;
                        };
                        if aspect_idx == n {
                            ui.set_item_default_focus();
                        }
                    }
                };

                if ui.button("calibrate") {
                    calibrate::get_homography(&feeds[0].mat, &feeds[1].mat, &mut homography)
                        .unwrap();
                };
                ui.same_line();
                if ui.button("reset calibration") {
                    homography = Mat::default();
                };
                if ui.button("save feed 1") {
                    imgcodecs::imwrite_def(&get_save_filename("f2.png"), &feeds[1].mat).unwrap();
                };
                ui.same_line();
                if ui.button("save feed 3") {
                    imgcodecs::imwrite_def(&get_save_filename("f3.png"), &feeds[2].mat).unwrap();
                };
                if ui.button(format!(
                    "{} recording",
                    if writer.is_none() { "start" } else { "stop" }
                )) {
                    if writer.is_none() {
                        writer = Some(
                            videoio::VideoWriter::new(
                                &get_save_filename("out.mp4"),
                                videoio::VideoWriter::fourcc('m', 'p', '4', 'v').unwrap(),
                                30.,
                                feeds[2].mat.size().unwrap(),
                                true,
                            )
                            .unwrap(),
                        );
                    } else {
                        writer = None;
                    }
                }
            });
        return Ok(());
    });

    return Ok(());
}

fn get_save_filepath(name: &str) -> String {
    let mut i = 0;
    let mut filepath = path::PathBuf::new();

    filepath.push(OUTPUT_FOLDER);
    if !filepath.exists() {
        fs::create_dir(filepath.clone()).expect("could not create dir");
    }

    for item in filepath.read_dir().expect("could not read dir") {
        if let Some((num_str, _)) = item
            .unwrap()
            .file_name()
            .into_string()
            .unwrap()
            .split_once("-")
        {
            let num: u32 = num_str.parse().expect("unexpected filename");
            if num > i {
                i = num;
            };
        };
    }

    filepath.push(format!("{}-{}", i + 1, name));
    filepath.to_str().unwrap().to_string()
}
