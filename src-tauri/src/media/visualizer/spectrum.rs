//! Bounded, allocation-free radix-2 FFT. No audio samples leave the native service.
pub const SIZE: usize = 2048;
pub const BANDS: usize = 32;
pub const RATE: u32 = 48000;

pub fn analyze(samples: &[f32]) -> [f32; BANDS] {
    if samples.is_empty() {
        return [0.0; BANDS];
    }
    let mut re = [0.0f32; SIZE];
    let mut im = [0.0f32; SIZE];
    let slice = &samples[samples.len().saturating_sub(SIZE)..];
    for (i, value) in slice.iter().enumerate() {
        re[i] = if value.is_finite() {
            value.clamp(-4.0, 4.0)
        } else {
            0.0
        } * (0.5 - 0.5 * (std::f32::consts::TAU * i as f32 / (SIZE - 1) as f32).cos());
    }
    for i in 0..SIZE {
        let j = i.reverse_bits() >> (usize::BITS - SIZE.ilog2());
        if i < j {
            re.swap(i, j);
        }
    }
    let mut width = 2;
    while width <= SIZE {
        for base in (0..SIZE).step_by(width) {
            for offset in 0..width / 2 {
                let angle = -std::f32::consts::TAU * offset as f32 / width as f32;
                let (sin, cos) = angle.sin_cos();
                let a = base + offset;
                let b = a + width / 2;
                let tr = cos * re[b] - sin * im[b];
                let ti = sin * re[b] + cos * im[b];
                re[b] = re[a] - tr;
                im[b] = im[a] - ti;
                re[a] += tr;
                im[a] += ti;
            }
        }
        width *= 2;
    }
    let mut bands = [0.0f32; BANDS];
    for (band, level) in bands.iter_mut().enumerate() {
        let low = 20.0 * 800.0f32.powf(band as f32 / BANDS as f32);
        let high = 20.0 * 800.0f32.powf((band + 1) as f32 / BANDS as f32);
        let first = ((low * SIZE as f32 / RATE as f32).floor() as usize).max(1);
        let last = ((high * SIZE as f32 / RATE as f32).ceil() as usize).min(SIZE / 2);
        let power = (first..last.max(first + 1))
            .map(|i| re[i] * re[i] + im[i] * im[i])
            .sum::<f32>();
        *level = (power.sqrt() * 4.0 / SIZE as f32).min(4.0);
    }
    bands
}
pub fn combine(spectra: impl Iterator<Item = [f32; BANDS]>) -> [f32; BANDS] {
    let mut energy = [0.0f32; BANDS];
    for spectrum in spectra {
        for (total, v) in energy.iter_mut().zip(spectrum) {
            *total += v * v;
        }
    }
    energy.map(|p| ((20.0 * p.sqrt().max(0.000_001).log10() + 60.0) / 60.0).clamp(0.0, 1.0))
}

#[cfg(test)]
mod tests {
    use super::*;
    fn tone(hz: f32, amplitude: f32) -> Vec<f32> {
        (0..SIZE)
            .map(|n| (std::f32::consts::TAU * hz * n as f32 / RATE as f32).sin() * amplitude)
            .collect()
    }
    #[test]
    fn silence_nonfinite_and_known_tones() {
        assert_eq!(combine([analyze(&[0.0; SIZE])].into_iter()), [0.0; BANDS]);
        assert_eq!(analyze(&[f32::NAN; SIZE]), [0.0; BANDS]);
        let peak = |hz| {
            let s = analyze(&tone(hz, 0.5));
            s.iter()
                .enumerate()
                .max_by(|a, b| a.1.total_cmp(b.1))
                .unwrap()
                .0
        };
        assert!(peak(100.0) < peak(1000.0));
        assert!(peak(1000.0) < peak(8000.0));
        let quiet = combine([analyze(&tone(1000.0, 0.01))].into_iter());
        let loud = combine([analyze(&tone(1000.0, 0.5))].into_iter());
        assert!(loud.iter().sum::<f32>() > quiet.iter().sum::<f32>());
        let mixed = combine([analyze(&tone(100.0, 0.2)), analyze(&tone(8000.0, 0.2))].into_iter());
        assert!(mixed[peak(100.0)] > 0.5 && mixed[peak(8000.0)] > 0.5);
    }
}
