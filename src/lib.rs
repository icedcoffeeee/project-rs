pub mod app;
pub mod calibrate;
pub mod detection;
pub mod image;
pub mod utils;
pub mod window;

pub use imgui as im;
pub use std::{env, os, str};
pub use std::{fs, path};
pub use std::{sync::mpsc, thread};

pub use opencv::{core::*, prelude::*};
pub use opencv::{dnn, imgcodecs, imgproc, videoio, Result};

pub trait SizeToArray {
    fn to_array(self) -> [f32; 2];
}

impl SizeToArray for Size {
    fn to_array(self) -> [f32; 2] {
        [self.width as _, self.height as _]
    }
}
