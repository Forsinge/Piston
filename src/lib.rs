#[macro_export]
macro_rules! index {
    ($bb:expr) => {
        $bb.leading_zeros() as usize
    }
}

#[macro_export]
macro_rules! indexu8 {
    ($bb:expr) => {
        $bb.leading_zeros() as u8
    }
}