pub struct LowPass {
    value: f32,
    alpha: f32,
}

impl LowPass {
    pub fn new(alpha: f32, initial: f32) -> Self {
        Self {
            value: initial,
            alpha,
        }
    }

    pub fn update(&mut self, input: f32) -> f32 {
        self.value += self.alpha * (input - self.value);
        self.value
    }

    pub fn value(&mut self) -> f32 {
        self.value
    }
}
