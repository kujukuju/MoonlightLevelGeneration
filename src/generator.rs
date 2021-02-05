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
const SCREEN_SCALE: u32 = 1;

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
        let t3_t2_angle = t1_t3_angle + PI;
        let t3_t2_angle = t3_t2_angle + self.next() * PI * 0.1 - PI * 0.05;

        // generate the angle for the small area divider wall
        let t2_t1_angle = MathHelper::radians_between_angles(t3_t2_angle, t1_t3_angle);
        let t2_t1_angle = if t2_t1_angle > 0.0 {-PI * 2.0 + t2_t1_angle} else {t2_t1_angle};
        let t2_t1_angle = t3_t2_angle + t2_t1_angle * 0.6;

        // generate out the divider walls with random curves and metadata for the thickness along the path
        let mut t1_t3_wall = WallSection::default();
        let desired_wall_length = 30000.0 + 12000.0 * self.next();
        self.fill_wall(&mut t1_t3_wall, 45000.0, t1_t3_angle, t1_t3_angle, 0.02, None);
        let [mut t1_t3_wall_1, mut t1_t3_wall_2] = t1_t3_wall.thicken(self, 200.0, 1200.0);
        t1_t3_wall.render(self, 0x880044);
        t1_t3_wall_1.render(self, 0x000044);
        t1_t3_wall_2.render(self, 0x000044);

        let mut t3_t2_wall = WallSection::default();
        let desired_wall_length = 30000.0 + 12000.0 * self.next();
        self.fill_wall(&mut t3_t2_wall, 45000.0, t3_t2_angle, t3_t2_angle, 0.02, None);
        let [mut t3_t2_wall_1, mut t3_t2_wall_2] = t3_t2_wall.thicken(self, 200.0, 1200.0);
        t3_t2_wall.render(self, 0x880044);
        t3_t2_wall_1.render(self, 0x000044);
        t3_t2_wall_2.render(self, 0x000044);

        let mut t2_t1_wall = WallSection::default();
        let desired_wall_length = 30000.0 + 12000.0 * self.next();
        self.fill_wall(&mut t2_t1_wall, 45000.0, t2_t1_angle, t2_t1_angle, 0.02, Some(&t1_t3_wall_1));
        let [mut t2_t1_wall_1, mut t2_t1_wall_2] = t2_t1_wall.thicken(self, 200.0, 1200.0);
        t2_t1_wall.render(self, 0x880044);
        t2_t1_wall_1.render(self, 0x000044);
        t2_t1_wall_2.render(self, 0x000044);

        let [mut t1_wall_inner, mut t1_wall_lower, mut t1_wall_upper] = self.fill_between_wall(&t2_t1_wall_2, &t1_t3_wall_1, 12000.0);
        t1_wall_inner.render(self, 0x880044);
        t1_wall_lower.render(self, 0x000044);
        t1_wall_upper.render(self, 0x000044);

        // calculate the road segments at the exact edge of the safe zone
        self.generate_roads(0.0, 0.0, SAFE_ZONE_WIDTH, SAFE_ZONE_HEIGHT, 0xff0000, 0.8);


        // at the tip of each wall, extend the t3 t2 divider on both sides

        // extend the t3 t1 divider towards t3

        // extend the t2 t1 divider towards t2


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
                let angle = start_angle.0 + MathHelper::radians_between_angles(start_angle.0, end_angle.0) / 2.0;

                road_segments.push(RoadSegment::create(self, center, d, angle));

                start_road_segment = None;
                end_road_segment = None;
            }
        }

        for road_segment in &road_segments {
            road_segment.render(self);
        }
    }

    fn fill_wall(&mut self, wall: &mut WallSection, length: f32, angle: f32, desired_angle: f32, desired_angle_strength: f32, distance_wall: Option<&WallSection>) {
        let mut point = [
            angle.cos() * SAFE_ZONE_WIDTH / 2.0,
            angle.sin() * SAFE_ZONE_HEIGHT / 2.0,
        ];

        let inner_offset_length = self.next() * 400.0;
        point[0] -= angle.cos() * inner_offset_length;
        point[1] -= angle.sin() * inner_offset_length;

        let mut current_wall_length = 0.0;
        let mut current_angle = angle;

        let mut optional_distance = 800.0;

        wall.add_point(&point);
        while current_wall_length < length {
            let angle_mod = self.get_perlin_value(point[0] - 10240.0, point[1] - 10240.0, 1.0);
            current_angle += angle_mod * PI * 0.1;

            let desired_difference = MathHelper::radians_between_angles(current_angle, desired_angle);
            current_angle += desired_difference * desired_angle_strength;

            let real_current_angle = MathHelper::round_to_interval(current_angle, PI / 8.0);

            let segment_length = 200.0 + 400.0 * self.next();

            let dx = segment_length * real_current_angle.cos();
            let dy = segment_length * real_current_angle.sin();

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

            wall.add_point(&point);
        }
    }

    fn fill_between_wall(&mut self, wall1: &WallSection, wall2: &WallSection, start_length: f32) -> [WallSection; 3] {
        let mut distance1 = 0.0;
        let mut distance2 = 0.0;

        let mut index1: usize = 0;
        let mut index2: usize = 0;

        let mut wall_inner = WallSection::default();
        let mut wall_lower = WallSection::default();
        let mut wall_upper = WallSection::default();

        loop {
            if index1 >= wall1.lines.len() - 1 {
                break;
            }

            let point = wall1.lines[index1];
            let next_point = wall1.lines[index1 + 1];

            let dx = next_point[0] - point[0];
            let dy = next_point[1] - point[1];
            let d = (dx * dx + dy * dy).sqrt();

            if distance1 + d >= start_length {
                break;
            }

            distance1 += d;
            index1 += 1;
        }

        loop {
            if index2 >= wall2.lines.len() - 1 {
                break;
            }

            let point = wall2.lines[index2];
            let next_point = wall2.lines[index2 + 1];

            let dx = next_point[0] - point[0];
            let dy = next_point[1] - point[1];
            let d = (dx * dx + dy * dy).sqrt();

            if distance2 + d >= start_length {
                break;
            }

            distance2 += d;
            index2 += 1;
        }

        let start_index1 = index1;
        let start_index2 = index2;

        while index1 < wall1.lines.len() && index2 < wall2.lines.len() {
            let point1 = wall1.lines[index1];
            let point2 = wall2.lines[index2];

            let progress1 = (index1 as f32 - start_index1 as f32) / (wall1.lines.len() as f32 - start_index1 as f32 - 1.0);
            let progress2 = (index2 as f32 - start_index2 as f32) / (wall2.lines.len() as f32 - start_index2 as f32 - 1.0);
            let progress = progress1.max(progress2);

            let dx = point2[0] - point1[0];
            let dy = point2[1] - point1[1];
            let d = (dx * dx + dy * dy).sqrt();
            let dx = dx / d;
            let dy = dy / d;

            let maximum_thickness = d / 2.0;
            let thickness = (200.0 + 2000.0 * progress as f32).min(maximum_thickness);

            let point = [
                (point1[0] + point2[0]) / 2.0,
                (point1[1] + point2[1]) / 2.0,
            ];

            wall_inner.add_point(&point);

            let perlin1 = (self.get_perlin_value(point[0] + 3452.0, point[1] + 3452.0, 10.0) + 1.0) / 2.0;
            let perlin2 = (self.get_perlin_value(point[0] + 87362.0, point[1] + 87362.0, 10.0) + 1.0) / 2.0;
            let perlin = perlin1 * perlin2;
            let thickness_mod = perlin * 8.0 - 0.5;
            let thickness = (thickness * thickness_mod).max(200.0).min(maximum_thickness);

            wall_lower.add_point(&[point[0] - dx * thickness / 2.0, point[1] - dy * thickness / 2.0]);
            wall_upper.add_point(&[point[0] + dx * thickness / 2.0, point[1] + dy * thickness / 2.0]);

            index1 += 1;
            index2 += 1;
        }

        return [wall_inner, wall_lower, wall_upper];
    }

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

    fn next(&mut self) -> f32 {
        return self.random.next();
    }

    pub fn get_perlin_value(&mut self, x: f32, y: f32, scale: f32) -> f32 {
        return self.noise.perlin2(x * (NOISE_DETAIL / scale), y * (NOISE_DETAIL / scale) / 0.75);
    }
}

impl Default for Generator {
    fn default() -> Self {
        let seed: u32 = rand::random();
        // let seed: u32 = 2873571609;
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