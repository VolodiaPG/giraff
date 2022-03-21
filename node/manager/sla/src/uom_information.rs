use core::fmt;

use serde::de::Visitor;
use uom::si::f64::Information;
use uom::{fmt::DisplayStyle::Description, si::information};

use crate::utils::parse_quantity;
pub struct Helper;

impl serde_with::SerializeAs<Information> for Helper {
    fn serialize_as<S>(value: &Information, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(
            format!(
                "{:?}",
                value.into_format_args(information::megabyte, Description)
            )
            .as_str(),
        )
    }
}

impl<'de> serde_with::DeserializeAs<'de, Information> for Helper {
    fn deserialize_as<D>(deserializer: D) -> Result<Information, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_any(CustomVisitor)
    }
}

pub struct CustomVisitor;

impl<'de> Visitor<'de> for CustomVisitor {
    type Value = Information;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(
            formatter,
            "a quantity resembling '<value> <unit>' like '1.5 GiB'"
        )
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        parse_quantity(v).map_err(|e| E::custom(e.to_string()))
    }
}
