use ron::{
    de::from_bytes,
    error::{Error, Position, SpannedError},
    from_str, to_string,
    value::RawValue,
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
struct WithRawValue {
    a: bool,
    b: Box<RawValue>,
}

#[test]
fn test_raw_value_simple() {
    let raw: &RawValue = from_str("true").unwrap();
    assert_eq!(raw.get_ron(), "true");
    let ser = to_string(raw).unwrap();
    assert_eq!(ser, "true");
}

#[test]
fn test_raw_value_inner() {
    let raw: WithRawValue = from_str("(a: false, b: [1, /* lol */ 2, 3])").unwrap();
    assert_eq!(raw.b.get_ron(), "[1, /* lol */ 2, 3]");
    let ser = to_string(&raw).unwrap();
    assert_eq!(ser, "(a:false,b:[1, /* lol */ 2, 3])");
}

#[test]
fn test_raw_value_comment() {
    let raw: WithRawValue = from_str("(a: false, b: /* nope */ 4)").unwrap();
    assert_eq!(raw.b.get_ron(), "4");

    let raw: WithRawValue = from_str("(a: false, b: 4 /* yes */)").unwrap();
    assert_eq!(raw.b.get_ron(), "4 /* yes */");

    let raw: WithRawValue = from_str("(a: false, b: (/* this */ 4 /* too */))").unwrap();
    assert_eq!(raw.b.get_ron(), "(/* this */ 4 /* too */)");
}

#[test]
fn test_raw_value_invalid() {
    let err = from_str::<&RawValue>("4.d").unwrap_err();
    assert_eq!(
        err,
        SpannedError {
            code: Error::TrailingCharacters,
            position: Position { line: 1, col: 3 }
        }
    );

    let err = from_bytes::<&RawValue>(b"\0").unwrap_err();
    assert_eq!(
        err,
        SpannedError {
            code: Error::UnexpectedByte('\0'),
            position: Position { line: 1, col: 1 }
        }
    )
}

#[test]
fn test_raw_value_from_ron() {
    let raw = RawValue::from_ron("/* hi */ (None, 4.2) /* bye */").unwrap();
    assert_eq!(raw.get_ron(), "/* hi */ (None, 4.2) /* bye */");

    let err = RawValue::from_ron("4.d").unwrap_err();
    assert_eq!(
        err,
        SpannedError {
            code: Error::TrailingCharacters,
            position: Position { line: 1, col: 3 }
        }
    );

    let raw =
        RawValue::from_boxed_ron(String::from("/* hi */ (None, 4.2) /* bye */").into_boxed_str())
            .unwrap();
    assert_eq!(raw.get_ron(), "/* hi */ (None, 4.2) /* bye */");

    let err = RawValue::from_boxed_ron(String::from("(").into_boxed_str()).unwrap_err();
    assert_eq!(
        err,
        SpannedError {
            code: Error::Eof,
            position: Position { line: 1, col: 2 },
        }
    );
}

#[test]
fn test_raw_value_into_rust() {
    let raw = RawValue::from_ron("/* hi */ (a: false, b: None) /* bye */").unwrap();

    let with: WithRawValue = raw.into_rust().unwrap();
    assert_eq!(
        with,
        WithRawValue {
            a: false,
            b: from_str("None").unwrap(),
        }
    );

    let err = raw.into_rust::<i32>().unwrap_err();
    assert_eq!(
        err,
        SpannedError {
            code: Error::ExpectedInteger,
            position: Position { line: 1, col: 10 },
        }
    );
}

#[test]
fn test_raw_value_from_rust() {
    let raw = RawValue::from_rust(&42).unwrap();
    assert_eq!(raw.get_ron(), "42");

    let raw = RawValue::from_rust(&WithRawValue {
        a: true,
        b: from_str("4.2").unwrap(),
    })
    .unwrap();
    assert_eq!(raw.get_ron(), "(a:true,b:4.2)");
}

#[test]
fn test_raw_value_serde_json() {
    let raw = RawValue::from_ron("/* hi */ (None, 4.2) /* bye */").unwrap();

    let ser = serde_json::to_string(&WithRawValue {
        a: true,
        b: raw.to_owned(),
    })
    .unwrap();
    assert_eq!(ser, "{\"a\":true,\"b\":\"/* hi */ (None, 4.2) /* bye */\"}");

    let with: WithRawValue = serde_json::from_str(&ser).unwrap();
    assert_eq!(raw, &*with.b);

    let err =
        serde_json::from_str::<WithRawValue>("{\"a\":true,\"b\":\"/* hi */ (a:) /* bye */\"}")
            .unwrap_err();
    assert_eq!(
        err.to_string(),
        "invalid RON value at 1:13: Unexpected byte ')' at line 1 column 39"
    );

    let err = serde_json::from_str::<WithRawValue>("{\"a\":true,\"b\":42}").unwrap_err();
    assert_eq!(
        err.to_string(),
        "invalid type: integer `42`, expected any valid RON-value-string at line 1 column 16"
    );

    let err = serde_json::from_str::<&RawValue>("\"/* hi */ (a:) /* bye */\"").unwrap_err();
    assert_eq!(
        err.to_string(),
        "invalid RON value at 1:13: Unexpected byte ')' at line 1 column 25"
    );

    let err = serde_json::from_str::<&RawValue>("42").unwrap_err();
    assert_eq!(
        err.to_string(),
        "invalid type: integer `42`, expected any valid borrowed RON-value-string at line 1 column 2"
    );
}

#[test]
fn test_raw_value_clone_into() {
    let raw = RawValue::from_boxed_ron(String::from("(None, 4.2)").into_boxed_str()).unwrap();
    let raw2 = raw.clone();
    assert_eq!(raw, raw2);

    let boxed_str: Box<str> = raw2.into();
    assert_eq!(&*boxed_str, "(None, 4.2)");
}

#[test]
fn test_raw_value_debug_display() {
    let raw = RawValue::from_ron("/* hi */ (None, 4.2) /* bye */").unwrap();

    assert_eq!(format!("{}", raw), "/* hi */ (None, 4.2) /* bye */");
    assert_eq!(
        format!("{:#?}", raw),
        "\
RawValue(
    /* hi */ (None, 4.2) /* bye */,
)\
    "
    );
}

#[test]
fn test_boxed_raw_value_deserialise_from_string() {
    let string = serde::de::value::StringDeserializer::<Error>::new(String::from("4.2"));

    let err = <&RawValue>::deserialize(string.clone()).unwrap_err();
    assert_eq!(
        err,
        Error::InvalidValueForType {
            expected: String::from("any valid borrowed RON-value-string"),
            found: String::from("the string \"4.2\""),
        }
    );

    let boxed_raw = Box::<RawValue>::deserialize(string).unwrap();
    assert_eq!(boxed_raw.get_ron(), "4.2");

    let string = serde::de::value::StringDeserializer::<Error>::new(String::from("["));

    let err = Box::<RawValue>::deserialize(string).unwrap_err();
    assert_eq!(
        err,
        Error::Message(String::from(
            "invalid RON value at 1:2: Unexpected end of RON"
        ))
    );
}