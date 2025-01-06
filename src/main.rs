use project::*;

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long, default_value_t = false)]
    detection: bool,
    #[arg(short, long, default_value_t = false)]
    camera: bool,
}

#[derive(Default)]
struct State {
    base_px: i32,
    win_size: i32,
    win_shift: [i32; 2],
    cam_shift: [i32; 2],
    writer: Option<videoio::VideoWriter>,
}

type Cameras = [videoio::VideoCapture; 2];
type Feeds = [image::Image; 5];

fn main() {
    let args = Args::parse();
    let mut s = State {
        base_px: 80,
        win_size: 35,
        win_shift: [-93, 0],
        cam_shift: [-138, -50],
        writer: None,
    };

    let mut cameras = get_cameras(!args.camera);
    let mut feeds: Feeds = Default::default();

    let mut channels = Channels::new();
    let mut classes: Option<Classes> = None;
    if args.detection {
        detection::initialize_thread(channels.channel_there);
    }

    window::create(|ui, renderer| {
        if !read_cameras(&mut cameras, &mut feeds, !args.camera) {
            return;
        };

        let img_size = Size::new(s.base_px * 4, s.base_px * 3);
        let [f0, f1, f2, f00, f10] = &mut feeds;

        if !args.camera {
            flip(&f0.mat.clone(), &mut f0.mat, -1).unwrap();
            flip(&f1.mat.clone(), &mut f1.mat, 0).unwrap();
        }
        shift_cameras(&s, &mut f1.mat);

        {
            // DoLP = S1 / S0 = (I90 - I0) / (I90 + I0)
            //let mut sub = Mat::default();
            if !&f00.mat.empty() && !f10.mat.empty() {
                absdiff(&f0.mat.clone(), &f00.mat, &mut f0.mat).unwrap();
                absdiff(&f1.mat.clone(), &f10.mat, &mut f1.mat).unwrap();
            }
            absdiff(&f0.mat, &f1.mat, &mut f2.mat).unwrap();
            //divide2(&f2.mat.clone(), &f1.mat, &mut f2.mat, 255., CV_8UC3).unwrap();
        }

        if args.detection {
            get_detections(
                &channels.channel_here,
                &mut channels.body,
                &mut channels.first_sent,
                &mut feeds,
                &mut classes,
            );
        }

        let mut mini: [Rect; 2] = Default::default();
        draw_rois(&s, &mut feeds, &mut mini);
        all_feed_windows(ui, renderer, &mut feeds, img_size);

        ui.window("Control Panel")
            .content_size([500., 500.])
            .build(|| control_panel(&ui, &mut s, &mut feeds, &mut mini));
    });
}

fn get_cameras(dual_camera: bool) -> Cameras {
    use env::consts::OS;
    use videoio::VideoCapture as cam;
    use videoio::{CAP_ANY, CAP_V4L};

    const L: &str = "linux";
    const D: bool = true;
    match (OS, dual_camera) {
        (L, D) => [cam::new(0, CAP_V4L).unwrap(), cam::new(2, CAP_V4L).unwrap()],
        (L, _) => [cam::new(0, CAP_V4L).unwrap(), cam::default().unwrap()],
        (_, D) => [cam::new(0, CAP_ANY).unwrap(), cam::new(1, CAP_ANY).unwrap()],
        (_, _) => [cam::new(0, CAP_ANY).unwrap(), cam::default().unwrap()],
    }
}

/// returns true if all cameras are read successfully
fn read_cameras(cameras: &mut Cameras, feeds: &mut Feeds, dual_camera: bool) -> bool {
    (0..cameras.len())
        .map(|n| {
            cameras[[0, n][dual_camera as usize]]
                .read(&mut feeds[n].mat)
                .unwrap()
        })
        .all(|x| x)
}

fn shift_cameras(s: &State, mat: &mut Mat) {
    if s.cam_shift.iter().any(|i| *i != 0) {
        let size = mat.size().unwrap();
        let m = Mat::from_slice_2d(&[
            [1., 0., -s.cam_shift[0] as f32],
            [0., 1., -s.cam_shift[1] as f32],
        ])
        .unwrap();
        imgproc::warp_affine_def(&mat.clone(), mat, &m, size).unwrap();
    }
}

fn get_detections(
    channel: &Channel<Mat, Detections>,
    detections: &mut Option<Detections>,
    first_sent: &mut bool,
    feeds: &mut Feeds,
    classes: &mut Option<Classes>,
) {
    if channel
        .send_on_receive(|det| {
            *detections = Some(det);
            feeds[0].mat.clone()
        })
        .is_err()
    {
        if !*first_sent {
            *first_sent = false;
            let _ = channel.0.send(feeds[0].mat.clone());
        }
    }
    if let Some(ref det) = detections {
        detection::draw(&mut feeds[2].mat, &det, classes);
    }
}

fn draw_rois(s: &State, feeds: &mut Feeds, minis: &mut [Rect; 2]) {
    let size = feeds[0].mat.size().unwrap();
    let (wsize, shift) = (s.win_size, s.win_shift);
    let [x, y] = [(size.width - wsize) / 2, (size.height - wsize) / 2];

    let mini_center = Rect::new(x, y, wsize, wsize);
    let mini_left = Rect::new(x + shift[0], y + shift[1], wsize, wsize);

    for feed in feeds {
        imgproc::rectangle_def(&mut feed.mat, mini_left, [0., 0., 0., 255.].into()).unwrap();
        imgproc::rectangle_def(&mut feed.mat, mini_center, [0., 0., 0., 255.].into()).unwrap();
    }

    *minis = [mini_left, mini_center];
}

fn all_feed_windows(
    ui: &window::Ui,
    renderer: &mut window::AutoRenderer,
    feeds: &mut Feeds,
    img_size: Size,
) {
    for n in 0..3 {
        ui.window(["left", "right", "subtracted"][n])
            .size([0., 0.], im::Condition::Always)
            .content_size(img_size.to_array())
            .build(|| {
                feeds[n].make(renderer, img_size).build(ui);
            });
    }
    {
        let mut channels = Vector::<Mat>::new();
        split(&feeds[2].mat, &mut channels).unwrap();
        for (n, channel) in channels.iter().enumerate() {
            let mut feed = image::Image::default();
            channel.assign_to_def(&mut feed.mat).unwrap();

            ui.window(["blue", "green", "red"][n])
                .size([0., 0.], im::Condition::Always)
                .content_size(img_size.to_array())
                .build(|| {
                    feed.make(renderer, img_size).build(ui);
                });
        }
    }
}

fn control_panel(ui: &&mut window::Ui, s: &mut State, feeds: &mut Feeds, minis: &mut [Rect; 2]) {
    ui.slider("image base size", 1, 400, &mut s.base_px);

    ui.text("calibration:");
    ui.slider("camera 2 shift x", -400, 400, &mut s.cam_shift[0]);
    ui.slider("camera 2 shift y", -400, 400, &mut s.cam_shift[1]);
    ui.slider("window size", 1, 200, &mut s.win_size);

    if ui.button("auto calibrate") {
        calibrate::get_shift(&feeds[0].mat, &feeds[1].mat, s.win_size, &mut s.cam_shift);
    };
    ui.same_line();
    if ui.button("reset") {
        s.cam_shift = [0, 0];
    };

    if ui.button("save null") {
        feeds[5 - 2].mat.set(feeds[0].mat.clone()).unwrap();
        feeds[5 - 1].mat.set(feeds[1].mat.clone()).unwrap();
    }
    ui.same_line();
    if ui.button("reset null") {
        feeds[5 - 2].mat.set(Mat::default()).unwrap();
        feeds[5 - 1].mat.set(Mat::default()).unwrap();
    }

    ui.text("save:");
    for n in 0..3 {
        ui.same_line();
        if ui.button(format!("feed {}", n + 1)) {
            imgcodecs::imwrite_def(
                &utils::get_save_filepath(&format!("f{}.png", n + 1)),
                &feeds[n].mat,
            )
            .unwrap();
        };
    }

    ui.text("recording:");
    ui.same_line();
    match s.writer {
        Some(ref mut w) => {
            w.write(&feeds[2].mat).unwrap();
            if ui.button("stop") {
                s.writer = None;
            }
        }
        None => {
            if ui.button("start") {
                let filepath = utils::get_save_filepath("out.mp4");
                let avc1 = videoio::VideoWriter::fourcc('a', 'v', 'c', '1').unwrap();
                let (fps, size) = (15., feeds[2].mat.size().unwrap());
                let writer = videoio::VideoWriter::new(&filepath, avc1, fps, size, true).unwrap();
                s.writer = Some(writer);
            }
        }
    }
    ui.slider("left window x", -400, 400, &mut s.win_shift[0]);
    ui.slider("left window y", -400, 400, &mut s.win_shift[1]);
    for (a, mini) in minis.into_iter().enumerate() {
        ui.new_line();
        ui.text(["left", "center"][a]);
        if let Some(_) = ui.begin_table_header(
            "channel readings",
            ["feeds", "red", "green", "blue"].map(|h| im::TableColumnSetup::new(h)),
        ) {
            for n in 0..3 {
                let mut bgr = Vector::<Mat>::new();
                split(&feeds[n].mat.roi(*mini).unwrap(), &mut bgr).unwrap();
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
}
