use regex::Regex;
use std::{collections::HashMap, fs};

use thiserror::Error;
#[derive(Debug)]
enum AS3Data {
    Object(HashMap<String, Box<AS3Data>>),
    String(String),
    Map {
        KeyType: Box<AS3Data>,
        ValueType: Box<AS3Data>,
    },
    Boolean(bool),
    Integer(i64),
    Decimal(f64),
    List(Vec<AS3Data>),
}

#[derive(Debug)]
enum AS3Validator {
    Object(HashMap<String, AS3Validator>),
    String { regex: Option<String> },
    Integer { minimum: Option<i64> },
    Decimal { minimum: Option<i64> },
    List(Box<AS3Validator>),
}

impl AS3Validator {
    fn validate(&self, data: &AS3Data) -> bool {
        match (self, data) {
            (AS3Validator::Object(validator_inner), AS3Data::Object(data_inner)) => {
                validator_inner
                    .iter()
                    .all(|(validator_key, validator_value)| {
                        validator_value.validate(
                            data_inner
                                .get(validator_key)
                                // TODO! implement error
                                .expect(&format!("Key {validator_key} is not in {data_inner:#?}")),
                        )
                    })
            }
            (AS3Validator::Integer { minimum }, AS3Data::Integer(number)) => {
                let Some(minimum) = minimum else {
                    return true;
                };
                number >= &minimum
            }
            (AS3Validator::String { regex }, AS3Data::String(string)) => {
                let Some(regex) = regex else {
                    return true;
                };
                let re = Regex::new(regex).unwrap();

                if !re.is_match(string) {
                    // TODO! implement error
                    println!(" `{string}` does not follow the specified regex");
                    return false;
                }
                true
            }
            (AS3Validator::List(items_type), AS3Data::List(items)) => {
                items.iter().all(|item| items_type.validate(item))
            }

            _ => {
                // TODO! implement error
                println!("Excepted: {self:?} got {data:?}");
                return false;
            }
        }
    }
}
impl From<&serde_json::Value> for AS3Data {
    fn from(json: &serde_json::Value) -> AS3Data {
        match json {
            serde_json::Value::Object(inner) => AS3Data::Object(
                inner
                    .iter()
                    .map(|(key, value)| (key.clone(), Box::new(value.into())))
                    .collect(),
            ),
            serde_json::Value::Array(inner) => {
                AS3Data::List(inner.clone().iter().map(|e| e.into()).collect())
            }
            serde_json::Value::String(inner) => AS3Data::String(inner.clone()),
            serde_json::Value::Number(inner) => {
                if let Some(number) = inner.as_i64() {
                    AS3Data::Integer(number)
                } else {
                    AS3Data::Decimal(inner.as_f64().unwrap())
                }
            }
            serde_json::Value::Bool(inner) => AS3Data::Boolean(*inner),
            serde_json::Value::Null => panic!(),
        }
    }
}

#[derive(Error, Debug)]
enum AS3ValidationError {
    #[error("Mismatched types. Expected `{:?}` got `{:?}` . " , .Expected , .Got)]
    TypeError {
        Expected: AS3Validator,
        Got: AS3Data,
    },
}

fn main() {
    let data = fs::read_to_string("test.json").expect("Unable to read file");

    let json: serde_json::Value =
        serde_json::from_str(&data).expect("JSON does not have correct format.");

    let validator = AS3Validator::Object(HashMap::from([
        (
            "age".to_owned(),
            AS3Validator::Integer { minimum: Some(20) },
        ),
        (
            "children".to_owned(),
            AS3Validator::Integer { minimum: Some(2) },
        ),
        (
            "name".to_owned(),
            AS3Validator::String {
                // The name should start with an Uppercase letter
                regex: Some("^[A-Z][a-z]".to_owned()),
            },
        ),
        (
            "vehicles".to_owned(),
            AS3Validator::Object(HashMap::from([(
                "list".to_owned(),
                AS3Validator::List(Box::new(AS3Validator::Object(HashMap::from([
                    ("name".to_owned(), AS3Validator::String { regex: None }),
                    (
                        "maker".to_owned(),
                        AS3Validator::String {
                            regex: Some("^[A-Z][a-z]".to_owned()),
                        },
                    ),
                    ("year".to_owned(), AS3Validator::Integer { minimum: None }),
                ])))),
            )])),
        ),
    ]));

    let as3_data = AS3Data::from(&json);

    println!("AS3 : {:#?}", AS3Data::from(&json));
    // println!("Validator : {:?}", validator);
    println!(
        "Validator_result : {}",
        if validator.validate(&as3_data) {
            "✅"
        } else {
            "❌"
        }
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn should_run() {
        let json = json!({
          "age": 25,
          "children": 5,
          "name": "Dilec",
          "vehicles": {
            "list": [
              { "name": "model3", "maker": "Tesla", "year": 2018 },
              { "name": "Raptor", "maker": "Ford", "year": 2018 }
            ]
          }
        });

        let validator = AS3Validator::Object(HashMap::from([
            (
                "age".to_owned(),
                AS3Validator::Integer { minimum: Some(20) },
            ),
            (
                "children".to_owned(),
                AS3Validator::Integer { minimum: Some(2) },
            ),
            (
                "name".to_owned(),
                AS3Validator::String {
                    // The name should start with an Uppercase letter
                    regex: Some("^[A-Z][a-z]".to_owned()),
                },
            ),
            (
                "vehicles".to_owned(),
                AS3Validator::Object(HashMap::from([(
                    "list".to_owned(),
                    AS3Validator::List(Box::new(AS3Validator::Object(HashMap::from([
                        ("name".to_owned(), AS3Validator::String { regex: None }),
                        (
                            "maker".to_owned(),
                            AS3Validator::String {
                                regex: Some("^[A-Z][a-z]".to_owned()),
                            },
                        ),
                        ("year".to_owned(), AS3Validator::Integer { minimum: None }),
                    ])))),
                )])),
            ),
        ]));

        assert!(validator.validate(&AS3Data::from(&json)));
    }

    #[test]
    fn with_decimal_error() {
        let json = json!({
          "age": 25,
          "children": 5,
          "name": "Dilec",
          "vehicles": {
            "list": [
              { "name": "model3", "maker": "Tesla", "year": 2018 },
              { "name": "Raptor", "maker": "Ford", "year": 20.18 }
            ]
          }
        });

        let validator = AS3Validator::Object(HashMap::from([
            (
                "age".to_owned(),
                AS3Validator::Integer { minimum: Some(20) },
            ),
            (
                "children".to_owned(),
                AS3Validator::Integer { minimum: Some(2) },
            ),
            (
                "name".to_owned(),
                AS3Validator::String {
                    // The name should start with an Uppercase letter
                    regex: Some("^[A-Z][a-z]".to_owned()),
                },
            ),
            (
                "vehicles".to_owned(),
                AS3Validator::Object(HashMap::from([(
                    "list".to_owned(),
                    AS3Validator::List(Box::new(AS3Validator::Object(HashMap::from([
                        ("name".to_owned(), AS3Validator::String { regex: None }),
                        (
                            "maker".to_owned(),
                            AS3Validator::String {
                                regex: Some("^[A-Z][a-z]".to_owned()),
                            },
                        ),
                        ("year".to_owned(), AS3Validator::Integer { minimum: None }),
                    ])))),
                )])),
            ),
        ]));

        assert_eq!(validator.validate(&AS3Data::from(&json)), false);
    }
    #[test]
    fn with_string_error() {
        let json = json!({
          "age": 25,
          "children": 5,
          "name": "Dilec",
          "vehicles": {
            "list": [
              { "name": "model3", "maker": "Tesla", "year": 2018 },
              { "name": "Raptor", "maker": "Ford", "year": "2018" }
            ]
          }
        });

        let validator = AS3Validator::Object(HashMap::from([
            (
                "age".to_owned(),
                AS3Validator::Integer { minimum: Some(20) },
            ),
            (
                "children".to_owned(),
                AS3Validator::Integer { minimum: Some(2) },
            ),
            (
                "name".to_owned(),
                AS3Validator::String {
                    // The name should start with an Uppercase letter
                    regex: Some("^[A-Z][a-z]".to_owned()),
                },
            ),
            (
                "vehicles".to_owned(),
                AS3Validator::Object(HashMap::from([(
                    "list".to_owned(),
                    AS3Validator::List(Box::new(AS3Validator::Object(HashMap::from([
                        ("name".to_owned(), AS3Validator::String { regex: None }),
                        (
                            "maker".to_owned(),
                            AS3Validator::String {
                                regex: Some("^[A-Z][a-z]".to_owned()),
                            },
                        ),
                        ("year".to_owned(), AS3Validator::Integer { minimum: None }),
                    ])))),
                )])),
            ),
        ]));

        assert_eq!(validator.validate(&AS3Data::from(&json)), false);
    }

    fn with_regex_error() {
        let json = json!({
          "age": 25,
          "children": 5,
          "name": "Dilec",
          "vehicles": {
            "list": [
              { "name": "model3", "maker": "Tesla", "year": 2018 },
              { "name": "Raptor", "maker": "ford", "year": "2018" }
            ]
          }
        });

        let validator = AS3Validator::Object(HashMap::from([
            (
                "age".to_owned(),
                AS3Validator::Integer { minimum: Some(20) },
            ),
            (
                "children".to_owned(),
                AS3Validator::Integer { minimum: Some(2) },
            ),
            (
                "name".to_owned(),
                AS3Validator::String {
                    // The name should start with an Uppercase letter
                    regex: Some("^[A-Z][a-z]".to_owned()),
                },
            ),
            (
                "vehicles".to_owned(),
                AS3Validator::Object(HashMap::from([(
                    "list".to_owned(),
                    AS3Validator::List(Box::new(AS3Validator::Object(HashMap::from([
                        ("name".to_owned(), AS3Validator::String { regex: None }),
                        (
                            "maker".to_owned(),
                            AS3Validator::String {
                                regex: Some("^[A-Z][a-z]".to_owned()),
                            },
                        ),
                        ("year".to_owned(), AS3Validator::Integer { minimum: None }),
                    ])))),
                )])),
            ),
        ]));

        assert_eq!(validator.validate(&AS3Data::from(&json)), false);
    }
}
