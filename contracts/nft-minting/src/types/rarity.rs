use schemars::JsonSchema;
use serde::de::{self, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;

#[derive(Clone, Debug, PartialEq, JsonSchema)]
pub enum Rarity {
    Legendary,
    Epic,
    Rare,
    Uncommon,
    Common,
}

impl Serialize for Rarity {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u8(self.to_u8())
    }
}

impl<'de> Deserialize<'de> for Rarity {
    fn deserialize<D>(deserializer: D) -> Result<Rarity, D::Error>
    where
        D: Deserializer<'de>,
    {
        let as_u8 = deserializer.deserialize_u8(U8Visitor)?;
        Ok(Self::from_u8(as_u8))
    }
}

impl Rarity {
    fn to_u8(&self) -> u8 {
        match &self {
            Rarity::Legendary => 4u8,
            Rarity::Epic => 3u8,
            Rarity::Rare => 2u8,
            Rarity::Uncommon => 1u8,
            Rarity::Common => 0u8,
        }
    }

    fn from_u8(from: u8) -> Self {
        match from {
            4 => Rarity::Legendary,
            3 => Rarity::Epic,
            2 => Rarity::Rare,
            1 => Rarity::Uncommon,
            0 => Rarity::Common,
            _ => Rarity::Common,
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
