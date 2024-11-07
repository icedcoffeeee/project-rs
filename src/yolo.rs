use crate::*;
use mpsc::{Receiver, Sender};

type Detections = (Vector<i32>, Vector<f32>, Vector<Rect_<i32>>);

pub fn initialize_thread(r_feed: Receiver<Mat>, t_detections: Sender<Detections>) {
    let mut model = dnn::DetectionModel::new("yolo/yolov3.weights", "yolo/yolov3.cfg").unwrap();
    let mut init = false;
    loop {
        let feed: Mat = match r_feed.try_recv() {
            Ok(feed) => feed,
            _ => continue,
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
        model.detect_def(&feed, class_ids, scores, rects).unwrap();
        t_detections.send(detections).unwrap();
    }
}

pub fn draw_bounding_boxes(mat: &mut Mat, detections: &Detections, classes: &Vec<&str>) {
    let (class_ids, scores, rects) = detections;
    let mut indices = Vector::<i32>::new();
    dnn::nms_boxes_def(&rects, &scores, 0.5, 0.1, &mut indices).unwrap();

    for i in indices {
        let rect: Rect = rects.to_vec()[i as usize];
        let label = classes[class_ids.to_vec()[i as usize] as usize];
        let color: VecN<f64, 4> = [0., 0., 255., 255.].into();
        imgproc::rectangle_def(mat, rect, color).unwrap();

        let text_org = Point::new(rect.x - 10, rect.y - 10);
        let font = imgproc::FONT_HERSHEY_SIMPLEX;
        imgproc::put_text_def(mat, label, text_org, font, 0.5, color).unwrap();
    }
}
