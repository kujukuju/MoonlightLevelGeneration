use crate::generator::Generator;
use std::f32::consts::PI;
use crate::math_helper::MathHelper;

pub struct WallSection {
    pub lines: Vec<[f32; 2]>,
}

impl WallSection {
    pub fn render(&mut self, generator: &mut Generator, color: u32) {
        for index in 0..(self.lines.len() - 1) {
            let next_index = index + 1;

            let point = self.lines[index];
            let next_point = self.lines[next_index];

            generator.draw_line(point[0], point[1], next_point[0], next_point[1], color, 1.0);
        }
    }

    pub fn add_point(&mut self, point: &[f32; 2]) {
        self.lines.push(point.clone());
    }

    pub fn distance_to_wall(&self, point: &[f32; 2]) -> ([f32; 2], f32) {
        let mut min_point: Option<[f32; 2]> = None;
        let mut min_distance = f32::MAX;

        for index in 0..(self.lines.len() - 1) {
            let current_point = self.lines[index];
            let next_point = self.lines[index + 1];

            let (point, distance) = MathHelper::distance_to_line_segment(&[current_point, next_point], point);
            if distance < min_distance {
                min_point = Some(point);
                min_distance = distance;
            }
        }

        return (min_point.unwrap(), min_distance);
    }

    pub fn thicken(&mut self, generator: &mut Generator, start_thickness: f32, end_thickness: f32) -> [WallSection; 2] {
        let mut wall_1 = WallSection::default();
        let mut wall_2 = WallSection::default();

        for index in 0..self.lines.len() {
            let point = self.lines[index];

            let normal;
            if index == 0 {
                let next_point = self.lines[index + 1];

                let dx = next_point[0] - point[0];
                let dy = next_point[1] - point[1];
                normal = dy.atan2(dx) + PI / 2.0;
            } else if index == self.lines.len() - 1 {
                let prev_point = self.lines[index - 1];

                let dx = point[0] - prev_point[0];
                let dy = point[1] - prev_point[1];
                normal = dy.atan2(dx) + PI / 2.0;
            } else {
                let prev_point = self.lines[index - 1];
                let next_point = self.lines[index + 1];

                let dx1 = point[0] - prev_point[0];
                let dy1 = point[1] - prev_point[1];
                let dx2 = next_point[0] - point[0];
                let dy2 = next_point[1] - point[1];
                let angle = dy1.atan2(dx1) + MathHelper::radians_between_angles(dy1.atan2(dx1), dy2.atan2(dx2)) / 2.0;
                normal = angle + PI / 2.0;
            }

            let perlin1 = (generator.get_perlin_value(point[0] + 3452.0, point[1] + 3452.0, 10.0) + 1.0) / 2.0;
            let perlin2 = (generator.get_perlin_value(point[0] + 87362.0, point[1] + 87362.0, 10.0) + 1.0) / 2.0;
            let perlin = perlin1 * perlin2;
            let thickness_mod = perlin * 8.0 - 0.5;
            // let thickness_mod = thickness_mod + generator.get_perlin_value(point[0], point[1], 0.1) * 400.0;
            let thickness = (index as f32 / (self.lines.len() - 1) as f32) * (end_thickness - start_thickness) + start_thickness;
            let thickness = (thickness * thickness_mod).max(200.0);
            wall_1.add_point(&[point[0] + normal.cos() * thickness, point[1] + normal.sin() * thickness]);
            wall_2.add_point(&[point[0] - normal.cos() * thickness, point[1] - normal.sin() * thickness]);
        }

        return [wall_1, wall_2];
    }
}

impl Default for WallSection {
    fn default() -> Self {
        return WallSection {
            lines: Vec::new(),
        }
    }
}