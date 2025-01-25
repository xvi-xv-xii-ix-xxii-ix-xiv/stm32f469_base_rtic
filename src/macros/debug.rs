#[macro_export]
macro_rules! debug_print {
    ($($arg:tt)*) => {
        ::defmt::debug!($($arg)*);
    };
}
