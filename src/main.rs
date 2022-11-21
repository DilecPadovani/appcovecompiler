use serde::de::{value, IntoDeserializer};

use regex::Regex;
use std::{collections::HashMap, iter::Map};
use std::{fs, result};
use yaml_rust::{YamlEmitter, YamlLoader};

#[derive(PartialEq, Eq, Debug)]
enum AS3_data {
    Object(HashMap<String, Box<AS3_data>>),
    String(String),
    Map {
        KeyType: Box<AS3_data>,
        ValueType: Box<AS3_data>,
    },
    Boolean(bool),
    Integer(i64),
    Decimal(),
    Float(),
    // Date(),
    // DateTime(),
    List(Vec<AS3_data>),
    None, // Set(),
}

#[derive(Debug)]
enum AS3_validator {
    Object(HashMap<String, AS3_validator>),
    String { regex: Option<String> },
    // Map {
    //     KeyType: Box<AS3_data>,
    //     ValueType: Box<AS3_data>,
    // },
    // Boolean(),
    Integer { minimum: Option<i64> },
    // Decimal(),
    // Float(),
    List(Box<AS3_validator>),
    // None, // Set(),
}

impl AS3_validator {
    fn validate(&self, data: &AS3_data) -> bool {
        // have a samme iter implementation for AS3_validator and AS3_validator
        // so that we can go trought a 'tree' of the enum and check validity step by step
        // dbg!(self, other);
        match (self, data) {
            (AS3_validator::Object(validator_inner), AS3_data::Object(data_inner)) => {
                validator_inner
                    .iter()
                    .all(|(validator_key, validator_value)| {
                        validator_value.validate(
                            data_inner
                                .get(validator_key)
                                .expect(&format!("Key {validator_key} is not in {data_inner:#?}")),
                        )
                    })
            }
            (AS3_validator::Integer { minimum }, AS3_data::Integer(number)) => {
                let Some(minimum) = minimum else {
                    return true;
                };
                number >= &minimum
            }
            (AS3_validator::String { regex }, AS3_data::String(String)) => {
                let Some(regex) = regex else {
                    return true;
                };
                let re = Regex::new(regex).unwrap();
                re.is_match(String)
            }
            (AS3_validator::List(items_type), AS3_data::List(items)) => {
                items.iter().all(|item| items_type.validate(item))
            }

            _ => false,
        }
    }
}
impl From<&serde_json::Value> for AS3_data {
    fn from(json: &serde_json::Value) -> AS3_data {
        match json {
            serde_json::Value::Object(inner) => AS3_data::Object(
                inner
                    .iter()
                    .map(|(key, value)| (key.clone(), Box::new(value.into())))
                    .collect(),
            ),
            serde_json::Value::Array(inner) => {
                AS3_data::List(inner.clone().iter().map(|e| e.into()).collect())
            }
            serde_json::Value::String(inner) => AS3_data::String(inner.clone()),
            serde_json::Value::Number(inner) => AS3_data::Integer(inner.as_i64().unwrap()),
            serde_json::Value::Bool(inner) => AS3_data::Boolean(*inner),
            serde_json::Value::Null => panic!(),
        }
    }
}

fn main() {
    let data = fs::read_to_string("test.json").expect("Unable to read file");

    let json: serde_json::Value =
        serde_json::from_str(&data).expect("JSON does not have correct format.");

    let validator = AS3_validator::Object(HashMap::from([
        (
            "age".to_owned(),
            AS3_validator::Integer { minimum: Some(20) },
        ),
        (
            "children".to_owned(),
            AS3_validator::Integer { minimum: Some(2) },
        ),
        (
            "name".to_owned(),
            AS3_validator::String {
                // The name should start with an Uppercase letter
                regex: Some("^[A-Z][a-z]".to_owned()),
            },
        ),
        (
            "vehicles".to_owned(),
            AS3_validator::List(Box::new(AS3_validator::String { regex: None })),
        ),
    ]));

    let as3_data = AS3_data::from(&json);

    println!("AS3 : {:#?}", AS3_data::from(&json));
    println!("Validator : {:?}", validator);
    println!("Validator_result : {:?}", validator.validate(&as3_data));
}
