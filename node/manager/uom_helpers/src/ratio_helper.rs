use lazy_static::lazy_static;
use regex::Regex;
use uom::si::f64::Ratio;

use super::Error;

fn match_unit<T>(raw: &str) -> bool
where
    T: uom::si::Unit + Sized,
{
    raw == <T as uom::si::Unit>::singular()
        || raw == <T as uom::si::Unit>::abbreviation()
        || raw == <T as uom::si::Unit>::plural()
}

pub fn parse(quantity: &str) -> Result<Ratio, super::Error> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"^([0-9\.eE\-+]+)\s*(\w+)$").unwrap();
    }

    let captures = RE
        .captures(quantity)
        .ok_or_else(|| Error::QuantityParsing(quantity.to_string()))?;
    let measure = captures
        .get(1)
        .ok_or_else(|| Error::QuantityParsing(quantity.to_string()))?
        .as_str()
        .parse::<f64>()
        .map_err(|_| Error::QuantityParsing(quantity.to_string()))?;
    let unit = captures
        .get(2)
        .ok_or_else(|| Error::QuantityParsing(quantity.to_string()))?
        .as_str();

    if match_unit::<crate::cpu_ratio::cpu>(unit) {
        Ok(Ratio::new::<crate::cpu_ratio::cpu>(measure))
    } else if match_unit::<crate::cpu_ratio::millicpu>(unit) {
        Ok(Ratio::new::<crate::cpu_ratio::millicpu>(measure))
    } else {
        Err(Error::QuantityParsing(quantity.to_string()))
    }
}
