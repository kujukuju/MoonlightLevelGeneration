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

    pub fn connect_points_linear(&mut self, generator: &mut Generator, point1: [f32; 2], point2: [f32; 2]) {
        let dx = point2[0] - point1[0];
        let dy = point2[1] - point1[1];
        let distance = (dx * dx + dy * dy).sqrt();

        let steps = (distance / 800.0).ceil() as i32;

        self.add_point(&point1);

        for i in 1..steps {
            let progress = i as f32 / steps as f32;

            let point = [point1[0] + dx * progress, point1[1] + dy * progress];
            self.add_point(&point);
        }

        self.add_point(&point2);

        // for i in 1..(self.lines.len() - 1) {
        //     let point = &mut self.lines[i];
        //
        //     let perlin = generator.get_perlin_value(point[0] + 52365.0, point[1] + 75632.0, 1.0);
        //     let dx = point[0];
        //     let dy = point[1];
        //     let d = (dx * dx + dy * dy).sqrt();
        //
        //     // let perlin2 = generator.get_perlin_value(point[0] - 457643.0, point[1] + 156453.0, 10.0);
        //
        //     let dx = dx / d;
        //     let dy = dy / d;
        //
        //     point[0] += dx * perlin * 400.0 + dx * perlin2 * 2400.0;
        //     point[1] += dy * perlin * 400.0 + dy * perlin2 * 2400.0;
        // }
    }

    pub fn connect_points(&mut self, generator: &mut Generator, point1: [f32; 2], tangent1: [f32; 2], point2: [f32; 2], tangent2: [f32; 2]) {
        let d1 = (point1[0] * point1[0] + point1[1] * point1[1]).sqrt();
        let d2 = (point2[0] * point2[0] + point2[1] * point2[1]).sqrt();

        let ang1 = point1[1].atan2(point1[0]);
        let ang2 = point2[1].atan2(point2[0]);
        let delta_ang = MathHelper::radians_between_angles(ang1, ang2);

        self.add_point(&point1);

        let iterations = (delta_ang.abs() / 0.002).ceil() as i32;
        for i in 1..iterations {
            let progress = i as f32 / iterations as f32;
            let point = MathHelper::hermite(
                progress,
                [point1, point2],
                [tangent1, [-tangent2[0], -tangent2[1]]]);

            self.add_point(&point);

            // let ang = ang1 + (delta_ang / iterations as f32) * i as f32;
            //
            // let d = d1 + (d2 - d1) * progress;
            //
            // self.add_point(&[ang.cos() * d, ang.sin() * d]);
        }

        self.add_point(&point2);

        // let mut center_point = [
        //     (point1[0] + point2[0]) / 2.0,
        //     (point1[1] + point2[1]) / 2.0,
        // ];
        // // move the center point slightly more back towards the center
        // let center_dist = (center_point[0] * center_point[0] + center_point[1] * center_point[1]).sqrt();
        // center_point[0] -= center_point[0] / center_dist * 10000.0;
        // center_point[1] -= center_point[1] / center_dist * 10000.0;
        //
        // for i in 1..(self.lines.len() - 1) {
        //     let point = &mut self.lines[i];
        //
        //     let perlin = generator.get_perlin_value(point[0] + 56342.0, point[1] + 90678.0, 1.0);
        //     let dx = point[0] - center_point[0];
        //     let dy = point[1] - center_point[1];
        //     let d = (dx * dx + dy * dy).sqrt();
        //
        //     let dx = dx / d;
        //     let dy = dy / d;
        //
        //     point[0] += dx * perlin * 400.0;
        //     point[1] += dy * perlin * 400.0;
        // }
    }

    pub fn noiseify(&mut self, generator: &mut Generator, strength: f32, scale: f32, center: [f32; 2], offset_angle: f32) {
        for i in 1..(self.lines.len() - 1) {
            let progress = i as f32 / (self.lines.len() - 1) as f32;
            let ease = MathHelper::ease_in_out((progress * 2.0).min(1.0));
            let ease = ease.min(MathHelper::ease_in_out(((1.0 - progress) * 2.0).min(1.0)));

            let point = &mut self.lines[i];

            let perlin = generator.get_perlin_value(point[0] + 56342.0, point[1] + 90678.0, scale);
            let angle = (point[1] - center[1]).atan2(point[0] - center[0]);
            let angle = angle + offset_angle;
            let dx = angle.cos();
            let dy = angle.sin();

            point[0] += dx * perlin * strength * ease;
            point[1] += dy * perlin * strength * ease;
        }
    }

    pub fn join_wall(&mut self, mut wall: WallSection) {
        // println!("test {:?} {:?} {:?} {:?}", self.lines[0], self.lines[self.lines.len() - 1], wall.lines[0], wall.lines[wall.lines.len() - 1]);
        if self.lines[self.lines.len() - 1][0] == wall.lines[wall.lines.len() - 1][0] && self.lines[self.lines.len() - 1][1] == wall.lines[wall.lines.len() - 1][1] {
            for vertex in wall.lines.drain(0..(wall.lines.len() - 1)).rev() {
                self.lines.push(vertex);
            }
        } else if self.lines[0][0] == wall.lines[wall.lines.len() - 1][0] && self.lines[0][1] == wall.lines[wall.lines.len() - 1][1] {
            // incorrect and slow
            for vertex in self.lines.drain(1..) {
                wall.lines.push(vertex);
            }
            self.lines = wall.lines;
        } else if self.lines[self.lines.len() - 1][0] == wall.lines[0][0] && self.lines[self.lines.len() - 1][1] == wall.lines[0][1] {
            for vertex in wall.lines.drain(1..) {
                self.lines.push(vertex);
            }
        } else {
            panic!("Can not join walls that don't share a vertex.");
        }
    }

    pub fn get_first_point(&self) -> [f32; 2] {
        return self.lines[0];
    }

    pub fn get_last_point(&self) -> [f32; 2] {
        return self.lines[self.lines.len() - 1];
    }

    pub fn get_length(&self) -> f32 {
        let mut length = 0.0;
        for i in 0..(self.lines.len() - 1) {
            let point = self.lines[i];
            let next_point = self.lines[i + 1];

            let dx = next_point[0] - point[0];
            let dy = next_point[1] - point[1];
            let d = (dx * dx + dy * dy).sqrt();

            length += d;
        }

        return length;
    }

    pub fn get_point_at_length(&self, length: f32) -> [f32; 2] {
        let mut cur_length = 0.0;
        for i in 0..(self.lines.len() - 1) {
            let point = self.lines[i];
            let next_point = self.lines[i + 1];

            let dx = next_point[0] - point[0];
            let dy = next_point[1] - point[1];
            let d = (dx * dx + dy * dy).sqrt();

            let prev_length = cur_length;
            cur_length += d;

            if cur_length >= length {
                let percent = (cur_length - length) / (cur_length - prev_length);

                return [point[0] + dx * percent, point[1] + dy * percent];
            }
        }

        return self.lines[self.lines.len() - 1];
    }

    pub fn delete_after_length(&mut self, length: f32) {
        for i in (1..self.lines.len()).rev() {
            let point = self.lines[i];

            let d = (point[0] * point[0] + point[1] * point[1]).sqrt();
            if d < length {
                while self.lines.len() > i + 2 {
                    self.lines.remove(self.lines.len() - 1);
                }

                return;
            }
        }

        // let mut cur_length = 0.0;
        // for i in 0..(self.lines.len() - 1) {
        //     let point = self.lines[i];
        //     let next_point = &mut self.lines[i + 1];
        //
        //     let dx = next_point[0] - point[0];
        //     let dy = next_point[1] - point[1];
        //     let d = (dx * dx + dy * dy).sqrt();
        //
        //     // let prev_length = cur_length;
        //     cur_length += d;
        //
        //     if cur_length >= length {
        //         // let percent = (cur_length - length) / (cur_length - prev_length);
        //
        //         // next_point[0] = point[0] + dx * percent;
        //         // next_point[1] = point[1] + dy * percent;
        //
        //         while self.lines.len() > i + 2 {
        //             self.lines.remove(self.lines.len() - 1);
        //         }
        //
        //         return;
        //     }
        // }
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