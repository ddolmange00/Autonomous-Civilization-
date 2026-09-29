/// Seeded 2D gradient noise (Perlin-style) with fractal helpers.
/// Deterministic across platforms: only integer hashing and f32 arithmetic.
#[derive(Clone)]
pub struct Noise2 {
    perm: [u8; 512],
}

fn splitmix(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut z = *state;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

fn fade(t: f32) -> f32 {
    t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
}

fn grad(hash: u8, x: f32, y: f32) -> f32 {
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

impl Noise2 {
    pub fn new(seed: u64) -> Self {
        let mut p: [u8; 256] = std::array::from_fn(|i| i as u8);
        let mut s = seed;
        for i in (1..256).rev() {
            let j = (splitmix(&mut s) % (i as u64 + 1)) as usize;
            p.swap(i, j);
        }
        let mut perm = [0u8; 512];
        for i in 0..512 {
            perm[i] = p[i & 255];
        }
        Self { perm }
    }

    /// Single octave, roughly in [-1, 1].
    pub fn sample(&self, x: f32, y: f32) -> f32 {
        let (fx, fy) = (x.floor(), y.floor());
        let xi = (fx as i32 & 255) as usize;
        let yi = (fy as i32 & 255) as usize;
        let (xf, yf) = (x - fx, y - fy);
        let (u, v) = (fade(xf), fade(yf));
        let p = &self.perm;
        let aa = p[p[xi] as usize + yi];
        let ab = p[p[xi] as usize + yi + 1];
        let ba = p[p[xi + 1] as usize + yi];
        let bb = p[p[xi + 1] as usize + yi + 1];
        let x1 = grad(aa, xf, yf) + u * (grad(ba, xf - 1.0, yf) - grad(aa, xf, yf));
        let x2 = grad(ab, xf, yf - 1.0) + u * (grad(bb, xf - 1.0, yf - 1.0) - grad(ab, xf, yf - 1.0));
        x1 + v * (x2 - x1)
    }

    /// Fractal Brownian motion normalised by total amplitude.
    pub fn fbm(&self, x: f32, y: f32, octaves: u32) -> f32 {
        let (mut sum, mut amp, mut freq, mut norm) = (0.0, 1.0, 1.0, 0.0);
        for o in 0..octaves {
            // Per-octave offset breaks the lattice alignment at the origin.
            let off = o as f32 * 17.13;
            sum += self.sample(x * freq + off, y * freq - off) * amp;
            norm += amp;
            amp *= 0.5;
            freq *= 2.0;
        }
        sum / norm
    }

    /// Ridged variant: sharp crests for mountain chains.
    pub fn ridged(&self, x: f32, y: f32, octaves: u32) -> f32 {
        let (mut sum, mut amp, mut freq, mut norm) = (0.0, 1.0, 1.0, 0.0);
        for o in 0..octaves {
            let off = o as f32 * 31.7;
            let r = 1.0 - self.sample(x * freq + off, y * freq + off).abs();
            sum += r * r * amp;
            norm += amp;
            amp *= 0.5;
            freq *= 2.0;
        }
        sum / norm
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_seed_same_field_different_seed_differs() {
        let (a, b, c) = (Noise2::new(7), Noise2::new(7), Noise2::new(8));
        let pts = [(0.3, 0.7), (12.5, -4.25), (100.1, 3.3)];
        for (x, y) in pts {
            assert_eq!(a.fbm(x, y, 5).to_bits(), b.fbm(x, y, 5).to_bits());
        }
        assert!(pts.iter().any(|&(x, y)| a.fbm(x, y, 5) != c.fbm(x, y, 5)));
    }

    #[test]
    fn noise_is_bounded_and_continuous() {
        let n = Noise2::new(42);
        let mut prev = n.sample(0.0, 0.5);
        for i in 1..2000 {
            let v = n.sample(i as f32 * 0.01, 0.5);
            assert!(v.abs() <= 1.01);
            assert!((v - prev).abs() < 0.1, "jump at step {i}");
            prev = v;
        }
    }
}
