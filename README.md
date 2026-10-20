# koine

[<img alt="github" src="https://img.shields.io/badge/github-udoprog/koine-8da0cb?style=for-the-badge&logo=github" height="20">](https://github.com/udoprog/koine)
[<img alt="crates.io" src="https://img.shields.io/crates/v/koine.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/koine)
[<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-koine-66c2a5?style=for-the-badge&logoColor=white&logo=data:image/svg+xml;base64,PHN2ZyByb2xlPSJpbWciIHhtbG5zPSJodHRwOi8vd3d3LnczLm9yZy8yMDAwL3N2ZyIgdmlld0JveD0iMCAwIDUxMiA1MTIiPjxwYXRoIGZpbGw9IiNmNWY1ZjUiIGQ9Ik00ODguNiAyNTAuMkwzOTIgMjE0VjEwNS41YzAtMTUtOS4zLTI4LjQtMjMuNC0zMy43bC0xMDAtMzcuNWMtOC4xLTMuMS0xNy4xLTMuMS0yNS4zIDBsLTEwMCAzNy41Yy0xNC4xIDUuMy0yMy40IDE4LjctMjMuNCAzMy43VjIxNGwtOTYuNiAzNi4yQzkuMyAyNTUuNSAwIDI2OC45IDAgMjgzLjlWMzk0YzAgMTMuNiA3LjcgMjYuMSAxOS45IDMyLjJsMTAwIDUwYzEwLjEgNS4xIDIyLjEgNS4xIDMyLjIgMGwxMDMuOS01MiAxMDMuOSA1MmMxMC4xIDUuMSAyMi4xIDUuMSAzMi4yIDBsMTAwLTUwYzEyLjItNi4xIDE5LjktMTguNiAxOS45LTMyLjJWMjgzLjljMC0xNS05LjMtMjguNC0yMy40LTMzLjd6TTM1OCAyMTQuOGwtODUgMzEuOXYtNjguMmw4NS0zN3Y3My4zek0xNTQgMTA0LjFsMTAyLTM4LjIgMTAyIDM4LjJ2LjZsLTEwMiA0MS40LTEwMi00MS40di0uNnptODQgMjkxLjFsLTg1IDQyLjV2LTc5LjFsODUtMzguOHY3NS40em0wLTExMmwtMTAyIDQxLjQtMTAyLTQxLjR2LS42bDEwMi0zOC4yIDEwMiAzOC4ydi42em0yNDAgMTEybC04NSA0Mi41di03OS4xbDg1LTM4Ljh2NzUuNHptMC0xMTJsLTEwMiA0MS40LTEwMi00MS40di0uNmwxMDItMzguMiAxMDIgMzguMnYuNnoiPjwvcGF0aD48L3N2Zz4K" height="20">](https://docs.rs/koine)
[<img alt="build status" src="https://img.shields.io/github/actions/workflow/status/udoprog/koine/ci.yml?branch=main&style=for-the-badge" height="20">](https://github.com/udoprog/koine/actions?query=branch%3Amain)

Locale primitives: ISO 639 languages, ISO 3166-1 countries and the valid
language+country locales, as compact `Copy` value types over generated
static tables.

[`Language`], [`Country`] and [`Locale`] are four, four and eight bytes,
hash and order cheaply, display as their codes (`eng`, `SE`, `pt-BR`) and
parse from them in any case. Each is validated against its table, and
reaches the table's entry for names and other codes. The `no_std` tables
are generated ahead of time, so building koine runs no build script and
reads no files.

```rust
use koine::{Country, Language, Locale};

let locale: Locale = "sv-se".parse().unwrap();
assert_eq!(locale.language(), Language::new(b"swe"));
assert_eq!(locale.country(), Country::new(b"SE"));
assert_eq!(locale.to_string(), "sv-SE");
assert!(locale.is_listed());

let language = locale.language().to_iso().unwrap();
assert_eq!(language.name, "Swedish");
assert_eq!(language.part1, Some("sv"));

let country = locale.country().to_iso().unwrap();
assert_eq!(country.alpha3, "SWE");
```

<br>

## Features

Each table is its own feature, so a crate compiles only the tables it
uses; the language table is by far the largest, with every ISO 639-3
code. The default features are `std`, `language`, `country` and `locale`.

A dependency that is not yet 1.0 and appears in the API is named after
its version: `musli01` for musli 0.1, `sqll014` for sqll 0.14 and `yew023`
for yew 0.23. A later version is added beside it, so a koine upgrade never
forces an app onto a new musli, sqll or yew.

`std` is on by default and implies `alloc`. Each is passed on to the
dependencies that have a feature of the same name, so leaving `std` out
builds them without it too.

| Feature | Module | What it provides |
|---|---|---|
| `std` | | The standard library, in koine and its dependencies (implies `alloc`) |
| `alloc` | | The `alloc` crate, in koine and its dependencies; with `sqll014` it keeps the messages of the errors koine returns |
| `language` | [`language`] | [`Language`] and the ISO 639-3 table: codes, ISO 639-1 and 639-2 forms, scope, type, English name and a flag country |
| `country` | [`country`] | [`Country`] and the ISO 3166-1 table: alpha-2, alpha-3 and numeric codes, English name, UN M49 region and sub-region |
| `locale` | [`locale`] | [`Locale`] and the table of valid language+country combinations (implies `language` and `country`) |
| `serde` | | serde `Serialize` and `Deserialize` as the string forms |
| `musli01` | | musli 0.1 `Encode` and `Decode` as the string forms |
| `sqll014` | | sqll 0.14 `FromColumn` and `BindValue`, as a stable integer: the code's bytes big-endian for [`Language`] (`eng` is `0x656e6700`) and [`Country`] (`US` is `0x55530000`), [`Locale::to_u64`] for [`Locale`], and `0` for `DEFAULT`. A well-formed code missing from the tables reads back unchanged |
| `yew023` | | yew 0.23 `IntoPropValue<VNode>` for all three, `IntoPropValue<Option<AttrValue>>` for [`Country`] (its name) and [`Locale`] (its code), and `From<Country>` for `Key` (implies `alloc`) |

With `default-features = false`, a crate names every feature it needs,
`std` included: one that needs only countries takes
`features = ["std", "country"]` and compiles none of the language table,
and a `no_std` crate leaves out `std`, or names `alloc` alone.

<br>

## Using it

```toml
[dependencies]
koine = "0.1.0"
# Only the countries, with serde and the standard library.
koine = { version = "0.1.0", default-features = false, features = ["std", "country", "serde"] }
```

<br>

## Data sources

The tables are generated from data files vendored in the repository,
which a tools crate there downloads and regenerates the tables from. The
code of this crate is MIT OR Apache-2.0; the data each table is generated
from keeps its own terms:

* **Languages** (`language`): the ISO 639-3 code tables of
  [SIL International], the ISO 639-3 registration authority, from
  [iso639-3.sil.org]. The codes, names and classifications are SIL's.
  The downloaded file, `data/language/iso-639-3.tab`, is kept in the
  repository for regeneration and is not part of the published package.
  The codes are used under SIL's ISO 639-3 Code Tables Terms of Use,
  published on [iso639-3.sil.org], which is credited here as the source
  of the codes; the identifiers are used as published, neither modified
  nor extended.
* **Countries and locales** (`country`, `locale`): [Unicode CLDR]
  release 48.2, from [unicode.org], under the [Unicode License v3]
  (`Unicode-3.0`), whose text ships with this crate as
  `data/cldr/LICENSE`. The countries are the regular CLDR regions with an
  ISO 3166-1 alpha-3 and numeric code, with their English names and
  their UN M49 regions from CLDR's territory containment. The locales are
  each language and country combined by a locale in CLDR's `common/main`.
* **Language flags** ([`language::Entry::flag`]):
  `data/language/to-3166.csv`, a mapping from each language to the
  country whose flag can stand for it. It is written and maintained by
  hand in this repository, best-effort and not authoritative, and is
  licensed with the crate (MIT OR Apache-2.0). It is an addition of this
  crate's and not part of the ISO 639-3 standard.

CLDR data is Copyright © Unicode, Inc.; the full copyright and permission
notice is in `data/cldr/LICENSE`.

[SIL International]: https://www.sil.org
[iso639-3.sil.org]: https://iso639-3.sil.org
[Unicode CLDR]: https://cldr.unicode.org
[unicode.org]: https://unicode.org/Public/cldr/
[Unicode License v3]: https://www.unicode.org/license.txt

<br>

## Updating the data

In a checkout of the repository, `cargo run -p tools -- download` fetches
the ISO 639-3 table and the pinned CLDR release, and `cargo run -p tools`
regenerates every table from the vendored files; then review the diff.
`cargo run -p tools -- --check` fails if a table is out of date. The
CLDR release is pinned in `tools/src/cldr.rs`. The hand-maintained
`to-3166.csv` is never written by the tools.

[`Country`]: https://docs.rs/koine/latest/koine/country/struct.Country.html
[`country`]: https://docs.rs/koine/latest/koine/country/index.html
[`Language`]: https://docs.rs/koine/latest/koine/language/struct.Language.html
[`language`]: https://docs.rs/koine/latest/koine/language/index.html
[`language::Entry::flag`]: https://docs.rs/koine/latest/koine/language/struct.Entry.html#structfield.flag
[`Locale`]: https://docs.rs/koine/latest/koine/locale/struct.Locale.html
[`locale`]: https://docs.rs/koine/latest/koine/locale/index.html
[`Locale::to_u64`]: https://docs.rs/koine/latest/koine/locale/struct.Locale.html#method.to_u64
