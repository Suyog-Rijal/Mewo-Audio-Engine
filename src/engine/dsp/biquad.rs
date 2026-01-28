pub enum FilterType {
    HighPass,
    LowShelf,
}

pub struct BiquadFilter {
    // Coefficients
    a1: f32,
    a2: f32,
    b0: f32,
    b1: f32,
    b2: f32,

    // Delay elements (state)
    z1: f32,
    z2: f32,
}

impl BiquadFilter {
    pub fn new(filter_type: FilterType, sample_rate: f32, frequency: f32, q: f32, gain_db: f32) -> Self {
        let mut filter = Self {
            a1: 0.0, a2: 0.0, b0: 1.0, b1: 0.0, b2: 0.0,
            z1: 0.0, z2: 0.0,
        };
        filter.update_coefficients(filter_type, sample_rate, frequency, q, gain_db);
        filter
    }

    pub fn update_coefficients(&mut self, filter_type: FilterType, sample_rate: f32, frequency: f32, q: f32, gain_db: f32) {
        let w0 = 2.0 * std::f32::consts::PI * frequency / sample_rate;
        let alpha = w0.sin() / (2.0 * q);
        let a = 10.0f32.powf(gain_db / 40.0); // A = 10^(dB/40) for shelving

        match filter_type {
            FilterType::HighPass => {
                let cos_w0 = w0.cos();
                let b0 = (1.0 + cos_w0) / 2.0;
                let b1 = -(1.0 + cos_w0);
                let b2 = (1.0 + cos_w0) / 2.0;
                let a0 = 1.0 + alpha;
                let a1 = -2.0 * cos_w0;
                let a2 = 1.0 - alpha;

                self.b0 = b0 / a0;
                self.b1 = b1 / a0;
                self.b2 = b2 / a0;
                self.a1 = a1 / a0;
                self.a2 = a2 / a0;
            }
            FilterType::LowShelf => {
                let cos_w0 = w0.cos();
                let beta = (a.powi(2) + 1.0).sqrt() / q; // Actually it's sqrt(A)/Q in some formulas, but RBJ uses this:
                // For low shelf: 
                // beta = sqrt( (A^2 + 1)/S - (A-1)^2 ) where S is shelf slope. 
                // If S=1, then beta = sqrt(A)/Q matches.
                // Let's use the standard RBJ formula:
                let alpha_shelf = (w0.sin() / 2.0) * ((a + 1.0/a) * (1.0/q - 1.0) + 2.0).sqrt();

                let b0 = a * ((a + 1.0) - (a - 1.0) * cos_w0 + alpha_shelf);
                let b1 = 2.0 * a * ((a - 1.0) - (a + 1.0) * cos_w0);
                let b2 = a * ((a + 1.0) - (a - 1.0) * cos_w0 - alpha_shelf);
                let a0 = (a + 1.0) + (a - 1.0) * cos_w0 + alpha_shelf;
                let a1 = -2.0 * ((a - 1.0) + (a + 1.0) * cos_w0);
                let a2 = (a + 1.0) + (a - 1.0) * cos_w0 - alpha_shelf;

                self.b0 = b0 / a0;
                self.b1 = b1 / a0;
                self.b2 = b2 / a0;
                self.a1 = a1 / a0;
                self.a2 = a2 / a0;
            }
        }
    }

    #[inline]
    pub fn process(&mut self, input: f32) -> f32 {
        let output = self.b0 * input + self.z1;
        self.z1 = self.b1 * input - self.a1 * output + self.z2;
        self.z2 = self.b2 * input - self.a2 * output;
        output
    }

    pub fn reset(&mut self) {
        self.z1 = 0.0;
        self.z2 = 0.0;
    }
}
