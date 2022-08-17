use std::str::FromStr;

use serde::de::Error;
use serde::Deserializer;

use crate::big_decimal_json::ser::BigDecimalJsonWrap;
use crate::decimal::BigDecimal;
use crate::decimal::ToPrimitive;

#[derive(Debug, Clone)]
pub struct BigDecimalJson(BigDecimal);

pub mod ser {
    #[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
    pub struct BigDecimalJsonWrap {
        pub decimal: String,
        pub rounded: Option<f64>,
    }
}

impl serde::Serialize for BigDecimalJson {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer, {
        let exact_value = self.0.to_string();
        let rounded_value = self.0.to_f64();
        let json_struct = BigDecimalJsonWrap { decimal: exact_value, rounded: rounded_value };
        serializer.serialize_some(&json_struct)
    }
}

impl<'de> serde::Deserialize<'de> for BigDecimalJson {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
    {
        let value = serde_json::Value::deserialize(deserializer)?;
        let result = match &value {
            serde_json::Value::Object(object) => {
                if let Some(decimal) = object.get("decimal").and_then(|value| value.as_str()) {
                    let bigdecimal_result = BigDecimal::from_str(decimal).map_err(|err| D::Error::custom(err));
                    // let bigdecimal_result = BigDecimal::from_str(decimal).map_err(D::custom);
                    bigdecimal_result.map(|bigdecimal| BigDecimalJson(bigdecimal))
                } else {
                    Err(Error::custom(format_args!(
                        "`decimal` is not present` {}, expected {}",
                        value, "&str"
                    )))
                }
            }
            _ => Err(Error::custom(format_args!(
                "invalid type for value {}, expected {}",
                value, "array"
            ))),
        };

        result
    }
}

impl From<BigDecimal> for BigDecimalJson {
    fn from(value: BigDecimal) -> Self {
        BigDecimalJson(value)
    }
}

impl From<&BigDecimal> for BigDecimalJson {
    fn from(value: &BigDecimal) -> Self {
        BigDecimalJson(value.clone())
    }
}

impl Into<BigDecimal> for BigDecimalJson {
    fn into(self) -> BigDecimal {
        self.0
    }
}

// fn rr (){
//     let decimal = BigDecimal::zero();
//     let (bigint, exponent) = decimal.into_bigint_and_exponent();
// }
//
// pub fn deserialize_json_string<'de, D>(deserializer: D) -> Result<base_crate::decimal::BigDecimal, D::Error>
//     where
//         D: de::Deserializer<'de>,
// {
//     // define a visitor that deserializes
//     // `ActualData` encoded as json within a string
//     struct JsonStringVisitor;
//
//     impl<'de> de::Visitor<'de> for JsonStringVisitor {
//         type Value = base_crate::decimal::BigDecimal;
//
//         fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
//             formatter.write_str("a string containing json data")
//         }
//
//         fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
//             where
//                 E: de::Error, {
//             // unfortunately we lose some typed information
//             // from errors deserializing the json string
//             base_crate::decimal::BigDecimal::from_str(v).map_err(E::custom)
//         }
//     }
//
//     deserializer.deserialize_any(JsonStringVisitor)
// }
//
// {
// let x1 = x.to_f64().unwrap();
// s.serialize_f64(x1)
// }


