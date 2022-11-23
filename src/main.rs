use regex::Regex;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs};

use thiserror::Error;
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
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

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
enum AS3Validator {
    #[serde(rename(serialize = "+Object", deserialize = "+Object"))]
    Object(HashMap<String, AS3Validator>),
    #[serde(rename(serialize = "+String", deserialize = "+String"))]
    String { regex: Option<String> },
    #[serde(rename(serialize = "+Integer", deserialize = "+Integer"))]
    Integer { minimum: Option<i64> },
    #[serde(rename(serialize = "+Decimal", deserialize = "+Decimal"))]
    Decimal { minimum: Option<f64> },
    #[serde(rename(serialize = "+List", deserialize = "+List"))]
    List(Box<AS3Validator>),
}

impl AS3Validator {
    fn validate(&self, data: &AS3Data) -> Result<(), AS3ValidationError> {
        match (self, data) {
            (AS3Validator::Object(validator_inner), AS3Data::Object(data_inner)) => {
                let res: Vec<Result<(), AS3ValidationError>> = validator_inner
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
                    .collect::<Result<Vec<()>, AS3ValidationError>>()
                {
                    Ok(_) => Ok(()),
                    Err(e) => Err(e),
                }
            }
            (AS3Validator::Integer { minimum }, AS3Data::Integer(number)) => {
                let Some(minimum) = minimum else {
                    return Ok(());
                };
                if minimum > number {
                    Err(AS3ValidationError::Minimum {
                        number: *number as f64,
                        minimum: *minimum as f64,
                    })
                } else {
                    Ok(())
                }
            }
            (AS3Validator::Decimal { minimum }, AS3Data::Decimal(number)) => {
                let Some(minimum) = minimum else {
                    return Ok(());
                };
                if minimum > number {
                    Err(AS3ValidationError::Minimum {
                        number: *number as f64,
                        minimum: *minimum as f64,
                    })
                } else {
                    Ok(())
                }
            }
            (AS3Validator::String { regex }, AS3Data::String(string)) => {
                let Some(regex) = regex else {
                    return Ok(());
                };
                let re = Regex::new(regex).unwrap();

                if !re.is_match(string) {
                    // TODO! implement error
                    return Err(AS3ValidationError::RegexError {
                        word: string.to_owned(),
                        regex: regex.to_owned(),
                    });
                }
                Ok(())
            }
            (AS3Validator::List(items_type), AS3Data::List(items)) => {
                // Ok(items.iter().all(|item| items_type.validate(item)))

                let res = items
                    .iter()
                    .map(|item| items_type.validate(item))
                    .collect::<Vec<Result<(), AS3ValidationError>>>();

                match res
                    .into_iter()
                    .collect::<Result<Vec<()>, AS3ValidationError>>()
                {
                    Ok(_) => Ok(()),
                    Err(e) => Err(e),
                }
            }

            _ => Err(AS3ValidationError::TypeError {
                expected: self.clone(),
                got: data.clone(),
            }),
        }
    }

    fn to_yaml_string(self) -> String {
        let serialized_json = serde_json::to_string(&self).unwrap();
        let serialized_yaml: serde_yaml::Value =
            serde_yaml::from_str::<serde_yaml::Value>(&serialized_json).unwrap();
        serde_yaml::to_string(&serialized_yaml).unwrap()
        // serde_yaml::to_string(&serialized_yaml).unwrap()
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


impl AS3Validator {
    pub fn from(yaml_config: &serde_yaml::Value) -> Result<AS3Validator, String> {
        let serde_yaml::Value::Mapping(inner) = yaml_config else {
            println!("Definition must start with a Yaml Mapping");
            return Err("Definition must start with a Yaml Mapping".to_string());
           
        } ;
        let root_word: String = "Root".to_string();
        if !inner.contains_key(&root_word) {

            return Err("Missing root word from definition".to_string());
        };

        AS3Validator::build_from_yaml( &inner.get(root_word).unwrap())

        


    }

    fn build_from_yaml(
        // validator: &mut AS3Validator,
        yaml_config: &&serde_yaml::Value,
    ) -> Result<AS3Validator, String> {
        let Some(serde_yaml::Value::String(validator_type)) = yaml_config.get("+Type") else {
            return Err("Non ce il +type".to_string());
        };

        let validator = match (validator_type.as_str(), yaml_config) {
            ("Object", serde_yaml::Value::Mapping(inner)) => {
                // println!("quiii {inner:#?}");
                
                let x : HashMap<String, AS3Validator>= inner.into_iter()           
                    .filter(|(key, _)| key != &&serde_yaml::Value::String("+Type".to_string()))
                    .map(|(key, value)| {
                        (
                            key.as_str().unwrap().to_string(),
                            AS3Validator::build_from_yaml( &value).unwrap(),
                        )
                    }).collect();
                    AS3Validator::Object(x)
            },
            ("String", serde_yaml::Value::Mapping(inner)) => {
                
                    AS3Validator::String{ regex : None}
            }

            ("Integer", serde_yaml::Value::Mapping(inner)) => {
                
                AS3Validator::Integer { minimum: None }
        }
            _ => return Err("unsupported type".to_string()),
        };

        Ok(validator)
   
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

    #[error(" `{}` is under the minumum of `{}` . " , .number , .minimum)]
    Minimum { number: f64, minimum: f64 },
}

fn main() {


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

    let data = fs::read_to_string("test.json").expect("Unable to read file");
    let data_to_validate: serde_json::Value =
        serde_json::from_str(&data).expect("JSON does not have correct format.");

    let validator_schema = fs::read_to_string("validator_schema.yml").expect("Unable to read file");
    let schema_yaml: serde_yaml::Value = serde_yaml::from_str(&validator_schema).unwrap();
    if let Ok(validator) = AS3Validator::from(&schema_yaml) {
        println!("{:?}", validator.validate(&AS3Data::from(&data_to_validate)) )
    }



    // MAIN
    // let as3_data = AS3Data::from(&json);
    // println!("AS3 : {:#?}", AS3Data::from(&json));
    // println!("Validator : {:?}", validator);
    // validator.validate(&as3_data).unwrap();

    // VALIDATOR FROM FILE
    // let data = fs::read_to_string("test.json").expect("Unable to read file");
    // let json: serde_json::Value =
    //     serde_json::from_str(&data).expect("JSON does not have correct format.");


}

#[cfg(test)]
#[path = "integration_test.rs"]
mod test;