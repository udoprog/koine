//! ISO 3166-1 countries: the [`Country`] value type and the table it is
//! validated against.
//!
//! The table holds every ISO 3166-1 country with its alpha-2, alpha-3 and
//! numeric codes, English name, and UN M49 region and sub-region, from
//! [Unicode CLDR]. See [Data sources](crate#data-sources).
//!
//! [Unicode CLDR]: https://cldr.unicode.org
//!
//! ```
//! use koine::Country;
//!
//! let sweden: Country = "se".parse()?;
//! assert_eq!(sweden.to_string(), "SE");
//!
//! let entry = sweden.to_iso().unwrap();
//! assert_eq!(entry.name, "Sweden");
//! assert_eq!(entry.alpha3, "SWE");
//! assert_eq!(entry.numeric, 752);
//! assert_eq!(entry.region.map(|r| r.name), Some("Europe"));
//! assert_eq!(entry.sub_region.map(|r| r.name), Some("Northern Europe"));
//! # Ok::<_, koine::country::ParseCountryErr>(())
//! ```

use core::fmt;

/// A single ISO 3166-1 country in the table.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub struct Entry {
    /// The ISO 3166-1 alpha-2 code, uppercase.
    pub alpha2: &'static str,
    /// The ISO 3166-1 alpha-3 code, uppercase.
    pub alpha3: &'static str,
    /// The ISO 3166-1 numeric code, the same as its UN M49 code. Written with
    /// three digits, as in `format!("{:03}", entry.numeric)`.
    pub numeric: u16,
    /// The English name, as CLDR spells it.
    pub name: &'static str,
    /// The UN M49 region (Africa, Americas, Asia, Europe or Oceania), where
    /// CLDR places the country in one.
    pub region: Option<Region>,
    /// The UN M49 sub-region (for example Northern Europe or Western Africa),
    /// where CLDR places the country in one.
    pub sub_region: Option<Region>,
}

/// A UN M49 region or sub-region.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub struct Region {
    /// The M49 code, written with three digits (`150` for Europe, `011` for
    /// Western Africa).
    pub code: u16,
    /// The English name, as CLDR spells it.
    pub name: &'static str,
}

#[allow(clippy::match_like_matches_macro)]
#[rustfmt::skip]
mod generated;
use generated::ENTRIES;

/// Whether the given bytes are an ISO 3166-1 alpha-2 code in the table (two
/// uppercase ASCII letters).
///
/// ```
/// assert!(koine::country::is_valid_alpha2(b"US"));
/// assert!(!koine::country::is_valid_alpha2(b"us"));
/// assert!(!koine::country::is_valid_alpha2(b"ZZ"));
/// ```
pub const fn is_valid_alpha2(alpha2: &[u8]) -> bool {
    generated::is_valid_alpha2(alpha2)
}

/// Look up a country by its uppercase alpha-2 code.
///
/// The lookup is case-sensitive; pass an uppercase code such as `"US"`, or
/// parse a [`Country`] for any case.
///
/// ```
/// let us = koine::country::by_alpha2("US").unwrap();
/// assert_eq!(us.name, "United States");
///
/// assert!(koine::country::by_alpha2("us").is_none());
/// assert!(koine::country::by_alpha2("ZZ").is_none());
/// ```
pub fn by_alpha2(alpha2: &str) -> Option<&'static Entry> {
    generated::BY_ALPHA2
        .get(alpha2)
        .and_then(|&i| ENTRIES.get(i))
}

/// Look up a country by its uppercase alpha-3 code.
///
/// ```
/// assert_eq!(koine::country::by_alpha3("USA").map(|c| c.alpha2), Some("US"));
/// assert!(koine::country::by_alpha3("usa").is_none());
/// ```
pub fn by_alpha3(alpha3: &str) -> Option<&'static Entry> {
    generated::BY_ALPHA3
        .get(alpha3)
        .and_then(|&i| ENTRIES.get(i))
}

/// Look up a country by its numeric code.
///
/// ```
/// assert_eq!(koine::country::by_numeric(840).map(|c| c.alpha2), Some("US"));
/// assert!(koine::country::by_numeric(150).is_none(), "Europe is no country");
/// ```
pub fn by_numeric(numeric: u16) -> Option<&'static Entry> {
    ENTRIES.iter().find(|c| c.numeric == numeric)
}

/// Iterate over every country, ordered by alpha-2 code.
///
/// ```
/// assert!(koine::country::iter().any(|c| c.alpha2 == "US"));
/// assert!(koine::country::iter().is_sorted_by_key(|c| c.alpha2));
/// ```
pub fn iter() -> impl ExactSizeIterator<Item = &'static Entry> {
    ENTRIES.iter()
}

define_code! {
    /// A country identified by its ISO 3166-1 alpha-2 code, stored as four
    /// bytes: the two ASCII letters followed by `0` padding.
    ///
    /// The all-zero value is [`Country::DEFAULT`], a sentinel for "no
    /// country". It is never a real country.
    ///
    /// It displays as its uppercase alpha-2 code and parses from one in any
    /// case:
    ///
    /// ```
    /// use koine::Country;
    ///
    /// assert_eq!("gb".parse::<Country>()?, Country::GB);
    /// assert_eq!(Country::GB.to_string(), "GB");
    /// assert_eq!("default".parse::<Country>()?, Country::DEFAULT);
    /// assert!("ZZ".parse::<Country>().is_err());
    /// # Ok::<_, koine::country::ParseCountryErr>(())
    /// ```
    pub struct Country;
    /// Error produced when a string cannot be parsed as a [`Country`].
    pub struct ParseCountryErr("invalid country code");
    /// Build a country from its raw representation, in any case.
    ///
    /// If the input is not an ISO 3166-1 alpha-2 code in the table, returns
    /// [`Country::DEFAULT`].
    ///
    /// ```
    /// use koine::Country;
    ///
    /// assert_eq!(Country::new(b"US"), Country::US);
    /// assert_eq!(Country::new(b"us"), Country::US);
    /// assert_eq!(Country::new(b"ZZ"), Country::DEFAULT);
    /// ```
    const fn new valid by is_valid_alpha2;
    case to_ascii_uppercase, is_ascii_uppercase;
    expecting "a 2-letter ISO 3166-1 country code or 'default'";
    stored error "invalid stored country code";
}

impl Country {
    /// United States (`US`).
    pub const US: Country = Country::new(b"US");

    /// United Kingdom (`GB`).
    pub const GB: Country = Country::new(b"GB");

    /// Japan (`JP`).
    pub const JP: Country = Country::new(b"JP");

    /// Build from a 2-letter ISO 3166-1 alpha-2 code (case-insensitive). An
    /// empty string or `"default"` maps to [`Country::DEFAULT`]. Returns
    /// `None` for anything else.
    ///
    /// Use this to read back a stored or serialized value, where
    /// `"default"` has to round-trip. For input that must name a real
    /// country, such as user or browser input, use [`Country::from_code`].
    ///
    /// ```
    /// use koine::Country;
    ///
    /// assert_eq!(Country::from_iso("us"), Some(Country::US));
    /// assert_eq!(Country::from_iso(""), Some(Country::DEFAULT));
    /// assert_eq!(Country::from_iso("USA"), None);
    /// ```
    pub fn from_iso(code: &str) -> Option<Self> {
        let code = code.trim();

        if code.is_empty() || code.eq_ignore_ascii_case("default") {
            return Some(Self::DEFAULT);
        }

        let mut bytes = [0u8; 4];
        let upper = Self::normalize(&mut bytes, code)?;

        match *upper.as_bytes() {
            [a, b] if is_valid_alpha2(&[a, b]) => Some(Self([a, b, 0, 0])),
            _ => None,
        }
    }

    /// Build from a 2-letter ISO 3166-1 alpha-2 code (case-insensitive),
    /// like [`Country::from_iso`], but never returns [`Country::DEFAULT`]:
    /// an empty string or `"default"` gives `None`.
    ///
    /// Use this for input that must name a real country, such as user or
    /// browser input.
    ///
    /// ```
    /// use koine::Country;
    ///
    /// assert_eq!(Country::from_code("us"), Some(Country::US));
    /// assert_eq!(Country::from_code(""), None);
    /// assert_eq!(Country::from_code("default"), None);
    /// assert_eq!(Country::from_code("USA"), None);
    /// ```
    pub fn from_code(code: &str) -> Option<Self> {
        Self::from_iso(code).filter(|country| !country.is_default())
    }

    /// The table [`Entry`] for this country, or `None` for
    /// [`Country::DEFAULT`].
    #[inline]
    pub fn to_iso(&self) -> Option<&'static Entry> {
        if self.is_default() {
            return None;
        }

        by_alpha2(self.as_raw_code())
    }
}

impl fmt::Debug for Country {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Country({self})")
    }
}

impl Entry {
    /// The [`Country`] of this entry, never [`Country::DEFAULT`].
    ///
    /// Every entry in the table holds a valid alpha-2 code, so unlike parsing
    /// [`Entry::alpha2`] this cannot fail.
    ///
    /// ```
    /// use koine::Country;
    ///
    /// let us = koine::country::by_alpha3("USA").unwrap();
    /// assert_eq!(us.to_country(), Country::US);
    /// assert_eq!(Country::from(us), Country::US);
    /// ```
    #[inline]
    pub const fn to_country(&self) -> Country {
        match *self.alpha2.as_bytes() {
            [a, b] => Country::from_raw_unchecked([a, b, 0, 0]),
            _ => Country::DEFAULT,
        }
    }
}

impl From<&Entry> for Country {
    /// The entry's country, as [`Entry::to_country`].
    #[inline]
    fn from(entry: &Entry) -> Self {
        entry.to_country()
    }
}

impl PartialEq<&'static Entry> for Country {
    #[inline]
    fn eq(&self, other: &&'static Entry) -> bool {
        self.as_raw_code() == other.alpha2
    }
}

/// The country's English name, or `None` for [`Country::DEFAULT`].
#[cfg(feature = "yew023")]
#[cfg_attr(koine_docsrs, doc(cfg(feature = "yew023")))]
impl yew023::html::IntoPropValue<Option<yew023::AttrValue>> for Country {
    #[inline]
    fn into_prop_value(self) -> Option<yew023::AttrValue> {
        self.to_iso().map(|c| yew023::AttrValue::Static(c.name))
    }
}

/// Keyed by the alpha-2 code.
#[cfg(feature = "yew023")]
#[cfg_attr(koine_docsrs, doc(cfg(feature = "yew023")))]
impl From<Country> for yew023::virtual_dom::Key {
    #[inline]
    fn from(country: Country) -> Self {
        country.as_str().into()
    }
}

#[cfg(test)]
mod tests {
    use alloc::string::ToString;

    use super::*;

    #[test]
    fn every_country_round_trips() {
        for entry in iter() {
            let country = entry.to_country();
            assert!(!country.is_default(), "{entry:?}");
            assert_eq!(country, Country::new(entry.alpha2.as_bytes()));
            assert_eq!(country, entry);
            assert_eq!(country.to_iso(), Some(entry));
            assert_eq!(country.to_string().parse::<Country>(), Ok(country));
            assert_eq!(by_alpha3(entry.alpha3), Some(entry));
            assert_eq!(by_numeric(entry.numeric), Some(entry));
        }
    }
}
