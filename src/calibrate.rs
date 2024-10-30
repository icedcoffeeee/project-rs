use opencv::{calib3d, features2d, imgproc};
use opencv::{core::*, prelude::*};

pub fn get_homography(camera1: &Mat, camera2: &Mat, homography: &mut Mat) {
    let mut orb = features2d::ORB::create_def().unwrap();
    let mut mask = Mat::default();

    let (mut gray1, mut gray2) = (Mat::default(), Mat::default());
    imgproc::cvt_color_def(&camera1, &mut gray1, imgproc::COLOR_BGR2GRAY).unwrap();
    imgproc::cvt_color_def(&camera2, &mut gray2, imgproc::COLOR_BGR2GRAY).unwrap();

    let (mut keypoints1, mut descriptors1) = (Vector::new(), Mat::default());
    let (mut keypoints2, mut descriptors2) = (Vector::new(), Mat::default());
    orb.detect_and_compute_def(&gray1, &mask, &mut keypoints1, &mut descriptors1)
        .unwrap();
    orb.detect_and_compute_def(&gray2, &mask, &mut keypoints2, &mut descriptors2)
        .unwrap();

    let mut matcher = features2d::DescriptorMatcher::create("BruteForce-Hamming").unwrap();
    let mut matches = Vector::new();
    matcher.add(&descriptors1).unwrap();
    matcher.match__def(&descriptors2, &mut matches).unwrap();

    let mut matches = matches.to_vec();
    matches.sort_by(|x, y| x.distance.total_cmp(&y.distance));
    if matches.len() < 4 {
        println!("Not enough matches");
        return;
    } else {
        // cut the last 10%
        for _ in 0..(matches.len() / 10) {
            matches.pop();
        }
    }

    let (mut points1, mut points2) = (Vec::new(), Vec::new());
    for match_ in matches {
        points1.push(keypoints1.get(match_.train_idx as usize).unwrap().pt());
        points2.push(keypoints2.get(match_.query_idx as usize).unwrap().pt());
    }
    *homography = calib3d::find_homography(
        &Mat::from_slice(points1.as_slice()).unwrap(),
        &Mat::from_slice(points2.as_slice()).unwrap(),
        &mut mask,
        calib3d::RANSAC,
        0.5,
    )
    .unwrap();
    println!("{:?}", homography.to_vec_2d::<VecN<f64, 1>>().unwrap());
}

//fn get_gradient(mat: &Mat) -> Mat {
//    let (mut grad_x, mut grad_y) = (Mat::default(), Mat::default());
//    imgproc::sobel_def(&mat, &mut grad_x, CV_32FC1, 1, 0).unwrap();
//    imgproc::sobel_def(&mat, &mut grad_y, CV_32FC1, 1, 0).unwrap();
//
//    let mut grad = Mat::default();
//    add_weighted_def(&grad_x, 0.5, &grad_y, 0.5, 0., &mut grad).unwrap();
//    return grad;
//}

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
                diff_shift = [
                    a - (size.width - wind) / 2 - shift[0],
                    b - (size.height - wind) / 2 + shift[1],
                ];
            }
        }
    }
    *shift = diff_shift;
    println!("{:?}", diff_min);
    println!("{:?}", shift);
}
