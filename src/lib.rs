pub mod calibrate;
pub mod image;
pub mod window;

pub use std::{env, os};
pub use std::{fs, path};

pub use opencv::{core::*, prelude::*};
pub use opencv::{imgcodecs, imgproc, videoio, Result};

pub trait SizeToArray {
    fn to_array(self) -> [f32; 2];
}

impl SizeToArray for Size {
    fn to_array(self) -> [f32; 2] {
        [self.width as _, self.height as _]
    }
}
