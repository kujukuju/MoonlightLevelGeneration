use std::f32::consts::PI;

pub struct MathHelper {

}

impl MathHelper {
    pub fn radians_between_angles(from: f32, to: f32) -> f32 {
        if to < from {
            if from - to > PI {
                return PI * 2.0 - (from - to);
            } else {
                return -(from - to);
            }
        } else {
            if to - from > PI {
                return -(PI * 2.0 - (to - from));
            } else {
                return to - from;
            }
        }
    }

    pub fn round_to_interval(angle: f32, interval: f32) -> f32 {
        return (angle / interval).round() * interval;
    }

    pub fn distance_to_line_segment(line: &[[f32; 2]; 2], point: &[f32; 2]) -> ([f32; 2], f32) {
        let line_dx = line[1][0] - line[0][0];
        let line_dy = line[1][1] - line[0][1];
        let length2 = line_dx * line_dx + line_dy * line_dy;
        if length2 == 0.0 {
            let dx = line[0][0] - point[0];
            let dy = line[0][1] - point[1];
            return ([line[0][0], line[0][1]], (dx * dx + dy * dy).sqrt());
        }

        let t = ((point[0] - line[0][0]) * (line[1][0] - line[0][0]) + (point[1] - line[0][1]) * (line[1][1] - line[0][1])) / length2;
        if t < 0.0 {
            let dx = line[0][0] - point[0];
            let dy = line[0][1] - point[1];
            return ([line[0][0], line[0][1]], (dx * dx + dy * dy).sqrt());
        }
        if t > 1.0 {
            let dx = line[1][0] - point[0];
            let dy = line[1][1] - point[1];
            return ([line[1][0], line[1][1]], (dx * dx + dy * dy).sqrt());
        }

        let dx = line[0][0] + t * (line[1][0] - line[0][0]) - point[0];
        let dy = line[0][1] + t * (line[1][1] - line[0][1]) - point[1];
        return ([line[0][0] + t * (line[1][0] - line[0][0]), line[0][1] + t * (line[1][1] - line[0][1])], (dx * dx + dy * dy).sqrt());
    }
}