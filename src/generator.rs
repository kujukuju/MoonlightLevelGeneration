use crate::{SCREEN_WIDTH, SCREEN_HEIGHT};
use line_drawing::Bresenham;
use crate::perlin::Perlin;
use crate::random::Random;
use std::collections::HashMap;
use crate::math_helper::MathHelper;
use crate::helpers::wall_section::WallSection;
use std::f32::consts::PI;

const NOISE_DETAIL: f32 = 0.0005 / 0.75;
const APPROX_WIDTH: f32 = 10752.0;
const APPROX_HEIGHT: f32 = 10752.0;

const TILE_WIDTH: u32 = 4;
const TILE_HEIGHT: u32 = 3;

const TEXTURE_WIDTH: u32 = 32 * 4;
const TEXTURE_HEIGHT: u32 = 24 * 4;

// this value forces the rendering to be sampled down to 1 / screen_scale
const SCREEN_SCALE: u32 = 2;

const LEVEL_WIDTH: usize = SCREEN_WIDTH as usize / TILE_WIDTH as usize * SCREEN_SCALE as usize;
const LEVEL_HEIGHT: usize = SCREEN_HEIGHT as usize / TILE_HEIGHT as usize * SCREEN_SCALE as usize;

const NEW_SAFE_ZONE_SCALE_MUL: f32 = 2.0;

const SAFE_ZONE_WIDTH: f32 = 3072.0 * NEW_SAFE_ZONE_SCALE_MUL;
const SAFE_ZONE_HEIGHT: f32 = 2304.0 * NEW_SAFE_ZONE_SCALE_MUL;

pub struct Generator {
    pixels: Vec<u32>,
    random: Random,
    noise: Perlin,
    seed: i64,
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
        let big_area_angle_start = self.next() * PI * 2.0;
        let big_area_angle_end = big_area_angle_start + PI;
        let big_area_angle_end = big_area_angle_end + self.next() * PI * 0.1 - PI * 0.05;

        // generate the angle for the small area divider wall
        let mid_area_angle = MathHelper::radians_between_angles(big_area_angle_start, big_area_angle_end);
        let mid_area_angle = big_area_angle_start + mid_area_angle / 2.0 + PI;
        let mid_area_angle = mid_area_angle + self.next() * PI * 0.1 - PI * 0.05;

        // generate out the divider walls with random curves and metadata for the thickness along the path
        let mut t1_t3_wall = WallSection::default();
        let desired_wall_length = 30000.0 + 12000.0 * self.next();
        self.fill_wall(&mut t1_t3_wall, desired_wall_length, big_area_angle_start);
        let [mut t1_t3_wall_1, mut t1_t3_wall_2] = t1_t3_wall.thicken(self, 200.0, 1200.0);
        t1_t3_wall_1.render(self);
        t1_t3_wall_2.render(self);

        let mut t3_t2_wall = WallSection::default();
        let desired_wall_length = 30000.0 + 12000.0 * self.next();
        self.fill_wall(&mut t3_t2_wall, desired_wall_length, big_area_angle_end);
        let [mut t3_t2_wall_1, mut t3_t2_wall_2] = t3_t2_wall.thicken(self, 200.0, 1200.0);
        t3_t2_wall_1.render(self);
        t3_t2_wall_2.render(self);

        let mut t2_t1_wall = WallSection::default();
        let desired_wall_length = 30000.0 + 12000.0 * self.next();
        self.fill_wall(&mut t2_t1_wall, desired_wall_length, mid_area_angle);
        let [mut t2_t1_wall_1, mut t2_t1_wall_2] = t2_t1_wall.thicken(self, 200.0, 1200.0);
        t2_t1_wall_1.render(self);
        t2_t1_wall_2.render(self);


        // at the tip of each wall, extend the t3 t2 divider on both sides

        // extend the t3 t1 divider towards t3

        // extend the t2 t1 divider towards t2


    }

    fn fill_wall(&mut self, wall: &mut WallSection, length: f32, angle: f32) {
        let mut point = [
            angle.cos() * SAFE_ZONE_WIDTH / 2.0,
            angle.sin() * SAFE_ZONE_HEIGHT / 2.0,
        ];

        let inner_offset_length = self.next() * 400.0;
        point[0] -= angle.cos() * inner_offset_length;
        point[1] -= angle.sin() * inner_offset_length;

        let mut current_wall_length = 0.0;
        let mut current_angle = angle;

        wall.add_point(&point);
        while current_wall_length < length {
            let angle_mod = self.get_perlin_value(point[0] - 10240.0, point[1] - 10240.0, 1.0);
            current_angle += angle_mod * PI * 0.1;

            let desired_difference = MathHelper::radians_between_angles(current_angle, angle);
            current_angle += desired_difference * 0.02;

            let real_current_angle = MathHelper::round_to_interval(current_angle, PI / 8.0);

            let segment_length = 200.0 + 400.0 * self.next();

            let dx = segment_length * real_current_angle.cos();
            let dy = segment_length * real_current_angle.sin();

            current_wall_length += (dx * dx + dy * dy).sqrt();

            point[0] += dx;
            point[1] += dy;

            wall.add_point(&point);
        }
    }

    fn create_road_bool_tiles(&mut self) -> HashMap<i32, HashMap<i32, bool>> {
        let mut tiles: HashMap<i32, HashMap<i32, bool>> = HashMap::new();

        let aabb = [
            [-(LEVEL_WIDTH as i32) / 2, -(LEVEL_HEIGHT as i32) / 2],
            [LEVEL_WIDTH as i32 / 2, LEVEL_HEIGHT as i32 / 2],
        ];

        for x in aabb[0][0]..=aabb[1][0] {
            let tiles = tiles.entry(x).or_default();

            for y in aabb[0][1]..=aabb[1][1] {
                let position_x = (x * TEXTURE_WIDTH as i32 + TEXTURE_WIDTH as i32 / 2) as f32;
                let position_y = (y * TEXTURE_HEIGHT as i32 + TEXTURE_HEIGHT as i32 / 2) as f32;

                let distance = position_x * position_x + position_y * position_y;
                let scale = (2.0 as f32).max(10000000.0 / distance);

                let noise1 = self.get_perlin_value(position_x, position_y, scale);
                let noise2 = self.get_perlin_value(position_x + APPROX_WIDTH, position_y + APPROX_HEIGHT, scale);
                if noise1.abs() < 0.05 || noise2.abs() < 0.05 {
                    // gravel
                    tiles.insert(y, true);
                } else {
                    // grass
                    tiles.insert(y, false);
                }
            }
        }

        return tiles;
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
        // let seed = 784818861;

        return Generator {
            pixels: vec![0; (LEVEL_WIDTH * LEVEL_HEIGHT) as usize],
            random: Random::create(seed as i64),
            noise: Perlin::default(),
            seed: seed as i64,
        };
    }
}