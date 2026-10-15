//! The value types encode as their string forms.

#![cfg(feature = "locale")]

use koine::{Country, Language, Locale};

#[cfg(feature = "serde")]
#[test]
fn serde_uses_the_string_form() {
    let value = (Language::ENG, Country::US, Locale::EN_US, Locale::DEFAULT);
    let json = serde_json::to_string(&value).unwrap();
    assert_eq!(json, r#"["eng","US","en-US","default"]"#);
    assert_eq!(
        serde_json::from_str::<(Language, Country, Locale, Locale)>(&json).unwrap(),
        value
    );

    assert_eq!(
        serde_json::from_str::<Language>(r#""en""#).unwrap(),
        Language::ENG
    );
    assert!(serde_json::from_str::<Locale>(r#""en-ZZ""#).is_err());
}

#[cfg(feature = "musli01")]
#[test]
fn musli_uses_the_string_form() {
    let value = (Language::ENG, Country::US, Locale::EN_US, Locale::DEFAULT);
    let json = musli::json::to_string(&value).unwrap();
    assert_eq!(json, r#"["eng","US","en-US","default"]"#);
    assert_eq!(
        musli::json::from_str::<(Language, Country, Locale, Locale)>(&json).unwrap(),
        value
    );
}
