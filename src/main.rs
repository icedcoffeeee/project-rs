use project::*;

const OUTPUT_FOLDER: &str = "output";

fn main() {
    let aspects = [[4, 3], [16, 9]];
    let mut base_px = 100;
    let mut aspect_idx = 0;

    let [_ind, cap] = match env::consts::OS {
        "linux" => [2, videoio::CAP_V4L],
        _ => [1, videoio::CAP_ANY],
    };
    let mut cameras = [
        videoio::VideoCapture::new(0, cap).unwrap(),
        //videoio::VideoCapture::new(ind, cap).unwrap(),
    ];
    for cam in &mut cameras {
        cam.set(videoio::CAP_PROP_FPS, 30.).unwrap();
    }

    let mut feeds = [
        image::Image::default(),
        image::Image::default(),
        image::Image::default(),
    ];

    let homo = Mat::default();
    let mut shift = [0, 0];
    let mut writer: Option<videoio::VideoWriter> = None;

    window::begin(|renderer, ui| {
        let aspect = aspects[aspect_idx];
        let img_size = Size::new(base_px * aspect[0], base_px * aspect[1]);

        //for (n, camera) in cameras.iter_mut().enumerate() {
        //    camera
        //        .read(&mut feeds[n].mat)
        //        .unwrap();
        //}
        cameras[0].read(&mut feeds[0].mat).unwrap();
        cameras[0].read(&mut feeds[1].mat).unwrap();

        if shift.iter().any(|i| *i != 0) {
            let size = feeds[1].mat.size().unwrap();
            let m = Mat::from_slice_2d(&[[1., 0., shift[0] as f32], [0., 1., -shift[1] as f32]])
                .unwrap();
            imgproc::warp_affine_def(&feeds[1].mat.clone(), &mut feeds[1].mat, &m, size).unwrap();
        }

        if homo.size().unwrap().area() != 0 {
            let size = feeds[2].mat.size().unwrap();
            imgproc::warp_perspective_def(&feeds[2].mat.clone(), &mut feeds[2].mat, &homo, size)
                .unwrap();
        }

        absdiff(
            &feeds[0].mat.clone(),
            &feeds[1].mat.clone(),
            &mut feeds[2].mat,
        )
        .unwrap();

        for (n, feed) in feeds.iter_mut().enumerate() {
            imgproc::resize_def(&feed.mat.clone(), &mut feed.mat, img_size).unwrap();
            ui.window(format!("Camera {}", n + 1))
                .content_size(img_size.to_array())
                .build(|| {
                    feed.make(renderer).build(ui);
                });
        }

        if let Some(writer) = writer.as_mut() {
            let mut rgb = Mat::default();
            imgproc::cvt_color_def(&feeds[2].mat, &mut rgb, imgproc::COLOR_BGR2RGB).unwrap();
            writer.write(&rgb).unwrap();
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

                ui.slider("hor", -400, 400, &mut shift[0]);
                ui.slider("ver", -400, 400, &mut shift[1]);

                if ui.button("calibrate") {
                    //calibrate::get_homography(&feeds[0].mat, &feeds[1].mat, &mut homo);
                    calibrate::get_shift(&feeds[0].mat, &feeds[1].mat, &mut shift);
                };

                ui.text("save:");
                for i in 1..=3 {
                    ui.same_line();
                    if ui.button(format!("feed {}", i)) {
                        let mut rgb = Mat::default();
                        imgproc::cvt_color_def(&feeds[i - 1].mat, &mut rgb, imgproc::COLOR_BGR2RGB)
                            .unwrap();
                        imgcodecs::imwrite_def(&get_save_filepath(&format!("f{}.png", i)), &rgb)
                            .unwrap();
                    };
                }

                ui.text("recording:");
                ui.same_line();
                if writer.is_none() {
                    if ui.button("start") {
                        writer = Some(
                            videoio::VideoWriter::new(
                                &get_save_filepath("out.mp4"),
                                videoio::VideoWriter::fourcc('a', 'v', 'c', '1').unwrap(),
                                15.,
                                feeds[2].mat.size().unwrap(),
                                true,
                            )
                            .unwrap(),
                        );
                    }
                } else if ui.button("stop") {
                    writer = None;
                }
            });
    });
}

fn get_save_filepath(name: &str) -> String {
    let mut i = 0;
    let mut filepath = path::PathBuf::new();

    filepath.push(OUTPUT_FOLDER);
    if !filepath.exists() {
        fs::create_dir(filepath.clone()).unwrap();
    }

    for item in filepath.read_dir().unwrap() {
        if let Some((num_str, _)) = item
            .unwrap()
            .file_name()
            .into_string()
            .unwrap()
            .split_once("-")
        {
            let num: u32 = num_str.parse().unwrap();
            if num > i {
                i = num;
            };
        };
    }

    filepath.push(format!("{}-{}", i + 1, name));
    filepath.to_str().unwrap().to_string()
}
