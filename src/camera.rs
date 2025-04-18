use image::GenericImageView;
use nokhwa::pixel_format::RgbFormat;
use nokhwa::utils::*;

pub struct Camera {
    pub camera: nokhwa::Camera,
}

impl Camera {
    pub fn new(index: u32) -> Self {
        let index = CameraIndex::Index(index);
        let format = RequestedFormat::new::<RgbFormat>(RequestedFormatType::Exact(
            CameraFormat::new(Resolution::default(), FrameFormat::RAWRGB, 60),
        ));
        let camera = nokhwa::Camera::new(index, format).unwrap();
        Self { camera }
    }

    pub fn get(&mut self) -> Vec<u8> {
        self.camera.frame_raw().unwrap().into()
    }

    pub fn resolution(&self) -> (u32, u32) {
        let res = self.camera.resolution();
        return (res.width(), res.height());
    }
}

/// For testing
pub struct ImageCamera(pub image::DynamicImage);
impl ImageCamera {
    pub fn new(path: &str) -> Self {
        Self(image::open(path).unwrap())
    }
    pub fn get(&mut self) -> Vec<u8> {
        self.0.clone().to_rgb8().to_vec()
    }
    pub fn resolution(&self) -> (u32, u32) {
        self.0.clone().dimensions()
    }
}
