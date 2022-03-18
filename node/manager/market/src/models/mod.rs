use core::fmt;

use chrono::{DateTime, Utc};
use serde::de::Visitor;
use uuid::Uuid;
pub type ClientId = Uuid;
pub type BidId = Uuid;
pub type NodeId = Uuid;

struct DateTimeHelper;

impl serde_with::SerializeAs<DateTime<Utc>> for DateTimeHelper {
    fn serialize_as<S>(value: &DateTime<Utc>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&value.to_rfc3339())
    }
}

impl<'de> serde_with::DeserializeAs<'de, DateTime<Utc>> for DateTimeHelper {
    fn deserialize_as<D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_any(CustomVisitor)
    }
}

struct CustomVisitor;

impl<'de> Visitor<'de> for CustomVisitor {
    type Value = DateTime<Utc>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(
            formatter,
            "a date and time following the RFC3339 / ISO 8601 format"
        )
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(DateTime::parse_from_rfc3339(v)
            .map_err(|e| E::custom(e.to_string()))?
            .with_timezone(&Utc))
    }
}

mod outputs;
pub use outputs::{Bid, Satisfiable};

mod database;
pub use database::{BidRecord, NodeRecord};

mod inputs;
pub use inputs::{PatchNode, RegisterNode};
