use core::str::FromStr;
use std::fmt;

use lazy_static::lazy_static;
use schemars::gen::SchemaGenerator;
use schemars::schema::*;
use schemars::JsonSchema;
use serde::de::Visitor;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// encapsulate the UUIDs in custom struct to let the compiler differentiate
/// them
macro_rules! impl_id_encapsulation {
    ($name:ident) => {
        #[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd)]
        pub struct $name {
            id: Uuid,
        }

        impl Default for $name {
            fn default() -> Self {
                $name {
                    id: Uuid::from_str("10000000-0000-0000-0000-000000000000")
                        .unwrap(),
                }
            }
        }

        impl From<Uuid> for $name {
            #[inline(always)]
            fn from(id: Uuid) -> Self { $name { id } }
        }

        impl FromStr for $name {
            type Err = uuid::Error;

            #[inline(always)]
            fn from_str(s: &str) -> Result<Self, Self::Err> {
                Uuid::from_str(s).map(|id| id.into())
            }
        }

        impl fmt::Display for $name {
            #[inline(always)]
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "{}", self.id)
            }
        }

        impl Serialize for $name {
            #[inline(always)]
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

                    fn expecting(
                        &self,
                        formatter: &mut fmt::Formatter,
                    ) -> fmt::Result {
                        formatter.write_str(
                            format!("a {}, i.e., a UUIDv4", stringify!($name))
                                .as_str(),
                        )
                    }

                    fn visit_str<E>(
                        self,
                        value: &str,
                    ) -> Result<Self::Value, E>
                    where
                        E: serde::de::Error,
                    {
                        Ok($name {
                            id: Uuid::parse_str(value).map_err(E::custom)?,
                        })
                    }
                }

                deserializer.deserialize_str(MyVisitor)
            }
        }

        impl JsonSchema for $name {
            fn schema_name() -> String { String::from(stringify!($name)) }

            fn json_schema(_: &mut SchemaGenerator) -> Schema {
                SchemaObject {
                    instance_type: Some(InstanceType::String.into()),
                    format: Some("uuid".to_string()),
                    ..Default::default()
                }
                .into()
            }
        }
    };
}

macro_rules! impl_port_encapsulation {
    ($name:ident) => {
        #[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd)]
        pub struct $name {
            port: u16,
        }

        impl From<u16> for $name {
            #[inline(always)]
            fn from(port: u16) -> Self { $name { port } }
        }

        impl fmt::Display for $name {
            #[inline(always)]
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "{}", self.port)
            }
        }

        impl From<$name> for u16 {
            #[inline(always)]
            fn from(port: $name) -> u16 { port.port }
        }

        impl Serialize for $name {
            #[inline(always)]
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                serializer.serialize_str(self.port.to_string().as_str())
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

                    fn expecting(
                        &self,
                        formatter: &mut fmt::Formatter,
                    ) -> fmt::Result {
                        formatter.write_str(
                            format!(
                                "a {}, i.e., a u16 as a string (\\\"8080\\\")",
                                stringify!($name)
                            )
                            .as_str(),
                        )
                    }

                    fn visit_str<E>(
                        self,
                        value: &str,
                    ) -> Result<Self::Value, E>
                    where
                        E: serde::de::Error,
                    {
                        Ok($name { port: value.parse().map_err(E::custom)? })
                    }
                }

                deserializer.deserialize_str(MyVisitor)
            }
        }

        impl JsonSchema for $name {
            fn schema_name() -> String { String::from(stringify!($name)) }

            fn json_schema(_: &mut SchemaGenerator) -> Schema {
                SchemaObject {
                    instance_type: Some(InstanceType::Number.into()),
                    format: Some("port".to_string()),
                    ..Default::default()
                }
                .into()
            }
        }
    };
}

impl_id_encapsulation!(NodeId);
impl_id_encapsulation!(BidId);

// Port for fog node Rpc
impl_port_encapsulation!(FogNodeRPCPort);
// Port for fog node HTTP
impl_port_encapsulation!(FogNodeHTTPPort);
// Port for Market HTTP port
impl_port_encapsulation!(MarketHTTPPort);

#[derive(Debug, Clone)]
pub enum Reserved {
    MarketPing,
}

lazy_static! {
    static ref MARKET_PING: BidId = BidId::from(
        Uuid::from_str("00000000-0000-0000-0000-000000000001").unwrap()
    );
}

impl From<BidId> for Option<Reserved> {
    fn from(id: BidId) -> Option<Reserved> {
        if id.eq(&MARKET_PING) {
            Some(Reserved::MarketPing)
        } else {
            None
        }
    }
}

impl From<Reserved> for BidId {
    fn from(reserved: Reserved) -> BidId {
        match reserved {
            Reserved::MarketPing => (*MARKET_PING).clone(),
        }
    }
}

pub mod domain;
pub mod dto;
pub mod view;
