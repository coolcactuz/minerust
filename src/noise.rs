// Modulo di rumore procedurale (Perlin & Simplex / FBM) altamente ottimizzato in puro Rust

const PERMUTATION: [u8; 512] = [
    151, 160, 137, 91, 90, 15, 131, 13, 201, 95, 96, 53, 194, 233, 7, 225, 140, 36, 103, 30, 69,
    142, 8, 99, 37, 240, 21, 10, 23, 190, 6, 148, 247, 120, 234, 75, 0, 26, 197, 62, 94, 252, 219,
    203, 117, 35, 11, 32, 57, 177, 33, 88, 237, 149, 56, 87, 174, 20, 125, 136, 171, 168, 68,
    175, 74, 165, 71, 134, 139, 48, 27, 166, 77, 146, 158, 231, 83, 111, 229, 122, 60, 211, 133,
    230, 220, 105, 92, 41, 55, 46, 245, 40, 244, 102, 143, 54, 65, 25, 63, 161, 1, 216, 80, 73,
    209, 76, 132, 187, 208, 89, 18, 169, 200, 196, 135, 130, 116, 188, 159, 86, 164, 100, 109,
    198, 173, 186, 3, 64, 52, 217, 226, 250, 124, 123, 5, 202, 38, 147, 118, 126, 255, 82, 85,
    212, 207, 206, 59, 227, 47, 16, 58, 17, 182, 189, 28, 42, 223, 183, 170, 213, 119, 248, 152,
    2, 44, 154, 163, 70, 221, 153, 101, 155, 167, 43, 172, 9, 129, 22, 39, 253, 19, 98, 108, 110,
    79, 113, 224, 232, 178, 185, 112, 104, 218, 246, 97, 228, 251, 34, 242, 193, 238, 210, 144,
    12, 191, 179, 162, 241, 81, 51, 145, 235, 249, 14, 239, 107, 49, 192, 214, 31, 181, 199, 106,
    157, 184, 84, 204, 176, 115, 121, 50, 45, 127, 4, 150, 254, 138, 236, 205, 93, 222, 114, 67,
    29, 24, 72, 243, 141, 128, 195, 78, 66, 215, 61, 156, 180,
    // Duplicate per evitare overflow senza operatore modulo (%)
    151, 160, 137, 91, 90, 15, 131, 13, 201, 95, 96, 53, 194, 233, 7, 225, 140, 36, 103, 30, 69,
    142, 8, 99, 37, 240, 21, 10, 23, 190, 6, 148, 247, 120, 234, 75, 0, 26, 197, 62, 94, 252, 219,
    203, 117, 35, 11, 32, 57, 177, 33, 88, 237, 149, 56, 87, 174, 20, 125, 136, 171, 168, 68,
    175, 74, 165, 71, 134, 139, 48, 27, 166, 77, 146, 158, 231, 83, 111, 229, 122, 60, 211, 133,
    230, 220, 105, 92, 41, 55, 46, 245, 40, 244, 102, 143, 54, 65, 25, 63, 161, 1, 216, 80, 73,
    209, 76, 132, 187, 208, 89, 18, 169, 200, 196, 135, 130, 116, 188, 159, 86, 164, 100, 109,
    198, 173, 186, 3, 64, 52, 217, 226, 250, 124, 123, 5, 202, 38, 147, 118, 126, 255, 82, 85,
    212, 207, 206, 59, 227, 47, 16, 58, 17, 182, 189, 28, 42, 223, 183, 170, 213, 119, 248, 152,
    2, 44, 154, 163, 70, 221, 153, 101, 155, 167, 43, 172, 9, 129, 22, 39, 253, 19, 98, 108, 110,
    79, 113, 224, 232, 178, 185, 112, 104, 218, 246, 97, 228, 251, 34, 242, 193, 238, 210, 144,
    12, 191, 179, 162, 241, 81, 51, 145, 235, 249, 14, 239, 107, 49, 192, 214, 31, 181, 199, 106,
    157, 184, 84, 204, 176, 115, 121, 50, 45, 127, 4, 150, 254, 138, 236, 205, 93, 222, 114, 67,
    29, 24, 72, 243, 141, 128, 195, 78, 66, 215, 61, 156, 180,
];

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

/// Rumore di Perlin 2D standard in [-1.0, 1.0]
pub fn perlin_2d(x: f64, y: f64) -> f64 {
    let xi = (x.floor() as i64 & 255) as usize;
    let yi = (y.floor() as i64 & 255) as usize;

    let xf = x - x.floor();
    let yf = y - y.floor();

    let u = fade(xf);
    let v = fade(yf);

    let aa = PERMUTATION[PERMUTATION[xi] as usize + yi];
    let ab = PERMUTATION[PERMUTATION[xi] as usize + yi + 1];
    let ba = PERMUTATION[PERMUTATION[xi + 1] as usize + yi];
    let bb = PERMUTATION[PERMUTATION[xi + 1] as usize + yi + 1];

    let x1 = lerp(grad2(aa, xf, yf), grad2(ba, xf - 1.0, yf), u);
    let x2 = lerp(grad2(ab, xf, yf - 1.0), grad2(bb, xf - 1.0, yf - 1.0), u);

    lerp(x1, x2, v)
}

/// Rumore di Perlin 3D standard in [-1.0, 1.0]
pub fn perlin_3d(x: f64, y: f64, z: f64) -> f64 {
    let xi = (x.floor() as i64 & 255) as usize;
    let yi = (y.floor() as i64 & 255) as usize;
    let zi = (z.floor() as i64 & 255) as usize;

    let xf = x - x.floor();
    let yf = y - y.floor();
    let zf = z - z.floor();

    let u = fade(xf);
    let v = fade(yf);
    let w = fade(zf);

    let a = PERMUTATION[xi] as usize + yi;
    let aa = PERMUTATION[a] as usize + zi;
    let ab = PERMUTATION[a + 1] as usize + zi;
    let b = PERMUTATION[xi + 1] as usize + yi;
    let ba = PERMUTATION[b] as usize + zi;
    let bb = PERMUTATION[b + 1] as usize + zi;

    let p000 = grad3(PERMUTATION[aa], xf, yf, zf);
    let p100 = grad3(PERMUTATION[ba], xf - 1.0, yf, zf);
    let p010 = grad3(PERMUTATION[ab], xf, yf - 1.0, zf);
    let p110 = grad3(PERMUTATION[bb], xf - 1.0, yf - 1.0, zf);
    let p001 = grad3(PERMUTATION[aa + 1], xf, yf, zf - 1.0);
    let p101 = grad3(PERMUTATION[ba + 1], xf - 1.0, yf, zf - 1.0);
    let p011 = grad3(PERMUTATION[ab + 1], xf, yf - 1.0, zf - 1.0);
    let p111 = grad3(PERMUTATION[bb + 1], xf - 1.0, yf - 1.0, zf - 1.0);

    let x1 = lerp(p000, p100, u);
    let x2 = lerp(p010, p110, u);
    let y1 = lerp(x1, x2, v);

    let x3 = lerp(p001, p101, u);
    let x4 = lerp(p011, p111, u);
    let y2 = lerp(x3, x4, v);

    lerp(y1, y2, w)
}

/// Fractal Brownian Motion (FBM) 2D per generare altitudini naturali e biomi
pub fn fbm_2d(x: f64, y: f64, octaves: usize, persistence: f64, lacunarity: f64) -> f64 {
    let mut total = 0.0;
    let mut frequency = 1.0;
    let mut amplitude = 1.0;
    let mut max_value = 0.0;

    for _ in 0..octaves {
        total += perlin_2d(x * frequency, y * frequency) * amplitude;
        max_value += amplitude;
        amplitude *= persistence;
        frequency *= lacunarity;
    }

    total / max_value
}

/// Ridged Multi-Fractal 2D: ideale per vette montuose aguzze e creste rocciose
pub fn ridged_fbm_2d(x: f64, y: f64, octaves: usize, persistence: f64, lacunarity: f64) -> f64 {
    let mut total = 0.0;
    let mut frequency = 1.0;
    let mut amplitude = 1.0;
    let mut max_value = 0.0;

    for _ in 0..octaves {
        let n = 1.0 - perlin_2d(x * frequency, y * frequency).abs();
        total += n * n * amplitude;
        max_value += amplitude;
        amplitude *= persistence;
        frequency *= lacunarity;
    }

    total / max_value
}

/// FBM 3D per caverne, gallerie sotterranee e spugne/cavità
pub fn fbm_3d(x: f64, y: f64, z: f64, octaves: usize, persistence: f64, lacunarity: f64) -> f64 {
    let mut total = 0.0;
    let mut frequency = 1.0;
    let mut amplitude = 1.0;
    let mut max_value = 0.0;

    for _ in 0..octaves {
        total += perlin_3d(x * frequency, y * frequency, z * frequency) * amplitude;
        max_value += amplitude;
        amplitude *= persistence;
        frequency *= lacunarity;
    }

    total / max_value
}
