use project::*;

const DETECTION: bool = true;
const DUAL_CAMERA: bool = false;

fn main() -> Result<()> {
    let aspects = [[4, 3], [16, 9]];
    let mut base_px = 60;
    let mut aspect = &aspects[0];

    let [ind, cap] = match env::consts::OS {
        "linux" => [2, videoio::CAP_V4L],
        _ => [1, videoio::CAP_ANY],
    };
    let mut cameras = if DUAL_CAMERA {
        [
            videoio::VideoCapture::new(0, cap)?,
            videoio::VideoCapture::new(ind, cap)?,
        ]
    } else {
        [
            videoio::VideoCapture::new(0, cap)?,
            videoio::VideoCapture::default()?,
        ]
    };
    let mjpg = videoio::VideoWriter::fourcc('m', 'j', 'p', 'g')? as f64;
    for cam in &mut cameras {
        cam.set(videoio::CAP_PROP_FOURCC, mjpg)?;
    }

    let mut feeds = [
        image::Image::default(),
        image::Image::default(),
        image::Image::default(),
    ];

    let mut shift_camera_2 = [0, 0];
    let mut shift_left_window = [0, 0];
    let mut window_size = 100;
    let mut threshold = 10.;
    let mut writer: Option<videoio::VideoWriter> = None;

    let (t_feed, r_feed) = mpsc::channel();
    let (t_detections, r_detections) = mpsc::channel();
    let mut detections = None;
    let mut first_send = true;
    let mut classes: Option<Vec<String>> = None;
    if DETECTION {
        thread::spawn(move || detection::initialize_thread(r_feed, t_detections));
    }

    window::create(|ui, renderer| {
        let img_size = Size::new(base_px * aspect[0], base_px * aspect[1]);

        for n in 0..cameras.len() {
            if !cameras[[0, n][DUAL_CAMERA as usize]].read(&mut feeds[n].mat)? {
                return Ok(());
            }
        }

        if shift_camera_2.iter().any(|i| *i != 0) {
            let size = feeds[1].mat.size()?;
            let m = Mat::from_slice_2d(&[
                [1., 0., -shift_camera_2[0] as f32],
                [0., 1., -shift_camera_2[1] as f32],
            ])?;
            imgproc::warp_affine_def(&feeds[1].mat.clone(), &mut feeds[1].mat, &m, size)?;
        }

        absdiff(
            &feeds[0].mat.clone(),
            &feeds[1].mat.clone(),
            &mut feeds[2].mat,
        )?;
        imgproc::threshold(
            &feeds[2].mat.clone(),
            &mut feeds[2].mat,
            threshold,
            255.,
            imgproc::THRESH_TOZERO,
        )?;

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

        let size = feeds[0].mat.size()?;
        let mini_center = Rect::new(
            (size.width - window_size) / 2,
            (size.height - window_size) / 2,
            window_size,
            window_size,
        );
        let mini_left = Rect::new(
            (size.width - window_size) / 2 + shift_left_window[0],
            (size.height - window_size) / 2 + shift_left_window[1],
            window_size,
            window_size,
        );
        for feed in &mut feeds {
            imgproc::rectangle_def(&mut feed.mat, mini_left, [0., 0., 0., 255.].into())?;
            imgproc::rectangle_def(&mut feed.mat, mini_center, [0., 0., 0., 255.].into())?;
        }

        for (n, feed) in feeds.iter_mut().enumerate() {
            ui.window(["left", "right", "subtracted"][n])
                .content_size(img_size.to_array())
                .build(|| {
                    feed.make(renderer, img_size).build(ui);
                });
        }
        {
            let mut channels = Vector::<Mat>::new();
            split(&feeds[2].mat, &mut channels)?;
            for (n, channel) in channels.iter().enumerate() {
                let mut feed = image::Image::default();
                channel.assign_to_def(&mut feed.mat)?;

                ui.window(["blue", "green", "red"][n])
                    .content_size(img_size.to_array())
                    .build(|| {
                        feed.make(renderer, img_size).build(ui);
                    });
            }
        }

        if let Some(writer) = writer.as_mut() {
            writer.write(&feeds[2].mat)?;
        }

        ui.window("Control Panel")
            .content_size([500., 500.])
            .build::<Result<()>, _>(|| {
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
                ui.slider("threshold", 0., 255., &mut threshold);

                ui.text("calibration:");
                ui.slider("camera 2 shift x", -400, 400, &mut shift_camera_2[0]);
                ui.slider("camera 2 shift y", -400, 400, &mut shift_camera_2[1]);
                ui.slider("window size", 1, 200, &mut window_size);
                if ui.button("auto calibrate") {
                    calibrate::get_shift(
                        &feeds[0].mat,
                        &feeds[1].mat,
                        window_size,
                        &mut shift_camera_2,
                    );
                };
                ui.same_line();
                if ui.button("reset") {
                    shift_camera_2 = [0, 0];
                };

                ui.text("save:");
                for n in 0..3 {
                    ui.same_line();
                    if ui.button(format!("feed {}", n + 1)) {
                        imgcodecs::imwrite_def(
                            &utils::get_save_filepath(&format!("f{}.png", n + 1)),
                            &feeds[n].mat,
                        )?;
                    };
                }

                ui.text("recording:");
                ui.same_line();
                if writer.is_none() {
                    if ui.button("start") {
                        writer = Some(videoio::VideoWriter::new(
                            &utils::get_save_filepath("out.mp4"),
                            videoio::VideoWriter::fourcc('a', 'v', 'c', '1')?,
                            15.,
                            feeds[2].mat.size()?,
                            true,
                        )?);
                    }
                } else if ui.button("stop") {
                    writer = None;
                }

                ui.slider("left window x", -400, 400, &mut shift_left_window[0]);
                ui.slider("left window y", -400, 400, &mut shift_left_window[1]);
                for (a, mini) in [mini_left, mini_center].into_iter().enumerate() {
                    ui.new_line();
                    ui.text(["left", "center"][a]);
                    if let Some(_) = ui.begin_table_header(
                        "channel readings",
                        ["feeds", "red", "green", "blue"].map(|h| im::TableColumnSetup::new(h)),
                    ) {
                        for (n, feed) in feeds.iter().enumerate() {
                            let mut bgr = Vector::<Mat>::new();
                            split(&feed.mat.roi(mini)?, &mut bgr)?;
                            ui.table_next_column();
                            ui.text(format!("camera {}", n + 1));
                            for i in 0..3 {
                                ui.table_next_column();
                                ui.text(format!("{:.2}", mean_def(&bgr.get(2 - i)?)?[0]));
                            }
                        }
                    }
                }
                Ok(())
            });
        Ok(())
    });
    Ok(())
}
