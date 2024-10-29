use project::*;

const OUTPUT_FOLDER: &str = "output";

fn main() {
    let aspects = [[4, 3], [16, 9]];
    let mut base_px = 100;
    let mut aspect_idx = 0;

    let cap = match env::consts::OS {
        "linux" => videoio::CAP_V4L,
        _ => videoio::CAP_ANY,
    };
    let mut cameras = [
        videoio::VideoCapture::new(0, cap).expect("could not read camera 0"),
        videoio::VideoCapture::new(2, cap).expect("could not read camera 1"),
    ];
    for cam in &mut cameras {
        cam.set(videoio::CAP_PROP_FPS, 30.)
            .expect("could not set fps");
    }

    let mut feeds = [
        image::Image::default(),
        image::Image::default(),
        image::Image::default(),
    ];

    let mut shift = [190, 30];
    //let mut homography = Mat::default();
    let mut writer: Option<videoio::VideoWriter> = None;

    window::begin(|renderer, ui| {
        let aspect = aspects[aspect_idx];
        let img_size = Size::new(base_px * aspect[0], base_px * aspect[1]);

        for (n, camera) in cameras.iter_mut().enumerate() {
            camera
                .read(&mut feeds[n].mat)
                .expect(&format!("could not read from camera {}", n));
        }

        if shift.iter().any(|i| *i != 0) {
            let size = feeds[1].mat.clone().size().unwrap();
            imgproc::warp_affine_def(
                &feeds[1].mat.clone(),
                &mut feeds[1].mat,
                &Mat::from_slice_2d(&[[1., 0., shift[0] as f32], [0., 1., -shift[1] as f32]])
                    .unwrap(),
                size,
            )
            .expect("could not warp");
        }
        //if homography.rows() > 0 {
        //    let clone = feeds[1].mat.clone();
        //    imgproc::warp_perspective_def(
        //        &clone,
        //        &mut feeds[1].mat,
        //        &homography,
        //        clone.size().unwrap(),
        //    )
        //    .expect("could not warp");
        //}

        {
            let mut float1 = Mat::default();
            let mut float2 = Mat::default();
            feeds[0].mat.convert_to_def(&mut float1, CV_32FC3).unwrap();
            feeds[1].mat.convert_to_def(&mut float2, CV_32FC3).unwrap();
            subtract_def(&float1, &float2, &mut feeds[2].mat).expect("could not subtract");
            abs(&feeds[2].mat.clone())
                .expect("could not absolute")
                .to_mat()
                .expect("could not be mat")
                .convert_to_def(&mut feeds[2].mat, CV_8UC3)
                .expect("could not convert to u8");
        }

        for (n, feed) in feeds.iter_mut().enumerate() {
            imgproc::resize_def(&feed.mat.clone(), &mut feed.mat, img_size)
                .expect("could not resize");
            ui.window(format!("Camera {}", n + 1))
                .content_size(img_size.to_array())
                .build(|| {
                    feed.make(renderer).build(ui);
                });
        }

        if let Some(writer) = writer.as_mut() {
            let mut rgb = Mat::default();
            imgproc::cvt_color_def(&feeds[2].mat, &mut rgb, imgproc::COLOR_BGR2RGB)
                .expect("could not to rgb");
            writer.write(&rgb).expect("could not write");
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

                ui.slider("hor", 0, 400, &mut shift[0]);
                ui.slider("ver", 0, 400, &mut shift[1]);
                //if ui.button("calibrate") {
                //    calibrate::get_homography(&feeds[0].mat, &feeds[1].mat, &mut homography);
                //};
                //ui.same_line();
                //if ui.button("reset calibration") {
                //    homography = Mat::default();
                //};

                ui.text("save:");
                for i in 1..=3 {
                    ui.same_line();
                    if ui.button(format!("feed {}", i)) {
                        let mut rgb = Mat::default();
                        imgproc::cvt_color_def(&feeds[i - 1].mat, &mut rgb, imgproc::COLOR_BGR2RGB)
                            .expect("could not to rgb");
                        imgcodecs::imwrite_def(&get_save_filepath(&format!("f{}.png", i)), &rgb)
                            .expect("could not save png");
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
