use core::str::FromStr;
use lazy_static::lazy_static;
use serde::{de::Visitor, Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

/// encapsulate the UUIDs in custom struct to let the compiler differentiate them
macro_rules! impl_id_encapsulation {
    ($name: ident) => {
        #[derive(Debug, Clone, PartialEq, Eq, Hash)]
        pub struct $name {
            id: Uuid,
        }

        impl Default for $name {
            fn default() -> Self {
                $name {
                    id: Uuid::from_str("10000000-0000-0000-0000-000000000000").unwrap(),
                }
            }
        }

        impl From<Uuid> for $name {
            fn from(id: Uuid) -> Self {
                $name { id }
            }
        }

        impl FromStr for $name {
            type Err = uuid::Error;
            fn from_str(s: &str) -> Result<Self, Self::Err> {
                Uuid::from_str(s).map(|id| id.into())
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "{}", self.id)
            }
        }

        impl Serialize for $name {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                serializer.serialize_str(&self.id.to_string())
            }
        }

        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct MyVisitor;

                impl<'de> Visitor<'de> for MyVisitor {
                    type Value = $name;

                    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                        formatter.write_str("a $name, i.e., a UUIDv4")
                    }

                    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
                    where
                        E: serde::de::Error,
                    {
                        Ok($name {
                            id: Uuid::parse_str(value).unwrap(),
                        })
                    }
                }

                deserializer.deserialize_str(MyVisitor)
            }
        }
    };
}

impl_id_encapsulation!(NodeId);
impl_id_encapsulation!(BidId);

#[derive(Debug, Clone)]
pub enum Reserved {
    MarketPing,
}

impl From<BidId> for Option<Reserved> {
    fn from(id: BidId) -> Option<Reserved> {
        lazy_static! {
            static ref MARKET_PING: BidId = Uuid::from_str("00000000-0000-0000-0000-000000000001")
                .unwrap()
                .into();
        }

        if id.eq(&MARKET_PING) {
            Some(Reserved::MarketPing)
        } else {
            None
        }
    }
}
