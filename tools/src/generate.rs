//! Turning the vendored source data into the generated table modules.
//!
//! Every function here is pure: it takes the contents of the data files and
//! returns the module source, so the same inputs always produce the same bytes.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;

use anyhow::{Context, Result, anyhow, bail};
use serde::{Deserialize, Serialize};

use crate::cldr::{self, Country};

/// Where `data/language/iso-639-3.tab` is downloaded from (SIL International,
/// the ISO 639-3 registration authority).
pub const LANGUAGES_URL: &str =
    "https://iso639-3.sil.org/sites/iso639-3/files/downloads/iso-639-3.tab";

/// One row of the optional `--mapping-path` CSV: the flag each language ended
/// up with.
#[derive(Serialize)]
pub struct Mapping {
    #[serde(rename = "iso-639-3")]
    id: String,
    #[serde(rename = "iso-639-1")]
    part1: Option<String>,
    #[serde(rename = "iso-3166-1")]
    flag: Option<String>,
}

#[derive(Debug)]
struct LanguageRow {
    id: String,
    part2b: Option<String>,
    part2t: Option<String>,
    part1: Option<String>,
    scope: Scope,
    ty: Type,
    name: String,
    comment: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Scope {
    Individual,
    Macrolanguage,
    Special,
}

impl Scope {
    fn parse(value: &str) -> Result<Self> {
        match value {
            "I" => Ok(Self::Individual),
            "M" => Ok(Self::Macrolanguage),
            "S" => Ok(Self::Special),
            _ => bail!("Unknown scope code: {value}"),
        }
    }

    fn rust_variant(self) -> &'static str {
        match self {
            Self::Individual => "Individual",
            Self::Macrolanguage => "Macrolanguage",
            Self::Special => "Special",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Type {
    Living,
    Extinct,
    Historical,
    Constructed,
    Special,
}

impl Type {
    fn parse(value: &str) -> Result<Self> {
        match value {
            "L" => Ok(Self::Living),
            "E" => Ok(Self::Extinct),
            "H" => Ok(Self::Historical),
            "C" => Ok(Self::Constructed),
            "S" => Ok(Self::Special),
            _ => bail!("Unknown language type code: {value}"),
        }
    }

    fn rust_variant(self) -> &'static str {
        match self {
            Self::Living => "Living",
            Self::Extinct => "Extinct",
            Self::Historical => "Historical",
            Self::Constructed => "Constructed",
            Self::Special => "Special",
        }
    }
}

#[derive(Debug, Deserialize)]
struct RawLanguageRow {
    #[serde(rename = "Id")]
    id: String,
    #[serde(rename = "Part2b", deserialize_with = "deserialize_optional")]
    part2b: Option<String>,
    #[serde(rename = "Part2t", deserialize_with = "deserialize_optional")]
    part2t: Option<String>,
    #[serde(rename = "Part1", deserialize_with = "deserialize_optional")]
    part1: Option<String>,
    #[serde(rename = "Scope")]
    scope: String,
    #[serde(rename = "Language_Type")]
    ty: String,
    #[serde(rename = "Ref_Name")]
    name: String,
    #[serde(rename = "Comment", deserialize_with = "deserialize_optional")]
    comment: Option<String>,
}

/// A row of the hand-maintained ISO 639 -> ISO 3166-1 mapping: which country's
/// flag stands for a language.
#[derive(Debug, Deserialize)]
struct FlagRow {
    #[serde(rename = "iso-639-3")]
    iso_639_3: String,
    #[serde(rename = "iso-639-1", default)]
    iso_639_1: Option<String>,
    #[serde(rename = "iso-3166-1")]
    iso_3166_1: String,
}

/// Generate `src/language/generated.rs`: `is_valid_id`, the `ENTRIES` table
/// and its `BY_ID` and `BY_PART1` indexes.
///
/// Each entry's `flag` comes from the `to_3166` mapping, kept only when it
/// names a country in `valid_countries` (the ISO 3166-1 table this crate
/// ships), so a flag always resolves in `koine::country`.
pub fn language_module(
    tab: &str,
    to_3166: &str,
    valid_countries: &BTreeSet<String>,
) -> Result<(String, Vec<Mapping>)> {
    let entries = parse_rows(tab)?;
    let (flags1, flags3) = flag_map(to_3166, valid_countries)?;

    let mut o = String::new();
    write_header(
        &mut o,
        &[
            &format!("data/language/iso-639-3.tab, downloaded from {LANGUAGES_URL}"),
            "data/language/to-3166.csv, maintained by hand in this repository",
        ],
    )?;
    writeln!(o, "use super::{{Entry, Scope, Type}};")?;
    writeln!(o)?;

    writeln!(o, "pub(super) const fn is_valid_id(id: &[u8]) -> bool {{")?;
    writeln!(o, "    match id {{")?;

    for e in &entries {
        writeln!(o, "        b\"{}\" => true,", e.id)?;
    }

    writeln!(o, "        _ => false,")?;
    writeln!(o, "    }}")?;
    writeln!(o, "}}")?;
    writeln!(o)?;

    let mapping = write_entries(&mut o, &entries, &flags1, &flags3)?;
    language_maps(&mut o, &entries)?;
    Ok((o, mapping))
}

/// Build the ISO 639-1 and ISO 639-3 -> ISO 3166-1 flag maps, keeping only
/// rows whose country is in `valid_countries`.
fn flag_map(
    csv: &str,
    valid_countries: &BTreeSet<String>,
) -> Result<(BTreeMap<String, String>, BTreeMap<String, String>)> {
    let mut map1 = BTreeMap::new();
    let mut map3 = BTreeMap::new();

    let mut reader = csv::Reader::from_reader(csv.as_bytes());

    for (index, row) in reader.deserialize::<FlagRow>().enumerate() {
        let row = row.with_context(|| anyhow!("Reading mapping entry #{index}"))?;
        let country = row.iso_3166_1.trim().to_ascii_uppercase();

        if country.is_empty() {
            continue;
        }

        if !valid_countries.contains(&country) {
            bail!(
                "to-3166.csv maps {} to {country:?}, which is not an ISO 3166-1 country in the vendored CLDR data",
                row.iso_639_3
            );
        }

        if let Some(part1) = &row.iso_639_1 {
            map1.insert(part1.to_ascii_lowercase(), country.clone());
        }

        map3.insert(row.iso_639_3.to_ascii_lowercase(), country);
    }

    Ok((map1, map3))
}

fn write_entries(
    out: &mut String,
    entries: &[LanguageRow],
    flags1: &BTreeMap<String, String>,
    flags3: &BTreeMap<String, String>,
) -> Result<Vec<Mapping>> {
    let mut mapping = Vec::new();

    writeln!(out, "pub(super) const ENTRIES: &[Entry] = &[")?;

    for row in entries {
        let flag1 = row
            .part1
            .as_deref()
            .and_then(|id| flags1.get(id))
            .map(String::as_str);

        // `UN` is reserved for the United Nations, which speaks no language,
        // so its flag stands for nothing here. No row in `to-3166.csv` maps
        // to it today, and this is what would keep one out.
        let flag3 = flags3
            .get(&row.id)
            .map(String::as_str)
            .filter(|flag| !flag.eq_ignore_ascii_case("UN"));

        let flag = flag1
            .or(flag3)
            .filter(|flag| !flag.eq_ignore_ascii_case("UN"));

        mapping.push(Mapping {
            id: row.id.clone(),
            part1: row.part1.clone(),
            flag: flag.map(str::to_owned),
        });

        writeln!(out, "    Entry {{")?;
        writeln!(out, "        id: {:?},", row.id)?;
        writeln!(out, "        part2b: {},", optional(row.part2b.as_deref()))?;
        writeln!(out, "        part2t: {},", optional(row.part2t.as_deref()))?;
        writeln!(out, "        part1: {},", optional(row.part1.as_deref()))?;
        writeln!(out, "        flag: {},", optional(flag3))?;
        writeln!(out, "        scope: Scope::{},", row.scope.rust_variant())?;
        writeln!(out, "        ty: Type::{},", row.ty.rust_variant())?;
        writeln!(out, "        name: {:?},", row.name)?;
        writeln!(
            out,
            "        comment: {},",
            optional(row.comment.as_deref())
        )?;
        writeln!(out, "    }},")?;
    }

    writeln!(out, "];")?;
    writeln!(out)?;
    Ok(mapping)
}

fn language_maps(out: &mut String, rows: &[LanguageRow]) -> Result<()> {
    let mut by_id = phf_codegen::Map::new();
    let mut by_part1 = phf_codegen::Map::new();

    for (i, row) in rows.iter().enumerate() {
        by_id.entry(&row.id, i.to_string());

        if let Some(part1) = &row.part1 {
            by_part1.entry(part1, i.to_string());
        }
    }

    writeln!(
        out,
        "pub(super) static BY_PART1: phf::Map<&'static str, usize> = {};",
        by_part1.build()
    )?;

    writeln!(
        out,
        "pub(super) static BY_ID: phf::Map<&'static str, usize> = {};",
        by_id.build()
    )?;

    Ok(())
}

/// Generate `src/country/generated.rs`: `is_valid_alpha2`, the `ENTRIES`
/// table ordered by alpha-2 code, and its `BY_ALPHA2` and `BY_ALPHA3`
/// indexes.
pub fn country_module(countries: &[Country]) -> Result<String> {
    let mut o = String::new();
    write_header(
        &mut o,
        &[&format!(
            "data/cldr/{{{}, {}, {}}}, from Unicode CLDR {} ({})",
            cldr::REGION_VALIDITY,
            cldr::SUPPLEMENTAL_DATA,
            cldr::ENGLISH,
            cldr::VERSION,
            cldr::url()
        )],
    )?;

    writeln!(o, "use super::{{Entry, Region}};")?;
    writeln!(o)?;
    writeln!(
        o,
        "pub(super) const fn is_valid_alpha2(alpha2: &[u8]) -> bool {{"
    )?;
    writeln!(o, "    match alpha2 {{")?;

    for c in countries {
        writeln!(o, "        b\"{}\" => true,", c.alpha2)?;
    }

    writeln!(o, "        _ => false,")?;
    writeln!(o, "    }}")?;
    writeln!(o, "}}")?;
    writeln!(o)?;
    writeln!(o, "pub(super) const ENTRIES: &[Entry] = &[")?;

    for c in countries {
        writeln!(o, "    Entry {{")?;
        writeln!(o, "        alpha2: {:?},", c.alpha2)?;
        writeln!(o, "        alpha3: {:?},", c.alpha3)?;
        writeln!(o, "        numeric: {},", c.numeric)?;
        writeln!(o, "        name: {:?},", c.name)?;
        writeln!(o, "        region: {},", region(c.region.as_ref()))?;
        writeln!(o, "        sub_region: {},", region(c.sub_region.as_ref()))?;
        writeln!(o, "    }},")?;
    }

    writeln!(o, "];")?;
    writeln!(o)?;

    let mut by_alpha2 = phf_codegen::Map::new();
    let mut by_alpha3 = phf_codegen::Map::new();

    for (i, c) in countries.iter().enumerate() {
        by_alpha2.entry(c.alpha2.as_str(), i.to_string());
        by_alpha3.entry(c.alpha3.as_str(), i.to_string());
    }

    writeln!(
        o,
        "pub(super) static BY_ALPHA2: phf::Map<&'static str, usize> = {};",
        by_alpha2.build()
    )?;

    writeln!(
        o,
        "pub(super) static BY_ALPHA3: phf::Map<&'static str, usize> = {};",
        by_alpha3.build()
    )?;

    Ok(o)
}

fn region(region: Option<&cldr::Region>) -> String {
    match region {
        Some(r) => format!("Some(Region {{ code: {}, name: {:?} }})", r.code, r.name),
        None => String::from("None"),
    }
}

/// The ISO 639-3 ids of the language table, and the ISO 639-1 code -> id map.
pub struct LanguageCodes {
    ids: BTreeSet<String>,
    by_part1: BTreeMap<String, String>,
}

impl LanguageCodes {
    /// Resolve a CLDR language subtag: ISO 639-1 where one exists, else
    /// ISO 639-3.
    fn resolve(&self, code: &str) -> Option<&str> {
        match code.len() {
            2 => self.by_part1.get(code).map(String::as_str),
            3 => self.ids.get(code).map(String::as_str),
            _ => None,
        }
    }
}

/// The codes of the language dataset.
pub fn language_codes(tab: &str) -> Result<LanguageCodes> {
    let rows = parse_rows(tab)?;

    Ok(LanguageCodes {
        by_part1: rows
            .iter()
            .filter_map(|r| Some((r.part1.clone()?, r.id.clone())))
            .collect(),
        ids: rows.into_iter().map(|r| r.id).collect(),
    })
}

/// Generate `src/locale/generated.rs` (`ENTRIES` and `BY_KEY`): one entry per
/// language and country of the CLDR `common/main` locales that have a
/// territory, where the language is in the ISO 639-3 table and the territory
/// is an ISO 3166-1 country. Script and variant subtags are dropped, so
/// `sr_Cyrl_RS` and `sr_Latn_RS` are one entry, `srp`-`RS`.
pub fn locale_module(
    listing: &str,
    languages: &LanguageCodes,
    countries: &[Country],
) -> Result<String> {
    let countries = countries
        .iter()
        .map(|c| c.alpha2.as_str())
        .collect::<BTreeSet<_>>();

    let mut entries = BTreeSet::new();

    for locale in cldr::main_locales(listing) {
        let Some(language) = languages.resolve(locale.language) else {
            continue;
        };

        if !countries.contains(locale.territory) {
            continue;
        }

        entries.insert((language.to_owned(), locale.territory.to_owned()));
    }

    if entries.is_empty() {
        bail!("No valid locale combinations found");
    }

    let mut out = String::new();
    write_header(
        &mut out,
        &[
            &format!(
                "data/cldr/{}, the locales in common/main of Unicode CLDR {} ({})",
                cldr::MAIN_LOCALES,
                cldr::VERSION,
                cldr::url()
            ),
            "filtered to the languages in data/language/iso-639-3.tab and the ISO 3166-1 countries",
        ],
    )?;
    writeln!(out, "use super::Entry;")?;
    writeln!(out)?;
    writeln!(out, "pub(super) const ENTRIES: &[Entry] = &[")?;

    for (language, country) in &entries {
        writeln!(
            out,
            "    Entry {{ language: {language:?}, country: {country:?} }},"
        )?;
    }

    writeln!(out, "];")?;
    writeln!(out)?;

    let keys = entries
        .iter()
        .map(|(language, country)| format!("{language}-{country}"))
        .collect::<Vec<_>>();

    let mut by_key = phf_codegen::Map::new();

    for (i, key) in keys.iter().enumerate() {
        by_key.entry(key.as_str(), i.to_string());
    }

    writeln!(
        out,
        "pub(super) static BY_KEY: phf::Map<&'static str, usize> = {};",
        by_key.build()
    )?;

    Ok(out)
}

/// Check that a downloaded ISO 639-3 table parses before it replaces the
/// vendored one.
pub fn validate_languages(tab: &str) -> Result<()> {
    parse_rows(tab).map(drop)
}

fn parse_rows(input: &str) -> Result<Vec<LanguageRow>> {
    if input.trim().is_empty() {
        bail!("Dataset is empty");
    }

    let mut reader = csv::ReaderBuilder::new()
        .delimiter(b'\t')
        .from_reader(input.trim_start_matches('\u{feff}').as_bytes());

    let mut rows = Vec::new();

    for (index, row) in reader.deserialize::<RawLanguageRow>().enumerate() {
        let line_no = index + 2;
        let row = row.with_context(|| anyhow!("Parsing row at line {line_no}"))?;

        let id = row.id.trim();
        let scope = row.scope.trim();
        let ty = row.ty.trim();
        let name = row.name.trim();

        if id.len() != 3 || !id.bytes().all(|b| b.is_ascii_lowercase()) {
            bail!("Invalid language id at line {line_no}: {id:?}");
        }

        if name.is_empty() {
            bail!("Empty ref name at line {line_no}");
        }

        let scope =
            Scope::parse(scope).with_context(|| anyhow!("Invalid scope at line {line_no}"))?;
        let ty =
            Type::parse(ty).with_context(|| anyhow!("Invalid language type at line {line_no}"))?;

        rows.push(LanguageRow {
            id: id.to_owned(),
            part2b: row.part2b,
            part2t: row.part2t,
            part1: row.part1,
            scope,
            ty,
            name: name.to_owned(),
            comment: row.comment,
        });
    }

    if rows.is_empty() {
        bail!("Dataset has no data rows");
    }

    rows.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(rows)
}

fn deserialize_optional<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = Option::<String>::deserialize(deserializer)?;

    Ok(value.and_then(|v| {
        let v = v.trim();
        (!v.is_empty()).then(|| v.to_owned())
    }))
}

fn write_header(out: &mut String, sources: &[&str]) -> Result<()> {
    writeln!(
        out,
        "// @generated by `cargo run -p tools`; do not edit by hand."
    )?;

    for source in sources {
        writeln!(out, "// Source: {source}")?;
    }

    writeln!(out)?;
    Ok(())
}

/// A Rust expression for an optional string literal.
fn optional(value: Option<&str>) -> String {
    match value {
        Some(value) => format!("Some({value:?})"),
        None => String::from("None"),
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::{LanguageRow, Scope, Type, write_entries};

    fn row(id: &str) -> LanguageRow {
        LanguageRow {
            id: id.to_owned(),
            part2b: None,
            part2t: None,
            part1: None,
            scope: Scope::Individual,
            ty: Type::Living,
            name: "Test".to_owned(),
            comment: None,
        }
    }

    /// **A language mapped to `UN` is given no flag at all.** No row in
    /// `to-3166.csv` maps to it today, so nothing in the committed data
    /// reaches the guard and deleting it would leave every test green, which
    /// is what makes it worth a test rather than a comment alone. `UN` is
    /// reserved for the United Nations, which speaks no language.
    #[test]
    fn a_language_is_never_given_the_un_flag() {
        let entries = [row("qaa"), row("qab")];

        let flags3 = BTreeMap::from([
            ("qaa".to_owned(), "UN".to_owned()),
            ("qab".to_owned(), "SE".to_owned()),
        ]);

        let mut out = String::new();
        let mapping = write_entries(&mut out, &entries, &BTreeMap::new(), &flags3).unwrap();

        assert_eq!(mapping[0].flag, None, "a UN row must yield no flag");
        assert_eq!(mapping[1].flag.as_deref(), Some("SE"));
        assert!(
            !out.contains("\"UN\""),
            "the table must not carry UN either"
        );
    }

    const FIXTURE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/fixtures/cldr");

    fn fixture(path: &str) -> String {
        std::fs::read_to_string(format!("{FIXTURE}/{path}")).unwrap()
    }

    fn fixture_countries() -> Vec<crate::cldr::Country> {
        crate::cldr::countries(
            &fixture(crate::cldr::REGION_VALIDITY),
            &fixture(crate::cldr::SUPPLEMENTAL_DATA),
            &fixture(crate::cldr::ENGLISH),
        )
        .unwrap()
    }

    /// Regular regions with an alpha-3 and an ISO numeric code are countries;
    /// `AC` and `IC` (no numeric code) and `XK` (user-assigned 983) are not.
    #[test]
    fn countries_from_cldr() {
        let countries = fixture_countries();

        let codes = countries
            .iter()
            .map(|c| c.alpha2.as_str())
            .collect::<Vec<_>>();

        assert_eq!(
            codes,
            [
                "AD", "AQ", "BA", "CI", "DE", "FR", "GB", "JP", "NG", "SE", "US"
            ]
        );

        let find = |code: &str| countries.iter().find(|c| c.alpha2 == code).unwrap();

        let ba = find("BA");
        assert_eq!(ba.name, "Bosnia & Herzegovina");
        assert_eq!(ba.alpha3, "BIH");
        assert_eq!(ba.numeric, 70);
        assert_eq!(ba.region.as_ref().unwrap().name, "Europe");
        assert_eq!(ba.sub_region.as_ref().unwrap().code, 39);

        // Grouping containment (Sub-Saharan Africa, 202) is not the M49 tree.
        let ng = find("NG");
        assert_eq!(ng.region.as_ref().unwrap().code, 2);
        assert_eq!(ng.sub_region.as_ref().unwrap().name, "Western Africa");

        // Antarctica sits under Outlying Oceania (QO), which is not M49.
        let aq = find("AQ");
        assert_eq!(aq.region, None);
        assert_eq!(aq.sub_region, None);

        assert_eq!(find("US").name, "United States");
    }

    #[test]
    fn locales_from_cldr() {
        let tab = std::fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../data/language/iso-639-3.tab"
        ))
        .unwrap();

        let languages = super::language_codes(&tab).unwrap();
        let module = super::locale_module(
            &fixture("main-locales.txt"),
            &languages,
            &fixture_countries(),
        )
        .unwrap();

        let entries = module
            .lines()
            .filter(|line| line.trim_start().starts_with("Entry {"))
            .map(str::trim)
            .collect::<Vec<_>>();

        let expected = [
            ("bos", "BA"),
            ("cat", "AD"),
            ("deu", "DE"),
            ("eng", "GB"),
            ("eng", "US"),
            ("fra", "CI"),
            ("fra", "FR"),
            ("hau", "NG"),
            ("jpn", "JP"),
            ("srp", "BA"),
            ("swe", "SE"),
        ]
        .map(|(l, c)| format!("Entry {{ language: {l:?}, country: {c:?} }},"));

        assert_eq!(entries, expected);
    }
}
