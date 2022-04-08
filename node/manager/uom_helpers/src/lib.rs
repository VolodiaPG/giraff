#[macro_use]
extern crate uom;
use std::str::FromStr;

use lazy_static::lazy_static;

use regex::Regex;
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Unable to parse the quantity: {0}")]
    QuantityParsing(String),
}

pub fn parse_quantity<T>(quantity: &str) -> Result<T, Error>
where
    T: FromStr,
{
    lazy_static! {
        static ref RE: Regex = Regex::new(r"^([0-9\.eE\-+]+)\s*(\w+)$").unwrap();
    }

    let captures = RE
        .captures(quantity)
        .ok_or_else(|| Error::QuantityParsing(quantity.to_string()))?;
    let measure = captures
        .get(1)
        .ok_or_else(|| Error::QuantityParsing(quantity.to_string()))?;
    let unit = captures
        .get(2)
        .ok_or_else(|| Error::QuantityParsing(quantity.to_string()))?;

    let qty = format!("{} {}", measure.as_str(), unit.as_str())
        .parse::<T>()
        .map_err(|_| Error::QuantityParsing(quantity.to_string()))?;

    Ok(qty)
}

pub mod cpu_ratio;
pub mod information;
pub mod ratio;
pub mod time;
