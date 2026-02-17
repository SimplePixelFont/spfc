use kurbo::BezPath;
use std::collections::HashMap;

pub struct PixelGrid {
    pub width: usize,
    pub height: usize,
    pub pixel_size: f64,
    pub pixels: Vec<Vec<bool>>,
}

type P = (i32, i32);

impl PixelGrid {
    pub fn empty(width: usize, height: usize, pixel_size: f64) -> Self {
        Self {
            width,
            height,
            pixel_size,
            pixels: vec![vec![false; width]; height],
        }
    }
    pub fn draw_rectangle(&mut self, x: usize, y: usize, width: usize, height: usize) {
        for offset_x in x..x + width {
            self.pixels[y][offset_x] = true;
            self.pixels[y + height - 1][offset_x] = true;
        }
        for offset_y in y..y + height {
            self.pixels[offset_y][x] = true;
            self.pixels[offset_y][x + width - 1] = true;
        }
    }

    pub fn to_bezpath(&self, descender_pixels: usize) -> BezPath {
        let ps = self.pixel_size;
        let h = self.height as i32;
        let baseline_offset = -(descender_pixels as f64 * ps);

        let to_font = |gx: i32, gy: i32| -> (f64, f64) {
            (gx as f64 * ps, baseline_offset + (h - gy) as f64 * ps)
        };

        let mut adj: HashMap<P, Vec<P>> = HashMap::new();
        for y in 0..h {
            for x in 0..self.width as i32 {
                if !self.pixel_at(x, y) {
                    continue;
                }
                if !self.pixel_at(x, y - 1) {
                    adj.entry((x, y)).or_default().push((x + 1, y));
                }
                if !self.pixel_at(x + 1, y) {
                    adj.entry((x + 1, y)).or_default().push((x + 1, y + 1));
                }
                if !self.pixel_at(x, y + 1) {
                    adj.entry((x + 1, y + 1)).or_default().push((x, y + 1));
                }
                if !self.pixel_at(x - 1, y) {
                    adj.entry((x, y + 1)).or_default().push((x, y));
                }
            }
        }

        let mut path = BezPath::new();

        while !adj.is_empty() {
            let start = *adj.keys().next().unwrap();
            let (fx, fy) = to_font(start.0, start.1);
            path.move_to((fx, fy));

            let mut current = start;
            let mut prev_dir: P = (0, 0);

            loop {
                let nexts = match adj.get(&current) {
                    Some(v) if !v.is_empty() => v.clone(),
                    _ => break,
                };

                let next = if nexts.len() == 1 {
                    nexts[0]
                } else {
                    *nexts
                        .iter()
                        .max_by_key(|&&n| {
                            let d = (n.0 - current.0, n.1 - current.1);
                            // 2-D cross product. If positive = right turn in y-down
                            prev_dir.0 * d.1 - prev_dir.1 * d.0
                        })
                        .unwrap()
                };

                prev_dir = (next.0 - current.0, next.1 - current.1);

                if let Some(v) = adj.get_mut(&current) {
                    v.retain(|&p| p != next);
                    if v.is_empty() {
                        adj.remove(&current);
                    }
                }

                if next == start {
                    break; // Close the contour
                }

                let (fx, fy) = to_font(next.0, next.1);
                path.line_to((fx, fy));
                current = next;
            }

            path.close_path();
        }

        path
    }

    fn pixel_at(&self, x: i32, y: i32) -> bool {
        if x < 0 || y < 0 || x >= self.width as i32 || y >= self.height as i32 {
            return false;
        }
        self.pixels[y as usize][x as usize]
    }
}
