use adler32::RollingAdler32;

pub trait Ad32 {
    fn ad32(&self) -> u32;
}

impl Ad32 for &[u8] {
    fn ad32(&self) -> u32 {
        let rolling = RollingAdler32::from_buffer(self);
        rolling.hash()
    }
}
