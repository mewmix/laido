// Simple xorshift32 for deterministic randomness without external deps
#[derive(Copy, Clone, Debug)]
pub struct XorShift32 { state: u32 }

impl XorShift32 {
    pub fn new(seed: u32) -> Self { Self { state: seed.max(1) } }
    pub fn next_u32(&mut self) -> u32 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.state = x;
        x
    }
    pub fn next_f32(&mut self) -> f32 { (self.next_u32() as f32) / (u32::MAX as f32) }
    pub fn range_u64(&mut self, min: u64, max: u64) -> u64 {
        min + (self.next_u32() as u64 % ((max - min + 1).max(1)))
    }
}
