// Deterministic procedural noise module (Perlin, FBM, Ridged Multi-Fractal) based on Seed

#[derive(Clone, Debug)]
pub struct NoiseGenerator {
    #[allow(dead_code)]
    pub seed: u64,
    perm: [u8; 512],
}

impl Default for NoiseGenerator {
    fn default() -> Self {
        Self::new(1337)
    }
}

impl NoiseGenerator {
    pub fn new(seed: u64) -> Self {
        let mut perm = [0u8; 512];
        let mut p: [u8; 256] = std::array::from_fn(|i| i as u8);

        // SplitMix64 PRNG: robust, deterministic 64-bit algorithm
        let mut s = seed.wrapping_add(0x9e3779b97f4a7c15);
        let mut next_u64 = || {
            s = s.wrapping_add(0x9e3779b97f4a7c15);
            let mut z = s;
            z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
            z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
            z ^ (z >> 31)
        };

        // Fisher-Yates shuffle to randomize permutation table using the seed
        for i in (1..256).rev() {
            let j = (next_u64() % (i as u64 + 1)) as usize;
            p.swap(i, j);
        }

        perm[..256].copy_from_slice(&p);
        perm[256..].copy_from_slice(&p);

        Self { seed, perm }
    }

    /// Standard 2D Perlin noise in [-1.0, 1.0]
    pub fn perlin_2d(&self, x: f64, y: f64) -> f64 {
        let (x_floor, xi) = fast_floor(x);
        let (y_floor, yi) = fast_floor(y);

        let xf = x - x_floor;
        let yf = y - y_floor;

        let u = fade(xf);
        let v = fade(yf);

        let aa = self.perm[self.perm[xi] as usize + yi];
        let ab = self.perm[self.perm[xi] as usize + yi + 1];
        let ba = self.perm[self.perm[xi + 1] as usize + yi];
        let bb = self.perm[self.perm[xi + 1] as usize + yi + 1];

        let x1 = lerp(grad2(aa, xf, yf), grad2(ba, xf - 1.0, yf), u);
        let x2 = lerp(grad2(ab, xf, yf - 1.0), grad2(bb, xf - 1.0, yf - 1.0), u);

        lerp(x1, x2, v)
    }

    /// Standard 3D Perlin noise in [-1.0, 1.0]
    pub fn perlin_3d(&self, x: f64, y: f64, z: f64) -> f64 {
        let (x_floor, xi) = fast_floor(x);
        let (y_floor, yi) = fast_floor(y);
        let (z_floor, zi) = fast_floor(z);

        let xf = x - x_floor;
        let yf = y - y_floor;
        let zf = z - z_floor;

        let u = fade(xf);
        let v = fade(yf);
        let w = fade(zf);

        let a = self.perm[xi] as usize + yi;
        let aa = self.perm[a] as usize + zi;
        let ab = self.perm[a + 1] as usize + zi;
        let b = self.perm[xi + 1] as usize + yi;
        let ba = self.perm[b] as usize + zi;
        let bb = self.perm[b + 1] as usize + zi;

        let p000 = grad3(self.perm[aa], xf, yf, zf);
        let p100 = grad3(self.perm[ba], xf - 1.0, yf, zf);
        let p010 = grad3(self.perm[ab], xf, yf - 1.0, zf);
        let p110 = grad3(self.perm[bb], xf - 1.0, yf - 1.0, zf);
        let p001 = grad3(self.perm[aa + 1], xf, yf, zf - 1.0);
        let p101 = grad3(self.perm[ba + 1], xf - 1.0, yf, zf - 1.0);
        let p011 = grad3(self.perm[ab + 1], xf, yf - 1.0, zf - 1.0);
        let p111 = grad3(self.perm[bb + 1], xf - 1.0, yf - 1.0, zf - 1.0);

        let x1 = lerp(p000, p100, u);
        let x2 = lerp(p010, p110, u);
        let y1 = lerp(x1, x2, v);

        let x3 = lerp(p001, p101, u);
        let x4 = lerp(p011, p111, u);
        let y2 = lerp(x3, x4, v);

        lerp(y1, y2, w)
    }

    /// 2D Fractal Brownian Motion (FBM) for terrain elevation and biomes
    #[inline]
    pub fn fbm_2d(&self, x: f64, y: f64, octaves: usize, persistence: f64, lacunarity: f64) -> f64 {
        if octaves == 3 && (persistence - 0.5).abs() < 1e-6 && (lacunarity - 2.0).abs() < 1e-6 {
            let o1 = self.perlin_2d(x, y);
            let o2 = self.perlin_2d(x * 2.0, y * 2.0);
            let o3 = self.perlin_2d(x * 4.0, y * 4.0);
            return (o1 + 0.5 * o2 + 0.25 * o3) / 1.75;
        }

        let mut total = 0.0;
        let mut frequency = 1.0;
        let mut amplitude = 1.0;
        let mut max_value = 0.0;

        for _ in 0..octaves {
            total += self.perlin_2d(x * frequency, y * frequency) * amplitude;
            max_value += amplitude;
            amplitude *= persistence;
            frequency *= lacunarity;
        }

        total / max_value
    }

    /// 2D Ridged Multi-Fractal: ideal for mountain peaks and rocky ridges
    #[inline]
    pub fn ridged_fbm_2d(
        &self,
        x: f64,
        y: f64,
        octaves: usize,
        persistence: f64,
        lacunarity: f64,
    ) -> f64 {
        if octaves == 4 && (persistence - 0.5).abs() < 1e-6 && (lacunarity - 2.0).abs() < 1e-6 {
            let n1 = 1.0 - self.perlin_2d(x, y).abs();
            let n2 = 1.0 - self.perlin_2d(x * 2.0, y * 2.0).abs();
            let n3 = 1.0 - self.perlin_2d(x * 4.0, y * 4.0).abs();
            let n4 = 1.0 - self.perlin_2d(x * 8.0, y * 8.0).abs();
            return (n1 * n1 + 0.5 * (n2 * n2) + 0.25 * (n3 * n3) + 0.125 * (n4 * n4)) / 1.875;
        }

        let mut total = 0.0;
        let mut frequency = 1.0;
        let mut amplitude = 1.0;
        let mut max_value = 0.0;

        for _ in 0..octaves {
            let n = 1.0 - self.perlin_2d(x * frequency, y * frequency).abs();
            total += n * n * amplitude;
            max_value += amplitude;
            amplitude *= persistence;
            frequency *= lacunarity;
        }

        total / max_value
    }

    /// 3D FBM for underground caves and tunnels
    #[inline]
    pub fn fbm_3d(
        &self,
        x: f64,
        y: f64,
        z: f64,
        octaves: usize,
        persistence: f64,
        lacunarity: f64,
    ) -> f64 {
        if octaves == 2 && (persistence - 0.5).abs() < 1e-6 && (lacunarity - 2.0).abs() < 1e-6 {
            let o1 = self.perlin_3d(x, y, z);
            let o2 = self.perlin_3d(x * 2.0, y * 2.0, z * 2.0);
            return (o1 + 0.5 * o2) / 1.5;
        }

        let mut total = 0.0;
        let mut frequency = 1.0;
        let mut amplitude = 1.0;
        let mut max_value = 0.0;

        for _ in 0..octaves {
            total += self.perlin_3d(x * frequency, y * frequency, z * frequency) * amplitude;
            max_value += amplitude;
            amplitude *= persistence;
            frequency *= lacunarity;
        }

        total / max_value
    }
}

#[inline(always)]
fn fast_floor(x: f64) -> (f64, usize) {
    let xi = x as i64;
    let offset = (x < xi as f64) as i64;
    let floor_val = xi - offset;
    (floor_val as f64, (floor_val & 255) as usize)
}

#[inline(always)]
const fn fade(t: f64) -> f64 {
    t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
}

#[inline(always)]
const fn lerp(a: f64, b: f64, t: f64) -> f64 {
    a + t * (b - a)
}

#[inline(always)]
const fn grad2(hash: u8, x: f64, y: f64) -> f64 {
    match hash & 7 {
        0 => x + y,
        1 => -x + y,
        2 => x - y,
        3 => -x - y,
        4 => x,
        5 => -x,
        6 => y,
        _ => -y,
    }
}

#[inline(always)]
const fn grad3(hash: u8, x: f64, y: f64, z: f64) -> f64 {
    match hash & 15 {
        0 | 12 => x + y,
        1 | 14 => -x + y,
        2 => x - y,
        3 => -x - y,
        4 => x + z,
        5 => -x + z,
        6 => x - z,
        7 => -x - z,
        8 => y + z,
        9 | 13 => -y + z,
        10 => y - z,
        _ => -y - z,
    }
}
