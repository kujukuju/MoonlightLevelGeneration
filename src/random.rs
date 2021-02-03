pub struct Random {
    seed: i64,
}

impl Random {
    pub fn next(&mut self) -> f32 {
        self.seed = (self.seed * 9301 + 49297) % 233280;

        return (self.seed as f64 / 233280.0) as f32;
    }
}

impl Random {
    pub fn create(seed: i64) -> Self {
        return Random {
            seed: seed.abs(),
        }
    }
}