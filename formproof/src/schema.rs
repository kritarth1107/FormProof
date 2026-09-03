use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use thiserror::Error;

pub const MAX_PROPERTIES: usize = 8;
pub const MAX_ENUM_VARIANTS: usize = 8;
pub const MAX_STRING_LENGTH: usize = 64;

#[derive(Error, Debug)]
pub enum SchemaError {
    #[error("schema must be an object type")]
    NotAnObject,
    #[error("too many properties: {0} (max {MAX_PROPERTIES})")]
    TooManyProperties(usize),
    #[error("too many enum variants for '{0}': {1} (max {MAX_ENUM_VARIANTS})")]
    TooManyEnumVariants(String, usize),
    #[error("string maxLength too large for '{0}': {1} (max {MAX_STRING_LENGTH})")]
    StringTooLong(String, usize),
    #[error("unsupported type for property '{0}': {1}")]
    UnsupportedType(String, String),
    #[error("required property '{0}' not defined in properties")]
    RequiredNotDefined(String),
    #[error("invalid JSON: {0}")]
    InvalidJson(#[from] serde_json::Error),
    #[error("missing 'type' field for property '{0}'")]
    MissingType(String),
    #[error("enum variants must be strings for property '{0}'")]
    EnumNotStrings(String),
    #[error("bytes32 must have exactly 32 bytes, got {0}")]
    InvalidBytes32Length(usize),
    #[error("minimum must be <= maximum for property '{0}'")]
    MinGreaterThanMax(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PropertyType {
    U64 {
        minimum: Option<u64>,
        maximum: Option<u64>,
    },
    Enum {
        variants: Vec<String>,
    },
    Bytes32,
    String {
        max_length: usize,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Property {
    pub name: String,
    pub prop_type: PropertyType,
    pub required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FormProofSchema {
    pub properties: Vec<Property>,
}

impl FormProofSchema {
    pub fn from_json(json: &str) -> Result<Self, SchemaError> {
        let value: serde_json::Value = serde_json::from_str(json)?;
        Self::from_value(&value)
    }

    pub fn from_value(value: &serde_json::Value) -> Result<Self, SchemaError> {
        let obj = value.as_object().ok_or(SchemaError::NotAnObject)?;

        let schema_type = obj.get("type").and_then(|t| t.as_str());
        if schema_type != Some("object") {
            return Err(SchemaError::NotAnObject);
        }

        let props = obj
            .get("properties")
            .and_then(|p| p.as_object())
            .ok_or(SchemaError::NotAnObject)?;

        if props.len() > MAX_PROPERTIES {
            return Err(SchemaError::TooManyProperties(props.len()));
        }

        let required: Vec<String> = obj
            .get("required")
            .and_then(|r| r.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default();

        let mut properties = Vec::new();

        for (name, prop_value) in props {
            let prop_obj = prop_value.as_object().ok_or_else(|| {
                SchemaError::UnsupportedType(name.clone(), "not an object".to_string())
            })?;

            let prop_type = Self::parse_property_type(name, prop_obj)?;
            let is_required = required.contains(name);

            properties.push(Property {
                name: name.clone(),
                prop_type,
                required: is_required,
            });
        }

        for req in &required {
            if !properties.iter().any(|p| &p.name == req) {
                return Err(SchemaError::RequiredNotDefined(req.clone()));
            }
        }

        properties.sort_by(|a, b| a.name.cmp(&b.name));

        Ok(FormProofSchema { properties })
    }

    fn parse_property_type(
        name: &str,
        obj: &serde_json::Map<String, serde_json::Value>,
    ) -> Result<PropertyType, SchemaError> {
        if let Some(enum_values) = obj.get("enum") {
            let variants = enum_values
                .as_array()
                .ok_or_else(|| SchemaError::EnumNotStrings(name.to_string()))?;

            if variants.len() > MAX_ENUM_VARIANTS {
                return Err(SchemaError::TooManyEnumVariants(
                    name.to_string(),
                    variants.len(),
                ));
            }

            let string_variants: Result<Vec<String>, _> = variants
                .iter()
                .map(|v| {
                    v.as_str()
                        .map(String::from)
                        .ok_or_else(|| SchemaError::EnumNotStrings(name.to_string()))
                })
                .collect();

            return Ok(PropertyType::Enum {
                variants: string_variants?,
            });
        }

        let type_str = obj
            .get("type")
            .and_then(|t| t.as_str())
            .ok_or_else(|| SchemaError::MissingType(name.to_string()))?;

        match type_str {
            "integer" => {
                let minimum = obj.get("minimum").and_then(|v| v.as_u64());
                let maximum = obj.get("maximum").and_then(|v| v.as_u64());

                if let (Some(min), Some(max)) = (minimum, maximum) {
                    if min > max {
                        return Err(SchemaError::MinGreaterThanMax(name.to_string()));
                    }
                }

                Ok(PropertyType::U64 { minimum, maximum })
            }
            "string" => {
                if obj.get("format").and_then(|f| f.as_str()) == Some("bytes32") {
                    return Ok(PropertyType::Bytes32);
                }

                let max_length = obj
                    .get("maxLength")
                    .and_then(|v| v.as_u64())
                    .map(|v| v as usize)
                    .unwrap_or(MAX_STRING_LENGTH);

                if max_length > MAX_STRING_LENGTH {
                    return Err(SchemaError::StringTooLong(name.to_string(), max_length));
                }

                Ok(PropertyType::String { max_length })
            }
            other => Err(SchemaError::UnsupportedType(
                name.to_string(),
                other.to_string(),
            )),
        }
    }

    pub fn to_json(&self) -> String {
        let mut props = BTreeMap::new();

        for prop in &self.properties {
            let mut prop_obj = serde_json::Map::new();

            match &prop.prop_type {
                PropertyType::U64 { minimum, maximum } => {
                    prop_obj.insert("type".to_string(), serde_json::json!("integer"));
                    if let Some(min) = minimum {
                        prop_obj.insert("minimum".to_string(), serde_json::json!(min));
                    }
                    if let Some(max) = maximum {
                        prop_obj.insert("maximum".to_string(), serde_json::json!(max));
                    }
                }
                PropertyType::Enum { variants } => {
                    prop_obj.insert("enum".to_string(), serde_json::json!(variants));
                }
                PropertyType::Bytes32 => {
                    prop_obj.insert("type".to_string(), serde_json::json!("string"));
                    prop_obj.insert("format".to_string(), serde_json::json!("bytes32"));
                }
                PropertyType::String { max_length } => {
                    prop_obj.insert("type".to_string(), serde_json::json!("string"));
                    prop_obj.insert("maxLength".to_string(), serde_json::json!(max_length));
                }
            }

            props.insert(prop.name.clone(), serde_json::Value::Object(prop_obj));
        }

        let required: Vec<&str> = self
            .properties
            .iter()
            .filter(|p| p.required)
            .map(|p| p.name.as_str())
            .collect();

        let schema = serde_json::json!({
            "type": "object",
            "properties": props,
            "required": required
        });

        serde_json::to_string_pretty(&schema).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_schema() {
        let schema_json = r#"{
            "type": "object",
            "properties": {
                "amount": {
                    "type": "integer",
                    "minimum": 0,
                    "maximum": 100
                },
                "currency": {
                    "enum": ["USD", "EUR", "GBP"]
                }
            },
            "required": ["amount", "currency"]
        }"#;

        let schema = FormProofSchema::from_json(schema_json).unwrap();
        assert_eq!(schema.properties.len(), 2);

        let amount = schema
            .properties
            .iter()
            .find(|p| p.name == "amount")
            .unwrap();
        assert!(amount.required);
        match &amount.prop_type {
            PropertyType::U64 { minimum, maximum } => {
                assert_eq!(*minimum, Some(0));
                assert_eq!(*maximum, Some(100));
            }
            _ => panic!("Expected U64 type"),
        }

        let currency = schema
            .properties
            .iter()
            .find(|p| p.name == "currency")
            .unwrap();
        assert!(currency.required);
        match &currency.prop_type {
            PropertyType::Enum { variants } => {
                assert_eq!(variants, &vec!["USD", "EUR", "GBP"]);
            }
            _ => panic!("Expected Enum type"),
        }
    }

    #[test]
    fn test_parse_bytes32() {
        let schema_json = r#"{
            "type": "object",
            "properties": {
                "hash": {
                    "type": "string",
                    "format": "bytes32"
                }
            },
            "required": ["hash"]
        }"#;

        let schema = FormProofSchema::from_json(schema_json).unwrap();
        let hash = &schema.properties[0];
        assert_eq!(hash.prop_type, PropertyType::Bytes32);
    }

    #[test]
    fn test_parse_string_with_maxlength() {
        let schema_json = r#"{
            "type": "object",
            "properties": {
                "name": {
                    "type": "string",
                    "maxLength": 32
                }
            },
            "required": []
        }"#;

        let schema = FormProofSchema::from_json(schema_json).unwrap();
        let name = &schema.properties[0];
        match &name.prop_type {
            PropertyType::String { max_length } => {
                assert_eq!(*max_length, 32);
            }
            _ => panic!("Expected String type"),
        }
    }

    #[test]
    fn test_too_many_properties() {
        let mut props = String::new();
        for i in 0..10 {
            if i > 0 {
                props.push_str(", ");
            }
            props.push_str(&format!(r#""prop{i}": {{"type": "integer"}}"#));
        }

        let schema_json = format!(
            r#"{{
            "type": "object",
            "properties": {{ {props} }},
            "required": []
        }}"#
        );

        let result = FormProofSchema::from_json(&schema_json);
        assert!(matches!(result, Err(SchemaError::TooManyProperties(_))));
    }

    #[test]
    fn test_too_many_enum_variants() {
        let schema_json = r#"{
            "type": "object",
            "properties": {
                "status": {
                    "enum": ["a", "b", "c", "d", "e", "f", "g", "h", "i", "j"]
                }
            },
            "required": []
        }"#;

        let result = FormProofSchema::from_json(schema_json);
        assert!(matches!(
            result,
            Err(SchemaError::TooManyEnumVariants(_, _))
        ));
    }

    #[test]
    fn test_string_too_long() {
        let schema_json = r#"{
            "type": "object",
            "properties": {
                "name": {
                    "type": "string",
                    "maxLength": 100
                }
            },
            "required": []
        }"#;

        let result = FormProofSchema::from_json(schema_json);
        assert!(matches!(result, Err(SchemaError::StringTooLong(_, _))));
    }

    #[test]
    fn test_min_greater_than_max() {
        let schema_json = r#"{
            "type": "object",
            "properties": {
                "amount": {
                    "type": "integer",
                    "minimum": 100,
                    "maximum": 50
                }
            },
            "required": []
        }"#;

        let result = FormProofSchema::from_json(schema_json);
        assert!(matches!(result, Err(SchemaError::MinGreaterThanMax(_))));
    }

    #[test]
    fn test_roundtrip() {
        let schema_json = r#"{
            "type": "object",
            "properties": {
                "amount": {
                    "type": "integer",
                    "minimum": 0,
                    "maximum": 50
                },
                "currency": {
                    "enum": ["USD", "EUR"]
                }
            },
            "required": ["amount", "currency"]
        }"#;

        let schema = FormProofSchema::from_json(schema_json).unwrap();
        let regenerated = schema.to_json();
        let schema2 = FormProofSchema::from_json(&regenerated).unwrap();

        assert_eq!(schema, schema2);
    }
}
