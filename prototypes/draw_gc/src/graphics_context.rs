use bronze::*;

#[derive(Clone, Copy)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

simple_empty_finalize_trace![Point];

pub struct GraphicsContext<'a> {
    pixel_buffer: &'a mut [u8],
    pixel_width: u32,
    pixel_height: u32,
    size: winit::dpi::LogicalSize<f64>,
}

impl<'a> GraphicsContext<'a> {
    pub fn new(pixel_buffer: &'a mut [u8], pixel_width: u32, pixel_height: u32, size: winit::dpi::LogicalSize<f64>) -> Self {
        GraphicsContext {pixel_buffer, pixel_width, pixel_height, size}
    }

    fn coord_to_pixel(&self, point: Point) -> (u32, u32) {
        // Origin is at top left.
        let pixel_x = (point.x/self.size.width * (self.pixel_width as f64)) as u32;
        let pixel_y = (point.y/self.size.height * (self.pixel_height as f64)) as u32;

        return (pixel_x, pixel_y);
    }

    fn pixel_to_array_slice(&mut self, pixel_x: u32, pixel_y: u32) -> &mut [u8] {
        let index = ((pixel_x + (pixel_y * self.pixel_width)) * 4) as usize;
        return &mut self.pixel_buffer[index..index+4];
    }

    fn width_to_pixel_width(&mut self, width: f64) -> u32 {
        (width * (self.size.width / self.size.width)) as u32
    }


    fn height_to_pixel_height(&mut self, height: f64) -> u32 {
        (height * (self.size.height / self.size.height)) as u32
    }

    pub fn draw_rect(&mut self, top_left: Point, width: f64, height: f64, color: [u8; 4]) {
        let (top_left_pixel_x, top_left_pixel_y) = self.coord_to_pixel(top_left);
        let bottom_right_pixel_x = top_left_pixel_x + self.width_to_pixel_width(width);
        let bottom_right_pixel_y = top_left_pixel_y + self.height_to_pixel_height(height);

        for x in top_left_pixel_x..bottom_right_pixel_x {
            for y in top_left_pixel_y..bottom_right_pixel_y {
                let slice = self.pixel_to_array_slice(x, y);
                slice.copy_from_slice(&color);
            }
        }

    }
}