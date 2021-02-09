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

    pub fn intersect_ray_ray(start1: [f32; 2], direction1: [f32; 2], start2: [f32; 2], direction2: [f32; 2]) -> Option<[f32; 2]> {
        let difx = start2[0] - start1[0];
        let dify = start2[1] - start1[1];
        let d1xd2 = direction1[0] * direction2[1] - direction1[1] * direction2[0];
        if d1xd2 == 0.0 {
            return None;
        }
        let d2sx = direction2[0] / d1xd2;
        let d2sy = direction2[1] / d1xd2;
        let length = difx * d2sy - dify * d2sx;

        return Some([
            start1[0] + direction1[0] * length,
            start1[1] + direction1[1] * length,
        ]);
    }

    pub fn intersect_line_line(line1: [[f32; 2]; 2], line2: [[f32; 2]; 2]) -> Option<[f32; 2]> {
        let x1 = line1[0][0];
        let y1 = line1[0][1];
        let x2 = line1[1][0];
        let y2 = line1[1][1];
        let x3 = line2[0][0];
        let y3 = line2[0][1];
        let x4 = line2[1][0];
        let y4 = line2[1][1];

        let d = (y4 - y3) * (x2 - x1) - (x4 - x3) * (y2 - y1);
        if d == 0.0 {
            return None;
        }

        let yd = y1 - y3;
        let xd = x1 - x3;
        let ua = ((x4 - x3) * yd - (y4 - y3) * xd) / d;
        if ua < 0.0 || ua > 1.0 {
            return None;
        }

        let ub = ((x2 - x1) * yd - (y2 - y1) * xd) / d;
        if ub < 0.0 || ub > 1.0 {
            return None;
        }

        return Some([x1 + (x2 - x1) * ua, y1 + (y2 - y1) * ua]);
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

    pub fn distance_to_ellipse(center_x: f32, center_y: f32, semi_major: f32, semi_minor: f32, point: &[f32; 2]) -> ([f32; 2], f32) {
        let point = [point[0] - center_x, point[1] - center_y];

        let mut px = point[0].abs();
        let mut py = point[1].abs();

        let mut tx = 0.707;
        let mut ty = 0.707;

        let mut a = semi_major;
        let mut b = semi_minor;

        for x in 0..3 {
            let x = a * tx;
            let y = b * ty;

            let ex = (a * a - b * b) * tx.powf(3.0) / a;
            let ey = (b * b - a * a) * ty.powf(3.0) / b;

            let rx = x - ex;
            let ry = y - ey;

            let qx = px - ex;
            let qy = py - ey;

            let r = (ry * ry + rx * rx).sqrt();
            let q = (qy * qy + qx * qx).sqrt();

            tx = (1.0 as f32).min((0.0 as f32).max((qx * r / q + ex) / a));
            ty = (1.0 as f32).min((0.0 as f32).max((qy * r / q + ey) / b));
            let t = (ty * ty + tx * tx).sqrt();
            tx = tx / t;
            ty = ty / t;
        }

        let x = (a * tx).abs() * point[0].signum();
        let y = (b * ty).abs() * point[1].signum();

        return ([x + center_x, y + center_y], (x * x + y * y).sqrt());
    }

    pub fn is_point_inside_ellipse(point: [f32; 2], center: [f32; 2], dimensions: [f32; 2]) -> bool {
        let point = [
            point[0] - center[0],
            point[1] - center[1],
        ];
        let dx = point[0] / (dimensions[0] / 2.0);
        let dy = point[1] / (dimensions[1] / 2.0);

        return dx * dx + dy * dy <= 1.0;
    }

    pub fn hermite(t: f32, points: [[f32; 2]; 2], tangentials: [[f32; 2]; 2]) -> [f32; 2] {
        let n1 = 2.0 * t * t * t - 3.0 * t * t + 1.0;
        let n2 = t * t * t - 2.0 * t * t + t;
        let n3 = -2.0 * t * t * t + 3.0 * t * t;
        let n4 = t * t * t - t * t;

        return [
            n1 * points[0][0] + n2 * tangentials[0][0] + n3 * points[1][0] + n4 * tangentials[1][0],
            n1 * points[0][1] + n2 * tangentials[0][1] + n3 * points[1][1] + n4 * tangentials[1][1],
        ];
    }

    pub fn ease_in_out(t: f32) -> f32 {
        let p = 2.0 * t * t;
        return if t < 0.5 {p} else {-p + (4.0 * t) - 1.0};
    }
}