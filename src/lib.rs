pub use opencv::core::*;
pub use opencv::imgproc::{resize_def, warp_affine_def};
pub use opencv::videoio::{VideoCapture, VideoCaptureTrait, CAP_ANY};
pub use opencv::Result;

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
