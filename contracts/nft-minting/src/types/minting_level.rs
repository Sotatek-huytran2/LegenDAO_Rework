use schemars::JsonSchema;
use serde::de::{self, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;

#[derive(Clone, Debug, PartialEq, JsonSchema)]
pub enum MintingLevel {
    Disabled,
    AdminOnly,
    Whitelist,
    Public,
}

impl ToString for MintingLevel {
    fn to_string(&self) -> String {
        let as_ref = match self {
            MintingLevel::Public => "public",
            MintingLevel::Whitelist => "whitelist",
            MintingLevel::AdminOnly => "admin_only",
            MintingLevel::Disabled => "disabled",
        };

        as_ref.to_string()
    }
}

impl Serialize for MintingLevel {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u8(self.to_u8())
    }
}

impl<'de> Deserialize<'de> for MintingLevel {
    fn deserialize<D>(deserializer: D) -> Result<MintingLevel, D::Error>
    where
        D: Deserializer<'de>,
    {
        let as_u8 = deserializer.deserialize_u8(U8Visitor)?;
        Ok(Self::from_u8(as_u8))
    }
}

impl MintingLevel {
    fn to_u8(&self) -> u8 {
        match &self {
            MintingLevel::Public => 3u8,
            MintingLevel::Whitelist => 2u8,
            MintingLevel::AdminOnly => 1u8,
            MintingLevel::Disabled => 0u8,
        }
    }

    fn from_u8(from: u8) -> Self {
        match from {
            3 => MintingLevel::Public,
            2 => MintingLevel::Whitelist,
            1 => MintingLevel::AdminOnly,
            0 => MintingLevel::Disabled,
            _ => MintingLevel::Disabled,
        }
    }
}

struct U8Visitor;

impl<'de> Visitor<'de> for U8Visitor {
    type Value = u8;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("an integer between 0 and 2^32")
    }

    fn visit_u8<E>(self, value: u8) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(value)
    }

    fn visit_u32<E>(self, value: u32) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        use std::u8;
        if value >= u32::from(u8::MIN) && value <= u32::from(u8::MAX) {
            Ok(value as u8)
        } else {
            Err(E::custom(format!("u8 out of range: {}", value)))
        }
    }

    fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        use std::u8;
        if value >= u64::from(u8::MIN) && value <= u64::from(u8::MAX) {
            Ok(value as u8)
        } else {
            Err(E::custom(format!("u8 out of range: {}", value)))
        }
    }
}
