use std::f32::consts::PI;

pub enum FilterType {
    HighPass,
    LowShelf,
    LowPass,
    HighShelf,
}

pub struct BiquadFilter {
    b0: f32,
    b1: f32,
    b2: f32,
    a1: f32,
    a2: f32,
    z1: f32,
    z2: f32,
}

impl BiquadFilter {
    pub fn new(
        filter_type: FilterType,
        sample_rate: f32,
        frequency: f32,
        q: f32,
        gain_db: f32,
    ) -> Self {
        let mut f = Self {
            b0: 1.0,
            b1: 0.0,
            b2: 0.0,
            a1: 0.0,
            a2: 0.0,
            z1: 0.0,
            z2: 0.0,
        };
        f.update(filter_type, sample_rate, frequency, q, gain_db);
        f
    }

    pub fn update(
        &mut self,
        filter_type: FilterType,
        sample_rate: f32,
        frequency: f32,
        q: f32,
        gain_db: f32,
    ) {
        let w0 = 2.0 * PI * frequency / sample_rate;
        let cos = w0.cos();
        let sin = w0.sin();
        let a = 10.0f32.powf(gain_db / 40.0);

        match filter_type {
            FilterType::HighPass => {
                let alpha = sin / (2.0 * q);
                let b0 = (1.0 + cos) / 2.0;
                let b1 = -(1.0 + cos);
                let b2 = (1.0 + cos) / 2.0;
                let a0 = 1.0 + alpha;
                let a1 = -2.0 * cos;
                let a2 = 1.0 - alpha;

                self.b0 = b0 / a0;
                self.b1 = b1 / a0;
                self.b2 = b2 / a0;
                self.a1 = a1 / a0;
                self.a2 = a2 / a0;
            }

            FilterType::LowPass => {
                let alpha = sin / (2.0 * q);
                let b0 = (1.0 - cos) / 2.0;
                let b1 = 1.0 - cos;
                let b2 = (1.0 - cos) / 2.0;
                let a0 = 1.0 + alpha;
                let a1 = -2.0 * cos;
                let a2 = 1.0 - alpha;

                self.b0 = b0 / a0;
                self.b1 = b1 / a0;
                self.b2 = b2 / a0;
                self.a1 = a1 / a0;
                self.a2 = a2 / a0;
            }

            FilterType::LowShelf => {
                let alpha = sin / 2.0 * ((a + 1.0 / a) * (1.0 / q - 1.0) + 2.0).sqrt();

                let b0 = a * ((a + 1.0) - (a - 1.0) * cos + alpha);
                let b1 = 2.0 * a * ((a - 1.0) - (a + 1.0) * cos);
                let b2 = a * ((a + 1.0) - (a - 1.0) * cos - alpha);
                let a0 = (a + 1.0) + (a - 1.0) * cos + alpha;
                let a1 = -2.0 * ((a - 1.0) + (a + 1.0) * cos);
                let a2 = (a + 1.0) + (a - 1.0) * cos - alpha;

                self.b0 = b0 / a0;
                self.b1 = b1 / a0;
                self.b2 = b2 / a0;
                self.a1 = a1 / a0;
                self.a2 = a2 / a0;
            }

            FilterType::HighShelf => {
                let alpha = sin / 2.0 * ((a + 1.0 / a) * (1.0 / q - 1.0) + 2.0).sqrt();

                let b0 = a * ((a + 1.0) + (a - 1.0) * cos + alpha);
                let b1 = -2.0 * a * ((a - 1.0) + (a + 1.0) * cos);
                let b2 = a * ((a + 1.0) + (a - 1.0) * cos - alpha);
                let a0 = (a + 1.0) - (a - 1.0) * cos + alpha;
                let a1 = 2.0 * ((a - 1.0) - (a + 1.0) * cos);
                let a2 = (a + 1.0) - (a - 1.0) * cos - alpha;

                self.b0 = b0 / a0;
                self.b1 = b1 / a0;
                self.b2 = b2 / a0;
                self.a1 = a1 / a0;
                self.a2 = a2 / a0;
            }
        }
    }

    #[inline]
    pub fn process(&mut self, x: f32) -> f32 {
        let y = self.b0 * x + self.z1;
        self.z1 = self.b1 * x - self.a1 * y + self.z2;
        self.z2 = self.b2 * x - self.a2 * y;
        y
    }

    pub fn reset(&mut self) {
        self.z1 = 0.0;
        self.z2 = 0.0;
    }
}