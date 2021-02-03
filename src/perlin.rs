#[derive(Copy, Clone)]
struct Grad(f32, f32, f32);

impl Grad {
    fn dot2(&self, x: f32, y: f32) -> f32 {
        return self.0 * x + self.1 * y;
    }

    fn dot3(&self, x: f32, y: f32, z: f32) -> f32 {
        return self.0 * x + self.1 * y + self.2 * z;
    }
}

impl Default for Grad {
    fn default() -> Self {
        return Grad(0.0, 0.0, 0.0);
    }
}

pub struct Perlin {
    perm: [usize; 512],
    grad_p: [Grad; 512],
    p: [i32; 256],
    grad_3: [Grad; 12],
    f2: f32,
    g2: f32,
    f3: f32,
    g3: f32,
}

impl Perlin {
    pub fn seed(&mut self, seed: f32) {
        self.seed_int((seed * 65536.0) as i32);
    }

    pub fn perlin2(&mut self, x: f32, y: f32) -> f32 {
        let x_floor = x.floor() as i32;
        let y_floor = y.floor() as i32;
        let x = x - x_floor as f32;
        let y = y - y_floor as f32;

        let x_floor = (x_floor & 0xff) as usize;
        let y_floor = (y_floor & 0xff) as usize;

        let n00 = self.grad_p[x_floor + self.perm[y_floor]].dot2(x, y);
        let n01 = self.grad_p[x_floor + self.perm[y_floor + 1]].dot2(x, y - 1.0);
        let n10 = self.grad_p[x_floor + 1 + self.perm[y_floor]].dot2(x - 1.0, y);
        let n11 = self.grad_p[x_floor + 1 + self.perm[y_floor + 1]].dot2(x - 1.0, y - 1.0);

        let u = self.fade(x);

        return self.lerp(
            self.lerp(n00, n10, u),
            self.lerp(n01, n11, u),
            self.fade(y));
    }

    fn seed_int(&mut self, mut seed: i32) {
        if seed < 256 {
            seed |= seed << 8;
        }

        for i in 0..256 {
            let v;
            if i & 1 == 1 {
                v = (self.p[i] ^ (seed & 255)) as usize;
            } else {
                v = (self.p[i] ^ ((seed >> 8) & 255)) as usize;
            }

            self.perm[i] = v;
            self.perm[i + 256] = v;

            self.grad_p[i] = self.grad_3[v % 12];
            self.grad_p[i + 256] = self.grad_3[v % 12];
        }
    }

    fn fade(&self, t: f32) -> f32 {
        return t * t * t * (t * (t * 6.0 - 15.0) + 10.0);
    }

    fn lerp(&self, a: f32, b: f32, t: f32) -> f32 {
        return (1.0 - t) * a + t * b;
    }
}

impl Default for Perlin {
    fn default() -> Self {
        return Perlin {
            perm: [0; 512],
            grad_p: [Grad::default(); 512],
            p: [
                151,160,137,91,90,15,
                131,13,201,95,96,53,194,233,7,225,140,36,103,30,69,142,8,99,37,240,21,10,23,
                190, 6,148,247,120,234,75,0,26,197,62,94,252,219,203,117,35,11,32,57,177,33,
                88,237,149,56,87,174,20,125,136,171,168, 68,175,74,165,71,134,139,48,27,166,
                77,146,158,231,83,111,229,122,60,211,133,230,220,105,92,41,55,46,245,40,244,
                102,143,54, 65,25,63,161, 1,216,80,73,209,76,132,187,208, 89,18,169,200,196,
                135,130,116,188,159,86,164,100,109,198,173,186, 3,64,52,217,226,250,124,123,
                5,202,38,147,118,126,255,82,85,212,207,206,59,227,47,16,58,17,182,189,28,42,
                223,183,170,213,119,248,152, 2,44,154,163, 70,221,153,101,155,167, 43,172,9,
                129,22,39,253, 19,98,108,110,79,113,224,232,178,185, 112,104,218,246,97,228,
                251,34,242,193,238,210,144,12,191,179,162,241, 81,51,145,235,249,14,239,107,
                49,192,214, 31,181,199,106,157,184, 84,204,176,115,121,50,45,127, 4,150,254,
                138,236,205,93,222,114,67,29,24,72,243,141,128,195,78,66,215,61,156,180,
            ],
            grad_3: [
                Grad(1.0,1.0,0.0), Grad(-1.0,1.0,0.0), Grad(1.0,-1.0,0.0), Grad(-1.0,-1.0,0.0),
                Grad(1.0,0.0,1.0), Grad(-1.0,0.0,1.0), Grad(1.0,0.0,-1.0), Grad(-1.0,0.0,-1.0),
                Grad(0.0,1.0,1.0), Grad(0.0,-1.0,1.0), Grad(0.0,1.0,-1.0), Grad(0.0,-1.0,-1.0),
            ],
            f2: 0.5 * ((3.0 as f32).sqrt() as f32 - 1.0),
            g2: (3.0 - (3.0 as f32).sqrt() as f32) / 6.0,
            f3: 1.0 / 3.0,
            g3: 1.0 / 6.0,
        };
    }
}