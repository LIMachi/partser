use std::fmt::Debug;
use std::ops::Neg;
use crate::decimal::Decimal;
use super::{StringReader, Number, ParserOut, ParserError, ExpectedChar};

pub fn number(unsigned: bool, multiplier: bool) -> impl Fn(StringReader) -> ParserOut<Number> {
    move |reader| Number::read(unsigned, multiplier, reader)
}

macro_rules! numbers {
    ($(($fn:ident,$unsinged:expr,$multiplier:expr,$type:ty)),* $(,)?) => {
        $(
            pub fn $fn(input: StringReader) -> ParserOut<$type> {
                Number::read($unsinged, $multiplier, input).and_then(|(reader, num)| <$type>::try_from(num).map(|num| (reader, num)))
            }
        )*
    };
}

numbers!(
    (f32,false,true,f32),
    (f64,false,true,f64),
    (uf32,true,true,f32),
    (uf64,true,true,f64),
    (usize,true,true,usize),
    (isize,false,true,isize),
    (u8,true,true,u8),
    (i8,false,true,i8),
    (u16,true,true,u16),
    (i16,false,true,i16),
    (u32,true,true,u32),
    (i32,false,true,i32),
    (u64,true,true,u64),
    (i64,false,true,i64),
    (u128,true,true,u128),
    (i128,false,true,i128),
);

fn saturating_unsigned_cast<T: TryFrom<u128> + Copy + TryInto<u128>>(from: u128, to_max: T) -> T where <T as TryInto<u128>>::Error: Debug {
    let max = to_max.try_into().unwrap();
    if from >= max {
        to_max
    } else {
        T::try_from(from).unwrap_or(to_max)
    }
}

fn saturating_signed_cast<T: TryFrom<u128> + Copy + TryInto<u128> + Neg<Output = T> + Debug>(negative: bool, from: u128, to_min: T, to_max: T) -> T where <T as TryInto<u128>>::Error: Debug {
    let max = to_max.try_into().unwrap();
    if negative {
        if from < max + 1 {
            T::try_from(from).map_or(to_min, |v| -v)
        } else {
            to_min
        }
    } else {
        if from < max {
            T::try_from(from).unwrap_or(to_max)
        } else {
            to_max
        }
    }
}

macro_rules! signed_try_from {
    ($($num:ty),*) => {
        $(
            impl TryFrom<Number> for $num {
                type Error = ParserError;

                fn try_from(value: Number) -> Result<Self, Self::Error> {
                    Ok(saturating_signed_cast(value.negative, value.integer().ok_or(ParserError::InvalidNumberCast { from: value, to: "$ty" })?, Self::MIN, Self::MAX))
                }
            }
        )*
    };
}

signed_try_from!(isize, i128, i64, i32, i16, i8);

impl TryFrom<Number> for u128 {
    type Error = ParserError;

    fn try_from(value: Number) -> Result<Self, Self::Error> {
        if value.negative {
            Err(ParserError::InvalidNumberCast { from: value, to: "u128" })?;
        }
        value.integer().ok_or(ParserError::InvalidNumberCast { from: value, to: "u128" })
    }
}

macro_rules! unsigned_try_from {
    ($($num:ty),*) => {
        $(
            impl TryFrom<Number> for $num {
                type Error = ParserError;

                fn try_from(value: Number) -> Result<Self, Self::Error> {
                    if value.negative {
                        Err(ParserError::InvalidNumberCast { from: value, to: "$ty" })?;
                    }
                    Ok(saturating_unsigned_cast(value.integer().ok_or(ParserError::InvalidNumberCast { from: value, to: "$ty" })?, Self::MAX))
                }
            }
        )*
    }
}

unsigned_try_from!(usize, u64, u32, u16, u8);

impl TryFrom<Number> for f64 {
    type Error = ParserError;

    fn try_from(value: Number) -> Result<Self, Self::Error> {
        let frac = if value.frac > 0 {
            value.frac as f64 / 10f64.powf(value.frac.decimal_digits() as f64)
        } else {
            0.
        };
        Ok((if value.negative { -1. } else { 1. }) * (value.integer as f64 + frac) * 10f64.powf(value.exponent as f64))
    }
}

impl TryFrom<Number> for f32 {
    type Error = ParserError;

    fn try_from(value: Number) -> Result<Self, Self::Error> {
        Ok(f64::try_from(value)? as f32)
    }
}

//TODO add support for separator (ex: '_' or ' ')
#[derive(PartialEq)]
enum NumberState {
    Start, //first symbols might be white space, +, - (or can switch to E, Integer or Dot)
    Integer, //any decimal characters, can switch to Dot, E or Finished
    Dot, //the . character, will transform to fractional or finished, can't accept an 'e' directly)
    Fractional, //any decimal character, can switch to E or Finished
    E, //the e character, can switch to Exponent (does not allow a Finished quantity kmgtep)
    Exponent, //any decimal characters, can switch to Finished
    Finished //optional quantity qualifier
}

impl Number {
    pub fn integer(&self) -> Option<u128> {
        let mut integer = self.integer;

        if self.exponent < 0 {
            let zeroes = integer.trailing_zeros();
            let exp = -self.exponent as u32;
            if zeroes >= exp {
                integer /= 10u128.pow(exp);
            } else {
                None?;
            }
        } else if self.exponent > 0 {
            integer = integer.saturating_mul(10u128.pow(self.exponent as u32));
        }

        if self.frac > 0 {
            let len = self.frac.decimal_digits() as i16;
            if self.exponent >= len {
                let exp = self.exponent - len;
                if exp >= 0 {
                    integer += self.frac as u128 * 10u128.pow(exp as u32);
                }
            } else {
                None?;
            }
        }

        Some(integer)
    }

    pub fn read(unsigned: bool, multiplier: bool, reader: StringReader) -> ParserOut<Self> {
        let r = reader.clone();
        let mut r = r.skip_whitespaces();
        let mut state = NumberState::Start;
        let mut out = Self::default();
        let mut negative_exponent = false;
        loop {
            match r[0] {
                '-' => {
                    if state == NumberState::Start && !unsigned { out.negative = !out.negative; }
                    else if state == NumberState::E { negative_exponent = !negative_exponent; }
                    else {
                        return Err(ParserError::InvalidCharacter { pos: r.true_index(0), char: '-', expected: ExpectedChar::Any("+0123456789".to_string()) });
                    }
                }
                '+' => {
                    if state != NumberState::Start && state != NumberState::E {
                        return Ok((r, out));
                    }
                }
                '.' if state == NumberState::Integer || state == NumberState::Start => { state = NumberState::Dot; },
                'e' if state == NumberState::Integer || state == NumberState::Fractional => { state = NumberState::E; },
                '0' ..= '9' => {
                    let v = r[0] as u64 - '0' as u64;
                    match state {
                        NumberState::Integer | NumberState::Start => if out.integer <= 34028236692093846346337460743176821145 || v <= 5 {
                            state = NumberState::Integer;
                            out.integer = out.integer * 10 + v as u128;
                        },
                        NumberState::Fractional | NumberState::Dot => if out.frac <= 1844674407370955161 || v <= 5 {
                            state = NumberState::Fractional;
                            out.frac = out.frac * 10 + v;
                        },
                        NumberState::Exponent | NumberState::E => if out.exponent <= 200 {
                            state = NumberState::Exponent;
                            out.exponent = out.exponent * 10 + v as i16;
                        }
                        _ => {
                            return Ok((r, out));
                        }
                    }
                }
                'y' | 'z' | 'a' | 'f' | 'p' | 'n' | 'u' | 'm' | 'k' | 'K' | 'M' | 'G' | 'T' | 'P' | 'E' | 'Z' | 'Y' if multiplier && (state == NumberState::Integer || state == NumberState::Exponent || state == NumberState::Fractional) => {
                    if negative_exponent {
                        negative_exponent = false;
                        out.exponent = -out.exponent;
                    }
                    match r[0] {
                        'y' => out.exponent -= 24, //yocto
                        'z' => out.exponent -= 21, //zepto
                        'a' => out.exponent -= 18, //atto
                        'f' => out.exponent -= 15, //femto
                        'p' => out.exponent -= 12, //pico
                        'n' => out.exponent -= 9, //nano
                        'u' => out.exponent -= 6, //micro
                        'm' => out.exponent -= 3, //milli
                        'k' | 'K' => out.exponent += 3, //kilo
                        'M' => out.exponent += 6, //mega
                        'G' => out.exponent += 9, //giga
                        'T' => out.exponent += 12, //tera
                        'P' => out.exponent += 15, //peta
                        'E' => out.exponent += 18, //exa
                        'Z' => out.exponent += 21, //zetta
                        'Y' => out.exponent += 24, //yotta
                        _ => {}
                    }
                    r = r.move_head(1)?;
                    state = NumberState::Finished;
                }
                _ => {
                    if state == NumberState::Start {
                        return Err(ParserError::InvalidCharacter { pos: r.true_index(0), char: r[0], expected: ExpectedChar::Any(format!("+{}0123456789", if unsigned { "" } else { "-" })) });
                    }
                    if negative_exponent {
                        out.exponent = -out.exponent;
                    }
                    return Ok((r, out));
                }
            }
            let t = r.move_head(1);
            if t.is_err() || state == NumberState::Finished {
                if negative_exponent {
                    out.exponent = -out.exponent;
                }
                return Ok((r, out));
            }
            r = t.unwrap();
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{Any, Parseable};
    use crate::prelude::Mappable;
    use super::*;

    #[test]
    fn complete() {
        let (rem, num) = Number::read(false, true, "---+123.45e-1k".into()).unwrap();
        assert_eq!(rem.view(1).as_str(), "");
        assert_eq!(num, Number {
            negative: true,
            integer: 123,
            frac: 45,
            exponent: 2,
        });
        assert_eq!(i64::try_from(num).unwrap(), -12345);
        assert_eq!("123.5e-1,49k".parse_with(true, (f32, ',', i32)).unwrap(), (12.35, ',', 49000));
        assert_eq!("123.4".parse_with(true, (i32.map_ok(|v| v as f32 + 5000.), f32).any()).unwrap(), 123.4);
        assert_eq!("123".parse_with(true, (i32.map_ok(|v| v as f32 + 5000.), f32).any()).unwrap(), 5123.);
        assert_eq!("123m".parse_with(true, (i32.map_ok(|v| v as f32 + 5000.), f32).any()).unwrap(), 0.123);
    }
}