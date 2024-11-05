use opencv::core::*;

pub fn get_shift(mat1: &Mat, mat2: &Mat, window: i32, shift: &mut [i32; 2]) {
    let size = mat1.size().unwrap();
    let rect = mat1
        .roi(Rect::new(
            (size.width - window) / 2,
            (size.height - window) / 2,
            window,
            window,
        ))
        .unwrap();
    let mut diff_min = 1e5;
    let mut diff_shift = [0, 0];
    let mut roi_ = Mat::default();
    for a in 0..size.width - window {
        for b in 0..size.height - window {
            let roi = mat2.roi(Rect::new(a, b, window, window)).unwrap();
            let mut diff = Mat::default();

            absdiff(&roi, &rect, &mut diff).unwrap();
            let min = mean_def(&diff)
                .unwrap()
                .map::<_, f64>(|x| x)
                .into_iter()
                .reduce(|a, v| a + v)
                .unwrap();
            if min < diff_min {
                diff_min = min;
                diff_shift = [
                    a - (size.width - window) / 2,
                    b - (size.height - window) / 2,
                ];
                roi_.set_to_def(&roi).unwrap();
            }
        }
    }
    *shift = diff_shift;
}
