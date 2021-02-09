use crate::generator::{Generator, SAFE_ZONE_WIDTH, SAFE_ZONE_HEIGHT};
use std::f32::consts::PI;
use crate::math_helper::MathHelper;
use std::collections::HashSet;
use rstar::{RTree, AABB, RTreeObject};
use std::fmt::{Debug, Formatter, Display};

#[derive(Copy, Clone)]
struct Line {
    lines: &Vec<[f32; 2]>,
    index1: usize,
    index2: usize,
}

impl Line {
    fn get_line(&self) -> [[f32; 2]; 2] {
        return [self.lines[self.index1], self.lines[self.index2]];
    }
}

impl Display for Line {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        return write!(f, "{:?} {:?}", self.index1, self.index2);
    }
}

impl Debug for Line {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        return write!(f, "{:?} {:?}", self.index1, self.index2);
    }
}

impl PartialEq for Line {
    fn eq(&self, other: &Self) -> bool {
        return self.index1 == other.index1 && self.index2 == other.index2;
    }
}

impl RTreeObject for Line {
    type Envelope = AABB<[f32; 2]>;

    fn envelope(&self) -> Self::Envelope {
        let min = [self.lines[self.index1][0].min(self.lines[self.index2][0]), self.lines[self.index1][1].min(self.lines[self.index2][1])];
        let max = [self.lines[self.index1][0].max(self.lines[self.index2][0]), self.lines[self.index1][1].max(self.lines[self.index2][1])];

        return AABB::from_corners(min, max);
    }
}

pub struct WallSection {
    pub lines: Vec<[f32; 2]>,
}

impl WallSection {
    pub fn render(&mut self, generator: &mut Generator, color: u32) {
        for index in 0..(self.lines.len() - 1) {
            let next_index = index + 1;

            let point = self.lines[index];
            let next_point = self.lines[next_index];

            let red = index as f32 / (self.lines.len() - 2) as f32;
            let red = red * 255.0;
            let red = red.round() as u32;
            let color = (color & 0x00ffff) | (red << 16);

            generator.draw_line(point[0], point[1], next_point[0], next_point[1], color, 1.0);
        }
    }

    pub fn connect_points_linear(&mut self, generator: &mut Generator, point1: [f32; 2], point2: [f32; 2]) {
        let dx = point2[0] - point1[0];
        let dy = point2[1] - point1[1];
        let distance = (dx * dx + dy * dy).sqrt();

        // let steps = (distance / 800.0).ceil() as i32;
        let steps = (distance.powf(0.65) / 15.0).ceil() as i32;

        self.add_point(&point1);

        for i in 1..steps {
            let progress = i as f32 / steps as f32;

            let point = [point1[0] + dx * progress, point1[1] + dy * progress];
            self.add_point(&point);
        }

        self.add_point(&point2);
    }

    pub fn connect_points(&mut self, generator: &mut Generator, point1: [f32; 2], tangent1: [f32; 2], point2: [f32; 2], tangent2: [f32; 2]) {
        // let d1 = (point1[0] * point1[0] + point1[1] * point1[1]).sqrt();
        // let d2 = (point2[0] * point2[0] + point2[1] * point2[1]).sqrt();

        // let ang1 = point1[1].atan2(point1[0]);
        // let ang2 = point2[1].atan2(point2[0]);
        // let delta_ang = MathHelper::radians_between_angles(ang1, ang2);

        let dx = point2[0] - point1[0];
        let dy = point2[1] - point1[1];
        let d = (dx * dx + dy * dy).sqrt();

        self.add_point(&point1);

        let iterations = (d.powf(0.65) / 15.0).ceil() as i32;

        for i in 1..iterations {
            let progress = i as f32 / iterations as f32;
            let point = MathHelper::hermite(
                progress,
                [point1, point2],
                [tangent1, [-tangent2[0], -tangent2[1]]]);

            self.add_point(&point);
        }

        self.add_point(&point2);
    }

    pub fn round_to_angle(&mut self, interval: f32) {
        // start at index 0
        for i in 0..self.lines.len() {
            // get the current line of index [i, i + 1], not wrapping back around
            let mut current_line = [&mut self.lines[i], &mut self.lines[(i + 1) % self.lines.len()]];
            let current_angle = (current_line[1][1] - current_line[0][1]).atan2(current_line[1][0] - current_line[0][0]);
            // get the next (infinite) line of index [i + 1, i + 2], wrapping back around if needed
            let mut next_line = [&mut self.lines[(i + 1) % self.lines.len()], &mut self.lines[(i + 2) % self.lines.len()]];
            let next_angle = (next_line[1][1] - next_line[0][1]).atan2(next_line[1][0] - next_line[0][0]);

            // round the current line angle up and down to the nearest angles
            let lower_angle = (current_angle / interval).floor() * interval;
            let upper_angle = (current_angle / interval).ceil() * interval;

            // find the intersection points of both rounded lines, and the next (infinite) line
            let lower_intersection = MathHelper::intersect_ray_ray(
                *current_line[0],
                [lower_angle.cos(), lower_angle.sin()],
                *next_line[0],
                [next_angle.cos(), next_angle.sin()]);
            let upper_intersection = MathHelper::intersect_ray_ray(
                *current_line[0],
                [upper_angle.cos(), upper_angle.sin()],
                *next_line[0],
                [next_angle.cos(), next_angle.sin()]);

            if current_line[0][0] == current_line[1][0] && current_line[0][1] == current_line[1][1] {
                panic!("1 found a segment of length 0 {:?}", self.lines);
            }

            if next_line[0][0] == next_line[1][0] && next_line[0][1] == next_line[1][1] {
                panic!("2 found a segment of length 0 {:?}", self.lines);
            }

            // whichever intersection point is closest to the original vertex [i + 1], [i + 1] becomes that point
            if let (Some(lower_intersection), Some(upper_intersection)) = (lower_intersection, upper_intersection) {
                let lower_dx = next_line[0][0] - lower_intersection[0];
                let lower_dy = next_line[0][1] - lower_intersection[1];
                let lower_d = lower_dx * lower_dx + lower_dy * lower_dy;

                let upper_dx = next_line[0][0] - upper_intersection[0];
                let upper_dy = next_line[0][1] - upper_intersection[1];
                let upper_d = upper_dx * upper_dx + upper_dy * upper_dy;

                if lower_d < upper_d {
                    next_line[0][0] = lower_intersection[0];
                    next_line[0][1] = lower_intersection[1];
                } else {
                    next_line[0][0] = upper_intersection[0];
                    next_line[0][1] = upper_intersection[1];
                }
            } else if let Some(lower_intersection) = lower_intersection {
                next_line[0][0] = lower_intersection[0];
                next_line[0][1] = lower_intersection[1];
            } else if let Some(upper_intersection) = upper_intersection {
                next_line[0][0] = upper_intersection[0];
                next_line[0][1] = upper_intersection[1];
            } else {
                println!("{:?} {:?}", current_line, next_line);
                panic!("this maybe shouldnt happen");
                let lower_angle = lower_angle - interval;
                let upper_angle = upper_angle - interval;

                let lower_intersection = MathHelper::intersect_ray_ray(
                    *current_line[0],
                    [lower_angle.cos(), lower_angle.sin()],
                    *next_line[0],
                    [next_angle.cos(), next_angle.sin()]);
                let upper_intersection = MathHelper::intersect_ray_ray(
                    *current_line[0],
                    [upper_angle.cos(), upper_angle.sin()],
                    *next_line[0],
                    [next_angle.cos(), next_angle.sin()]);

                if let (Some(lower_intersection), Some(upper_intersection)) = (lower_intersection, upper_intersection) {
                    let lower_dx = next_line[0][0] - lower_intersection[0];
                    let lower_dy = next_line[0][1] - lower_intersection[1];
                    let lower_d = lower_dx * lower_dx + lower_dy * lower_dy;

                    let upper_dx = next_line[0][0] - upper_intersection[0];
                    let upper_dy = next_line[0][1] - upper_intersection[1];
                    let upper_d = upper_dx * upper_dx + upper_dy * upper_dy;

                    if lower_d < upper_d {
                        next_line[0][0] = lower_intersection[0];
                        next_line[0][1] = lower_intersection[1];
                    } else {
                        next_line[0][0] = upper_intersection[0];
                        next_line[0][1] = upper_intersection[1];
                    }
                } else if let Some(lower_intersection) = lower_intersection {
                    next_line[0][0] = lower_intersection[0];
                    next_line[0][1] = lower_intersection[1];
                } else if let Some(upper_intersection) = upper_intersection {
                    next_line[0][0] = upper_intersection[0];
                    next_line[0][1] = upper_intersection[1];
                } else {
                    panic!("This should actually mathematically never happen.");
                }
            }
        }
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
                if self.lines[0][0] == vertex[0] && self.lines[0][1] == vertex[1] {
                    continue;
                }

                self.lines.push(vertex);
            }
        } else if self.lines[self.lines.len() - 1][0] == wall.lines[0][0] && self.lines[self.lines.len() - 1][1] == wall.lines[0][1] {
            for vertex in wall.lines.drain(1..) {
                if self.lines[0][0] == vertex[0] && self.lines[0][1] == vertex[1] {
                    continue;
                }

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

    pub fn split_for_path(&mut self, length: f32, thickness: f32) -> [WallSection; 2] {
        let mut wall_section1 = WallSection::default();
        let mut wall_section2 = WallSection::default();

        let start_length = length - thickness / 2.0;
        let end_length = length + thickness / 2.0;

        let mut cur_length = 0.0;
        for index in 0..(self.lines.len() - 1) {
            let point = self.lines[index];
            let next_point = self.lines[index + 1];

            let dx = next_point[0] - point[0];
            let dy = next_point[1] - point[1];
            let d = (dx * dx + dy * dy).sqrt();

            if cur_length < start_length {
                wall_section1.add_point(&point);
            }

            // if this step contains the desired start length
            if cur_length < start_length && cur_length + d >= start_length {
                let percent = (start_length - cur_length) / d;

                let next_point = [
                    point[0] + dx * percent,
                    point[1] + dy * percent,
                ];
                wall_section1.add_point(&next_point);
            }

            // if this step contains the desired end length
            if cur_length < end_length && cur_length + d >= end_length {
                let percent = (end_length - cur_length) / d;

                let next_point = [
                    point[0] + dx * percent,
                    point[1] + dy * percent,
                ];
                wall_section2.add_point(&next_point);
            }

            cur_length += d;

            if cur_length >= end_length {
                wall_section2.add_point(&next_point);
            }
        }

        return [wall_section1, wall_section2];
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

    pub fn fill_wall(&mut self, generator: &mut Generator, length: f32, angle: f32, desired_angle: f32, desired_angle_strength: f32, distance_wall: Option<&WallSection>) {
        let mut point = [
            angle.cos() * SAFE_ZONE_WIDTH / 2.0,
            angle.sin() * SAFE_ZONE_HEIGHT / 2.0,
        ];

        let inner_offset_length = generator.next() * 400.0;
        point[0] -= angle.cos() * inner_offset_length;
        point[1] -= angle.sin() * inner_offset_length;

        let mut current_wall_length = 0.0;
        let mut current_angle = angle;

        let mut optional_distance = 800.0;

        self.add_point(&point);
        while current_wall_length < length {
            let angle_mod = generator.get_perlin_value(point[0] - 10240.0, point[1] - 10240.0, 1.0);
            current_angle += angle_mod * PI * 0.1;

            let desired_difference = MathHelper::radians_between_angles(current_angle, desired_angle);
            current_angle += desired_difference * desired_angle_strength;

            // let real_current_angle = MathHelper::round_to_interval(current_angle, PI / 8.0);

            let segment_length = 400.0 + 600.0 * generator.next();

            let dx = segment_length * current_angle.cos();
            let dy = segment_length * current_angle.sin();

            current_wall_length += (dx * dx + dy * dy).sqrt();

            point[0] += dx;
            point[1] += dy;

            if let Some(distance_wall) = distance_wall {
                let (nearest_point, nearest_distance) = distance_wall.distance_to_wall(&point);
                if nearest_distance < optional_distance {
                    let difference = optional_distance - nearest_distance;
                    let dx = point[0] - nearest_point[0];
                    let dy = point[1] - nearest_point[1];
                    let d = (dx * dx + dy * dy).sqrt();
                    let dx = dx / d;
                    let dy = dy / d;

                    point[0] += difference * dx;
                    point[1] += difference * dy;
                }
            }

            optional_distance += 200.0;

            self.add_point(&point);
        }
    }

    pub fn remove_loops(&mut self) {
        // create the lookupable tree of lines
        let mut rtree: RTree<Line> = RTree::new();

        let mut new_lines: Vec<Line> = Vec::new();

        // I decided to go with a sub optimal solution that can potentially destroy entire polygons
        // if the intersection happens the wrong way, like by extending through an existing line
        // instead of being a loop

        for index in 0..self.lines.len() {
            let line = Line {
                lines: &self.lines,
                index1: index,
                index2: (index + 1) % self.lines.len(),
            };
            let line_line = line.get_line();
            let line_aabb = line.envelope();

            let collisions = rtree.locate_in_envelope_intersecting(&line_aabb);
            for existing in collisions.into_iter() {
                // if the line we're testing is the previous line skip it

                if existing.index2 == line.index1 || existing.index1 == line.index2 {
                    continue;
                }

                let existing_line = existing.get_line();
                if let Some(intersection) = MathHelper::intersect_line_line(line_line, existing_line) {
                    println!("found collision {:?} {:?}", existing, line);
                    self.lines[line.index1][0] = intersection[0];
                    self.lines[line.index1][1] = intersection[1];

                    // delete from [existing_line.index2, line_line.index1)
                    // so basically keep popping off the last entry until you get to the entry where index2 is existing_line.index2
                    // then change existing_line.index2 to line_line.index1
                    println!("removing from {:?} to {:?}", new_lines[new_lines.len() - 1].index2, existing.index2);
                    while new_lines[new_lines.len() - 1].index2 > existing.index2 {
                        let removed_line = new_lines.pop();

                        if let Some(removed_line) = removed_line {
                            rtree.remove(&removed_line);
                        }
                    }
                    new_lines[new_lines.len() - 1].index2 = line.index1;

                    break;
                }
            }

            new_lines.push(line);
            rtree.insert(line);
        }

        self.lines = new_lines.iter().map(|line| {
            return [self.lines[line.index1][0], self.lines[line.index1][1]];
        }).collect();
    }
}

impl Default for WallSection {
    fn default() -> Self {
        return WallSection {
            lines: Vec::new(),
        }
    }
}