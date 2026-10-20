//! The value types encode as their string forms, and are stored in sqll as
//! integers.

#![cfg(all(
    feature = "locale",
    any(feature = "serde", feature = "musli01", feature = "sqll014")
))]

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
    let err = serde_json::from_str::<Locale>(r#""en-ZZ""#).unwrap_err();
    assert!(
        err.to_string().contains("unknown country in locale code"),
        "{err}"
    );
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

/// The stored integers are a stable format: changing any of them breaks every
/// database written before.
#[cfg(feature = "sqll014")]
#[test]
fn sqll_stored_values() -> sqll014::Result<()> {
    use sqll014::Connection;

    assert_eq!(Locale::EN_US.to_u64(), 0x5553_0000_656e_6700);
    assert_eq!(Locale::DEFAULT.to_u64(), 0);

    let c = Connection::open_in_memory()?;
    c.execute("CREATE TABLE t (language INTEGER, country INTEGER, locale INTEGER)")?;

    let mut insert = c.prepare("INSERT INTO t VALUES (?, ?, ?)")?;
    insert.execute((Language::ENG, Country::US, Locale::EN_US))?;
    insert.reset()?;
    insert.execute((Language::DEFAULT, Country::DEFAULT, Locale::DEFAULT))?;

    let mut select = c.prepare("SELECT language, country, locale FROM t ORDER BY rowid")?;
    let raw = select
        .iter::<(i64, i64, i64)>()
        .collect::<sqll014::Result<Vec<_>>>()?;
    assert_eq!(
        raw,
        [(0x656e_6700, 0x5553_0000, 0x5553_0000_656e_6700), (0, 0, 0)]
    );

    select.reset()?;
    let typed = select
        .iter::<(Language, Country, Locale)>()
        .collect::<sqll014::Result<Vec<_>>>()?;
    assert_eq!(
        typed,
        [
            (Language::ENG, Country::US, Locale::EN_US),
            (Language::DEFAULT, Country::DEFAULT, Locale::DEFAULT)
        ]
    );
    Ok(())
}

/// A well-formed code missing from the tables, such as one retired after it
/// was stored, reads back intact; a malformed one is an error.
#[cfg(feature = "sqll014")]
#[test]
fn sqll_keeps_codes_missing_from_the_tables() -> sqll014::Result<()> {
    use sqll014::Connection;

    // "qqq" and "QQ" are well formed but in neither table.
    let language = 0x7171_7100_i64;
    let country = 0x5151_0000_i64;
    let locale = (country << 32) | language;

    let c = Connection::open_in_memory()?;
    c.execute("CREATE TABLE t (language INTEGER, country INTEGER, locale INTEGER)")?;
    let mut insert = c.prepare("INSERT INTO t VALUES (?, ?, ?)")?;
    insert.execute((language, country, locale))?;
    insert.reset()?;
    // A known language with an unknown country.
    insert.execute((language, country, (country << 32) | 0x656e_6700))?;

    let mut select = c.prepare("SELECT language, country, locale FROM t ORDER BY rowid")?;
    let rows = select
        .iter::<(Language, Country, Locale)>()
        .collect::<sqll014::Result<Vec<_>>>()?;
    assert_eq!(rows[0].0.as_str(), "qqq");
    assert_eq!(rows[0].1.as_str(), "QQ");
    assert_eq!(rows[0].2.to_string(), "qqq-QQ");
    assert_eq!(rows[1].2.to_string(), "en-QQ");

    // Write the read values back and check nothing was lost.
    c.execute("DELETE FROM t")?;
    insert.reset()?;
    for row in &rows {
        insert.execute(*row)?;
        insert.reset()?;
    }

    select.reset()?;
    let raw = select
        .iter::<(i64, i64, i64)>()
        .collect::<sqll014::Result<Vec<_>>>()?;
    assert_eq!(
        raw,
        [
            (language, country, locale),
            (language, country, (country << 32) | 0x656e_6700)
        ]
    );

    // Malformed: digits, a gap in the padding, the wrong case.
    for bad in [0x3132_3300_i64, 0x7100_7100, 0x5555_5500] {
        let mut select = c.prepare("SELECT ?")?;
        select.bind((bad,))?;
        assert!(select.next::<(Locale,)>().is_err(), "{bad:#x}");
    }

    Ok(())
}
