//! ISO 639 languages: the [`Language`] value type and the ISO 639-3 table it
//! is validated against.
//!
//! The table holds every ISO 639-3 code with its ISO 639-1 and 639-2 forms,
//! scope, type and English reference name, from the code tables of
//! [SIL International], the ISO 639-3 registration authority. See
//! [Data sources](crate#data-sources).
//!
//! [SIL International]: https://iso639-3.sil.org
//!
//! ```
//! use koine::Language;
//!
//! let english: Language = "en".parse()?;
//! assert_eq!(english, Language::ENG);
//! assert_eq!(english.to_string(), "eng");
//! assert_eq!(english.to_part1(), Some("en"));
//!
//! let entry = koine::language::by_id("eng").unwrap();
//! assert_eq!(entry.name, "English");
//! assert_eq!(entry.flag, Some("US"));
//! # Ok::<_, koine::language::ParseLanguageErr>(())
//! ```

use core::fmt;

/// The scope of an ISO 639 [`Entry`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Scope {
    /// An individual language.
    Individual,
    /// A macrolanguage that subsumes several closely related individual
    /// languages.
    Macrolanguage,
    /// A special code (for example for undetermined or multiple languages).
    Special,
}

/// The type of an ISO 639 [`Entry`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Type {
    /// A language still in everyday use by a living community.
    Living,
    /// A language with no remaining living speakers.
    Extinct,
    /// A language that is no longer in common use but is distinct from its
    /// modern descendants.
    Historical,
    /// An artificially created language (for example Esperanto).
    Constructed,
    /// A special-purpose code rather than a natural language.
    Special,
}

/// A single ISO 639 language in the table.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub struct Entry {
    /// The 3-letter ISO 639-3 code, the primary identifier for this language.
    pub id: &'static str,
    /// The 3-letter ISO 639-2/B (bibliographic) code, when one exists.
    pub part2b: Option<&'static str>,
    /// The 3-letter ISO 639-2/T (terminological) code, when one exists.
    pub part2t: Option<&'static str>,
    /// The 2-letter ISO 639-1 code, when one exists.
    pub part1: Option<&'static str>,
    /// The ISO 3166-1 alpha-2 code of a country whose flag can stand for this
    /// language, when one is mapped.
    ///
    /// This comes from a best-effort mapping maintained by hand in this
    /// repository, not from any standard: a language is spoken in many
    /// countries and a flag stands for none of them exactly. Every code here
    /// is in [`koine::country`](crate#features), but whether an app has a
    /// flag to draw for it is the app's to check.
    pub flag: Option<&'static str>,
    /// The scope of this entry.
    pub scope: Scope,
    /// The type of this entry.
    pub ty: Type,
    /// The English reference name of the language.
    pub name: &'static str,
    /// A free-form comment about the entry, when present.
    pub comment: Option<&'static str>,
}

#[allow(clippy::match_like_matches_macro)]
#[rustfmt::skip]
mod generated;
use generated::ENTRIES;

/// Whether the given bytes are an ISO 639-3 code in the table (three
/// lowercase ASCII letters).
///
/// ```
/// assert!(koine::language::is_valid_id(b"eng"));
/// assert!(!koine::language::is_valid_id(b"ENG"));
/// assert!(!koine::language::is_valid_id(b"zzz"));
/// ```
pub const fn is_valid_id(id: &[u8]) -> bool {
    generated::is_valid_id(id)
}

/// Look up an [`Entry`] by its 2-letter ISO 639-1 code [`Entry::part1`].
///
/// ```
/// let english = koine::language::by_part1("en").unwrap();
/// assert_eq!(english.id, "eng");
/// assert_eq!(english.name, "English");
///
/// assert!(koine::language::by_part1("zz").is_none());
/// ```
pub fn by_part1(part1: &str) -> Option<&'static Entry> {
    generated::BY_PART1.get(part1).and_then(|&i| ENTRIES.get(i))
}

/// Look up an [`Entry`] by its 3-letter ISO 639-3 code [`Entry::id`].
///
/// ```
/// let english = koine::language::by_id("eng").unwrap();
/// assert_eq!(english.part1, Some("en"));
/// assert_eq!(english.name, "English");
///
/// assert!(koine::language::by_id("zzz").is_none());
/// ```
pub fn by_id(id: &str) -> Option<&'static Entry> {
    generated::BY_ID.get(id).and_then(|&i| ENTRIES.get(i))
}

/// Iterate over every [`Entry`], ordered by ISO 639-3 code.
///
/// ```
/// assert!(koine::language::iter().any(|lang| lang.id == "eng"));
/// assert!(koine::language::iter().is_sorted_by_key(|lang| lang.id));
/// ```
pub fn iter() -> impl ExactSizeIterator<Item = &'static Entry> {
    ENTRIES.iter()
}

/// Iterate over each distinct [`Entry::flag`] country once, as uppercase
/// ISO 3166-1 alpha-2 codes in alphabetical order.
///
/// These are the flags an app needs to draw every language that has one.
///
/// ```
/// let flags: Vec<_> = koine::language::flags().collect();
/// assert!(flags.contains(&"US"));
/// assert!(flags.is_sorted());
///
/// for language in koine::language::iter() {
///     if let Some(flag) = language.flag {
///         assert!(flags.contains(&flag));
///     }
/// }
/// ```
pub fn flags() -> impl ExactSizeIterator<Item = &'static str> {
    generated::FLAGS.iter().copied()
}

define_code! {
    /// A language identified by its 3-letter ISO 639-3 code, stored as four
    /// bytes: the three ASCII letters followed by a `0` pad.
    ///
    /// The all-zero value is [`Language::DEFAULT`], a sentinel for "no
    /// language" (an app may read it as "the original language"). It is never
    /// a real language.
    ///
    /// It displays as its ISO 639-3 code and parses from an ISO 639-1 or
    /// ISO 639-3 code in any case:
    ///
    /// ```
    /// use koine::Language;
    ///
    /// assert_eq!("sv".parse::<Language>()?.to_string(), "swe");
    /// assert_eq!("SWE".parse::<Language>()?.to_string(), "swe");
    /// assert_eq!("default".parse::<Language>()?, Language::DEFAULT);
    /// assert!("zz".parse::<Language>().is_err());
    /// # Ok::<_, koine::language::ParseLanguageErr>(())
    /// ```
    pub struct Language;
    /// Error produced when a string cannot be parsed as a [`Language`].
    ///
    /// [`kind`](ParseLanguageErr::kind) says whether the code was unknown or
    /// not a code at all:
    ///
    /// ```
    /// use koine::Language;
    /// use koine::language::ParseLanguageErrKind;
    ///
    /// let err = "zz".parse::<Language>().unwrap_err();
    /// assert_eq!(err.kind(), ParseLanguageErrKind::Unknown);
    /// assert_eq!(err.to_string(), "unknown language code");
    ///
    /// let err = "english".parse::<Language>().unwrap_err();
    /// assert_eq!(err.kind(), ParseLanguageErrKind::Malformed);
    /// assert_eq!(err.to_string(), "malformed language code");
    /// ```
    pub struct ParseLanguageErr;
    /// How a string failed to parse as a [`Language`], from
    /// [`ParseLanguageErr::kind`].
    pub enum ParseLanguageErrKind {
        /// A 2- or 3-letter code that is not in the ISO 639 table, such as
        /// `"zz"`.
        Unknown("unknown language code"),
        /// Not a 2- or 3-letter code, such as `"english"` or `"e1"`.
        Malformed("malformed language code"),
    }
    /// Build a language from its raw representation, in any case.
    ///
    /// If the input is not an ISO 639-3 code in the table, returns
    /// [`Language::DEFAULT`]. Unlike [`Language::from_iso`] it takes no
    /// ISO 639-1 codes, so it can be `const`.
    ///
    /// ```
    /// use koine::Language;
    ///
    /// assert_eq!(Language::new(b"eng"), Language::ENG);
    /// assert_eq!(Language::new(b"ENG"), Language::ENG);
    /// assert_eq!(Language::new(b"zzz"), Language::DEFAULT);
    /// ```
    const fn new valid by is_valid_id;
    case to_ascii_lowercase, is_ascii_lowercase;
    expecting "a 2- or 3-letter ISO 639 language code or 'default'";
    stored error "invalid stored language code";
}

impl Language {
    /// English (`eng`).
    pub const ENG: Language = Language::new(b"eng");

    /// Undetermined (`und`): the ISO 639 code for a language that is not
    /// known, for example of a string handed over without saying which
    /// language it is in.
    pub const UND: Language = Language::new(b"und");

    /// Build from a 2- or 3-letter ISO 639 code (case-insensitive). A 2-letter
    /// code is resolved to its 3-letter form through the table. An empty
    /// string or `"default"` maps to [`Language::DEFAULT`]. Returns `None` for
    /// anything else.
    ///
    /// Use this to read back a stored or serialized value, where
    /// `"default"` has to round-trip. For input that must name a real
    /// language, such as user or browser input, use [`Language::from_code`].
    ///
    /// ```
    /// use koine::Language;
    ///
    /// assert_eq!(Language::from_iso("en"), Some(Language::ENG));
    /// assert_eq!(Language::from_iso(" Eng "), Some(Language::ENG));
    /// assert_eq!(Language::from_iso(""), Some(Language::DEFAULT));
    /// assert_eq!(Language::from_iso("english"), None);
    /// ```
    #[inline]
    pub fn from_iso(code: &str) -> Option<Self> {
        Self::parse(code).ok()
    }

    /// As [`Language::from_iso`], saying why on failure.
    pub(crate) fn parse(code: &str) -> Result<Self, ParseLanguageErr> {
        let code = code.trim();

        if code.is_empty() || code.eq_ignore_ascii_case("default") {
            return Ok(Self::DEFAULT);
        }

        let mut bytes = [0u8; 4];

        let Some(lower) = Self::normalize(&mut bytes, code) else {
            return Err(ParseLanguageErr::malformed());
        };

        let bytes = match lower.len() {
            2 => {
                let Some(&[a, b, c]) = by_part1(lower).map(|e| e.id.as_bytes()) else {
                    return Err(ParseLanguageErr::unknown());
                };

                [a, b, c, 0]
            }
            3 if is_valid_id(lower.as_bytes()) => bytes,
            3 => return Err(ParseLanguageErr::unknown()),
            _ => return Err(ParseLanguageErr::malformed()),
        };

        Ok(Self(bytes))
    }

    /// Build from a 2- or 3-letter ISO 639 code (case-insensitive), like
    /// [`Language::from_iso`], but never returns [`Language::DEFAULT`]: an
    /// empty string or `"default"` gives `None`.
    ///
    /// Use this for input that must name a real language, such as user or
    /// browser input.
    ///
    /// ```
    /// use koine::Language;
    ///
    /// assert_eq!(Language::from_code("en"), Some(Language::ENG));
    /// assert_eq!(Language::from_code(" Eng "), Some(Language::ENG));
    /// assert_eq!(Language::from_code(""), None);
    /// assert_eq!(Language::from_code("default"), None);
    /// assert_eq!(Language::from_code("english"), None);
    /// ```
    pub fn from_code(code: &str) -> Option<Self> {
        Self::from_iso(code).filter(|language| !language.is_default())
    }

    /// The table [`Entry`] for this language, or `None` for
    /// [`Language::DEFAULT`].
    ///
    /// ```
    /// use koine::Language;
    ///
    /// assert_eq!(Language::ENG.to_iso().map(|e| e.name), Some("English"));
    /// assert!(Language::DEFAULT.to_iso().is_none());
    /// ```
    pub fn to_iso(&self) -> Option<&'static Entry> {
        if self.is_default() {
            return None;
        }

        by_id(self.as_raw_code())
    }

    /// The 3-letter ISO 639-3 code, or `None` for [`Language::DEFAULT`].
    pub fn to_id(&self) -> Option<&'static str> {
        self.to_iso().map(|e| e.id)
    }

    /// The 2-letter ISO 639-1 code, or `None` for [`Language::DEFAULT`] and
    /// languages without one.
    ///
    /// ```
    /// use koine::Language;
    ///
    /// assert_eq!(Language::ENG.to_part1(), Some("en"));
    /// assert_eq!(Language::UND.to_part1(), None);
    /// ```
    pub fn to_part1(&self) -> Option<&'static str> {
        self.to_iso()?.part1
    }
}

impl fmt::Debug for Language {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl Entry {
    /// The [`Language`] of this entry, never [`Language::DEFAULT`].
    ///
    /// Every entry in the table holds a valid ISO 639-3 code, so unlike
    /// parsing [`Entry::id`] this cannot fail.
    ///
    /// ```
    /// use koine::Language;
    ///
    /// let english = koine::language::by_id("eng").unwrap();
    /// assert_eq!(english.to_language(), Language::ENG);
    /// assert_eq!(Language::from(english), Language::ENG);
    /// ```
    #[inline]
    pub const fn to_language(&self) -> Language {
        match *self.id.as_bytes() {
            [a, b, c] => Language::from_raw_unchecked([a, b, c, 0]),
            _ => Language::DEFAULT,
        }
    }
}

impl From<&Entry> for Language {
    /// The entry's language, as [`Entry::to_language`].
    #[inline]
    fn from(entry: &Entry) -> Self {
        entry.to_language()
    }
}

impl PartialEq<&'static Entry> for Language {
    #[inline]
    fn eq(&self, other: &&'static Entry) -> bool {
        self.as_raw_code() == other.id
    }
}

#[cfg(test)]
mod tests {
    use alloc::string::ToString;

    use super::*;

    /// `flags` is every distinct entry flag once, in alphabetical order.
    #[test]
    fn flags_are_the_distinct_entry_flags() {
        let expected: alloc::collections::BTreeSet<_> =
            ENTRIES.iter().filter_map(|e| e.flag).collect();
        assert!(flags().eq(expected));
    }

    /// Every language with an ISO 639-1 code has a flag in the mapping, which
    /// is what keeps `to-3166.csv` complete when SIL adds one.
    #[test]
    fn every_part1_language_has_a_flag() {
        for entry in iter() {
            if entry.part1.is_none() {
                continue;
            }

            assert!(
                entry.flag.is_some(),
                "language {:?} with ISO 639-1 code {:?} is missing from data/language/to-3166.csv",
                entry.id,
                entry.part1
            );
        }
    }

    #[cfg(feature = "country")]
    #[test]
    fn every_flag_is_a_country() {
        for entry in iter() {
            if let Some(flag) = entry.flag {
                assert!(
                    crate::country::by_alpha2(flag).is_some(),
                    "{flag:?} of {:?} is not a country",
                    entry.id
                );
            }
        }
    }

    #[test]
    fn every_entry_is_its_language() {
        for entry in iter() {
            let language = entry.to_language();
            assert!(!language.is_default(), "{entry:?}");
            assert_eq!(language, Language::new(entry.id.as_bytes()));
            assert_eq!(language, entry);
            assert_eq!(language.to_iso(), Some(entry));
        }
    }

    #[test]
    fn round_trips_through_its_string() {
        for entry in iter().step_by(97) {
            let language = entry.to_language();
            assert_eq!(language.as_str(), entry.id);
            assert_eq!(language.to_string().parse::<Language>(), Ok(language));
        }

        assert_eq!(
            Language::DEFAULT.to_string().parse::<Language>(),
            Ok(Language::DEFAULT)
        );
    }
}
