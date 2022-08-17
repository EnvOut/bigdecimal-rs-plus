use std::cmp::Ordering;
use std::convert::TryFrom;
use std::fmt::{Debug, Formatter};
use std::iter::Sum;
use std::ops::{Add, AddAssign, Div, Mul, Neg, Rem, Sub};
use std::result::Result as StdErr;
use std::str::FromStr;

pub use bigdecimal::{FromPrimitive, Num, num_bigint::Sign, num_traits::Pow, One, ParseBigDecimalError, Signed, ToPrimitive, Zero};
use bigdecimal::BigDecimal as InnerDecimal;
pub use bigdecimal::num_bigint::ParseBigIntError;

use crate::errors::BaseCrateError;
use crate::errors::Result;

pub struct BigDecimal {
    inner: InnerDecimal,
    #[cfg(debug_assertions)]
    _debug_value: String,
}

impl Pow<i32> for BigDecimal {
    type Output = BigDecimal;

    fn pow(self, rhs: i32) -> Self::Output {
        let value_f64 = ToPrimitive::to_f64(&self.inner).unwrap().pow(rhs);
        BigDecimal::from_f64(value_f64).unwrap()
    }
}

impl AddAssign for BigDecimal {
    fn add_assign(&mut self, rhs: Self) {
        self.inner += rhs.inner;
    }
}

impl Default for BigDecimal {
    fn default() -> Self {
        let inner = InnerDecimal::default();
        Self::new_from_inner(inner)
    }
}

impl Neg for BigDecimal {
    type Output = BigDecimal;

    fn neg(self) -> Self::Output {
        Self::new_from_inner(self.inner.neg())
    }
}

impl<'a> Neg for &'a BigDecimal {
    type Output = BigDecimal;

    #[inline]
    fn neg(self) -> BigDecimal {
        let inner = -self.inner.clone();
        BigDecimal::new_from_inner(inner)
    }
}

// impl <'a>Neg for &BigDecimal {
//     type Output = &'a BigDecimal;
//
//     fn neg(self) -> Self::Output {
//         let inner = -&self.inner.neg();
//         Self::new_from_inner(inner)
//     }
// }

impl TryFrom<f64> for BigDecimal {
    type Error = ParseBigDecimalError;

    fn try_from(value: f64) -> StdErr<Self, Self::Error> {
        InnerDecimal::try_from(value)
            .map(|inner| Self::new_from_inner(inner))
    }
}

impl Debug for BigDecimal {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.inner.fmt(f)
    }
}

impl Clone for BigDecimal {
    fn clone(&self) -> Self {
        let inner = self.inner.clone();
        Self::new_from_inner(inner)
    }
}

impl BigDecimal {
    // #[cfg(debug_assertions)]
    // pub fn zero()->Self{
    //     let inner_value = InnerDecimal::zero();
    //     Self { inner: inner_value, debug_value: inner_value.to_string() }
    // }
    // #[cfg(not(debug_assertions))]
    // pub fn zero()->Self{
    //     Self { inner: InnerDecimal::zero() }
    // }

    #[cfg(debug_assertions)]
    pub fn new_from_inner(inner: InnerDecimal) -> Self {
        Self { _debug_value: inner.to_string(), inner: inner }
    }

    #[cfg(not(debug_assertions))]
    pub fn new_from_inner(inner: InnerDecimal) -> Self {
        Self { inner: inner }
    }

    pub fn string_to_bigdecimal(value: impl ToString, scale: i64) -> StdErr<Self, ParseBigIntError> {
        let digits_string = value.to_string();
        let digits = bigdecimal::num_bigint::BigInt::from_str(&digits_string)?;
        let inner = InnerDecimal::new(digits, scale);
        Ok(Self::new_from_inner(inner))
    }

    pub fn divide_and_into_bigdecimal(first: impl ToString, second: impl ToString) -> BigDecimal {
        let first = InnerDecimal::from_str(&first.to_string()).unwrap();
        let second = InnerDecimal::from_str(&second.to_string()).unwrap();

        let inner = first / second;
        Self::new_from_inner(inner)
    }

    pub fn normalized(&self) -> BigDecimal {
        Self::new_from_inner(self.inner.normalized())
    }

    pub fn gt_zero(&self) -> bool {
        self.is_positive() && !self.is_zero()
    }

    pub fn is_positive(&self) -> bool {
        self.inner.is_positive()
    }

    pub fn is_zero(&self) -> bool {
        self.inner.is_zero()
    }

    pub fn is_negative(&self) -> bool {
        self.inner.is_negative()
    }

    pub fn scale_to(&self, scale: i64) -> Self {
        // BigDecimal::string_to_bigdecimal(self.to_string(), scale)
        let (big_value, exp) = self.inner.as_bigint_and_exponent();

        let decimal = {
            let i = (exp - scale).abs() as u32;
            let ir: i64 = 10i64.pow(i);

            let new_big_value = if exp > scale {
                big_value / ir
            } else {
                big_value * ir
            };

            let inner = InnerDecimal::new(new_big_value, scale);
            BigDecimal::new_from_inner(inner)
        };

        decimal
    }

    pub fn scale_to2(&self, scale: i64) -> StdErr<Self, ParseBigDecimalError> {
        let to_string = self.to_string();
        let big_decimal_parts: Vec<String> = {
            let to_string = to_string.clone();
            let big_decimal_parts: Vec<String> = to_string.split(".").map(|it| it.to_string()).collect();
            big_decimal_parts
        };
        let scaled_str_value: String = match big_decimal_parts.len() {
            1 => big_decimal_parts.get(0).cloned().unwrap(),
            2 => {
                let first = big_decimal_parts.get(0).unwrap();
                let second = {
                    let part = big_decimal_parts.get(1).unwrap();
                    let part_len = part.len();
                    let scale = scale as usize;
                    if scale < part_len {
                        part[0..scale].to_string()
                    } else {
                        part.to_string()
                    }
                };
                format!("{}.{}", first, second)
            }
            _ => unimplemented!(),
        };
        BigDecimal::from_str(&scaled_str_value)
    }

    pub fn round(&self, round_digits: i64) -> BigDecimal {
        let new_inner = self.inner.round(round_digits);
        Self::new_from_inner(new_inner)
    }

    pub fn round_safe(&self, round_digits: i64) -> BigDecimal {
        let string_value = self.to_string();

        let mut split = string_value.splitn(2, ".");
        let left = split.next().unwrap();
        let mut rigth: &str = split.next().unwrap_or("0");
        if rigth.len() > round_digits as usize {
            let rigth2: &str = &rigth[0..(round_digits as usize)];
            rigth = rigth2;
        }
        Self::from_str(&format!("{}.{}", left, rigth)).unwrap()
    }
}

impl FromStr for BigDecimal {
    type Err = ParseBigDecimalError;

    #[inline]
    fn from_str(s: &str) -> StdErr<Self, Self::Err> {
        let inner = InnerDecimal::from_str(s)?;
        Ok(BigDecimal::new_from_inner(inner))
    }
}

impl FromPrimitive for BigDecimal {
    #[inline]
    fn from_i64(n: i64) -> Option<Self> {
        let inner = InnerDecimal::from(n);
        Some(Self::new_from_inner(inner))
    }

    #[inline]
    fn from_u64(n: u64) -> Option<Self> {
        let inner = InnerDecimal::from(n);
        Some(Self::new_from_inner(inner))
    }

    #[inline]
    fn from_f32(n: f32) -> Option<Self> {
        InnerDecimal::try_from(n).ok()
            .map(|inner| Self::new_from_inner(inner))
    }

    #[inline]
    fn from_f64(n: f64) -> Option<Self> {
        InnerDecimal::try_from(n).ok()
            .map(|inner| Self::new_from_inner(inner))
    }
}

impl PartialEq for BigDecimal {
    fn eq(&self, other: &Self) -> bool {
        self.inner.eq(&other.inner)
    }
}

impl Add<BigDecimal> for BigDecimal {
    type Output = BigDecimal;

    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        Self::new_from_inner(self.inner.add(rhs.inner))
    }
}

impl<'a> Add<&'a BigDecimal> for BigDecimal {
    type Output = BigDecimal;

    #[inline]
    fn add(self, rhs: &'a BigDecimal) -> Self::Output {
        Self::new_from_inner(self.inner.add(&rhs.inner))
    }
}

impl Add<i32> for BigDecimal {
    type Output = BigDecimal;

    #[inline]
    fn add(self, rhs: i32) -> Self::Output {
        self + BigDecimal::from(rhs)
    }
}

impl<'a> Add<BigDecimal> for &'a BigDecimal {
    type Output = BigDecimal;

    #[inline]
    fn add(self, rhs: BigDecimal) -> Self::Output {
        let inner = &self.inner + rhs.inner;
        BigDecimal::new_from_inner(inner)
    }
}

impl<'a, 'b> Add<&'b BigDecimal> for &'a BigDecimal {
    type Output = BigDecimal;

    #[inline]
    fn add(self, rhs: &'b BigDecimal) -> Self::Output {
        let inner = &self.inner + &rhs.inner;
        BigDecimal::new_from_inner(inner)
    }
}

impl Zero for BigDecimal {
    #[inline]
    fn zero() -> Self {
        let inner = InnerDecimal::zero();
        Self::new_from_inner(inner)
    }

    #[inline]
    fn is_zero(&self) -> bool {
        self.inner.is_zero()
    }
}

impl One for BigDecimal {
    fn one() -> Self {
        Self::new_from_inner(InnerDecimal::one())
    }
}

impl Sub<BigDecimal> for BigDecimal {
    type Output = BigDecimal;

    fn sub(self, rhs: Self) -> Self::Output {
        BigDecimal::new_from_inner(self.inner.sub(rhs.inner))
    }
}

impl Sub<&BigDecimal> for BigDecimal {
    type Output = BigDecimal;

    fn sub(self, rhs: &Self) -> Self::Output {
        BigDecimal::new_from_inner(self.inner.sub(&rhs.inner))
    }
}

impl<'a> Sub<BigDecimal> for &'a BigDecimal {
    type Output = BigDecimal;

    fn sub(self, rhs: BigDecimal) -> Self::Output {
        let inner = &self.inner - rhs.inner;
        BigDecimal::new_from_inner(inner)
    }
}

impl<'a> Sub<&'a BigDecimal> for &'a BigDecimal {
    type Output = BigDecimal;

    fn sub(self, rhs: &'a BigDecimal) -> Self::Output {
        let inner = &self.inner - &rhs.inner;
        BigDecimal::new_from_inner(inner)
    }
}

impl Sub<i32> for BigDecimal {
    type Output = BigDecimal;

    #[inline]
    fn sub(self, rhs: i32) -> Self::Output {
        self - BigDecimal::from(rhs)
    }
}

impl<'a> Sub<i32> for &'a BigDecimal {
    type Output = BigDecimal;

    fn sub(self, rhs: i32) -> Self::Output {
        self - BigDecimal::from(rhs)
    }
}

impl Mul<BigDecimal> for BigDecimal {
    type Output = BigDecimal;

    fn mul(self, rhs: Self) -> Self::Output {
        BigDecimal::new_from_inner(self.inner.mul(rhs.inner))
    }
}

impl Mul<i32> for BigDecimal {
    type Output = BigDecimal;

    fn mul(self, rhs: i32) -> Self::Output {
        BigDecimal::new_from_inner(self.inner.mul(InnerDecimal::from(rhs)))
    }
}

impl Div<i32> for &BigDecimal {
    type Output = Result<BigDecimal>;

    fn div(self, rhs: i32) -> Self::Output {
        if rhs.is_zero() {
            Err(BaseCrateError::DivisionByZero)
        } else {
            let inner = &self.inner / rhs;
            Ok(BigDecimal::new_from_inner(inner))
        }
    }
}

impl Div<i32> for BigDecimal {
    type Output = crate::errors::Result<BigDecimal>;

    fn div(self, rhs: i32) -> Self::Output {
        if rhs.is_zero() {
            Err(BaseCrateError::DivisionByZero)
        } else {
            let inner = self.inner / rhs;
            Ok(BigDecimal::new_from_inner(inner))
        }
    }
}

impl<'a> Div for &'a BigDecimal {
    type Output = crate::errors::Result<BigDecimal>;

    fn div(self, rhs: Self) -> Self::Output {
        if rhs.is_zero() {
            Err(BaseCrateError::DivisionByZero)
        } else {
            let inner = &self.inner / &rhs.inner;
            Ok(BigDecimal::new_from_inner(inner))
        }
    }
}

impl<'a> Sum<&'a BigDecimal> for BigDecimal {
    #[inline]
    fn sum<I: Iterator<Item=&'a BigDecimal>>(iter: I) -> BigDecimal {
        iter.fold(Zero::zero(), |a, b| BigDecimal::new_from_inner(a.inner + &b.inner))
    }
}

impl<'a> Mul<&'a BigDecimal> for BigDecimal {
    type Output = BigDecimal;

    fn mul(self, rhs: &'a BigDecimal) -> Self::Output {
        let inner = self.inner * &rhs.inner;
        BigDecimal::new_from_inner(inner)
    }
}

impl<'a> Mul<BigDecimal> for &'a BigDecimal {
    type Output = BigDecimal;

    fn mul(self, rhs: BigDecimal) -> Self::Output {
        let inner = &self.inner * rhs.inner;
        BigDecimal::new_from_inner(inner)
    }
}

impl<'a> Mul<&'a BigDecimal> for &'a BigDecimal {
    type Output = BigDecimal;

    fn mul(self, rhs: &'a BigDecimal) -> Self::Output {
        let inner = &self.inner * &rhs.inner;
        BigDecimal::new_from_inner(inner)
    }
}

impl Sum for BigDecimal {
    #[inline]
    fn sum<I: Iterator<Item=BigDecimal>>(iter: I) -> BigDecimal {
        iter.fold(Zero::zero(), |a, b| BigDecimal::new_from_inner(a.inner + b.inner))
    }
}

impl Div<BigDecimal> for BigDecimal {
    type Output = crate::errors::Result<BigDecimal>;

    fn div(self, rhs: Self) -> Self::Output {
        if rhs.is_zero() {
            Err(BaseCrateError::DivisionByZero)?
        } else {
            Ok(BigDecimal::new_from_inner(self.inner.div(rhs.inner)))
        }
    }
}

impl Div<&BigDecimal> for BigDecimal {
    type Output = crate::errors::Result<BigDecimal>;

    fn div(self, rhs: &Self) -> Self::Output {
        if rhs.is_zero() {
            Err(BaseCrateError::DivisionByZero)?
        } else {
            Ok(BigDecimal::new_from_inner(self.inner.div(&rhs.inner)))
        }
    }
}

impl Rem<BigDecimal> for BigDecimal {
    type Output = BigDecimal;

    fn rem(self, rhs: Self) -> Self::Output {
        BigDecimal::new_from_inner(self.inner.rem(rhs.inner))
    }
}

impl ToPrimitive for BigDecimal {
    fn to_i64(&self) -> Option<i64> {
        self.inner.to_i64()
    }
    fn to_u64(&self) -> Option<u64> {
        self.inner.to_u64()
    }

    fn to_f64(&self) -> Option<f64> {
        self.inner.to_f64()
    }
}

impl PartialOrd for BigDecimal {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.inner.partial_cmp(&other.inner)
    }

    fn lt(&self, other: &Self) -> bool {
        self.inner.lt(&other.inner)
    }

    fn le(&self, other: &Self) -> bool {
        self.inner.le(&other.inner)
    }

    fn gt(&self, other: &Self) -> bool {
        self.inner.gt(&other.inner)
    }

    fn ge(&self, other: &Self) -> bool {
        self.inner.ge(&other.inner)
    }
}

impl PartialEq<f64> for BigDecimal {
    fn eq(&self, other: &f64) -> bool {
        let other = BigDecimal::from_f64(other.clone());
        if let Some(other) = other {
            self.eq(&other)
        } else {
            false
        }
    }
}

impl PartialOrd<f64> for BigDecimal {
    fn partial_cmp(&self, other: &f64) -> Option<Ordering> {
        let other = BigDecimal::from_f64(other.clone()).unwrap_or_default();
        self.partial_cmp(&other)
    }
}

impl ToString for BigDecimal {
    fn to_string(&self) -> String {
        self.inner.to_string()
    }
}

impl From<i32> for BigDecimal {
    fn from(value: i32) -> Self {
        let inner = InnerDecimal::from(value);
        Self::new_from_inner(inner)
    }
}

#[cfg(test)]
mod test {
    use std::str::FromStr;

    use bigdecimal::{One, Zero};

    use crate::decimal::BigDecimal;
    use crate::errors::BaseCrateError;

    #[test]
    fn div_by_zero_test(){
        let div_zero = BigDecimal::one() / BigDecimal::zero();
        assert!(div_zero.is_err())
    }
}
