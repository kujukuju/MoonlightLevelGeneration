use crate::{SCREEN_WIDTH, SCREEN_HEIGHT};
use line_drawing::Bresenham;
use crate::perlin::Perlin;
use crate::random::Random;
use std::collections::{HashMap, HashSet};
use crate::math_helper::MathHelper;
use crate::helpers::wall_section::WallSection;
use std::f32::consts::PI;
use std::hash::{Hash, Hasher};
use std::cmp::Ordering;
use crate::helpers::road_segment::RoadSegment;

const NOISE_DETAIL: f32 = 0.0005 / 0.75;
const APPROX_WIDTH: f32 = 10752.0;
const APPROX_HEIGHT: f32 = 10752.0;

const TILE_WIDTH: u32 = 4;
const TILE_HEIGHT: u32 = 3;

pub const TEXTURE_WIDTH: u32 = 32 * 4;
pub const TEXTURE_HEIGHT: u32 = 24 * 4;

// this value forces the rendering to be sampled down to 1 / screen_scale
const SCREEN_SCALE: u32 = 2;

const LEVEL_WIDTH: usize = SCREEN_WIDTH as usize / TILE_WIDTH as usize * SCREEN_SCALE as usize;
const LEVEL_HEIGHT: usize = SCREEN_HEIGHT as usize / TILE_HEIGHT as usize * SCREEN_SCALE as usize;

const NEW_SAFE_ZONE_SCALE_MUL: f32 = 2.0;

pub const SAFE_ZONE_WIDTH: f32 = 3072.0 * NEW_SAFE_ZONE_SCALE_MUL;
pub const SAFE_ZONE_HEIGHT: f32 = 2304.0 * NEW_SAFE_ZONE_SCALE_MUL;

pub struct Generator {
    pixels: Vec<u32>,
    grass: Vec<bool>,
    random: Random,
    noise: Perlin,
    seed: i64,
}

#[derive(Copy, Clone)]
pub struct Angle(pub f32);

impl PartialEq for Angle {
    fn eq(&self, other: &Self) -> bool {
        let bad_angle1 = (self.0 * 100000.0).round() as i32;
        let bad_angle2 = (other.0 * 100000.0).round() as i32;

        return bad_angle1 == bad_angle2;
    }
}

impl PartialOrd for Angle {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let bad_angle1 = (self.0 * 100000.0).round() as i32;
        let bad_angle2 = (other.0 * 100000.0).round() as i32;
        return Some(bad_angle1.cmp(&bad_angle2));
    }
}

impl Eq for Angle {}

impl Hash for Angle {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let bad_angle = (self.0 * 100000.0).round() as i32;
        bad_angle.hash(state);
    }
}

impl Generator {
    pub fn render(&mut self, frame: &mut [u8]) {
        for value in &mut self.pixels {
            *value = 0x000000ff;
        }

        self.generate_level();

        for (i, pixel) in frame.chunks_exact_mut(4).enumerate() {

            let x = (i % SCREEN_WIDTH as usize) as f64 / SCREEN_WIDTH as f64;
            let y = (i / SCREEN_WIDTH as usize) as f64 / SCREEN_HEIGHT as f64;
            let x = (x * LEVEL_WIDTH as f64) as usize;
            let y = (y * LEVEL_HEIGHT as f64) as usize;

            let index = y * LEVEL_WIDTH + x;
            let color = self.pixels[index];

            pixel[0] = ((color & 0xff000000) >> 24) as u8;
            pixel[1] = ((color & 0x00ff0000) >> 16) as u8;
            pixel[2] = ((color & 0x0000ff00) >> 8) as u8;
            pixel[3] = (color & 0x000000ff) as u8;
        }
    }

    fn generate_level(&mut self) {
        self.noise.seed(self.next());

        // road tiles
        let mut bool_tiles = self.create_road_bool_tiles();
        for x in 0..LEVEL_WIDTH as i32 {
            for y in 0..LEVEL_HEIGHT as i32 {
                let index = y * LEVEL_WIDTH as i32 + x;
                let ix = x - LEVEL_WIDTH as i32 / 2;
                let iy = y - LEVEL_HEIGHT as i32 / 2;
                if !bool_tiles.contains_key(&ix) || !bool_tiles[&ix].contains_key(&iy) {
                    continue;
                }

                if bool_tiles[&ix][&iy] {
                    self.grass[index as usize] = false;
                }
            }
        }
        for (x, map) in bool_tiles.into_iter() {
            for (y, gravel) in map.into_iter() {
                self.draw_tile(x, y, if gravel {0xffffff} else {0x43711d}, 1.0);
            }
        }

        // safe zone
        self.draw_oval(0.0, 0.0, SAFE_ZONE_WIDTH, SAFE_ZONE_HEIGHT, 0x39a8e7, 0.5);

        // walls
        self.generate_walls();
    }

    fn generate_walls(&mut self) {
        // generate the angle for the big area walls
        let t1_t3_angle = self.next() * PI * 2.0;
        let t1_t3_angle_original = t1_t3_angle;
        let t3_t2_angle = t1_t3_angle + PI;
        let t3_t2_angle = t3_t2_angle + self.next() * PI * 0.1 - PI * 0.05;

        // generate the angle for the small area divider wall
        let t2_t1_angle = MathHelper::radians_between_angles(t3_t2_angle, t1_t3_angle);
        let t2_t1_angle = if t2_t1_angle > 0.0 {-PI * 2.0 + t2_t1_angle} else {t2_t1_angle};
        let t2_t1_angle = t3_t2_angle + t2_t1_angle * 0.6;

        // generate out the divider walls with random curves and metadata for the thickness along the path
        let mut t1_t3_wall = WallSection::default();
        let desired_wall_length = 30000.0 + 12000.0 * self.next();
        t1_t3_wall.fill_wall(self, 45000.0, t1_t3_angle, t1_t3_angle, 0.08, None);
        let [mut t1_t3_wall_1, mut t1_t3_wall_2] = t1_t3_wall.thicken(self, 200.0, 1200.0);

        let mut t3_t2_wall = WallSection::default();
        let desired_wall_length = 30000.0 + 12000.0 * self.next();
        t3_t2_wall.fill_wall(self, 45000.0, t3_t2_angle, t3_t2_angle, 0.08, None);
        let [mut t3_t2_wall_1, mut t3_t2_wall_2] = t3_t2_wall.thicken(self, 200.0, 1200.0);

        let mut t2_t1_wall = WallSection::default();
        let desired_wall_length = 30000.0 + 12000.0 * self.next();
        t2_t1_wall.fill_wall(self, 45000.0, t2_t1_angle, t2_t1_angle, 0.08, Some(&t1_t3_wall_1));
        let [mut t2_t1_wall_1, mut t2_t1_wall_2] = t2_t1_wall.thicken(self, 200.0, 1200.0);

        // TODO I should do this some better way than literally halving it
        let point1 = t2_t1_wall_2.get_point_at_length(12000.0);
        let point2 = t1_t3_wall_1.get_point_at_length(12000.0);
        let dx = point2[0] - point1[0];
        let dy = point2[1] - point1[1];
        let d = (dx * dx + dy * dy).sqrt();
        let center = [
            (point1[0] + dx / 2.0),
            (point1[1] + dy / 2.0),
        ];
        let dx = dx / d;
        let dy = dy / d;
        let t1_wall_lower_start = [center[0] - dx * 200.0, center[1] - dy * 200.0];
        let t1_wall_upper_start = [center[0] + dx * 200.0, center[1] + dy * 200.0];

        let point1 = t2_t1_wall_2.get_last_point();
        let point2 = t1_t3_wall_1.get_last_point();
        let dx = point2[0] - point1[0];
        let dy = point2[1] - point1[1];
        let d = (dx * dx + dy * dy).sqrt();
        let center = [
            (point1[0] + dx / 2.0),
            (point1[1] + dy / 2.0),
        ];
        let dx = dx / d;
        let dy = dy / d;
        let t1_wall_lower_end = [center[0] - dx * 900.0, center[1] - dy * 900.0];
        let t1_wall_upper_end = [center[0] + dx * 900.0, center[1] + dy * 900.0];

        let mut t1_wall_lower = WallSection::default();
        t1_wall_lower.connect_points_linear(self, t1_wall_lower_start, t1_wall_lower_end);
        t1_wall_lower.noiseify(self, 6000.0, 12.0, [0.0, 0.0], PI / 2.0);

        let mut t1_wall_upper = WallSection::default();
        t1_wall_upper.connect_points_linear(self, t1_wall_upper_start, t1_wall_upper_end);
        t1_wall_upper.noiseify(self, 6000.0, 12.0, [0.0, 0.0], PI / 2.0);

        // t1 inner closing wall
        let mut t1_wall_closing = self.close_walls(&t1_wall_lower, &t1_wall_upper);

        // t1 t3 closing wall
        let mut t1_t3_wall_closing = self.close_walls(&t1_t3_wall_1, &t1_t3_wall_2);

        // t3 t2 closing wall
        let mut t3_t2_wall_closing = self.close_walls(&t3_t2_wall_1, &t3_t2_wall_2);

        let mut t2_t1_wall_closing = self.close_walls(&t2_t1_wall_1, &t2_t1_wall_2);

        // calculate the road segments at the exact edge of the safe zone
        self.generate_roads(0.0, 0.0, SAFE_ZONE_WIDTH, SAFE_ZONE_HEIGHT, 0xff0000, 0.8);

        // back walls
        let tangent_strength = 24000.0;

        // t1 upper bounding wall
        let t1_upper_point = t1_wall_upper.get_last_point();
        let mut length = (t1_upper_point[0] * t1_upper_point[0] + t1_upper_point[1] * t1_upper_point[1]).sqrt().min(32000.0);
        t1_wall_upper.delete_after_length(length);
        let t1_upper_point = t1_wall_upper.get_last_point();
        let t1_upper_tangent = [t1_upper_point[1].atan2(t1_upper_point[0]).cos() * tangent_strength, t1_upper_point[1].atan2(t1_upper_point[0]).sin() * tangent_strength];

        t1_t3_wall_1.delete_after_length(length);
        let t1_t3_point = t1_t3_wall_1.get_last_point();
        let t1_t3_tangent = [t1_t3_point[1].atan2(t1_t3_point[0]).cos() * tangent_strength, t1_t3_point[1].atan2(t1_t3_point[0]).sin() * tangent_strength];

        let mut t1_upper_back_wall = WallSection::default();
        t1_upper_back_wall.connect_points(self, t1_upper_point, t1_upper_tangent, t1_t3_point, t1_t3_tangent);
        let mut center_point = [
            (t1_upper_point[0] + t1_t3_point[0]) / 2.0,
            (t1_upper_point[1] + t1_t3_point[1]) / 2.0,
        ];
        // move the center point slightly more back towards the center
        let center_dist = (center_point[0] * center_point[0] + center_point[1] * center_point[1]).sqrt();
        center_point[0] -= center_point[0] / center_dist * 10000.0;
        center_point[1] -= center_point[1] / center_dist * 10000.0;
        t1_upper_back_wall.noiseify(self, 400.0, 1.0, center_point, 0.0);

        // t1 lower bounding wall
        let t1_lower_point = t1_wall_lower.get_last_point();
        let length = (t1_lower_point[0] * t1_lower_point[0] + t1_lower_point[1] * t1_lower_point[1]).sqrt().min(32000.0);
        t1_wall_lower.delete_after_length(length);
        let t1_lower_point = t1_wall_lower.get_last_point();
        let t1_lower_tangent = [t1_lower_point[1].atan2(t1_lower_point[0]).cos() * tangent_strength, t1_lower_point[1].atan2(t1_lower_point[0]).sin() * tangent_strength];

        t2_t1_wall_2.delete_after_length(length);
        let t2_t1_point = t2_t1_wall_2.get_last_point();
        let t2_t1_tangent = [t2_t1_point[1].atan2(t2_t1_point[0]).cos() * tangent_strength, t2_t1_point[1].atan2(t2_t1_point[0]).sin() * tangent_strength];

        let mut t1_lower_back_wall = WallSection::default();
        t1_lower_back_wall.connect_points(self, t2_t1_point, t2_t1_tangent, t1_lower_point, t1_lower_tangent);
        let mut center_point = [
            (t2_t1_point[0] + t1_lower_point[0]) / 2.0,
            (t2_t1_point[1] + t1_lower_point[1]) / 2.0,
        ];
        // move the center point slightly more back towards the center
        let center_dist = (center_point[0] * center_point[0] + center_point[1] * center_point[1]).sqrt();
        center_point[0] -= center_point[0] / center_dist * 10000.0;
        center_point[1] -= center_point[1] / center_dist * 10000.0;
        t1_lower_back_wall.noiseify(self, 400.0, 1.0, center_point, 0.0);

        let tangent_strength = 54000.0;

        // t2 bounding wall
        let mut length = 18000.0;
        t3_t2_wall_2.delete_after_length(length);
        let t3_t2_point = t3_t2_wall_2.get_last_point();
        let t3_t2_tangent = [t3_t2_point[1].atan2(t3_t2_point[0]).cos() * tangent_strength, t3_t2_point[1].atan2(t3_t2_point[0]).sin() * tangent_strength];

        t2_t1_wall_1.delete_after_length(length);
        let t2_t1_point = t2_t1_wall_1.get_last_point();
        let t2_t1_tangent = [t2_t1_point[1].atan2(t2_t1_point[0]).cos() * tangent_strength, t2_t1_point[1].atan2(t2_t1_point[0]).sin() * tangent_strength];

        let mut t2_back_wall = WallSection::default();
        t2_back_wall.connect_points(self, t3_t2_point, t3_t2_tangent, t2_t1_point, t2_t1_tangent);
        let mut center_point = [
            (t3_t2_point[0] + t2_t1_point[0]) / 2.0,
            (t3_t2_point[1] + t2_t1_point[1]) / 2.0,
        ];
        // move the center point slightly more back towards the center
        let center_dist = (center_point[0] * center_point[0] + center_point[1] * center_point[1]).sqrt();
        center_point[0] -= center_point[0] / center_dist * 10000.0;
        center_point[1] -= center_point[1] / center_dist * 10000.0;
        t2_back_wall.noiseify(self, 400.0, 1.0, center_point, 0.0);

        // t3 back wall
        let length = 24000.0;
        t1_t3_wall_2.delete_after_length(length);
        let t1_t3_point = t1_t3_wall_2.get_last_point();
        let t1_t3_dist = (t1_t3_point[0] * t1_t3_point[0] + t1_t3_point[1] * t1_t3_point[1]).sqrt();
        let t1_t3_angle = t1_t3_point[1].atan2(t1_t3_point[0]);

        let length = 18000.0;
        t3_t2_wall_1.delete_after_length(length);
        let t3_t2_point = t3_t2_wall_1.get_last_point();
        let t3_t2_dist = (t3_t2_point[0] * t3_t2_point[0] + t3_t2_point[1] * t3_t2_point[1]).sqrt();
        let t3_t2_angle = t3_t2_point[1].atan2(t3_t2_point[0]);

        let t1_t3_angle = t1_t3_point[1].atan2(t1_t3_point[0]);
        let t3_t2_angle = t3_t2_point[1].atan2(t3_t2_point[0]);
        let mut angle_diff = MathHelper::radians_between_angles(t3_t2_angle, t1_t3_angle);
        if angle_diff < 0.0 {
            angle_diff += PI * 2.0;
        }

        let start_angle_offset = ((angle_diff - (144.0 / 180.0 * PI)).max(15.0 / 180.0 * PI) + (24.0 / 180.0 * PI)).min(50.0 / 180.0 * PI);
        let start_angle_offset = start_angle_offset;
        let start_angle = t1_t3_angle - start_angle_offset;
        // I remember basic trig
        // adding this back makes it so it doesn't extend as far when bend obtusely or whatever
        let start_length = t1_t3_dist / (start_angle_offset + MathHelper::radians_between_angles(t1_t3_angle, t1_t3_angle_original).min(0.0)).cos();
        let start_point = [start_angle.cos() * start_length, start_angle.sin() * start_length];

        let end_angle_offset = 40.0 / 180.0 * PI;
        let end_length = t3_t2_dist / end_angle_offset.cos();
        let end_angle = t3_t2_angle + end_angle_offset;
        let end_point = [end_angle.cos() * end_length, end_angle.sin() * end_length];

        let back_wall_angle = (end_point[1] - start_point[1]).atan2(end_point[0] - start_point[0]);

        let mut t3_back_wall = WallSection::default();
        t3_back_wall.connect_points_linear(self, start_point, end_point);

        let tangent_strength = 8000.0;

        let mut t1_t3_side_wall = WallSection::default();
        t1_t3_side_wall.connect_points(
            self,
            t1_t3_point,
            [t1_t3_angle.cos() * tangent_strength, t1_t3_angle.sin() * tangent_strength],
            start_point,
            [-back_wall_angle.cos() * tangent_strength, -back_wall_angle.sin() * tangent_strength]);
        t1_t3_side_wall.join_wall(t3_back_wall);
        let mut t3_back_wall = t1_t3_side_wall;
        // t3_back_wall.join_wall(t1_t3_side_wall);

        let mut t3_t2_side_wall = WallSection::default();
        t3_t2_side_wall.connect_points(
            self,
            end_point,
            [back_wall_angle.cos() * tangent_strength, back_wall_angle.sin() * tangent_strength],
            t3_t2_point,
            [t3_t2_angle.cos() * tangent_strength, t3_t2_angle.sin() * tangent_strength]);
        t3_back_wall.join_wall(t3_t2_side_wall);

        t3_back_wall.noiseify(self, 8000.0, 20.0, [0.0, 0.0], 0.0);
        t3_back_wall.noiseify(self, 400.0, 1.0, [0.0, 0.0], 0.0);

        // self.draw_line(start_point[0], start_point[1], end_point[0], end_point[1], 0xff0000, 1.0);

        // t2 t1 connector
        let t2_t1_wall_1_length = t2_t1_wall_1.get_length();
        let t2_t1_wall_2_length = t2_t1_wall_2.get_length();
        let t2_t1_path_length = t2_t1_wall_1_length.min(t2_t1_wall_2_length) * (0.6 + self.next() * 0.3);

        let t2_t1_path_thickness = 900.0 + 1400.0 * self.next();
        let [mut t2_t1_wall_1_split_1, mut t2_t1_wall_1_split_2] = t2_t1_wall_1.split_for_path(t2_t1_path_length, t2_t1_path_thickness);

        let [mut t2_t1_wall_2_split_1, mut t2_t1_wall_2_split_2] = t2_t1_wall_2.split_for_path(t2_t1_path_length, t2_t1_path_thickness);

        let mut t2_t1_path_wall_1 = WallSection::default();
        let point1 = t2_t1_wall_1_split_1.get_last_point();
        let tangent1 = [
            point1[1].atan2(point1[0]).cos() * 1000.0,
            point1[1].atan2(point1[0]).sin() * 1000.0,
        ];
        let point2 = t2_t1_wall_2_split_1.get_last_point();
        let tangent2 = [
            point2[1].atan2(point2[0]).cos() * 1000.0,
            point2[1].atan2(point2[0]).sin() * 1000.0,
        ];
        t2_t1_path_wall_1.connect_points(self, point1, tangent1, point2, tangent2);

        let mut t2_t1_path_wall_2 = WallSection::default();
        let point1 = t2_t1_wall_1_split_2.get_first_point();
        let tangent1 = [
            -point1[1].atan2(point1[0]).cos() * 1000.0,
            -point1[1].atan2(point1[0]).sin() * 1000.0,
        ];
        let point2 = t2_t1_wall_2_split_2.get_first_point();
        let tangent2 = [
            -point2[1].atan2(point2[0]).cos() * 1000.0,
            -point2[1].atan2(point2[0]).sin() * 1000.0,
        ];
        t2_t1_path_wall_2.connect_points(self, point1, tangent1, point2, tangent2);

        t2_t1_wall_1_split_1.join_wall(t2_t1_path_wall_1);
        t2_t1_wall_1_split_1.join_wall(t2_t1_wall_2_split_1);
        t2_t1_wall_1_split_1.join_wall(t2_t1_wall_closing);

        let mut t2_t1_inner = t2_t1_wall_1_split_1;

        // t3 t2 connector
        let t3_t2_wall_1_length = t3_t2_wall_1.get_length();
        let t3_t2_wall_2_length = t3_t2_wall_2.get_length();
        let t3_t2_path_length = t3_t2_wall_1_length.min(t3_t2_wall_2_length) * (0.6 + self.next() * 0.3);

        let t3_t2_path_thickness = 900.0 + 1400.0 * self.next();
        let [mut t3_t2_wall_1_split_1, mut t3_t2_wall_1_split_2] = t3_t2_wall_1.split_for_path(t3_t2_path_length, t3_t2_path_thickness);

        let [mut t3_t2_wall_2_split_1, mut t3_t2_wall_2_split_2] = t3_t2_wall_2.split_for_path(t3_t2_path_length, t3_t2_path_thickness);

        let mut t3_t2_path_wall_1 = WallSection::default();
        let point1 = t3_t2_wall_1_split_1.get_last_point();
        let tangent1 = [
            point1[1].atan2(point1[0]).cos() * 1000.0,
            point1[1].atan2(point1[0]).sin() * 1000.0,
        ];
        let point2 = t3_t2_wall_2_split_1.get_last_point();
        let tangent2 = [
            point2[1].atan2(point2[0]).cos() * 1000.0,
            point2[1].atan2(point2[0]).sin() * 1000.0,
        ];
        t3_t2_path_wall_1.connect_points(self, point1, tangent1, point2, tangent2);

        let mut t3_t2_path_wall_2 = WallSection::default();
        let point1 = t3_t2_wall_1_split_2.get_first_point();
        let tangent1 = [
            -point1[1].atan2(point1[0]).cos() * 1000.0,
            -point1[1].atan2(point1[0]).sin() * 1000.0,
        ];
        let point2 = t3_t2_wall_2_split_2.get_first_point();
        let tangent2 = [
            -point2[1].atan2(point2[0]).cos() * 1000.0,
            -point2[1].atan2(point2[0]).sin() * 1000.0,
        ];
        t3_t2_path_wall_2.connect_points(self, point1, tangent1, point2, tangent2);

        t3_t2_wall_1_split_1.join_wall(t3_t2_path_wall_1);
        t3_t2_wall_1_split_1.join_wall(t3_t2_wall_2_split_1);
        t3_t2_wall_1_split_1.join_wall(t3_t2_wall_closing);

        let mut t3_t2_inner = t3_t2_wall_1_split_1;

        // t1 first connector
        let t1_wall_lower_length = t1_wall_lower.get_length();
        let t1_wall_upper_length = t1_wall_upper.get_length();
        let t1_path_length = t1_wall_lower_length.min(t1_wall_upper_length) * (0.25 + self.next() * 0.25);

        let t1_path_thickness = 900.0 + 1400.0 * self.next();
        let [mut t1_wall_lower_split_1, mut t1_wall_lower_split_2] = t1_wall_lower.split_for_path(t1_path_length, t1_path_thickness);

        let [mut t1_wall_upper_split_1, mut t1_wall_upper_split_2] = t1_wall_upper.split_for_path(t1_path_length, t1_path_thickness);

        let mut t1_path_wall_1 = WallSection::default();
        let point1 = t1_wall_lower_split_1.get_last_point();
        let tangent1 = [
            point1[1].atan2(point1[0]).cos() * 1000.0,
            point1[1].atan2(point1[0]).sin() * 1000.0,
        ];
        let point2 = t1_wall_upper_split_1.get_last_point();
        let tangent2 = [
            point2[1].atan2(point2[0]).cos() * 1000.0,
            point2[1].atan2(point2[0]).sin() * 1000.0,
        ];
        t1_path_wall_1.connect_points(self, point1, tangent1, point2, tangent2);

        let mut t1_path_wall_2 = WallSection::default();
        let point1 = t1_wall_lower_split_2.get_first_point();
        let tangent1 = [
            -point1[1].atan2(point1[0]).cos() * 1000.0,
            -point1[1].atan2(point1[0]).sin() * 1000.0,
        ];
        let point2 = t1_wall_upper_split_2.get_first_point();
        let tangent2 = [
            -point2[1].atan2(point2[0]).cos() * 1000.0,
            -point2[1].atan2(point2[0]).sin() * 1000.0,
        ];
        t1_path_wall_2.connect_points(self, point1, tangent1, point2, tangent2);

        t1_wall_lower_split_1.join_wall(t1_path_wall_1);
        t1_wall_lower_split_1.join_wall(t1_wall_upper_split_1);
        t1_wall_lower_split_1.join_wall(t1_wall_closing);

        let mut t1_inner_1 = t1_wall_lower_split_1;

        // t1 second connector
        let t1_wall_lower_length = t1_wall_lower_split_2.get_length();
        let t1_wall_upper_length = t1_wall_upper_split_2.get_length();
        let t1_path_length = t1_wall_lower_length.min(t1_wall_upper_length) * (0.50 + self.next() * 0.40);

        let t1_path_thickness = 900.0 + 1400.0 * self.next();
        let [mut t1_wall_lower_split_2, mut t1_wall_lower_split_3] = t1_wall_lower_split_2.split_for_path(t1_path_length, t1_path_thickness);

        let [mut t1_wall_upper_split_2, mut t1_wall_upper_split_3] = t1_wall_upper_split_2.split_for_path(t1_path_length, t1_path_thickness);

        let mut t1_path_wall_3 = WallSection::default();
        let point1 = t1_wall_lower_split_2.get_last_point();
        let tangent1 = [
            point1[1].atan2(point1[0]).cos() * 1000.0,
            point1[1].atan2(point1[0]).sin() * 1000.0,
        ];
        let point2 = t1_wall_upper_split_2.get_last_point();
        let tangent2 = [
            point2[1].atan2(point2[0]).cos() * 1000.0,
            point2[1].atan2(point2[0]).sin() * 1000.0,
        ];
        t1_path_wall_3.connect_points(self, point1, tangent1, point2, tangent2);

        let mut t1_path_wall_4 = WallSection::default();
        let point1 = t1_wall_lower_split_3.get_first_point();
        let tangent1 = [
            -point1[1].atan2(point1[0]).cos() * 1000.0,
            -point1[1].atan2(point1[0]).sin() * 1000.0,
        ];
        let point2 = t1_wall_upper_split_3.get_first_point();
        let tangent2 = [
            -point2[1].atan2(point2[0]).cos() * 1000.0,
            -point2[1].atan2(point2[0]).sin() * 1000.0,
        ];
        t1_path_wall_4.connect_points(self, point1, tangent1, point2, tangent2);

        t1_wall_lower_split_2.join_wall(t1_path_wall_3);
        t1_wall_lower_split_2.join_wall(t1_wall_upper_split_2);
        t1_wall_lower_split_2.join_wall(t1_path_wall_2);

        let mut t1_inner_2 = t1_wall_lower_split_2;

        t1_t3_wall_2.join_wall(t3_back_wall);
        t1_t3_wall_2.join_wall(t3_t2_wall_1_split_2);
        t1_t3_wall_2.join_wall(t3_t2_path_wall_2);
        t1_t3_wall_2.join_wall(t3_t2_wall_2_split_2);
        t1_t3_wall_2.join_wall(t2_back_wall);
        t1_t3_wall_2.join_wall(t2_t1_wall_1_split_2);
        t1_t3_wall_2.join_wall(t2_t1_path_wall_2);
        t1_t3_wall_2.join_wall(t2_t1_wall_2_split_2);
        t1_t3_wall_2.join_wall(t1_lower_back_wall);
        t1_t3_wall_2.join_wall(t1_wall_lower_split_3);
        t1_t3_wall_2.join_wall(t1_path_wall_4);
        t1_t3_wall_2.join_wall(t1_wall_upper_split_3);
        t1_t3_wall_2.join_wall(t1_upper_back_wall);
        t1_t3_wall_2.join_wall(t1_t3_wall_1);
        t1_t3_wall_2.join_wall(t1_t3_wall_closing);

        let mut outer_wall = t1_t3_wall_2;

        t1_inner_1.round_to_angle(PI / 8.0);
        t1_inner_1.remove_loops();
        t1_inner_2.round_to_angle(PI / 8.0);
        t1_inner_2.remove_loops();
        t2_t1_inner.round_to_angle(PI / 8.0);
        t2_t1_inner.remove_loops();
        t3_t2_inner.round_to_angle(PI / 8.0);
        t3_t2_inner.remove_loops();
        outer_wall.round_to_angle(PI / 8.0);
        outer_wall.remove_loops();

        t1_inner_1.render(self, 0x000000);
        t1_inner_2.render(self, 0x000000);

        t2_t1_inner.render(self, 0x000000);
        t3_t2_inner.render(self, 0x000000);

        outer_wall.render(self, 0x000000);

        // t1_t3_wall.render(self, 0x880044);
        // t1_t3_wall_1.render(self, 0x000044);
        // t1_t3_wall_2.render(self, 0x000044);
        //
        // // t3_t2_wall.render(self, 0x880044);
        // // t3_t2_wall_1.render(self, 0x000044);
        // // t3_t2_wall_2.render(self, 0x000044);
        // t3_t2_wall_1_split_2.render(self, 0x440088);
        // t3_t2_wall_2_split_2.render(self, 0x440088);
        // t3_t2_path_wall_2.render(self, 0x880044);
        //
        // // t2_t1_wall.render(self, 0x880044);
        // // t2_t1_wall_1_split_1.render(self, 0x000000);
        // t2_t1_wall_1_split_2.render(self, 0x000000);
        // // t2_t1_wall_1.render(self, 0x000044);
        // // t2_t1_wall_2_split_1.render(self, 0x000000);
        // t2_t1_wall_2_split_2.render(self, 0x000000);
        // // t2_t1_wall_2.render(self, 0x000044);
        //
        // // t2_t1_path_wall_1.render(self, 0x440088);
        // t2_t1_path_wall_2.render(self, 0x440088);
        //
        // // t1_wall_inner.render(self, 0x880044);
        // // t1_wall_lower.render(self, 0x000044);
        // // t1_wall_upper.render(self, 0x000044);
        // // t1_wall_lower_split_2.render(self, 0x000044);
        // // t1_wall_upper_split_2.render(self, 0x000044);
        // // t1_path_wall_2.render(self, 0x880044);
        // t1_wall_lower_split_3.render(self, 0x000044);
        // t1_wall_upper_split_3.render(self, 0x000044);
        // t1_path_wall_4.render(self, 0x000044);
        //
        // t1_upper_back_wall.render(self, 0x440088);
        // t1_lower_back_wall.render(self, 0x440088);
        //
        // t2_back_wall.render(self, 0x440088);
        // t3_back_wall.render(self, 0x440088);
        //
        // // t1_wall_closing.render(self, 0x440088);
        // t1_t3_wall_closing.render(self, 0x440088);
        // // t3_t2_wall_closing.render(self, 0x440088);
        // // t2_t1_wall_closing.render(self, 0x440088);
    }

    pub fn close_walls(&mut self, lower: &WallSection, upper: &WallSection) -> WallSection {
        let mut wall_closing = WallSection::default();
        let lower_point = lower.get_first_point();
        let lower_angle = lower_point[1].atan2(lower_point[0]);
        let upper_point = upper.get_first_point();
        let upper_angle = upper_point[1].atan2(upper_point[0]);
        wall_closing.connect_points(
            self,
            lower_point,
            [-lower_angle.cos() * 2000.0, -lower_angle.sin() * 2000.0],
            upper_point,
            [-upper_angle.cos() * 2000.0, -upper_angle.sin() * 2000.0]);

        return wall_closing;
    }

    pub fn generate_roads(&mut self, center_x: f32, center_y: f32, width: f32, height: f32, color: u32, alpha: f32) {
        let mut point_map = HashMap::new();

        let max = width.max(height);
        for angle in 0..200 {
            let angle = (angle as f32 / 200.0) * PI * 2.0;

            let point = [angle.cos() * max, angle.sin() * max];
            let (point, distance) = MathHelper::distance_to_ellipse(center_x, center_y, width / 2.0, height / 2.0, &point);
            let point = [
                (point[0] / TEXTURE_WIDTH as f32) as i32,
                (point[1] / TEXTURE_HEIGHT as f32) as i32,
            ];

            let angle = (point[1] as f32).atan2(point[0] as f32);
            point_map.insert(Angle(angle), point);
        }

        let mut sorted_tiles: Vec<(Angle, [i32; 2])> = point_map.into_iter().collect();
        sorted_tiles.sort_by(|first, second| {
            return first.0.partial_cmp(&second.0).unwrap();
        });

        // find the first grass tile to start on, to ensure no roads are cut in half
        let mut start_index = 0;
        for (index, (angle, tile)) in sorted_tiles.iter().enumerate() {
            if self.is_tile_grass(tile[0], tile[1]) {
                start_index = index;
                break;
            }
        }

        let mut road_segments = Vec::new();
        let mut start_road_segment: Option<usize> = None;
        let mut end_road_segment: Option<usize> = None;
        // less than or equal so it will do +1 allowing it to finish off the last road
        for i in 0..=sorted_tiles.len() {
            let index = (start_index + i) % sorted_tiles.len();
            let (angle, tile) = sorted_tiles[index];

            if !self.is_tile_grass(tile[0], tile[1]) {
                if start_road_segment.is_none() {
                    start_road_segment = Some(index);
                }

                end_road_segment = Some(index);
            }
            if self.is_tile_grass(tile[0], tile[1]) && start_road_segment.is_some() {
                let (start_angle, start_tile) = sorted_tiles[start_road_segment.unwrap()];
                let (end_angle, end_tile) = sorted_tiles[end_road_segment.unwrap()];

                let start = [
                    start_tile[0] as f32 * TEXTURE_WIDTH as f32,
                    start_tile[1] as f32 * TEXTURE_HEIGHT as f32,
                ];
                let end = [
                    end_tile[0] as f32 * TEXTURE_WIDTH as f32,
                    end_tile[1] as f32 * TEXTURE_HEIGHT as f32,
                ];
                // let dx = end[0] - start[0];
                // let dy = end[1] - start[1];
                // let d = (dx * dx + dy * dy).sqrt();
                // let dx = dx / d;
                // let dy = dy / d;
                // let start = [
                //     start[0] - dx * TEXTURE_WIDTH as f32 / 2.0,
                //     start[1] - dy * TEXTURE_HEIGHT as f32 / 2.0,
                // ];
                // let end = [
                //     end[0] + dx * TEXTURE_WIDTH as f32 / 2.0,
                //     end[1] + dy * TEXTURE_HEIGHT as f32 / 2.0,
                // ];
                let center = [
                    (start[0] + end[0]) / 2.0,
                    (start[1] + end[1]) / 2.0,
                ];
                let dx = end[0] - start[0];
                let dy = end[1] - start[1];
                let d = (dx * dx + dy * dy).sqrt();

                // this is bad

                road_segments.push(RoadSegment::create(self, center, start_angle.0, end_angle.0, d));

                start_road_segment = None;
                end_road_segment = None;
            }
        }

        for road_segment in &mut road_segments {
            let children = road_segment.extend(self, 40000.0);
            road_segment.render(self);
            for child in children {s
                child.render(self);
            }
        }
    }

    // fn fill_wall(&mut self, wall: &mut WallSection, length: f32, angle: f32, desired_angle: f32, desired_angle_strength: f32, distance_wall: Option<&WallSection>) {
    //     let mut point = [
    //         angle.cos() * SAFE_ZONE_WIDTH / 2.0,
    //         angle.sin() * SAFE_ZONE_HEIGHT / 2.0,
    //     ];
    //
    //     let inner_offset_length = self.next() * 400.0;
    //     point[0] -= angle.cos() * inner_offset_length;
    //     point[1] -= angle.sin() * inner_offset_length;
    //
    //     let mut current_wall_length = 0.0;
    //     let mut current_angle = angle;
    //
    //     let mut optional_distance = 800.0;
    //
    //     wall.add_point(&point);
    //     while current_wall_length < length {
    //         let angle_mod = self.get_perlin_value(point[0] - 10240.0, point[1] - 10240.0, 1.0);
    //         current_angle += angle_mod * PI * 0.1;
    //
    //         let desired_difference = MathHelper::radians_between_angles(current_angle, desired_angle);
    //         current_angle += desired_difference * desired_angle_strength;
    //
    //         let real_current_angle = MathHelper::round_to_interval(current_angle, PI / 8.0);
    //
    //         let segment_length = 200.0 + 400.0 * self.next();
    //
    //         let dx = segment_length * real_current_angle.cos();
    //         let dy = segment_length * real_current_angle.sin();
    //
    //         current_wall_length += (dx * dx + dy * dy).sqrt();
    //
    //         point[0] += dx;
    //         point[1] += dy;
    //
    //         if let Some(distance_wall) = distance_wall {
    //             let (nearest_point, nearest_distance) = distance_wall.distance_to_wall(&point);
    //             if nearest_distance < optional_distance {
    //                 let difference = optional_distance - nearest_distance;
    //                 let dx = point[0] - nearest_point[0];
    //                 let dy = point[1] - nearest_point[1];
    //                 let d = (dx * dx + dy * dy).sqrt();
    //                 let dx = dx / d;
    //                 let dy = dy / d;
    //
    //                 point[0] += difference * dx;
    //                 point[1] += difference * dy;
    //             }
    //         }
    //
    //         optional_distance += 200.0;
    //
    //         wall.add_point(&point);
    //     }
    // }

    fn create_road_bool_tiles(&mut self) -> HashMap<i32, HashMap<i32, bool>> {
        let mut tiles: HashMap<i32, HashMap<i32, bool>> = HashMap::new();

        let width = (SAFE_ZONE_WIDTH / TEXTURE_WIDTH as f32) as i32;
        let height = (SAFE_ZONE_HEIGHT / TEXTURE_HEIGHT as f32) as i32;

        let center_x = 0;
        let center_y = 0;

        let aabb = [
            [-width / 2, -height / 2],
            [width / 2, height / 2],
        ];

        for x in (-(LEVEL_WIDTH as i32) / 2)..(LEVEL_WIDTH as i32 / 2) {
            let tiles = tiles.entry(x).or_default();

            for y in (-(LEVEL_HEIGHT as i32) / 2)..(LEVEL_HEIGHT as i32 / 2) {
                let inside_center = true;
                let inside_center = inside_center && x >= aabb[0][0];
                let inside_center = inside_center && x < aabb[1][0];
                let inside_center = inside_center && y >= aabb[0][1];
                let inside_center = inside_center && y < aabb[1][1];

                if inside_center {
                    let dx = (x - center_x) as f32 / (width as f32 / 2.0);
                    let dy = (y - center_y) as f32 / (height as f32 / 2.0);
                    let d2 = dx * dx + dy * dy;

                    if d2 <= 1.0 {
                        let position_x = (x * TEXTURE_WIDTH as i32 + TEXTURE_WIDTH as i32 / 2) as f32;
                        let position_y = (y * TEXTURE_HEIGHT as i32 + TEXTURE_HEIGHT as i32 / 2) as f32;

                        let (is_road, road_strength) = self.sample_road(position_x, position_y);
                        if is_road {
                            // gravel
                            tiles.insert(y, true);
                        } else {
                            // grass
                            tiles.insert(y, false);
                        }
                    } else {
                        tiles.insert(y, false);
                    }
                } else {
                    tiles.insert(y, false);
                }
            }
        }

        return tiles;
    }

    pub fn sample_road(&mut self, x: f32, y: f32) -> (bool, f32) {
        let distance = x * x + y * y;
        let scale = (2.0 as f32).max(10000000.0 / distance);
        let noise1 = self.get_perlin_value(x, y, scale);
        let noise2 = self.get_perlin_value(x + APPROX_WIDTH, x + APPROX_HEIGHT, scale);
        if noise1.abs() < 0.05 || noise2.abs() < 0.05 {
            return (true, noise1.abs().min(noise2.abs()));
        }

        return (false, noise1.abs().min(noise2.abs()));
    }

    pub fn draw_tile(&mut self, x: i32, y: i32, color: u32, alpha: f32) {
        let center_x = (LEVEL_WIDTH / 2) as i32;
        let center_y = (LEVEL_HEIGHT / 2) as i32;

        // dont draw outside bounds, itll crash at casting
        if (x + center_x as i32) < 0 || (y + center_y as i32) < 0 {
            return;
        }

        let x = (center_x + x) as u32;
        let y = (center_y + y) as u32;

        self.draw_rect(x, y, 1, 1, color, alpha);
    }

    pub fn draw_oval(&mut self, center_x: f32, center_y: f32, width: f32, height: f32, color: u32, alpha: f32) {
        let center_x = (center_x / TEXTURE_WIDTH as f32) as i32;
        let center_y = (center_y / TEXTURE_HEIGHT as f32) as i32;

        let width = (width / TEXTURE_WIDTH as f32) as i32;
        let height = (height / TEXTURE_HEIGHT as f32) as i32;

        for x in (center_x - width as i32 / 2)..(center_x + width as i32 / 2) {
            for y in (center_y - height as i32 / 2)..(center_y + height as i32 / 2) {
                let dx = (x - center_x) as f32 / (width as f32 / 2.0);
                let dy = (y - center_y) as f32 / (height as f32 / 2.0);
                let d2 = dx * dx + dy * dy;

                if d2 <= 1.0 {
                    self.draw_tile(x, y, color, alpha);
                }
            }
        }
    }

    pub fn draw_line(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, color: u32, alpha: f32) {
        let x1 = (x1 / TEXTURE_WIDTH as f32) as i32;
        let y1 = (y1 / TEXTURE_HEIGHT as f32) as i32;
        let x2 = (x2 / TEXTURE_WIDTH as f32) as i32;
        let y2 = (y2 / TEXTURE_HEIGHT as f32) as i32;

        for (x, y) in Bresenham::new((x1 as i32, y1 as i32), (x2 as i32, y2 as i32)) {
            self.draw_tile(x, y, color, alpha);
        }
    }

    pub fn draw_line_thickness(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, radius: f32, color: u32, alpha: f32) {
        let aabb = [
            [((x1.min(x2) - radius) / TEXTURE_WIDTH as f32).floor() as i32, ((y1.min(y2) - radius) / TEXTURE_HEIGHT as f32).floor() as i32],
            [((x1.max(x2) + radius) / TEXTURE_WIDTH as f32).ceil() as i32, ((y1.max(y2) + radius) / TEXTURE_HEIGHT as f32).ceil() as i32],
        ];

        for x in aabb[0][0]..=aabb[1][0] {
            for y in aabb[0][1]..=aabb[1][1] {
                let cx = x as f32 * TEXTURE_WIDTH as f32 + TEXTURE_WIDTH as f32 / 2.0;
                let cy = y as f32 * TEXTURE_HEIGHT as f32 + TEXTURE_HEIGHT as f32 / 2.0;

                if MathHelper::is_point_inside_ellipse([cx, cy], [0.0, 0.0], [SAFE_ZONE_WIDTH - TEXTURE_WIDTH as f32 * 2.0, SAFE_ZONE_HEIGHT - TEXTURE_HEIGHT as f32 * 2.0]) {
                    continue;
                }

                let (point, distance) = MathHelper::distance_to_line_segment(&[[x1, y1], [x2, y2]], &[cx, cy]);
                if distance - 0.000001 <= radius {
                    self.draw_tile(x, y, color, alpha);
                }
            }
        }
    }

    fn is_tile_grass(&self, x: i32, y: i32) -> bool {
        if let Some(tile_index) = Generator::grid_index((x + LEVEL_WIDTH as i32 / 2) as u32, (y + LEVEL_HEIGHT as i32 / 2) as u32) {
            return self.grass[tile_index];
        }

        return false;
    }

    fn draw_rect(&mut self, x: u32, y: u32, width: u32, height: u32, color: u32, alpha: f32) {
        for offset_y in 0..height {
            let y = y + offset_y;

            for offset_x in 0..width {
                let x = x + offset_x;

                if let Some(index) = Generator::grid_index(x, y) {
                    let orig_color_r = (self.pixels[index] & 0xff000000) >> 24;
                    let orig_color_g = (self.pixels[index] & 0x00ff0000) >> 16;
                    let orig_color_b = (self.pixels[index] & 0x0000ff00) >> 8;

                    let color_r = (color & 0xff0000) >> 16;
                    let color_g = (color & 0x00ff00) >> 8;
                    let color_b = color & 0x0000ff;

                    let new_color_r = (orig_color_r as f32 * (1.0 - alpha) + color_r as f32 * alpha).round() as u32;
                    let new_color_g = (orig_color_g as f32 * (1.0 - alpha) + color_g as f32 * alpha).round() as u32;
                    let new_color_b = (orig_color_b as f32 * (1.0 - alpha) + color_b as f32 * alpha).round() as u32;

                    self.pixels[index] = (new_color_r << 24) | (new_color_g << 16) | (new_color_b << 8) | 0x000000ff;
                }
            }
        }
    }

    fn grid_index(x: u32, y: u32) -> Option<usize> {
        let index = y * LEVEL_WIDTH as u32 + x;
        if index < 0 || index >= (LEVEL_WIDTH * LEVEL_HEIGHT) as u32 {
            return None;
        }

        return Some(index as usize);
    }

    pub fn next(&mut self) -> f32 {
        return self.random.next();
    }

    pub fn get_perlin_value(&mut self, x: f32, y: f32, scale: f32) -> f32 {
        return self.noise.perlin2(x * (NOISE_DETAIL / scale), y * (NOISE_DETAIL / scale) / 0.75);
    }
}

impl Default for Generator {
    fn default() -> Self {
        let seed: u32 = rand::random();
        // let seed: u32 = 2996010972;

        // TODO cursed seed to try before finalizing
        // let seed: u32 = 1835892476;
        println!("SEED {:?}", seed);

        return Generator {
            pixels: vec![0; (LEVEL_WIDTH * LEVEL_HEIGHT) as usize],
            grass: vec![true; (LEVEL_WIDTH * LEVEL_HEIGHT) as usize],
            random: Random::create(seed as i64),
            noise: Perlin::default(),
            seed: seed as i64,
        };
    }
}
