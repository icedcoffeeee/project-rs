use opencv::{calib3d, features2d, Result};
use opencv::{core::*, prelude::*};

pub fn get_homography(camera1: &Mat, camera2: &Mat, homography: &mut Mat) -> Result<()> {
    let mut orb = features2d::ORB::create_def()?;
    let mut mask = Mat::default();

    let (mut keypoints1, mut descriptors1) = (Vector::new(), Mat::default());
    let (mut keypoints2, mut descriptors2) = (Vector::new(), Mat::default());
    orb.detect_and_compute_def(camera1, &mask, &mut keypoints1, &mut descriptors1)?;
    orb.detect_and_compute_def(camera2, &mask, &mut keypoints2, &mut descriptors2)?;

    let mut matcher = features2d::DescriptorMatcher::create("BruteForce-Hamming")?;
    let mut matches = Vector::new();
    matcher.add(&descriptors1)?;
    matcher.match__def(&descriptors2, &mut matches)?;

    let mut matches = matches.to_vec();
    matches.sort_by(|x, y| x.distance.total_cmp(&y.distance));
    if matches.len() < 4 {
        println!("Not enough matches");
        return Ok(());
    } else {
        // cut the last 10%
        for _ in 0..(matches.len() / 10) {
            matches.pop();
        }
    }

    let (mut points1, mut points2) = (Vec::new(), Vec::new());
    for match_ in matches {
        points1.push(keypoints1.get(match_.train_idx as usize)?.pt());
        points2.push(keypoints2.get(match_.query_idx as usize)?.pt());
    }
    *homography = calib3d::find_homography(
        &Mat::from_slice(points1.as_slice())?,
        &Mat::from_slice(points2.as_slice())?,
        &mut mask,
        calib3d::RANSAC,
        0.5,
    )?;
    println!("{:?}", homography.to_vec_2d::<VecN<f64, 1>>()?);
    return Ok(());
}
