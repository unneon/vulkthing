#[repr(align(4))]
pub struct SpvArray<const N: usize>(pub [u8; N]);
