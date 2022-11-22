use regex::Regex;
use std::{collections::HashMap, fs, ops::Deref};

use thiserror::Error;
#[derive(Debug, PartialEq, Clone)]
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

#[derive(Debug, PartialEq, Clone)]
enum AS3Validator {
    Object(HashMap<String, AS3Validator>),
    String { regex: Option<String> },
    Integer { minimum: Option<i64> },
    Decimal { minimum: Option<i64> },
    List(Box<AS3Validator>),
}

impl AS3Validator {
    fn validate(&self, data: &AS3Data) -> Result<bool, AS3ValidationError> {
        match (self, data) {
            (AS3Validator::Object(validator_inner), AS3Data::Object(data_inner)) => {
                let res: Vec<Result<bool, AS3ValidationError>> = validator_inner
                    .into_iter()
                    .map(|(validator_key, validator_value)| {
                        if let Some(value_from_key) = data_inner.get(validator_key) {
                            return validator_value.validate(value_from_key);
                        }
                        Err(AS3ValidationError::MissingKey {
                            key: validator_key.clone(),
                            // context: data_inner.into_iter().map().collect(),
                        })
                    })
                    .collect();

                match res
                    .into_iter()
                    .collect::<Result<Vec<bool>, AS3ValidationError>>()
                {
                    Ok(ins) => Ok(ins.iter().all(|e| *e)),
                    Err(e) => Err(e),
                }
            }
            (AS3Validator::Integer { minimum }, AS3Data::Integer(number)) => {
                let Some(minimum) = minimum else {
                    return Ok(true);
                };
                Ok(number >= &minimum)
            }
            (AS3Validator::String { regex }, AS3Data::String(string)) => {
                let Some(regex) = regex else {
                    return Ok(true);
                };
                let re = Regex::new(regex).unwrap();

                if !re.is_match(string) {
                    // TODO! implement error
                    return Err(AS3ValidationError::RegexError {
                        word: string.to_owned(),
                        regex: regex.to_owned(),
                    });
                }
                Ok(true)
            }
            (AS3Validator::List(items_type), AS3Data::List(items)) => {
                // Ok(items.iter().all(|item| items_type.validate(item)))

                let res = items
                    .iter()
                    .map(|item| items_type.validate(item))
                    .collect::<Vec<Result<bool, AS3ValidationError>>>();

                match res
                    .into_iter()
                    .collect::<Result<Vec<bool>, AS3ValidationError>>()
                {
                    Ok(ins) => Ok(ins.iter().all(|e| *e)),
                    Err(e) => Err(e),
                }
            }

            _ => Err(AS3ValidationError::TypeError {
                expected: self.clone(),
                got: data.clone(),
            }),
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

#[derive(Error, Debug, PartialEq)]
enum AS3ValidationError {
    #[error("Mismatched types. Expected `{:?}` got `{:?}` . " , .expected , .got)]
    TypeError {
        expected: AS3Validator,
        got: AS3Data,
    },
    #[error("Key {} is not in " , .key )]
    // .expect(&format!("Key {validator_key} is not in {data_inner:#?}")),
    MissingKey {
        key: String,
        // context: HashMap<String, Box<AS3Data>>,
    },
    #[error("Word {} is not following the `{}` regex " , .word, .regex )]
    RegexError { word: String, regex: String },
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
        if let Ok(res) = validator.validate(&as3_data) {
            if res {
                "OK!"
            } else {
                "NO!"
            }
        } else {
            "Err"
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

        assert_eq!(validator.validate(&AS3Data::from(&json)), Ok(true));
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

        assert_eq!(
            validator.validate(&AS3Data::from(&json)),
            Err(AS3ValidationError::TypeError {
                expected: AS3Validator::Integer { minimum: None },
                got: AS3Data::Decimal(20.18)
            })
        );
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

        assert_eq!(
            validator.validate(&AS3Data::from(&json)),
            Err(AS3ValidationError::TypeError {
                expected: AS3Validator::Integer { minimum: None },
                got: AS3Data::String("2018".to_string())
            })
        );
    }
    #[test]
    fn with_regex_error() {
        let json = json!({
          "age": 25,
          "children": 5,
          "name": "Dilec",
          "vehicles": {
            "list": [
              { "name": "model3", "maker": "Tesla", "year": 2018},
              { "name": "Raptor", "maker": "ford", "year": 2018 }
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

        assert_eq!(
            validator.validate(&AS3Data::from(&json)),
            Err(AS3ValidationError::RegexError {
                word: "ford".to_string(),
                regex: "^[A-Z][a-z]".to_string()
            })
        )
    }
}
