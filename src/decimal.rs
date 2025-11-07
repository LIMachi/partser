use std::ops::{DivAssign, Rem};

pub trait Decimal {
    #[allow(dead_code)]
    fn trailing_decimal_zeroes(&self) -> usize;
    fn decimal_digits(&self) -> usize;
}

const POWERS_USIZE: &'static [(usize, usize)] = &[
    #[cfg(target_pointer_width = "64")]
    (10000000000000000000, 19),
    (1000000000, 9),
    (10000, 4),
    (100, 2),
    (10, 1)
];

const POWERS_U128: &'static [(u128, usize)] = &[
    (100000000000000000000000000000000000000, 38),
    (10000000000000000000, 19),
    (1000000000, 9),
    (10000, 4),
    (100, 2),
    (10, 1)
];

const POWERS_U64: &'static [(u64, usize)] = &[
    (10000000000000000000, 19),
    (1000000000, 9),
    (10000, 4),
    (100, 2),
    (10, 1)
];

const POWERS_U32: &'static [(u32, usize)] = &[
    (1000000000, 9),
    (10000, 4),
    (100, 2),
    (10, 1)
];

const POWERS_U16: &'static [(u16, usize)] = &[
    (10000, 4),
    (100, 2),
    (10, 1)
];

const POWERS_U8: &'static [(u8, usize)] = &[
    (100, 2),
    (10, 1)
];

macro_rules! unsigned_decimal_impl {
    ($($num:ty=>$powers:expr),*) => {
        $(
            impl Decimal for $num {
                fn trailing_decimal_zeroes(&self) -> usize {
                    trailing_decimal_zeroes(*self, $powers, 0)
                }

                fn decimal_digits(&self) -> usize {
                    decimal_digits(*self, $powers, 0)
                }
            }
        )*
    }
}

unsigned_decimal_impl!(
    usize=>POWERS_USIZE,
    u128=>POWERS_U128,
    u64=>POWERS_U64,
    u32=>POWERS_U32,
    u16=>POWERS_U16,
    u8=>POWERS_U8
);

macro_rules! signed_trailing_decimal_zeroes {
    ($($num:ty),*) => {
        $(
            impl Decimal for $num {
                fn trailing_decimal_zeroes(&self) -> usize {
                    self.unsigned_abs().trailing_decimal_zeroes()
                }

                fn decimal_digits(&self) -> usize {
                    self.unsigned_abs().decimal_digits()
                }
            }
        )*
    }
}

signed_trailing_decimal_zeroes!(isize, i128, i64, i32, i16, i8);

#[allow(dead_code)]
pub fn trailing_decimal_zeroes<T: Default + Copy + PartialOrd + PartialEq + DivAssign + Rem<Output = T>>(mut num: T, powers: &[(T, usize)], zero: T) -> usize {
    if num == zero {
        return 0;
    }
    let mut count = 0;
    for &(pow, len) in powers {
        while num % pow == zero {
            num /= pow;
            count += len;
        }
    }
    count
}

pub fn decimal_digits<T: Copy + PartialOrd + DivAssign>(mut num: T, powers: &[(T, usize)], zero: T) -> usize {
    if num == zero {
        return 1;
    }
    let mut digits = 1;
    for &(pow, len) in powers {
        while num >= pow {
            num /= pow;
            digits += len;
        }
    }
    digits
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn trailing() {
        assert_eq!(100u8.trailing_decimal_zeroes(), 2);
        assert_eq!(10230u16.trailing_decimal_zeroes(), 1);
        assert_eq!(32u32.trailing_decimal_zeroes(), 0);
        assert_eq!(15000000000000000000u64.trailing_decimal_zeroes(), 18);
        assert_eq!(1000000004400003000000u128.trailing_decimal_zeroes(), 6);
        assert_eq!(1200234050usize.trailing_decimal_zeroes(), 1);

        assert_eq!(10i8.trailing_decimal_zeroes(), 1);
        assert_eq!(1024u16.trailing_decimal_zeroes(), 0);
        assert_eq!(640000u32.trailing_decimal_zeroes(), 4);
        assert_eq!(100000000000000i64.trailing_decimal_zeroes(), 14);
        assert_eq!(10000000000000000000000000i128.trailing_decimal_zeroes(), 25);
        assert_eq!(1230000000isize.trailing_decimal_zeroes(), 7);

        assert_eq!(0u128.trailing_decimal_zeroes(), 0);
        assert_eq!(1u128.trailing_decimal_zeroes(), 0);
        assert_eq!(10u128.trailing_decimal_zeroes(), 1);
        assert_eq!(100u128.trailing_decimal_zeroes(), 2);
        assert_eq!(1000u128.trailing_decimal_zeroes(), 3);
        assert_eq!(1000000000000000000u128.trailing_decimal_zeroes(), 18);
        assert_eq!(100000000000000000000000000000000000000u128.trailing_decimal_zeroes(), 38);
        assert_eq!(1234567000000000u128.trailing_decimal_zeroes(), 9);

        assert_eq!(i128::MIN.trailing_decimal_zeroes(), 0);
        assert_eq!((-123000).trailing_decimal_zeroes(), 3);
    }

    #[test]
    fn digits() {
        assert_eq!(0u128.decimal_digits(), 1);
        assert_eq!(1u128.decimal_digits(), 1);
        assert_eq!(9u128.decimal_digits(), 1);
        assert_eq!(9999u128.decimal_digits(), 4);
        assert_eq!(10000u128.decimal_digits(), 5);
        assert_eq!(12345678u128.decimal_digits(), 8);
        assert_eq!(100000000000000000000000000000000000000u128.decimal_digits(), 39);
        assert_eq!((-123i32).decimal_digits(), 3);
    }
}