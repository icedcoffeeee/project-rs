use crate::*;
use mpsc::{Receiver, Sender};

const WEIGHTS_FILE: &str = "data/yolov3.weights";
const CONFIG_FILE: &str = "data/yolov3.cfg";
const CLASSES_FILE: &str = "data/yolov3.txt";

pub type Classes = Vec<String>;
pub type ClassID = i32;
pub type Score = f32;
pub type Detections = (Vector<ClassID>, Vector<Score>, Vector<Rect>);

pub type Channel<Here, There> = (Sender<Here>, Receiver<There>);
pub trait ChannelTrait<Here, There> {
    fn send_on_receive<SendFn: FnMut(There) -> Here>(&self, send_fn: SendFn) -> Result<(), &str>;
}
impl<Here, There> ChannelTrait<Here, There> for Channel<Here, There> {
    fn send_on_receive<SendFn: FnMut(There) -> Here>(
        &self,
        mut send_fn: SendFn,
    ) -> Result<(), &str> {
        let (send_from_here, receive_from_there) = self;
        let body = match receive_from_there.try_recv() {
            Ok(body) => body,
            _ => return Err("no body yet"),
        };
        let data = send_fn(body);
        let _ = send_from_here.send(data);
        Ok(())
    }
}

/// Here and There are relative to its creation!
/// Think of these as "data from here" and "data from there"
pub struct Channels<Here, There> {
    pub channel_here: Channel<Here, There>,
    pub channel_there: Channel<There, Here>,
    pub body: Option<There>,
    pub first_sent: bool,
}

impl<S, T> Channels<S, T> {
    pub fn new() -> Self {
        let (s1, r1) = mpsc::channel();
        let (s2, r2) = mpsc::channel();
        Channels {
            channel_here: (s1, r2),
            channel_there: (s2, r1),
            body: None,
            first_sent: false,
        }
    }
}

pub fn initialize_thread(channel: Channel<Detections, Mat>) {
    thread::spawn(move || {
        let mut model = dnn::DetectionModel::new(WEIGHTS_FILE, CONFIG_FILE).unwrap();
        let mut init = false;
        loop {
            let _ = channel.send_on_receive(|feed| {
                if !init {
                    init = true;
                    model.set_input_size(feed.size().unwrap()).unwrap();
                    model.set_input_scale(0.001.into()).unwrap();
                    model.set_input_mean([0.; 4].into()).unwrap();
                    model.set_input_swap_rb(true).unwrap();
                }
                let mut detections = Detections::default();
                let (ref mut class_ids, ref mut scores, ref mut rects) = &mut detections;
                let _ = model.detect_def(&feed, class_ids, scores, rects);
                detections
            });
        }
    });
}

pub fn draw(mat: &mut Mat, detections: &Detections, classes: &mut Option<Classes>) {
    if classes.is_none() {
        let raw = fs::read(CLASSES_FILE).unwrap();
        let string: Vec<String> = String::from_utf8(raw)
            .unwrap()
            .split("\n")
            .map(|x| x.to_string())
            .collect();
        *classes = Some(string);
    }
    let classes = classes.as_ref().unwrap();

    let (class_ids, scores, rects) = detections;
    let mut indices = Vector::<i32>::new();
    dnn::nms_boxes_def(&rects, &scores, 0.5, 0.1, &mut indices).unwrap();

    for i in indices {
        let rect: Rect = rects.to_vec()[i as usize];
        let label = classes[class_ids.to_vec()[i as usize] as usize].as_str();
        let color: Scalar = [0., 0., 255., 255.].into();
        imgproc::rectangle_def(mat, rect, color).unwrap();

        let text_org = Point::new(rect.x + 10, rect.y + 20);
        let font = imgproc::FONT_HERSHEY_SIMPLEX;
        imgproc::put_text_def(mat, label, text_org, font, 0.5, color).unwrap();
    }
}
