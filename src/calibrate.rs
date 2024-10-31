use opencv::core::*;

pub fn get_shift(camera1: &Mat, camera2: &Mat, shift: &mut [i32; 2]) {
    let wind = 20;
    let size = camera1.size().unwrap();
    let rect = camera1
        .roi(Rect::new(
            (size.width - wind) / 2,
            (size.height - wind) / 2,
            wind,
            wind,
        ))
        .unwrap();
    let mut diff_min = 1e5;
    let mut diff_shift = [0, 0];
    for a in 0..size.width - wind {
        for b in 0..size.height - wind {
            let roi = camera2.roi(Rect::new(a, b, wind, wind)).unwrap();
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
                diff_shift = [a - (size.width - wind) / 2, b - (size.height - wind) / 2];
            }
        }
    }
    *shift = diff_shift;
}
