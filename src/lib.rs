//! [<img alt="github" src="https://img.shields.io/badge/github-udoprog/koine-8da0cb?style=for-the-badge&logo=github" height="20">](https://github.com/udoprog/koine)
//! [<img alt="crates.io" src="https://img.shields.io/crates/v/koine.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/koine)
//! [<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-koine-66c2a5?style=for-the-badge&logoColor=white&logo=data:image/svg+xml;base64,PHN2ZyByb2xlPSJpbWciIHhtbG5zPSJodHRwOi8vd3d3LnczLm9yZy8yMDAwL3N2ZyIgdmlld0JveD0iMCAwIDUxMiA1MTIiPjxwYXRoIGZpbGw9IiNmNWY1ZjUiIGQ9Ik00ODguNiAyNTAuMkwzOTIgMjE0VjEwNS41YzAtMTUtOS4zLTI4LjQtMjMuNC0zMy43bC0xMDAtMzcuNWMtOC4xLTMuMS0xNy4xLTMuMS0yNS4zIDBsLTEwMCAzNy41Yy0xNC4xIDUuMy0yMy40IDE4LjctMjMuNCAzMy43VjIxNGwtOTYuNiAzNi4yQzkuMyAyNTUuNSAwIDI2OC45IDAgMjgzLjlWMzk0YzAgMTMuNiA3LjcgMjYuMSAxOS45IDMyLjJsMTAwIDUwYzEwLjEgNS4xIDIyLjEgNS4xIDMyLjIgMGwxMDMuOS01MiAxMDMuOSA1MmMxMC4xIDUuMSAyMi4xIDUuMSAzMi4yIDBsMTAwLTUwYzEyLjItNi4xIDE5LjktMTguNiAxOS45LTMyLjJWMjgzLjljMC0xNS05LjMtMjguNC0yMy40LTMzLjd6TTM1OCAyMTQuOGwtODUgMzEuOXYtNjguMmw4NS0zN3Y3My4zek0xNTQgMTA0LjFsMTAyLTM4LjIgMTAyIDM4LjJ2LjZsLTEwMiA0MS40LTEwMi00MS40di0uNnptODQgMjkxLjFsLTg1IDQyLjV2LTc5LjFsODUtMzguOHY3NS40em0wLTExMmwtMTAyIDQxLjQtMTAyLTQxLjR2LS42bDEwMi0zOC4yIDEwMiAzOC4ydi42em0yNDAgMTEybC04NSA0Mi41di03OS4xbDg1LTM4Ljh2NzUuNHptMC0xMTJsLTEwMiA0MS40LTEwMi00MS40di0uNmwxMDItMzguMiAxMDIgMzguMnYuNnoiPjwvcGF0aD48L3N2Zz4K" height="20">](https://docs.rs/koine)
//!
//! Locale primitives: ISO 639 languages, ISO 3166-1 countries and the valid
//! language+country locales, as compact `Copy` value types over generated
//! static tables.
//!
//! [`Language`], [`Country`] and [`Locale`] are four, four and eight bytes,
//! hash and order cheaply, display as their codes (`eng`, `SE`, `pt-BR`) and
//! parse from them in any case. Each is validated against its table, and
//! reaches the table's entry for names and other codes. The `no_std` tables
//! are generated ahead of time, so building koine runs no build script and
//! reads no files.
//!
//! ```
//! # #[cfg(feature = "locale")]
//! # {
//! use koine::{Country, Language, Locale};
//!
//! let locale: Locale = "sv-se".parse().unwrap();
//! assert_eq!(locale.language(), Language::new(b"swe"));
//! assert_eq!(locale.country(), Country::new(b"SE"));
//! assert_eq!(locale.to_string(), "sv-SE");
//! assert!(locale.is_listed());
//!
//! let language = locale.language().to_iso().unwrap();
//! assert_eq!(language.name, "Swedish");
//! assert_eq!(language.part1, Some("sv"));
//!
//! let country = locale.country().to_iso().unwrap();
//! assert_eq!(country.alpha3, "SWE");
//! # }
//! ```
//!
//! <br>
//!
//! ## Features
//!
//! The default features give you all three types and their tables, with the
//! standard library:
//!
//! ```toml
//! [dependencies]
//! koine = "0.3.2"
//! ```
//!
//! Everything else is opt-in, and you can turn off the defaults to build only
//! what you use (see [Choosing features](#choosing-features)).
//!
//! <br>
//!
//! ### Data tables
//!
//! Each type comes with its own table, so a crate compiles only the tables it
//! uses. The language table, which holds every ISO 639-3 code, is by far the
//! largest.
//!
//! | Feature | Default | What it lets you do |
//! |---|---|---|
//! | `language` | yes | Use [`Language`] and look up any ISO 639-3 language: its ISO 639-1 and 639-2 codes, English name, scope and type, and a country whose flag can stand for it. See [`language`]. |
//! | `country` | yes | Use [`Country`] and look up any ISO 3166-1 country: its alpha-2, alpha-3 and numeric codes, English name, and UN M49 region and sub-region. See [`country`]. |
//! | `locale` | yes | Use [`Locale`], a language with an optional country such as `pt-BR`, and check whether a language and country form a known locale. Turns on `language` and `country`. See [`locale`]. |
//!
//! <br>
//!
//! ### Integrations
//!
//! Each integration lets you use koine's types directly with another crate.
//! None is on by default.
//!
//! | Feature | What it lets you do |
//! |---|---|
//! | `serde` | Serialize and deserialize koine's types with serde, as their codes (`"eng"`, `"SE"`, `"pt-BR"`). |
//! | `musli02` | Encode and decode koine's types with musli 0.2, as their codes. |
//! | `sqll015` | Store koine's types in SQLite with sqll 0.15: bind them as query parameters and read them back from columns. They are stored as integers in a stable format; see [Storage in SQLite](#storage-in-sqlite). |
//! | `yew023` | Pass koine's types straight to yew 0.23: any of them as an `Html` prop, shown as its code; a [`Country`] or [`Locale`] as an attribute value, giving the country's English name or the locale's code; and a [`Country`] as a list `key`. Turns on `alloc`. |
//!
//! An integration with a crate that is not yet 1.0 is named after the version
//! it supports: `musli02` is musli 0.2, `sqll015` is sqll 0.15 and `yew023` is
//! yew 0.23. Support for a later version gets a feature of its own beside the
//! existing one, so upgrading koine never moves you to a new version of those
//! crates by surprise.
//!
//! <br>
//!
//! ### Platform
//!
//! | Feature | Default | What it lets you do |
//! |---|---|---|
//! | `std` | yes | Use the standard library. Turns on `alloc`. |
//! | `alloc` | with `std` | Use koine in a `no_std` crate that has an allocator. |
//!
//! koine itself needs neither: without them it is `no_std` and does not
//! allocate. Both are passed on to the integrations you turn on, so leaving
//! out `std` builds those without the standard library too.
//!
//! <br>
//!
//! ### Choosing features
//!
//! With `default-features = false`, name every feature you need, `std`
//! included. For example, either of:
//!
//! ```toml
//! [dependencies]
//! # Countries only, with serde, in a crate that uses the standard library.
//! koine = { version = "0.3.2", default-features = false, features = ["std", "country", "serde"] }
//! # Languages only, in a no_std crate without an allocator.
//! koine = { version = "0.3.2", default-features = false, features = ["language"] }
//! ```
//!
//! <br>
//!
//! ## Storage in SQLite
//!
//! With the `sqll015` feature, koine's types are stored in SQLite as integers.
//! This format is a compatibility promise: koine keeps it stable across
//! releases, so values you have stored keep reading back the same after you
//! upgrade.
//!
//! * A [`Language`] is its code's ASCII bytes, padded with zero bytes to four
//!   and read as a big-endian 32-bit integer: `eng` is `0x656e6700`.
//! * A [`Country`] is its alpha-2 code stored the same way: `US` is
//!   `0x55530000`.
//! * A [`Locale`] is [`Locale::to_u64`]: the country's integer in the high 32
//!   bits and the language's in the low 32 bits.
//! * The `DEFAULT` value of each type is `0`.
//!
//! The byte order is fixed, so the stored value is the same on every
//! platform. Reading a value checks only that it is well formed, not that it
//! is in the tables, so a code that a later release drops from its tables
//! still reads back unchanged. A malformed value is an error with sqll's
//! `MISMATCH` code; its message is kept only with the `alloc` feature.
//!
//! <br>
//!
//! ## Data sources
//!
//! The tables are generated from data files vendored in the repository,
//! which a tools crate there downloads and regenerates the tables from. The
//! code of this crate is MIT OR Apache-2.0; the data each table is generated
//! from keeps its own terms:
//!
//! * **Languages** (`language`): the ISO 639-3 code tables of
//!   [SIL International], the ISO 639-3 registration authority, from
//!   [iso639-3.sil.org]. The codes, names and classifications are SIL's.
//!   The downloaded file, `data/language/iso-639-3.tab`, is kept in the
//!   repository for regeneration and is not part of the published package.
//!   The codes are used under SIL's ISO 639-3 Code Tables Terms of Use,
//!   published on [iso639-3.sil.org], which is credited here as the source
//!   of the codes; the identifiers are used as published, neither modified
//!   nor extended.
//! * **Countries and locales** (`country`, `locale`): [Unicode CLDR]
//!   release 48.2, from [unicode.org], under the [Unicode License v3]
//!   (`Unicode-3.0`), whose text ships with this crate as
//!   `data/cldr/LICENSE`. The countries are the regular CLDR regions with an
//!   ISO 3166-1 alpha-3 and numeric code, with their English names and
//!   their UN M49 regions from CLDR's territory containment. The locales are
//!   each language and country combined by a locale in CLDR's `common/main`.
//! * **Language flags** ([`language::Entry::flag`]):
//!   `data/language/to-3166.csv`, a mapping from each language to the
//!   country whose flag can stand for it. It is written and maintained by
//!   hand in this repository, best-effort and not authoritative, and is
//!   licensed with the crate (MIT OR Apache-2.0). It is an addition of this
//!   crate's and not part of the ISO 639-3 standard.
//!
//! CLDR data is Copyright © Unicode, Inc.; the full copyright and permission
//! notice is in `data/cldr/LICENSE`.
//!
//! [SIL International]: https://www.sil.org
//! [iso639-3.sil.org]: https://iso639-3.sil.org
//! [Unicode CLDR]: https://cldr.unicode.org
//! [unicode.org]: https://unicode.org/Public/cldr/
//! [Unicode License v3]: https://www.unicode.org/license.txt
//!
//! <br>
//!
//! ## Updating the data
//!
//! In a checkout of the repository, `cargo run -p tools -- download` fetches
//! the ISO 639-3 table and the pinned CLDR release, and `cargo run -p tools`
//! regenerates every table from the vendored files; then review the diff.
//! `cargo run -p tools -- --check` fails if a table is out of date. The
//! CLDR release is pinned in `tools/src/cldr.rs`. The hand-maintained
//! `to-3166.csv` is never written by the tools.
//!
//! [`Country`]: https://docs.rs/koine/latest/koine/country/struct.Country.html
//! [`country`]: https://docs.rs/koine/latest/koine/country/index.html
//! [`Language`]: https://docs.rs/koine/latest/koine/language/struct.Language.html
//! [`language`]: https://docs.rs/koine/latest/koine/language/index.html
//! [`language::Entry::flag`]: https://docs.rs/koine/latest/koine/language/struct.Entry.html#structfield.flag
//! [`Locale`]: https://docs.rs/koine/latest/koine/locale/struct.Locale.html
//! [`locale`]: https://docs.rs/koine/latest/koine/locale/index.html
//! [`Locale::to_u64`]: https://docs.rs/koine/latest/koine/locale/struct.Locale.html#method.to_u64
#![no_std]
#![deny(missing_docs)]
#![cfg_attr(koine_docsrs, feature(doc_cfg))]

#[cfg(any(test, feature = "alloc"))]
extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

#[macro_use]
mod macros;

#[cfg(feature = "language")]
#[cfg_attr(koine_docsrs, doc(cfg(feature = "language")))]
pub mod language;
#[cfg(feature = "language")]
#[doc(inline)]
pub use self::language::Language;

#[cfg(feature = "country")]
#[cfg_attr(koine_docsrs, doc(cfg(feature = "country")))]
pub mod country;
#[cfg(feature = "country")]
#[doc(inline)]
pub use self::country::Country;

#[cfg(feature = "locale")]
#[cfg_attr(koine_docsrs, doc(cfg(feature = "locale")))]
pub mod locale;
#[cfg(feature = "locale")]
#[doc(inline)]
pub use self::locale::Locale;
