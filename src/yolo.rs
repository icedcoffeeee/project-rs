use crate::*;

const CONF_THRESH: f32 = 0.1;
const NMS_THRESH: f32 = 0.1;

pub fn detect(mat: &mut Mat, net: &mut dnn::Net, classes: &Vec<&str>) {
    let mut class_ids = Vec::<usize>::new();
    let mut confidences = Vec::<f32>::new();
    let mut rects = Vec::<Rect>::new();

    let size = mat.size().unwrap();
    let blob =
        dnn::blob_from_image(mat, 0.00392, size, [0.; 4].into(), true, false, CV_32F).unwrap();
    net.set_input_def(&blob).unwrap();

    let names = net.get_layer_names().unwrap();
    let mut layers = Vector::new();
    for i in net.get_unconnected_out_layers().unwrap() {
        layers.push(names.get(i as usize - 1).unwrap().as_str());
    }
    let mut detections = Vector::<Mat>::default();
    net.forward(&mut detections, &layers).unwrap();

    for output in detections {
        for row in output.to_vec_2d::<f32>().unwrap() {
            let (class_id, confidence) = row[5..row.len()]
                .iter()
                .enumerate()
                .reduce(|acc, val| {
                    return [val, acc][(acc.1 > val.1) as usize];
                })
                .unwrap();
            if *confidence > 0.5 {
                let [cx, cy] = [row[0] * size.width as f32, row[1] * size.height as f32];
                let [w, h] = [row[3] * size.height as f32, row[3] * size.height as f32];
                let [x, y] = [cx - w / 2., cy - h / 2.];

                class_ids.push(class_id);
                confidences.push(*confidence);
                rects.push(Rect::new(x as i32, y as i32, w as i32, h as i32));
            };
        }
    }

    let mut indices = Vector::<i32>::new();
    dnn::nms_boxes_def(
        &rects.clone().into(),
        &confidences.into(),
        CONF_THRESH,
        NMS_THRESH,
        &mut indices,
    )
    .unwrap();
    for i in indices {
        let rect: Rect = rects[i as usize];

        let label = classes[class_ids[i as usize]];
        let color: VecN<f64, 4> = [1., 0., 0., 0.].into();
        imgproc::rectangle_def(mat, rect, color).unwrap();

        let text_org = Point::new(rect.x - 10, rect.y - 10);
        let font = imgproc::FONT_HERSHEY_SIMPLEX;
        imgproc::put_text_def(mat, label, text_org, font, 0.5, color).unwrap();
    }
}
