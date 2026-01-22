use kurbo::BezPath;

pub struct PixelGrid {
    pub width: usize,
    pub height: usize,
    pub pixel_size: f64,
    pub pixels: Vec<Vec<bool>>,
}

impl PixelGrid {
    pub fn to_bezpath(&self) -> BezPath {
        let mut path = BezPath::new();
        let baseline_offset = 0.0;
        
        for y in 0..self.height {
            for x in 0..self.width {
                if self.pixels[y][x] {
                    let x_pos = x as f64 * self.pixel_size;
                    let y_pos = baseline_offset + (self.height - 1 - y) as f64 * self.pixel_size;
                    
                    let mut rect_path = BezPath::new();
                    rect_path.move_to((x_pos, y_pos));
                    rect_path.line_to((x_pos, y_pos + self.pixel_size));  // Up
                    rect_path.line_to((x_pos + self.pixel_size, y_pos + self.pixel_size));  // Right
                    rect_path.line_to((x_pos + self.pixel_size, y_pos));  // Down
                    rect_path.close_path();
                    
                    path.extend(&rect_path);
                }
            }
        }
        
        path
    }
}