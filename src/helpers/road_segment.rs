use crate::generator::{Generator, TEXTURE_WIDTH, TEXTURE_HEIGHT, SAFE_ZONE_WIDTH, SAFE_ZONE_HEIGHT};
use crate::math_helper::MathHelper;
use std::f32::consts::PI;

pub struct RoadSegment {
    point: [f32; 2],
    thickness: f32,
    angle: f32,
}

impl RoadSegment {
    pub fn render(&self, generator: &mut Generator) {
        let (edge_point, distance) = MathHelper::distance_to_ellipse(0.0, 0.0, SAFE_ZONE_WIDTH / 2.0, SAFE_ZONE_HEIGHT / 2.0, &self.point);
        // generator.draw_line(self.point[0], self.point[1], self.point[0] + self.angle.cos() * 1000.0, self.point[1] + self.angle.sin() * 1000.0, 0x00ffff, 1.0);
        generator.draw_line(edge_point[0], edge_point[1], edge_point[0] + self.angle.cos() * 1000.0, edge_point[1] + self.angle.sin() * 1000.0, 0x00ffff, 1.0);
    }
}

impl RoadSegment {
    pub fn create(generator: &mut Generator, point: [f32; 2], thickness: f32, angle: f32) -> Self {
        let (edge_point, distance) = MathHelper::distance_to_ellipse(0.0, 0.0, SAFE_ZONE_WIDTH / 2.0, SAFE_ZONE_HEIGHT / 2.0, &point);

        // smaller number?
        let mut vector = [0.0, 0.0];
        let mut vector_count = 0;
        for i in 0..24 {
            let angle = i as f32 / 24.0;
            let point = [
                edge_point[0] + angle.cos() * (thickness + TEXTURE_WIDTH as f32 * 2.0),
                edge_point[1] + angle.sin() * (thickness + TEXTURE_HEIGHT as f32 * 2.0),
            ];

            if MathHelper::is_point_inside_ellipse(point, [0.0, 0.0], [SAFE_ZONE_WIDTH, SAFE_ZONE_HEIGHT]) {
                continue;
            }

            let (is_road1, road_strength) = generator.sample_road(point[0], point[1]);
            let point2 = [
                edge_point[0] + angle.cos() * (thickness + TEXTURE_WIDTH as f32 * 4.0),
                edge_point[1] + angle.sin() * (thickness + TEXTURE_HEIGHT as f32 * 4.0),
            ];
            let (is_road2, road_strength) = generator.sample_road(point2[0], point2[1]);
            let is_road = is_road1 || is_road2;

            if is_road {
                let dx = point[0] - edge_point[0];
                let dy = point[1] - edge_point[1];
                let d = (dx * dx + dy * dy).sqrt();

                vector[0] += dx / d;
                vector[1] += dy / d;
                vector_count += 1;
            }
        }

        vector[0] /= vector_count as f32;
        vector[1] /= vector_count as f32;
        let road_angle;
        if vector_count > 0 {
            road_angle = vector[1].atan2(vector[0]);
        } else {
            road_angle = angle;
        }

        // 0.0 is confident
        // let mut most_confident_road_value = 1.0;
        // let mut most_confident_road_angle: Option<f32> = None;
        // for i in 0..12 {
        //     let angle = i as f32 / 12.0;
        //     let point = [
        //         edge_point[0] + angle.cos() * TEXTURE_WIDTH as f32 * 1.0,
        //         edge_point[1] + angle.sin() * TEXTURE_HEIGHT as f32 * 1.0,
        //     ];
        //
        //     let (is_road, road_strength) = generator.sample_road(point[0], point[1]);
        //     if road_strength < most_confident_road_value {
        //         most_confident_road_value = road_strength;
        //         most_confident_road_angle = Some(angle);
        //
        //         // if the found point is inside the ellipse flip it
        //         if MathHelper::is_point_inside_ellipse(point, [0.0, 0.0], [SAFE_ZONE_WIDTH, SAFE_ZONE_HEIGHT]) {
        //             most_confident_road_angle = Some(angle + PI);
        //         }
        //     }
        // }

        return RoadSegment {
            point, // edge_point,
            thickness,
            angle: road_angle,
        };
    }
}
