use std::str::FromStr;

use lazy_regex::regex;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Unable to parse the quantity: {0}")]
    QuantityParsing(String),
}

pub fn parse_quantity<T>(quantity: &str) -> Result<T, Error>
where
    T: FromStr,
{
    let re = regex!(r"^([0-9\.eE\-+]+)\s*(\w+)$");

    let captures =
        re.captures(quantity).ok_or_else(|| Error::QuantityParsing(quantity.to_string()))?;
    let measure = captures.get(1).ok_or_else(|| Error::QuantityParsing(quantity.to_string()))?;
    let unit = captures.get(2).ok_or_else(|| Error::QuantityParsing(quantity.to_string()))?;

    let qty = format!("{} {}", measure.as_str(), unit.as_str())
        .parse::<T>()
        .map_err(|_| Error::QuantityParsing(quantity.to_string()))?;

    Ok(qty)
}

macro_rules! impl_json_schema {
    ($type:ty => $instance_type:ident, $format:literal) => {
        impl_json_schema!($type => $instance_type, $format, Some($format.to_owned()));
    };
    ($type:ty => $instance_type:ident, $name:expr, $format:expr) => {
        use schemars::gen::SchemaGenerator;
        use schemars::schema::*;

        pub fn schema_function(_: &mut SchemaGenerator) -> Schema {
            SchemaObject {
                instance_type: Some(InstanceType::$instance_type.into()),
                format: $format,
                ..Default::default()
            }
            .into()
        }
    };
}

macro_rules! impl_serialize_as {
    ($type:ty, $unit:ty, $format:expr) => {
        impl_serialize_as!($type, $unit, $format, super::parse_quantity);
    };
    ($type:ty, $unit:ty, $format:expr, $parser:expr) => {
        use core::fmt;
        use serde::{de::Visitor, Deserializer, Serializer};

        use uom::fmt::DisplayStyle::Abbreviation;

        pub struct Helper;

        impl serde_with::SerializeAs<$type> for Helper {
            fn serialize_as<S>(value: &$type, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                serializer.serialize_str(
                    format!("{:?}", value.into_format_args($format, Abbreviation)).as_str(),
                )
            }
        }

        impl<'de> serde_with::DeserializeAs<'de, $type> for Helper {
            fn deserialize_as<D>(deserializer: D) -> Result<$type, D::Error>
            where
                D: Deserializer<'de>,
            {
                deserializer.deserialize_any(CustomVisitor)
            }
        }

        pub struct CustomVisitor;

        impl<'de> Visitor<'de> for CustomVisitor {
            type Value = $type;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                write!(
                    formatter,
                    "a quantity resembling '<value> <unit>' like '{:?}'",
                    <$type>::new::<$unit>(1.5).into_format_args($format, Abbreviation)
                )
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                $parser(v).map_err(|e| E::custom(e.to_string()))
            }
        }
    };
}

pub mod information {
    use uom::si::{f64::Information, information::megabyte};

    impl_serialize_as!(Information, megabyte, megabyte);
    impl_json_schema!(Information => String, "<value> <SI unit>");
}

pub mod time {
    use uom::si::{f64::Time, time::second};

    impl_serialize_as!(Time, second, second);
    impl_json_schema!(Time => String, "<value> <SI unit>");
}

pub mod ratio {
    use uom::si::f64::Ratio;

    use crate::helper::uom::{cpu_ratio::millicpu, ratio_helper::parse};

    impl_serialize_as!(Ratio, millicpu, millicpu, parse);
    impl_json_schema!(Ratio => String, "<value> <SI unit>");
}

pub mod cpu_ratio;
mod ratio_helper;
