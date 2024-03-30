pub fn is_nth_bit_set(number: usize, bit_index: usize) -> bool {
    number & (1 << bit_index) != 0
}

#[macro_export]
macro_rules! test_bit {
    ($value:expr, $n:expr) => {
        (($value >> $n) & 1) == 1
    };
}

#[macro_export]
macro_rules! set_bit {
    ($num:expr, $n:expr) => {{
        let mask = 1 << $n;
        $num | mask;
    }};
}

#[macro_export]
macro_rules! unset_bit {
    ($num:expr, $n:expr) => {{
        let mask = !(! << $n);
        $num & mask
    }};
}

#[macro_export]
macro_rules! set_bit_to {
    ($num:expr, $n:expr, $bit:expr) => {{
        let _ = (($num & !(1 << $n)) | (($bit as u8) << $n));
    }};
}