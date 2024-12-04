use project::*;

const DETECTION: bool = false;
const DUAL_CAMERA: bool = true;

#[derive(Default)]
struct State {
    base_px: i32,
    win_size: i32,
    win_shift: [i32; 2],
    cam_shift: [i32; 2],
    writer: Option<videoio::VideoWriter>,
}

type Cameras = [videoio::VideoCapture; 2];
type Feeds = [image::Image; 3];

fn main() {
    let mut s = State::default();
    s.win_size = 100;
    s.base_px = 80;

    let mut cameras = get_cameras();
    let mut feeds: Feeds = Default::default();

    let mut channels = Channels::new();
    let mut classes: Option<Classes> = None;
    if DETECTION {
        detection::initialize_thread(channels.channel_there);
    }

    window::create(|ui, renderer| {
        read_cameras(&mut cameras, &mut feeds);

        let img_size = Size::new(s.base_px * 4, s.base_px * 3);
        let [f0, f1, f2] = &mut feeds;

        flip(&f0.mat.clone(), &mut f0.mat, -1).unwrap();
        flip(&f1.mat.clone(), &mut f1.mat, 0).unwrap();
        shift_cameras(&s, &mut f1.mat);

        {
            // DoLP = S1 / S0 = (I90 - I0) / (I90 + I0)
            let mut sub = Mat::default();
            absdiff(&f0.mat, &f1.mat, &mut sub).unwrap();
            divide2_def(&sub, &f1.mat, &mut f2.mat).unwrap();
        }

        if DETECTION {
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

fn get_cameras() -> Cameras {
    use env::consts::OS;
    use videoio::VideoCapture as cam;
    use videoio::{CAP_ANY, CAP_V4L};

    const L: &str = "linux";
    const D: bool = true;
    match (OS, DUAL_CAMERA) {
        (L, D) => [cam::new(0, CAP_V4L).unwrap(), cam::new(2, CAP_V4L).unwrap()],
        (L, _) => [cam::new(0, CAP_V4L).unwrap(), cam::default().unwrap()],
        (_, D) => [cam::new(0, CAP_ANY).unwrap(), cam::new(1, CAP_ANY).unwrap()],
        (_, _) => [cam::new(0, CAP_ANY).unwrap(), cam::default().unwrap()],
    }
}

fn read_cameras(cameras: &mut Cameras, feeds: &mut Feeds) {
    for n in 0..cameras.len() {
        cameras[[0, n][DUAL_CAMERA as usize]]
            .read(&mut feeds[n].mat)
            .unwrap();
    }
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
    for (n, feed) in feeds.iter_mut().enumerate() {
        ui.window(["left", "right", "subtracted"][n])
            .content_size(img_size.to_array())
            .build(|| {
                feed.make(renderer, img_size).build(ui);
            });
    }
    {
        let mut channels = Vector::<Mat>::new();
        split(&feeds[2].mat, &mut channels).unwrap();
        for (n, channel) in channels.iter().enumerate() {
            let mut feed = image::Image::default();
            channel.assign_to_def(&mut feed.mat).unwrap();

            ui.window(["blue", "green", "red"][n])
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
            let filepath = utils::get_save_filepath("out.mp4");
            let avc1 = videoio::VideoWriter::fourcc('a', 'v', 'c', '1').unwrap();
            let (fps, size) = (15., feeds[2].mat.size().unwrap());
            let writer = videoio::VideoWriter::new(&filepath, avc1, fps, size, true).unwrap();
            if ui.button("start") {
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
            for (n, feed) in feeds.iter().enumerate() {
                let mut bgr = Vector::<Mat>::new();
                split(&feed.mat.roi(*mini).unwrap(), &mut bgr).unwrap();
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
