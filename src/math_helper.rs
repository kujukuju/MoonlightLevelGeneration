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
}