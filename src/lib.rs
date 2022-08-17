#![forbid(unsafe_code)]
#![deny(non_upper_case_globals)]
#![deny(non_camel_case_types)]
#![deny(non_snake_case)]
#![deny(unused_mut)]
#![deny(unused_variables)]
#![deny(dead_code)]
#![deny(unused_imports)]
#![deny(warnings)]

pub mod errors;

pub use bigdecimal::ParseBigDecimalError;

pub mod big_decimal_json;
pub mod decimal;
