use core::fmt;
use lazy_static::lazy_static;
use regex::Regex;
use serde::de::Visitor;
use uom::si::f64::Ratio;
use uom::{fmt::DisplayStyle::Abbreviation, si::Unit};

use crate::utils::{parse_quantity, Error};
pub struct Helper;

impl serde_with::SerializeAs<Ratio> for Helper {
    fn serialize_as<S>(value: &Ratio, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(
            format!(
                "{:?}",
                value.into_format_args(super::cpu_ratio::millicpu, Abbreviation)
            )
            .as_str(),
        )
    }
}

impl<'de> serde_with::DeserializeAs<'de, Ratio> for Helper {
    fn deserialize_as<D>(deserializer: D) -> Result<Ratio, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_any(CustomVisitor)
    }
}

pub struct CustomVisitor;

impl<'de> Visitor<'de> for CustomVisitor {
    type Value = Ratio;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(
            formatter,
            "a quantity resembling '<value> <unit>' like '250 millicpu'"
        )
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        parse(v).map_err(|e| E::custom(e.to_string()))
    }
}

fn match_unit<T>(raw: &str) -> bool
where
    T: uom::si::Unit + Sized,
{
    raw == <T as uom::si::Unit>::singular()
        || raw == <T as uom::si::Unit>::abbreviation()
        || raw == <T as uom::si::Unit>::plural()
}

fn parse(quantity: &str) -> Result<Ratio, crate::utils::Error> {
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
