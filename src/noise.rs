// Modulo di rumore procedurale (Perlin, FBM, Ridged Multi-Fractal) deterministico basato su Seed

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
        let mut p = [0u8; 256];
        for i in 0..256 {
            p[i] = i as u8;
        }

        // SplitMix64 PRNG: algoritmo robusto e deterministico a 64 bit
        let mut s = seed.wrapping_add(0x9e3779b97f4a7c15);
        let mut next_u64 = || {
            s = s.wrapping_add(0x9e3779b97f4a7c15);
            let mut z = s;
            z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
            z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
            z ^ (z >> 31)
        };

        // Fisher-Yates shuffle per mescolare la tabella in base al seed
        for i in (1..256).rev() {
            let j = (next_u64() % (i as u64 + 1)) as usize;
            p.swap(i, j);
        }

        for i in 0..256 {
            perm[i] = p[i];
            perm[256 + i] = p[i];
        }

        Self { seed, perm }
    }

    /// Rumore di Perlin 2D standard in [-1.0, 1.0]
    pub fn perlin_2d(&self, x: f64, y: f64) -> f64 {
        let xi = (x.floor() as i64 & 255) as usize;
        let yi = (y.floor() as i64 & 255) as usize;

        let xf = x - x.floor();
        let yf = y - y.floor();

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

    /// Rumore di Perlin 3D standard in [-1.0, 1.0]
    pub fn perlin_3d(&self, x: f64, y: f64, z: f64) -> f64 {
        let xi = (x.floor() as i64 & 255) as usize;
        let yi = (y.floor() as i64 & 255) as usize;
        let zi = (z.floor() as i64 & 255) as usize;

        let xf = x - x.floor();
        let yf = y - y.floor();
        let zf = z - z.floor();

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

    /// Fractal Brownian Motion (FBM) 2D per altitudini e biomi
    pub fn fbm_2d(&self, x: f64, y: f64, octaves: usize, persistence: f64, lacunarity: f64) -> f64 {
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

    /// Ridged Multi-Fractal 2D: ideale per vette montuose e creste rocciose
    pub fn ridged_fbm_2d(&self, x: f64, y: f64, octaves: usize, persistence: f64, lacunarity: f64) -> f64 {
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

    /// FBM 3D per caverne e gallerie sotterranee
    pub fn fbm_3d(&self, x: f64, y: f64, z: f64, octaves: usize, persistence: f64, lacunarity: f64) -> f64 {
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
fn fade(t: f64) -> f64 {
    t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
}

#[inline(always)]
fn lerp(a: f64, b: f64, t: f64) -> f64 {
    a + t * (b - a)
}

#[inline(always)]
fn grad2(hash: u8, x: f64, y: f64) -> f64 {
    match hash & 7 {
        0 => x + y,
        1 => -x + y,
        2 => x - y,
        3 => -x - y,
        4 => x,
        5 => -x,
        6 => y,
        7 => -y,
        _ => 0.0,
    }
}

#[inline(always)]
fn grad3(hash: u8, x: f64, y: f64, z: f64) -> f64 {
    let h = hash & 15;
    let u = if h < 8 { x } else { y };
    let v = if h < 4 {
        y
    } else if h == 12 || h == 14 {
        x
    } else {
        z
    };
    (if (h & 1) == 0 { u } else { -u }) + (if (h & 2) == 0 { v } else { -v })
}
