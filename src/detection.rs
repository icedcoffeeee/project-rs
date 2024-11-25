use crate::*;
use mpsc::{Receiver, Sender, TryRecvError};

const WEIGHTS_FILE: &str = "data/yolov3.weights";
const CONFIG_FILE: &str = "data/yolov3.cfg";
const CLASSES_FILE: &str = "data/yolov3.txt";

type Detections = (Vector<i32>, Vector<f32>, Vector<Rect_<i32>>);

pub fn initialize_thread(r_feed: Receiver<Mat>, t_detections: Sender<Detections>) {
    let mut model = dnn::DetectionModel::new(WEIGHTS_FILE, CONFIG_FILE).unwrap();
    let mut init = false;
    loop {
        let feed: Mat = match r_feed.try_recv() {
            Ok(feed) => feed,
            Err(TryRecvError::Empty) => continue,
            Err(TryRecvError::Disconnected) => break,
        };
        if !init {
            init = true;
            model.set_input_size(feed.size().unwrap()).unwrap();
            model.set_input_scale(0.001).unwrap();
            model.set_input_mean([0.; 4].into()).unwrap();
            model.set_input_swap_rb(true).unwrap();
        }
        let mut detections = Detections::default();
        let (ref mut class_ids, ref mut scores, ref mut rects) = &mut detections;
        let _ = model.detect_def(&feed, class_ids, scores, rects);
        let _ = t_detections.send(detections);
    }
}

pub fn draw_bounding_boxes(
    mat: &mut Mat,
    detections: &Detections,
    classes: &mut Option<Vec<String>>,
) {
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
