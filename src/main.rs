use project::*;

const OUTPUT_FOLDER: &str = "output";

fn main() {
    let aspects = [[4, 3], [16, 9]];
    let mut base_px = 100;
    let mut aspect_idx = 0;

    let [ind, cap] = match env::consts::OS {
        "linux" => [2, videoio::CAP_V4L],
        _ => [1, videoio::CAP_ANY],
    };
    let mut cameras = [
        videoio::VideoCapture::new(0, cap).unwrap(),
        videoio::VideoCapture::new(ind, cap).unwrap(),
    ];
    for cam in &mut cameras {
        cam.set(videoio::CAP_PROP_FPS, 30.).unwrap();
    }

    let mut feeds = [
        image::Image::default(),
        image::Image::default(),
        image::Image::default(),
    ];

    let mut shift = [0, 0];
    let mut window = 100;
    let mut writer: Option<videoio::VideoWriter> = None;

    let mut net = dnn::read_net("input/yolov3.weights", "input/yolov3.cfg", "").unwrap();
    let raw = fs::read("input/yolov3.txt").unwrap();
    let classes: Vec<&str> = str::from_utf8(raw.as_slice())
        .unwrap()
        .split("\n")
        .collect();

    window::begin(|renderer, ui| {
        let aspect = aspects[aspect_idx];
        let img_size = Size::new(base_px * aspect[0], base_px * aspect[1]);

        for (n, camera) in cameras.iter_mut().enumerate() {
            if !camera.read(&mut feeds[n].mat).unwrap() {
                return;
            };
        }

        if shift.iter().any(|i| *i != 0) {
            let size = feeds[1].mat.size().unwrap();
            let m = Mat::from_slice_2d(&[[1., 0., -shift[0] as f32], [0., 1., -shift[1] as f32]])
                .unwrap();
            imgproc::warp_affine_def(&feeds[1].mat.clone(), &mut feeds[1].mat, &m, size).unwrap();
        }

        absdiff(
            &feeds[0].mat.clone(),
            &feeds[1].mat.clone(),
            &mut feeds[2].mat,
        )
        .unwrap();

        {
            let size = feeds[0].mat.size().unwrap();
            imgproc::rectangle_def(
                &mut feeds[0].mat,
                Rect::new(
                    (size.width - window) / 2,
                    (size.height - window) / 2,
                    window,
                    window,
                ),
                [1., 0., 0., 1.].into(),
            )
            .unwrap();
        }

        yolo::detect(&mut feeds[0].mat, &mut net, &classes);

        for (n, feed) in feeds.iter_mut().enumerate() {
            ui.window(format!("Camera {}", n + 1))
                .content_size(img_size.to_array())
                .build(|| {
                    feed.make(renderer, img_size).build(ui);
                });
        }

        if let Some(writer) = writer.as_mut() {
            writer.write(&feeds[2].mat).unwrap();
        }

        ui.window("Control Panel")
            .content_size([500., 500.])
            .build(|| {
                ui.slider("image base size", 1, 400, &mut base_px);

                if ui
                    .begin_combo("aspect ratio", format!("{}x{}", aspect[0], aspect[1]))
                    .is_some()
                {
                    for (n, aspect) in aspects.iter().enumerate() {
                        if ui.selectable(format!("{}x{}", aspect[0], aspect[1])) {
                            aspect_idx = n;
                        };
                        if aspect_idx == n {
                            ui.set_item_default_focus();
                        }
                    }
                };

                ui.text("calibration:");
                ui.slider("horizontal", -400, 400, &mut shift[0]);
                ui.slider("vertical", -400, 400, &mut shift[1]);
                ui.slider("window size", 1, 200, &mut window);
                if ui.button("auto calibrate") {
                    calibrate::get_shift(&feeds[0].mat, &feeds[1].mat, window, &mut shift);
                };
                ui.same_line();
                if ui.button("reset") {
                    shift = [0, 0];
                };

                ui.text("save:");
                for n in 0..3 {
                    ui.same_line();
                    if ui.button(format!("feed {}", n + 1)) {
                        println!("{:?}", feeds[n].mat.size().unwrap());
                        imgcodecs::imwrite_def(
                            &get_save_filepath(&format!("f{}.png", n + 1)),
                            &feeds[n].mat,
                        )
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
            if let Ok(num) = num_str.parse() {
                if num > i {
                    i = num;
                }
            }
        };
    }

    filepath.push(format!("{}-{}", i + 1, name));
    filepath.to_str().unwrap().to_string()
}
