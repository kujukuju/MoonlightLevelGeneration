use crate::generator::{Generator, TEXTURE_WIDTH, TEXTURE_HEIGHT, SAFE_ZONE_WIDTH, SAFE_ZONE_HEIGHT};
use crate::math_helper::MathHelper;
use std::f32::consts::PI;

pub struct Point {
    x: f32,
    y: f32,
    thickness: f32,
}

pub struct RoadSegment {
    start_point: [f32; 2],
    start_angle: f32,
    end_angle: f32,
    thickness: f32,
    angle: f32,
    points: Vec<Point>
}

impl RoadSegment {
    pub fn render(&self, generator: &mut Generator) {
        for index in 0..(self.points.len() - 1) {
            let point = &self.points[index];
            let next_point = &self.points[index + 1];

            // generator.draw_line(point.x, point.y, next_point.x, next_point.y, 0xffffbb, 1.0);
            generator.draw_line_thickness(point.x, point.y, next_point.x, next_point.y, point.thickness / 2.0, 0xffffbb, 1.0);
        }
        // let (edge_point, distance) = MathHelper::distance_to_ellipse(0.0, 0.0, SAFE_ZONE_WIDTH / 2.0, SAFE_ZONE_HEIGHT / 2.0, &self.start_point);
        // // generator.draw_line(self.point[0], self.point[1], self.point[0] + self.angle.cos() * 1000.0, self.point[1] + self.angle.sin() * 1000.0, 0x00ffff, 1.0);
        // generator.draw_line(edge_point[0], edge_point[1], edge_point[0] + self.angle.cos() * 1000.0, edge_point[1] + self.angle.sin() * 1000.0, 0x00ffff, 1.0);
    }

    pub fn extend(&mut self, generator: &mut Generator, length: f32) -> Vec<RoadSegment> {
        let mut return_segments = Vec::new();

        let mut angle = self.angle;
        let mut distance = 0.0;
        let mut thickness = self.thickness;

        let mut point = [self.start_point[0], self.start_point[1]];

        self.points.push(Point {
            x: point[0],
            y: point[1],
            thickness: self.thickness,
        });

        let mut split_chance = 0.0;

        while distance < length {
            let next_segment_length = 250.0 + 850.0 * generator.next();

            // angle = MathHelper::round_to_interval(angle, PI / 8.0);
            // rounded angles aren't necessary here, but it leads to more interesting roads
            // because they tend to not overlap exactly and spread out more
            let real_angle = MathHelper::round_to_interval(angle, PI / 8.0);
            point[0] += real_angle.cos() * next_segment_length;
            point[1] += real_angle.sin() * next_segment_length;
            distance += next_segment_length;

            if distance >= length {
                break;
            }

            // angle = MathHelper::round_to_interval(angle, PI / 8.0);

            self.points.push(Point {
                x: point[0],
                y: point[1],
                thickness,
            });

            split_chance += (thickness.sqrt() - 12.0).max(0.0) / 18.0 * generator.next();
            let split = split_chance > 1.0;
            if split {
                let point1 = [
                    point[0] - (angle + PI / 2.0).cos() * thickness / 4.0,
                    point[1] - (angle + PI / 2.0).sin() * thickness / 4.0,
                ];
                let point2 = [
                    point[0] + (angle + PI / 2.0).cos() * thickness / 4.0,
                    point[1] + (angle + PI / 2.0).sin() * thickness / 4.0,
                ];

                let angle_diff = MathHelper::radians_between_angles(self.start_angle, self.end_angle);

                let mut segment1 = RoadSegment::create(generator, point1, angle - angle_diff * (0.6 + generator.next() * 0.5), angle, thickness / 1.2);
                let mut segment2 = RoadSegment::create(generator, point2, angle, angle + angle_diff * (0.6 + generator.next() * 0.5), thickness / 1.2);
                // let mut segment1 = RoadSegment::create(generator, point1, self.start_angle, self.end_angle, thickness / 1.5);
                // let mut segment2 = RoadSegment::create(generator, point2, self.start_angle, self.end_angle, thickness / 1.5);

                let children1 = segment1.extend(generator, length - distance);
                return_segments.push(segment1);
                for segment in children1 {
                    return_segments.push(segment);
                }
                let children2 = segment2.extend(generator, length - distance);
                return_segments.push(segment2);
                for segment in children2 {
                    return_segments.push(segment);
                }

                break;
            }/* else if 20.0 / thickness.sqrt() < generator.next() {
                // chance to split with no thickness cut
                let point1 = [
                    point[0] - (angle + PI / 2.0).cos() * thickness / 2.0,
                    point[1] - (angle + PI / 2.0).sin() * thickness / 2.0,
                ];
                let point2 = [
                    point[0] + (angle + PI / 2.0).cos() * thickness / 2.0,
                    point[1] + (angle + PI / 2.0).sin() * thickness / 2.0,
                ];

                let angle_diff = MathHelper::radians_between_angles(self.start_angle, self.end_angle);

                let mut segment1 = RoadSegment::create(generator, point1, angle - angle_diff / 2.0, angle, thickness);
                let mut segment2 = RoadSegment::create(generator, point2, angle, angle + angle_diff / 2.0, thickness);
                // let mut segment1 = RoadSegment::create(generator, point1, self.start_angle, self.end_angle, thickness / 1.5);
                // let mut segment2 = RoadSegment::create(generator, point2, self.start_angle, self.end_angle, thickness / 1.5);

                let children1 = segment1.extend(generator, length - distance);
                return_segments.push(segment1);
                for segment in children1 {
                    return_segments.push(segment);
                }
                let children2 = segment2.extend(generator, length - distance);
                return_segments.push(segment2);
                for segment in children2 {
                    return_segments.push(segment);
                }

                break;
            }*/

            // same logic as walls so they kind of bend the same way
            let angle_mod = generator.get_perlin_value(point[0] - 10240.0, point[1] - 10240.0, 1.0);
            angle += angle_mod * PI * 0.1;

            let desired_difference = MathHelper::radians_between_angles(angle, self.angle);
            angle += desired_difference * 0.02;

            let perlin1 = (generator.get_perlin_value(point[0] + 4532.0, point[1] + 7546.0, 10.0) + 1.0) / 2.0;
            let perlin2 = (generator.get_perlin_value(point[0] + 85623.0, point[1] + 68572.0, 10.0) + 1.0) / 2.0;
            let perlin = perlin1 * perlin2;
            let thickness_mod = perlin * 2.0 + 0.2;
            thickness = thickness * thickness_mod;
            thickness = thickness.max(20.0);

            let dthick = self.thickness - thickness;
            thickness += dthick / 2.0 * generator.next();
        }

        return return_segments;
    }
}

impl RoadSegment {
    pub fn create(generator: &mut Generator, point: [f32; 2], start_angle: f32, end_angle: f32, thickness: f32) -> Self {
        let (edge_point, distance) = MathHelper::distance_to_ellipse(0.0, 0.0, SAFE_ZONE_WIDTH / 2.0, SAFE_ZONE_HEIGHT / 2.0, &point);

        let angle_diff = MathHelper::radians_between_angles(start_angle, end_angle);
        let angle = start_angle + angle_diff / 2.0;

        // smaller number?
        let mut vector = [0.0, 0.0];
        let mut vector_count = 0;
        for i in 0..24 {
            let angle_to_edge = (edge_point[1] - point[1]).atan2(edge_point[0] - point[0]);
            let angle = i as f32 / 24.0;

            // force the outcoming road angles to be somewhat reasonable, aka not immediate 90 degree angles
            if MathHelper::radians_between_angles(angle, angle_to_edge).abs() > PI * 0.4 {
                continue;
            }

            let point = [
                edge_point[0] + angle.cos() * (thickness + TEXTURE_WIDTH as f32 * 0.0),
                edge_point[1] + angle.sin() * (thickness + TEXTURE_HEIGHT as f32 * 0.0),
            ];

            if MathHelper::is_point_inside_ellipse(point, [0.0, 0.0], [SAFE_ZONE_WIDTH, SAFE_ZONE_HEIGHT]) {
                continue;
            }

            let (is_road1, road_strength) = generator.sample_road(point[0], point[1]);
            let point2 = [
                edge_point[0] + angle.cos() * (thickness + TEXTURE_WIDTH as f32 * 2.0),
                edge_point[1] + angle.sin() * (thickness + TEXTURE_HEIGHT as f32 * 2.0),
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
        let mut road_angle;
        if vector_count > 0 {
            road_angle = vector[1].atan2(vector[0]);
        } else {
            road_angle = angle;
        }

        let angle_diff = MathHelper::radians_between_angles(road_angle, angle);
        if angle_diff.abs() > 45.0 / 180.0 * PI {
            road_angle += angle_diff.signum() * 45.0 / 180.0 * PI;
        }

        let angle_diff = MathHelper::radians_between_angles(start_angle, end_angle);

        return RoadSegment {
            start_point: point,
            start_angle: road_angle - angle_diff / 2.0,
            end_angle: road_angle + angle_diff / 2.0,
            thickness,
            angle: road_angle,
            points: Vec::new(),
        };
    }
}
