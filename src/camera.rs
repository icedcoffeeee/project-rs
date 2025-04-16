use eye::hal::platform::Stream as Feed;
use eye::hal::stream::Descriptor;
use eye::hal::traits::{Context, Device, Stream};
use eye::hal::PlatformContext;

pub struct Camera<'a> {
    pub feed: Feed<'a>,
    pub stream: Descriptor,
}

impl<'a> Camera<'a> {
    pub fn new() -> Self {
        let ctx = PlatformContext::all().next().unwrap();
        let dev = ctx.open_device(&ctx.devices().unwrap()[0].uri).unwrap();
        let stream = &mut dev.streams().unwrap()[0];
        Self {
            feed: dev.start_stream(stream).unwrap(),
            stream: stream.clone(),
        }
    }

    pub fn get_frame(&mut self) -> Vec<u8> {
        let frame = self.feed.next().unwrap().unwrap();
        match &self.stream.pixfmt {
            eye::hal::format::PixelFormat::Custom(fmt) if fmt == "YU12" => yv12_to_rgb(
                frame,
                self.stream.width as usize,
                self.stream.height as usize,
            ),
            _ => frame.to_vec(),
        }
    }
}

fn yv12_to_rgb(data: &[u8], width: usize, height: usize) -> Vec<u8> {
    // Calculate the sizes of the Y, U, and V planes
    let y_size = width * height;
    let u_size = (width / 2) * (height / 2);
    let v_size = (width / 2) * (height / 2);

    // Ensure the raw data length matches the expected size
    let expected_size = y_size + u_size + v_size;
    if data.len() != expected_size {
        panic!(
            "Invalid raw data size: expected {}, got {}",
            expected_size,
            data.len()
        );
    }

    // Extract Y, U, and V planes from the raw data
    let y_plane = &data[0..y_size];
    let u_plane = &data[y_size..y_size + u_size];
    let v_plane = &data[y_size + u_size..];

    // Create an RGB vector
    let mut rgb_image = vec![0u8; width * height * 3];

    // Process each pixel in a single loop
    for pixel_index in 0..(width * height) {
        let j = pixel_index / width; // Calculate the row index
        let i = pixel_index % width; // Calculate the column index

        let y = y_plane[pixel_index] as f32;

        // Calculate the corresponding U and V indices
        let u_index = (j / 2) * (width / 2) + (i / 2);
        let v_index = (j / 2) * (width / 2) + (i / 2);

        let u = u_plane[u_index] as f32 - 128.0;
        let v = v_plane[v_index] as f32 - 128.0;

        // Convert YUV to RGB
        let r = y + 1.402 * v;
        let g = y - 0.344136 * u - 0.714136 * v;
        let b = y + 1.772 * u;

        // Clamp values to [0, 255]
        let r = r.clamp(0.0, 255.0) as u8;
        let g = g.clamp(0.0, 255.0) as u8;
        let b = b.clamp(0.0, 255.0) as u8;

        // Store the RGB values in the output vector
        let rgb_index = pixel_index * 3;
        rgb_image[rgb_index] = r;
        rgb_image[rgb_index + 1] = g;
        rgb_image[rgb_index + 2] = b;
    }

    rgb_image
}
