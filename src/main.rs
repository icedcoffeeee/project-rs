use imgui::TableColumnSetup;
use project::*;

const OUTPUT_FOLDER: &str = "output";
const DETECTION: bool = true;
const DUAL_CAMERA: bool = true;

fn main() {
    let aspects = [[4, 3], [16, 9]];
    let mut base_px = 100;
    let mut aspect = &aspects[0];

    let [ind, cap] = match env::consts::OS {
        "linux" => [2, videoio::CAP_V4L],
        _ => [1, videoio::CAP_ANY],
    };
    let mut cameras = if DUAL_CAMERA {
        [
            videoio::VideoCapture::new(0, cap).unwrap(),
            videoio::VideoCapture::new(ind, cap).unwrap(),
        ]
    } else {
        [
            videoio::VideoCapture::new(0, cap).unwrap(),
            videoio::VideoCapture::default().unwrap(),
        ]
    };
    for cam in &mut cameras {
        cam.set(
            videoio::CAP_PROP_FOURCC,
            videoio::VideoWriter::fourcc('m', 'j', 'p', 'g').unwrap() as _,
        )
        .unwrap();
    }
    cameras[0]
        .set(videoio::CAP_PROP_WB_TEMPERATURE, 4600.)
        .unwrap();

    let mut feeds = [
        image::Image::default(),
        image::Image::default(),
        image::Image::default(),
    ];

    let mut shift = [0, 0];
    let mut window = 100;
    let mut writer: Option<videoio::VideoWriter> = None;

    let (t_feed, r_feed) = mpsc::channel();
    let (t_detections, r_detections) = mpsc::channel();
    let mut detections = None;
    let mut first_send = true;
    if DETECTION {
        thread::spawn(move || detection::initialize_thread(r_feed, t_detections));
    }

    let mut classes: Option<Vec<String>> = None;
    let mut shift_a = [0, 0];
    let mut init = true;

    window::create(|ui, renderer| {
        if init && !feeds[0].mat.empty() {
            init = false;
            calibrate::get_shift(&feeds[0].mat, &feeds[1].mat, window, &mut shift);
            window = 40;
            shift_a[0] = -85;
        }
        let img_size = Size::new(base_px * aspect[0], base_px * aspect[1]);

        if DUAL_CAMERA {
            for n in 0..cameras.len() {
                if !cameras[n].read(&mut feeds[n].mat).unwrap() {
                    return;
                }
            }
        } else {
            for n in 0..cameras.len() {
                if !cameras[0].read(&mut feeds[n].mat).unwrap() {
                    return;
                }
            }
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

        if DETECTION {
            if first_send {
                first_send = false;
                let _ = t_feed.send(feeds[0].mat.clone());
            }
            if let Ok(det) = r_detections.try_recv() {
                detections = Some(det);
                let _ = t_feed.send(feeds[0].mat.clone());
            }
            if let Some(ref detections) = detections {
                detection::draw_bounding_boxes(&mut feeds[2].mat, &detections, &mut classes);
            }
        }

        let size_b = feeds[0].mat.size().unwrap();
        let mini_b = Rect::new(
            (size_b.width - window) / 2,
            (size_b.height - window) / 2,
            window,
            window,
        );
        let size_a = feeds[0].mat.size().unwrap();
        let mini_a = Rect::new(
            (size_a.width - window) / 2 + shift_a[0],
            (size_a.height - window) / 2 + shift_a[1],
            window,
            window,
        );
        for feed in &mut feeds{
            imgproc::rectangle_def(&mut feed.mat, mini_a, [0., 0., 0., 255.].into()).unwrap();
            imgproc::rectangle_def(&mut feed.mat, mini_b, [0., 0., 0., 255.].into()).unwrap();
        }

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

                if let Some(_) =
                    ui.begin_combo("aspect ratio", format!("{}x{}", aspect[0], aspect[1]))
                {
                    for a in &aspects {
                        if aspect == a {
                            ui.set_item_default_focus();
                        }
                        if ui
                            .selectable_config(format!("{}x{}", a[0], a[1]))
                            .selected(aspect == a)
                            .build()
                        {
                            aspect = a;
                        }
                    }
                };

                ui.text("calibration:");
                ui.slider("horizontal_a", -400, 400, &mut shift_a[0]);
                ui.slider("vertical_a", -400, 400, &mut shift_a[1]);
                ui.slider("horizontal_b", -400, 400, &mut shift[0]);
                ui.slider("vertical_b", -400, 400, &mut shift[1]);
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

                for (a, mini) in [mini_a, mini_b].into_iter().enumerate() {
                    ui.new_line();
                    ui.text(["alive", "dead"][a]);
                    if let Some(_) = ui.begin_table_header(
                        "channel readings",
                        [
                            TableColumnSetup::new("feeds"),
                            TableColumnSetup::new("red"),
                            TableColumnSetup::new("green"),
                            TableColumnSetup::new("blue"),
                        ],
                    ) {
                        for (n, feed) in feeds.iter().enumerate() {
                            let mut bgr = Vector::<Mat>::new();
                            split(&feed.mat.roi(mini).unwrap(), &mut bgr).unwrap();
                            ui.table_next_column();
                            ui.text(format!("camera {}", n + 1));
                            for i in 0..3 {
                                ui.table_next_column();
                                ui.text(format!(
                                    "{:.2}",
                                    mean_def(&bgr.get(2 - i).unwrap()).unwrap()[0]
                                ));
                            }
                        }
                    }
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
