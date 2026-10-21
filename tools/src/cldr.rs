//! The Unicode CLDR release koine's country and locale tables come from: what
//! is vendored out of it, and reading those files.
//!
//! Only the handful of files the tables need are vendored, under `data/cldr/`
//! at the same paths they have in the release, plus the release's license:
//!
//! * `common/validity/region.xml`: which region codes are regular, and which
//!   are macro-regions, private use, deprecated or special.
//! * `common/supplemental/supplementalData.xml`: the alpha-3 and numeric code
//!   of each region (`codeMappings`) and the UN M49 containment tree
//!   (`territoryContainment`).
//! * `common/main/en.xml`: the English names of countries and regions.
//! * `main-locales.txt`: the names of the locales in `common/main`, one per
//!   line. It is the release's file listing, not a file of the release:
//!   the locale table needs the names alone, not the 1000-odd files.

use std::collections::{BTreeMap, BTreeSet};
use std::io::{Cursor, Read as _};

use anyhow::{Context, Result, anyhow, bail};
use quick_xml::XmlVersion;
use quick_xml::events::Event;

/// The pinned CLDR release. Bumping it is: change this, run
/// `cargo run -p tools -- download-cldr`, regenerate, review the diff.
pub const VERSION: &str = "48.2";

/// Where the pinned release is downloaded from.
pub fn url() -> String {
    format!("https://unicode.org/Public/cldr/{VERSION}/cldr-common-{VERSION}.zip")
}

/// The files vendored verbatim, by their path in the release.
pub const VENDORED: &[&str] = &[REGION_VALIDITY, SUPPLEMENTAL_DATA, ENGLISH];

pub const REGION_VALIDITY: &str = "common/validity/region.xml";
pub const SUPPLEMENTAL_DATA: &str = "common/supplemental/supplementalData.xml";
pub const ENGLISH: &str = "common/main/en.xml";

/// The listing of `common/main`, written by the download.
pub const MAIN_LOCALES: &str = "main-locales.txt";

/// The vendored copy of the release's license (Unicode-3.0).
pub const LICENSE: &str = "LICENSE";

/// The version the vendored files were taken from.
pub const VERSION_FILE: &str = "VERSION";

/// Names the release's license file may have at its root.
const LICENSE_NAMES: &[&str] = &[
    "unicode-license.txt",
    "LICENSE",
    "LICENSE.txt",
    "license.txt",
    "LICENSE.md",
];

/// Pull the vendored files out of a downloaded release archive, as pairs of
/// a path relative to `data/cldr/` and its contents.
pub fn extract(zip: &[u8]) -> Result<Vec<(String, String)>> {
    let mut archive = zip::ZipArchive::new(Cursor::new(zip)).context("Opening the CLDR archive")?;

    let names = archive
        .file_names()
        .map(|name| Ok(name?.into_owned()))
        .collect::<Result<BTreeSet<_>>>()
        .context("Reading CLDR archive file names")?;

    // The archive may or may not put everything in a top-level directory.
    let prefix = names
        .iter()
        .find_map(|name| name.strip_suffix(REGION_VALIDITY))
        .with_context(|| anyhow!("The archive has no {REGION_VALIDITY}"))?
        .to_owned();

    let mut files = Vec::new();

    for path in VENDORED {
        files.push((
            path.to_string(),
            read(&mut archive, &format!("{prefix}{path}"))?,
        ));
    }

    let license = LICENSE_NAMES
        .iter()
        .map(|name| format!("{prefix}{name}"))
        .find(|name| names.contains(name))
        .with_context(|| {
            anyhow!("The archive has no license file, looked for {LICENSE_NAMES:?}")
        })?;

    files.push((LICENSE.to_owned(), read(&mut archive, &license)?));

    let main = format!("{prefix}common/main/");
    let mut listing = format!(
        "# The locales in common/main of CLDR {VERSION} ({}).\n\
         # Written by `cargo run -p tools -- download-cldr`; do not edit by hand.\n",
        url()
    );

    for name in &names {
        if let Some(stem) = name
            .strip_prefix(&main)
            .and_then(|name| name.strip_suffix(".xml"))
            && !stem.contains('/')
        {
            listing.push_str(stem);
            listing.push('\n');
        }
    }

    files.push((MAIN_LOCALES.to_owned(), listing));
    files.push((VERSION_FILE.to_owned(), format!("{VERSION}\n")));
    Ok(files)
}

fn read(archive: &mut zip::ZipArchive<Cursor<&[u8]>>, name: &str) -> Result<String> {
    let mut file = archive
        .by_name(name)
        .with_context(|| anyhow!("The archive has no {name}"))?;

    let mut out = String::new();
    file.read_to_string(&mut out)
        .with_context(|| anyhow!("Reading {name} from the archive"))?;
    Ok(out)
}

/// A UN M49 region, as it appears in the generated table.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Region {
    pub code: u16,
    pub name: String,
}

/// One ISO 3166-1 country, as it appears in the generated table.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Country {
    pub alpha2: String,
    pub alpha3: String,
    pub numeric: u16,
    pub name: String,
    pub region: Option<Region>,
    pub sub_region: Option<Region>,
}

/// Build the ISO 3166-1 country table, ordered by alpha-2 code.
///
/// A country is a CLDR region whose validity is `regular` and which has both
/// an alpha-3 and a numeric code below 900 in `codeMappings`. That leaves out
/// the macro-regions, private-use, deprecated and special codes, the
/// exceptionally reserved codes CLDR lists as regular (such as `AC` and `IC`,
/// which have no numeric code), and user-assigned codes such as `XK`.
pub fn countries(region_xml: &str, supplemental_xml: &str, en_xml: &str) -> Result<Vec<Country>> {
    let validity = region_validity(region_xml)?;
    let mappings = code_mappings(supplemental_xml)?;
    let groups = containment(supplemental_xml)?;
    let names = territory_names(en_xml)?;

    let regular = validity
        .get("regular")
        .context("region.xml lists no regular regions")?;

    // The primary (not grouping, not deprecated) group directly containing
    // each region.
    let mut parent = BTreeMap::<&str, &str>::new();

    for group in groups.iter().filter(|g| g.primary) {
        for child in &group.contains {
            if let Some(existing) = parent.insert(child, &group.code) {
                bail!(
                    "{child} is in two primary containment groups, {existing} and {}",
                    group.code
                );
            }
        }
    }

    let region = |code: &str| -> Result<Option<Region>> {
        let Ok(numeric) = code.parse::<u16>() else {
            return Ok(None);
        };

        let name = names
            .get(code)
            .with_context(|| anyhow!("en.xml has no name for region {code}"))?;

        Ok(Some(Region {
            code: numeric,
            name: name.clone(),
        }))
    };

    let mut out = Vec::new();

    for alpha2 in regular {
        let Some(mapping) = mappings.get(alpha2) else {
            continue;
        };

        let (Some(alpha3), Some(numeric)) = (&mapping.alpha3, mapping.numeric) else {
            continue;
        };

        if numeric >= 900 {
            continue;
        }

        let name = names
            .get(alpha2)
            .with_context(|| anyhow!("en.xml has no name for {alpha2}"))?;

        // Country -> sub-region -> region -> world (001). A country directly
        // under a region has no sub-region; one under a non-M49 group, such as
        // Antarctica under Outlying Oceania (QO), has neither.
        let (region_code, sub_region_code) = match parent.get(alpha2.as_str()) {
            Some(&group) if group.parse::<u16>().is_ok() => match parent.get(group) {
                Some(&"001") => (Some(group), None),
                Some(&upper) => (Some(upper), Some(group)),
                None => (None, Some(group)),
            },
            _ => (None, None),
        };

        out.push(Country {
            alpha2: alpha2.clone(),
            alpha3: alpha3.clone(),
            numeric,
            name: name.clone(),
            region: region_code.map(region).transpose()?.flatten(),
            sub_region: sub_region_code.map(region).transpose()?.flatten(),
        });
    }

    if out.is_empty() {
        bail!("No ISO 3166-1 countries found in the CLDR data");
    }

    Ok(out)
}

/// One line of `main-locales.txt` that names a language and a territory.
#[derive(Debug, PartialEq, Eq)]
pub struct MainLocale<'a> {
    pub language: &'a str,
    pub territory: &'a str,
}

/// The locales in `main-locales.txt` that have a territory subtag, such as
/// `en_US` or `sr_Latn_RS` (script and variant subtags are dropped). Locales
/// without a territory, `root`, and those whose territory is a numeric
/// macro-region such as `es_419` are left out.
pub fn main_locales(listing: &str) -> Vec<MainLocale<'_>> {
    let mut out = Vec::new();

    for line in listing.lines() {
        let line = line.trim();

        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let mut parts = line.split('_');

        let Some(language) = parts.next() else {
            continue;
        };

        if language == "root" {
            continue;
        }

        // The subtags after the language are an optional 4-letter script, then
        // an optional region, then variants.
        let territory = parts
            .find(|part| part.len() != 4)
            .filter(|part| part.len() == 2 && part.bytes().all(|b| b.is_ascii_uppercase()));

        if let Some(territory) = territory {
            out.push(MainLocale {
                language,
                territory,
            });
        }
    }

    out
}

/// Region codes by validity status (`regular`, `macroregion`, `deprecated`,
/// ...), with `~` ranges expanded.
pub fn region_validity(xml: &str) -> Result<BTreeMap<String, BTreeSet<String>>> {
    let mut out = BTreeMap::<String, BTreeSet<String>>::new();

    for element in elements(xml)? {
        if element.name != "id" || element.attr("type") != Some("region") {
            continue;
        }

        let status = element
            .attr("idStatus")
            .context("A region validity entry without idStatus")?;

        let codes = out.entry(status.to_owned()).or_default();

        for token in element.text.split_whitespace() {
            expand_range(token, codes)?;
        }
    }

    Ok(out)
}

/// Expand `AC~G` into `AC AD AE AF AG`, and `001~3` into `001 002 003`.
fn expand_range(token: &str, out: &mut BTreeSet<String>) -> Result<()> {
    let Some((start, end)) = token.split_once('~') else {
        out.insert(token.to_owned());
        return Ok(());
    };

    let ([first @ .., from], [to]) = (start.as_bytes(), end.as_bytes()) else {
        bail!("Unsupported range in region.xml: {token:?}");
    };

    if from > to {
        bail!("Backwards range in region.xml: {token:?}");
    }

    for last in *from..=*to {
        let mut code = first.to_vec();
        code.push(last);
        out.insert(String::from_utf8(code).context("A non-ASCII range")?);
    }

    Ok(())
}

/// The other codes of a region, from `codeMappings`.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct CodeMapping {
    pub alpha3: Option<String>,
    pub numeric: Option<u16>,
}

/// Each region's alpha-3 and numeric codes, from the `territoryCodes`
/// entries of `supplementalData.xml`.
pub fn code_mappings(xml: &str) -> Result<BTreeMap<String, CodeMapping>> {
    let mut out = BTreeMap::new();

    for element in elements(xml)? {
        if element.name != "territoryCodes" {
            continue;
        }

        let code = element
            .attr("type")
            .context("A territoryCodes entry without type")?;

        let numeric = element
            .attr("numeric")
            .filter(|n| !n.is_empty())
            .map(|n| {
                n.parse::<u16>()
                    .with_context(|| anyhow!("Numeric code {n:?} of {code}"))
            })
            .transpose()?;

        out.insert(
            code.to_owned(),
            CodeMapping {
                alpha3: element
                    .attr("alpha3")
                    .filter(|a| !a.is_empty())
                    .map(str::to_owned),
                numeric,
            },
        );
    }

    Ok(out)
}

/// One `territoryContainment` group.
#[derive(Debug, PartialEq, Eq)]
pub struct Group {
    pub code: String,
    pub contains: Vec<String>,
    /// Neither a grouping (such as the EU or Latin America) nor deprecated:
    /// part of the strict M49 tree.
    pub primary: bool,
}

/// The `territoryContainment` groups of `supplementalData.xml`.
pub fn containment(xml: &str) -> Result<Vec<Group>> {
    let mut out = Vec::new();

    for element in elements(xml)? {
        if element.name != "group" || element.parent != "territoryContainment" {
            continue;
        }

        let code = element.attr("type").context("A group without type")?;
        let contains = element.attr("contains").unwrap_or_default();

        out.push(Group {
            code: code.to_owned(),
            contains: contains.split_whitespace().map(str::to_owned).collect(),
            primary: element.attr("status").is_none() && element.attr("grouping").is_none(),
        });
    }

    Ok(out)
}

/// The English name of each territory, from `en.xml`, leaving out the `alt`
/// forms (short names, variants).
pub fn territory_names(xml: &str) -> Result<BTreeMap<String, String>> {
    let mut out = BTreeMap::new();

    for element in elements(xml)? {
        if element.name != "territory"
            || element.parent != "territories"
            || element.attr("alt").is_some()
        {
            continue;
        }

        let code = element.attr("type").context("A territory without type")?;
        out.insert(code.to_owned(), element.text.trim().to_owned());
    }

    Ok(out)
}

/// A leaf-level view of one XML element: what the parsers here need.
struct Element {
    name: String,
    parent: String,
    attrs: Vec<(String, String)>,
    /// The text directly inside the element, entities resolved.
    text: String,
}

impl Element {
    fn attr(&self, name: &str) -> Option<&str> {
        self.attrs
            .iter()
            .find(|(key, _)| key == name)
            .map(|(_, value)| value.as_str())
    }
}

/// Every element in `xml` with its attributes and own text, in document
/// order of their end.
fn elements(xml: &str) -> Result<Vec<Element>> {
    let mut reader = quick_xml::Reader::from_str(xml);
    let mut stack = Vec::<Element>::new();
    let mut out = Vec::new();

    loop {
        let event = reader
            .read_event()
            .with_context(|| anyhow!("Parsing XML at byte {}", reader.buffer_position()))?;

        let empty = matches!(event, Event::Empty(_));

        match event {
            Event::Start(start) | Event::Empty(start) => {
                let parent = stack.last().map(|e| e.name.clone()).unwrap_or_default();
                let name = start.name().as_ref().to_owned();

                let mut attrs = Vec::new();

                for attr in start.attributes() {
                    let attr = attr?;
                    attrs.push((
                        attr.key.as_ref().to_owned(),
                        attr.normalized_value(XmlVersion::Implicit1_0)?.into_owned(),
                    ));
                }

                let element = Element {
                    name,
                    parent,
                    attrs,
                    text: String::new(),
                };

                if empty {
                    out.push(element);
                } else {
                    stack.push(element);
                }
            }
            Event::End(_) => {
                out.push(stack.pop().context("Unbalanced XML")?);
            }
            Event::Text(text) => {
                if let Some(element) = stack.last_mut() {
                    element.text.push_str(&text);
                }
            }
            Event::CData(data) => {
                if let Some(element) = stack.last_mut() {
                    element.text.push_str(&data);
                }
            }
            Event::GeneralRef(reference) => {
                let Some(element) = stack.last_mut() else {
                    continue;
                };

                if let Some(c) = reference.resolve_char_ref()? {
                    element.text.push(c);
                    continue;
                }

                let c = match &*reference {
                    "amp" => '&',
                    "lt" => '<',
                    "gt" => '>',
                    "quot" => '"',
                    "apos" => '\'',
                    other => bail!("Unknown XML entity &{other};"),
                };

                element.text.push(c);
            }
            Event::Eof => break,
            _ => {}
        }
    }

    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ranges_expand() {
        let mut out = BTreeSet::new();
        expand_range("AC~E", &mut out).unwrap();
        expand_range("001~3", &mut out).unwrap();
        expand_range("ZW", &mut out).unwrap();

        let expected = ["001", "002", "003", "AC", "AD", "AE", "ZW"];
        assert_eq!(out.iter().map(String::as_str).collect::<Vec<_>>(), expected);
    }

    #[test]
    fn main_locales_keep_those_with_a_territory() {
        let listing = "# comment\nroot\nen\nen_US\nen_001\nes_419\nsr_Latn_RS\nca_ES_VALENCIA\nyue_Hant_HK\nzh_Hans\n";

        let found = main_locales(listing)
            .into_iter()
            .map(|l| (l.language, l.territory))
            .collect::<Vec<_>>();

        assert_eq!(
            found,
            [("en", "US"), ("sr", "RS"), ("ca", "ES"), ("yue", "HK")]
        );
    }

    #[test]
    fn entities_in_names_are_resolved() {
        let xml = r#"<ldml><localeDisplayNames><territories>
            <territory type="BA">Bosnia &amp; Herzegovina</territory>
            <territory type="BA" alt="short">Bosnia</territory>
            <territory type="CI">C&#xF4;te d’Ivoire</territory>
        </territories></localeDisplayNames></ldml>"#;

        let names = territory_names(xml).unwrap();
        assert_eq!(names["BA"], "Bosnia & Herzegovina");
        assert_eq!(names["CI"], "Côte d’Ivoire");
    }
}
