use core::fmt;

use serde::de::Visitor;
use uom::si::f64::Time;
use uom::{fmt::DisplayStyle::Abbreviation, si::time};

use super::parse_quantity;
pub struct Helper;

impl serde_with::SerializeAs<Time> for Helper {
    fn serialize_as<S>(value: &Time, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(
            format!("{:?}", value.into_format_args(time::second, Abbreviation)).as_str(),
        )
    }
}

impl<'de> serde_with::DeserializeAs<'de, Time> for Helper {
    fn deserialize_as<D>(deserializer: D) -> Result<Time, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_any(CustomVisitor)
    }
}

pub struct CustomVisitor;

impl<'de> Visitor<'de> for CustomVisitor {
    type Value = Time;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(
            formatter,
            "a quantity resembling '<value> <unit>' like '1.5 seconds'"
        )
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        parse_quantity(v).map_err(|e| E::custom(e.to_string()))
    }
}
