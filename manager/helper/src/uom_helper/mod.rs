use lazy_regex::regex;
use num_rational::Ratio;
use num_traits::FromPrimitive as _;
use std::str::FromStr;

pub mod cpu_ratio;
mod ratio_helper;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Unable to parse the quantity: {0}")]
    QuantityParsing(String),
    #[error("Failed to convert the quantity {0} to {1}")]
    FailedConversion(String, String),
}

pub fn parse_quantity<T>(quantity: &str) -> Result<T, Error>
where
    T: FromStr,
{
    let re = regex!(r"^([0-9\.eE\-+]+)\s*(\w+)$");

    let captures = re
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

macro_rules! impl_serialize_as {
    ($type:ty, $unit:ty, $format:expr) => {
        impl_serialize_as!($type, $unit, $format, super::parse_quantity);
    };

    ($type:ty, $unit:ty, $format:expr, $parser:expr ) => {
        use uom::fmt::DisplayStyle::Abbreviation;
        pub fn serialize_quantity(value: &$type) -> String {
            format!("{:?}", value.into_format_args($format, Abbreviation))
        }

        impl_serialize_as!($type, $unit, $format, $parser, serialize_quantity);
    };
    ($type:ty, $unit:ty, $format:expr, $parser:expr, $serializer:expr) => {
        use core::fmt;
        use serde::de::Visitor;
        use serde::{Deserializer, Serializer};

        pub struct Helper;

        impl serde_with::SerializeAs<$type> for Helper {
            fn serialize_as<S>(
                value: &$type,
                serializer: S,
            ) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                serializer.serialize_str($serializer(value).as_str())
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

            fn expecting(
                &self,
                formatter: &mut fmt::Formatter,
            ) -> fmt::Result {
                write!(formatter, "a quantity resembling '<value> <unit>'")
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
    use super::*;
    use std::str::FromStr;
    use uom::si::information::megabyte;
    use uom::si::rational64::Information;

    pub fn parse_quantity<T>(quantity: &str) -> Result<T, Error>
    where
        T: FromStr,
    {
        let re = regex!(r"^([0-9\.eE\-+]+)\s*(\w+)$");

        let captures = re
            .captures(quantity)
            .ok_or_else(|| Error::QuantityParsing(quantity.to_string()))?;
        let measure = captures
            .get(1)
            .ok_or_else(|| Error::QuantityParsing(quantity.to_string()))?
            .as_str()
            .parse::<f64>()
            .map_err(|_| Error::QuantityParsing(quantity.to_string()))?;
        let measure: Ratio<i64> = num_rational::Ratio::from_f64(measure)
            .ok_or_else(|| {
                Error::FailedConversion(
                    quantity.to_string(),
                    "rational number".to_string(),
                )
            })?;
        let unit = captures
            .get(2)
            .ok_or_else(|| Error::QuantityParsing(quantity.to_string()))?
            .as_str();

        let qty = format!("{} {}", measure, unit)
            .parse::<T>()
            .map_err(|_| Error::QuantityParsing(quantity.to_string()))?;

        Ok(qty)
    }

    fn serialize_quantity(value: &Information) -> String {
        format!("{:?} MB", value.get::<megabyte>().numer())
    }

    impl_serialize_as!(
        Information,
        megabyte,
        megabyte,
        parse_quantity,
        serialize_quantity
    );

    #[cfg(test)]
    mod tests {
        use super::*;
        use anyhow::Result;
        use yare::parameterized;

        #[parameterized(
            byte = {"1000 kilobytes", 1},
            megabyte = {"1 MB", 1},
            gigabyte = {"1 GB", 1_000},
            gigabyte_floating = {"0.5 gigabytes", 500},
            megabyte_reel = {"7680.0 MB", 7680}
        )]
        fn test_serialize(ss: &str, qty: i64) -> Result<()> {
            assert_eq!(
                parse_quantity::<Information>(ss)?,
                Information::new::<megabyte>(num_rational::Ratio::new(qty, 1))
            );
            Ok(())
        }
        #[test]
        fn test_deserialize() -> Result<()> {
            assert_eq!(
                serialize_quantity(&Information::new::<megabyte>(
                    num_rational::Ratio::new(1000, 1)
                )),
                "1000 MB"
            );
            Ok(())
        }
    }
}

pub mod time {
    use uom::si::f64::Time;
    use uom::si::time::second;

    impl_serialize_as!(Time, second, second);
}

pub mod cpu {
    use crate::uom_helper::cpu_ratio::millicpu;
    use crate::uom_helper::ratio_helper::parse;
    use uom::num_traits::ToPrimitive;
    use uom::si::rational64::Ratio;

    fn serialize_quantity(value: &Ratio) -> String {
        format!("{:?} m", value.get::<millicpu>().to_f64().unwrap())
    }

    impl_serialize_as!(Ratio, millicpu, millicpu, parse, serialize_quantity);
    #[cfg(test)]
    mod tests {
        use super::*;
        use crate::uom_helper::cpu_ratio::cpu;
        use crate::uom_helper::ratio_helper::parse;
        use anyhow::Result;
        use yare::parameterized;

        #[parameterized(
            millicpu = {"1000 millicpu", 1000},
            m = {"1000 m", 1000},
            cpu = {"1 cpu", 1000},
            cpu_float = {"0.31 cpu", 310}
        )]
        fn test_serialize(ss: &str, qty: i64) -> Result<()> {
            assert_eq!(
                parse(ss)?,
                Ratio::new::<millicpu>(num_rational::Ratio::new(qty, 1))
            );
            Ok(())
        }
        #[test]
        fn test_deserialize() -> Result<()> {
            assert_eq!(
                serialize_quantity(&Ratio::new::<millicpu>(
                    num_rational::Ratio::new(1000, 1)
                )),
                "1000.0 m"
            );
            Ok(())
        }

        #[test]
        fn test_millicpu_cpu() -> Result<()> {
            assert_eq!(
                parse("2000.0 millicpu")?,
                Ratio::new::<cpu>(num_rational::Ratio::new(2, 1))
            );
            Ok(())
        }
    }
}

pub mod ratio {
    use uom::si::f64::Ratio;
    use uom::si::ratio::basis_point;

    impl_serialize_as!(Ratio, basis_point, basis_point);
}
