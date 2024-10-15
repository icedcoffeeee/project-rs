pub use opencv::{calib3d, features2d, imgcodecs, imgproc, videoio, Result};
pub use opencv::{core::*, prelude::*};

pub use std::fs;

pub mod image;
pub mod window;

pub trait Size2Array {
    fn to_array(self) -> [f32; 2];
}

impl Size2Array for Size {
    fn to_array(self) -> [f32; 2] {
        [self.width as _, self.height as _]
    }
}
