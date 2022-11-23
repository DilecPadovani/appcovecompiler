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

    assert_eq!(validator.validate(&AS3Data::from(&json)), Ok(()));
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

#[test]
fn with_minimum_error() {
    let json = json!({
      "age": 18,
      "children": 5,
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
    ]));

    assert_eq!(
        validator.validate(&AS3Data::from(&json)),
        Err(AS3ValidationError::Minimum {
            number: 18.0,
            minimum: 20.0
        })
    );

    let json = json!({
      "age": 20,
      "children": 0,
    });

    assert_eq!(
        validator.validate(&AS3Data::from(&json)),
        Err(AS3ValidationError::Minimum {
            number: 0.0,
            minimum: 2.0
        })
    );

    let json = json!({
      "age": 20,
      "children": 20,
    });

    assert_eq!(validator.validate(&AS3Data::from(&json)), Ok(()))
}

#[test]
fn with_missing_field_error_validator_derive() {
    let mut json = json!({
      "vehicles": {
        "name": "raptor",
        "year": 2018
      },
      "Truks": {
        "name": "hummer",
        "maker": "ford",
        "year": 2019
      }
    });

    let data = AS3Data::from(&json);

    let yaml: serde_yaml::Value = serde_yaml::from_str(
        &r#"
        Root:
            +Type: Object
            vehicles:
                +Type: Object
                name:
                    +Type: String
                maker:
                    +Type: String
                year:
                    +Type: Integer
            Truks:
                +Type: Object
                name:
                    +Type: String
                maker:
                    +Type: String
                year:
                    +Type: Integer
                    "#,
    )
    .unwrap();

    let validator = AS3Validator::from(&yaml).unwrap();

    assert_eq!(
        validator.validate(&data),
        Err(AS3ValidationError::MissingKey {
            key: "maker".to_string()
        })
    );

    json["vehicles"]["maker"] = serde_json::Value::String("tesla".to_string());

    assert_eq!(validator.validate(&AS3Data::from(&json)), Ok(()))
}
