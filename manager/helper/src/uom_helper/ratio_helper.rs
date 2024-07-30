use super::Error;
use crate::uom_helper::cpu_ratio;
use lazy_regex::regex;
use num_traits::FromPrimitive;
use uom::si::rational64::Ratio;

fn match_unit<T>(raw: &str) -> bool
where
    T: uom::si::Unit + Sized,
{
    raw == <T as uom::si::Unit>::singular()
        || raw == <T as uom::si::Unit>::abbreviation()
        || raw == <T as uom::si::Unit>::plural()
}

pub fn parse(quantity: &str) -> Result<Ratio, super::Error> {
    let re = regex!(r"^([0-9.eE\-+]+)\s*(\w+)$");

    let captures = re
        .captures(quantity)
        .ok_or_else(|| Error::QuantityParsing(quantity.to_string()))?;
    let measure = captures
        .get(1)
        .ok_or_else(|| Error::QuantityParsing(quantity.to_string()))?
        .as_str()
        .parse::<f64>()
        .map_err(|_| Error::QuantityParsing(quantity.to_string()))?;
    let measure = num_rational::Ratio::from_f64(measure).ok_or_else(|| {
        Error::FailedConversion(
            quantity.to_string(),
            "rational number".to_string(),
        )
    })?;
    let unit = captures
        .get(2)
        .ok_or_else(|| Error::QuantityParsing(quantity.to_string()))?
        .as_str();

    if match_unit::<cpu_ratio::cpu>(unit) {
        Ok(Ratio::new::<cpu_ratio::cpu>(measure))
    } else if match_unit::<cpu_ratio::millicpu>(unit) {
        Ok(Ratio::new::<cpu_ratio::millicpu>(measure))
    } else {
        Err(Error::QuantityParsing(quantity.to_string()))
    }
}
