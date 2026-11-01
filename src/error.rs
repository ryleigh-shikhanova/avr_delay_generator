#[derive(Debug)]
pub enum DelayError {
    InvalidIterations { requested: u32, maximum: u32 },
    CycleOverflow,
    UnsupportedRegister,
    UnachievableDelay,
}
