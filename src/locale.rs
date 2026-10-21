//! Locales: the [`Locale`] value type, a [`Language`] with an optional
//! [`Country`], and the table of valid language+country combinations.
//!
//! The table lists each language and country that a locale in
//! [Unicode CLDR]'s `common/main` combines, such as `en_US` or `sr_Latn_RS`
//! (scripts and variants dropped), where the language is in
//! [`koine::language`](crate::language) and the country in
//! [`koine::country`](crate::country). See [Data sources](crate#data-sources).
//!
//! [Unicode CLDR]: https://cldr.unicode.org
//!
//! ```
//! use koine::{Country, Language, Locale};
//!
//! let locale: Locale = "pt-BR".parse()?;
//! assert_eq!(locale.language(), Language::new(b"por"));
//! assert_eq!(locale.country(), Country::new(b"BR"));
//! assert_eq!(locale.to_string(), "pt-BR");
//!
//! assert!(koine::locale::contains("por", "BR"));
//! assert!(!koine::locale::contains("por", "SE"));
//! # Ok::<_, koine::locale::ParseLocaleErr>(())
//! ```

use core::fmt;
use core::str::FromStr;

use crate::{Country, Language};

/// A single valid language+country combination in the table.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub struct Entry {
    /// The ISO 639-3 code (3 letters, lowercase), as in
    /// [`language::Entry::id`](crate::language::Entry::id).
    pub language: &'static str,
    /// The ISO 3166-1 alpha-2 code (uppercase), as in
    /// [`country::Entry::alpha2`](crate::country::Entry::alpha2).
    pub country: &'static str,
}

impl Entry {
    /// The [`Language`] of this entry, never [`Language::DEFAULT`].
    ///
    /// Every entry in the table holds a valid ISO 639-3 code, so unlike
    /// parsing [`Entry::language`](#structfield.language) this cannot fail.
    ///
    /// ```
    /// use koine::Language;
    ///
    /// let entry = koine::locale::iter().find(|l| l.language == "eng").unwrap();
    /// assert_eq!(entry.to_language(), Language::ENG);
    /// ```
    #[inline]
    pub const fn to_language(&self) -> Language {
        match *self.language.as_bytes() {
            [a, b, c] => Language::from_raw_unchecked([a, b, c, 0]),
            _ => Language::DEFAULT,
        }
    }

    /// The [`Country`] of this entry, never [`Country::DEFAULT`].
    ///
    /// Every entry in the table holds a valid alpha-2 code, so unlike parsing
    /// [`Entry::country`](#structfield.country) this cannot fail.
    ///
    /// ```
    /// use koine::Country;
    ///
    /// let entry = koine::locale::iter().find(|l| l.country == "US").unwrap();
    /// assert_eq!(entry.to_country(), Country::US);
    /// ```
    #[inline]
    pub const fn to_country(&self) -> Country {
        match *self.country.as_bytes() {
            [a, b] => Country::from_raw_unchecked([a, b, 0, 0]),
            _ => Country::DEFAULT,
        }
    }

    /// The [`Locale`] of this entry: its language and country, always
    /// [listed](Locale::is_listed).
    ///
    /// ```
    /// use koine::Locale;
    ///
    /// let entry = koine::locale::iter()
    ///     .find(|l| l.language == "eng" && l.country == "US")
    ///     .unwrap();
    /// assert_eq!(entry.to_locale(), Locale::EN_US);
    /// assert_eq!(Locale::from(entry), Locale::EN_US);
    /// ```
    #[inline]
    pub const fn to_locale(&self) -> Locale {
        Locale::new(self.to_language(), self.to_country())
    }
}

impl From<&Entry> for Locale {
    /// The entry's locale, as [`Entry::to_locale`].
    #[inline]
    fn from(entry: &Entry) -> Self {
        entry.to_locale()
    }
}

#[rustfmt::skip]
mod generated;
use generated::ENTRIES;

/// Iterate over every valid combination, ordered by language then country.
///
/// ```
/// assert!(koine::locale::iter().any(|l| l.language == "eng" && l.country == "US"));
/// ```
pub fn iter() -> impl ExactSizeIterator<Item = &'static Entry> {
    ENTRIES.iter()
}

/// Iterate over the combinations of the given language, ordered by country.
///
/// Yields nothing for [`Language::DEFAULT`] and for a language that has no
/// locale in the table.
///
/// ```
/// use koine::{Country, Language};
///
/// let portuguese = Language::new(b"por");
/// assert!(koine::locale::by_language(portuguese).any(|l| l.to_country() == Country::new(b"BR")));
/// assert!(koine::locale::by_language(portuguese).all(|l| l.to_language() == portuguese));
///
/// assert_eq!(koine::locale::by_language(Language::UND).len(), 0);
/// ```
pub fn by_language(language: Language) -> impl ExactSizeIterator<Item = &'static Entry> {
    // The table is ordered by language code, which orders the same as the
    // code's raw bytes.
    let start = ENTRIES.partition_point(|e| e.to_language() < language);
    let len = ENTRIES[start..].partition_point(|e| e.to_language() == language);
    ENTRIES[start..start + len].iter()
}

/// Iterate over the combinations of the given country, ordered by language.
///
/// Yields nothing for [`Country::DEFAULT`] and for a country that has no
/// locale in the table.
///
/// ```
/// use koine::{Country, Language};
///
/// let languages: Vec<Language> = koine::locale::by_country(Country::new(b"CH"))
///     .map(|l| l.to_language())
///     .collect();
///
/// assert!(languages.contains(&Language::new(b"deu")));
/// assert!(languages.contains(&Language::new(b"fra")));
/// assert_eq!(koine::locale::by_country(Country::DEFAULT).count(), 0);
/// ```
pub fn by_country(country: Country) -> impl Iterator<Item = &'static Entry> {
    ENTRIES.iter().filter(move |e| e.to_country() == country)
}

/// Whether the given ISO 639-3 code and uppercase ISO 3166-1 alpha-2 code are
/// a valid combination.
///
/// ```
/// assert!(koine::locale::contains("eng", "US"));
/// assert!(!koine::locale::contains("eng", "ZZ"));
/// assert!(!koine::locale::contains("en", "US"), "takes ISO 639-3 codes");
/// ```
pub fn contains(language: &str, country: &str) -> bool {
    // The `"<language>-<country>"` key, built on the stack.
    let mut buf = [0u8; 8];
    let (language, country) = (language.as_bytes(), country.as_bytes());
    let len = language.len() + 1 + country.len();

    let Some(key) = buf.get_mut(..len) else {
        return false;
    };

    let (head, tail) = key.split_at_mut(language.len());
    head.copy_from_slice(language);
    tail[0] = b'-';
    tail[1..].copy_from_slice(country);

    match core::str::from_utf8(key) {
        Ok(key) => generated::BY_KEY.contains_key(key),
        Err(_) => false,
    }
}

/// A locale: a [`Language`] and an optional [`Country`], for example `en-US`,
/// `pt-BR` or a bare `en`.
///
/// It displays as the language's [`Language::to_tag`], its ISO 639-1 code
/// where it has one and its ISO 639-3 code otherwise, followed by `-` and
/// the country when one is set, and parses from that form with either
/// language code. [`Locale::display_name`] renders its English name
/// instead, such as `English (United States)`.
///
/// The 64-bit integer form ([`Locale::to_u64`]) keeps the language in the
/// low 32 bits and the country in the high 32 bits, so a value whose high
/// bits are zero is a bare [`Language`] with [`Country::DEFAULT`].
///
/// [`Locale::DEFAULT`] (both components default) is a sentinel for "no
/// locale". A locale that is not [`Locale::DEFAULT`] always names a real
/// language. A locale need not be in the [table](crate::locale::contains):
/// any language may pair with any country.
#[derive(Default, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Locale {
    language: Language,
    country: Country,
}

/// Error produced when a string cannot be parsed as a [`Locale`].
///
/// [`kind`](ParseLocaleErr::kind) says which part of the code was wrong:
///
/// ```
/// use koine::Locale;
/// use koine::locale::ParseLocaleErrKind;
///
/// let err = "zz-US".parse::<Locale>().unwrap_err();
/// assert_eq!(err.kind(), ParseLocaleErrKind::UnknownLanguage);
/// assert_eq!(err.to_string(), "unknown language in locale code");
///
/// let err = "en-ZZ".parse::<Locale>().unwrap_err();
/// assert_eq!(err.kind(), ParseLocaleErrKind::UnknownCountry);
/// assert_eq!(err.to_string(), "unknown country in locale code");
///
/// let err = "en_US".parse::<Locale>().unwrap_err();
/// assert_eq!(err.kind(), ParseLocaleErrKind::Malformed);
/// assert_eq!(err.to_string(), "malformed locale code");
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct ParseLocaleErr {
    kind: ParseLocaleErrKind,
}

/// How a string failed to parse as a [`Locale`], from
/// [`ParseLocaleErr::kind`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum ParseLocaleErrKind {
    /// The language part is a 2- or 3-letter code that is not in the ISO 639
    /// table, such as in `"zz-US"`.
    UnknownLanguage,
    /// The country part is a 2-letter code that is not an assigned
    /// ISO 3166-1 alpha-2 code, such as in `"en-ZZ"`.
    UnknownCountry,
    /// Not of the form `language[-country]`, such as `"en_US"`, `"en-"`,
    /// `"en-USA"` or `"default-US"`.
    Malformed,
}

impl ParseLocaleErr {
    /// Which part of the code was wrong.
    #[inline]
    pub const fn kind(&self) -> ParseLocaleErrKind {
        self.kind
    }

    #[inline]
    const fn new(kind: ParseLocaleErrKind) -> Self {
        Self { kind }
    }
}

impl fmt::Display for ParseLocaleErr {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self.kind {
            ParseLocaleErrKind::UnknownLanguage => "unknown language in locale code",
            ParseLocaleErrKind::UnknownCountry => "unknown country in locale code",
            ParseLocaleErrKind::Malformed => "malformed locale code",
        })
    }
}

impl core::error::Error for ParseLocaleErr {}

impl Locale {
    /// The sentinel for "no locale": [`Language::DEFAULT`] and
    /// [`Country::DEFAULT`]. It displays as `"default"`.
    pub const DEFAULT: Locale = Locale {
        language: Language::DEFAULT,
        country: Country::DEFAULT,
    };

    /// English (United States).
    ///
    /// ```
    /// use koine::{Country, Language, Locale};
    ///
    /// assert_eq!(Locale::EN_US, Locale::new(Language::ENG, Country::US));
    /// assert_eq!(Locale::EN_US.to_string(), "en-US");
    /// ```
    pub const EN_US: Locale = Locale {
        language: Language::ENG,
        country: Country::US,
    };

    /// The locale of a string whose language is unknown: [`Language::UND`]
    /// with no country.
    pub const UNKNOWN: Locale = Locale {
        language: Language::UND,
        country: Country::DEFAULT,
    };

    /// Construct a locale from its language and country components.
    #[inline]
    pub const fn new(language: Language, country: Country) -> Self {
        Self { language, country }
    }

    /// The language component.
    #[inline]
    pub const fn language(&self) -> Language {
        self.language
    }

    /// The country component.
    #[inline]
    pub const fn country(&self) -> Country {
        self.country
    }

    /// Whether the language is [`Language::DEFAULT`], which is what makes
    /// this the [`Locale::DEFAULT`] sentinel.
    #[inline]
    pub const fn is_default(&self) -> bool {
        self.language.is_default()
    }

    /// This locale, or `other` if this is [`Locale::DEFAULT`].
    #[inline]
    pub fn or(self, other: Self) -> Self {
        if self.is_default() { other } else { self }
    }

    /// This locale, or [`Locale::DEFAULT`] if `f` returns `false` for it.
    #[inline]
    pub fn filter(self, f: impl FnOnce(Self) -> bool) -> Self {
        if f(self) { self } else { Self::DEFAULT }
    }

    /// The ISO 3166-1 alpha-2 code of the flag that stands for this locale:
    /// the country's when one is set, the language's own
    /// [`flag`](crate::language::Entry::flag) otherwise. `None` for
    /// [`Locale::DEFAULT`] and a language with no flag mapped.
    ///
    /// Whether an app has the flag to draw is the app's to check; koine
    /// ships no flags.
    ///
    /// ```
    /// use koine::{Country, Language, Locale};
    ///
    /// assert_eq!(Locale::EN_US.flag(), Some("US"));
    /// assert_eq!(Locale::new(Language::new(b"swe"), Country::DEFAULT).flag(), Some("SE"));
    /// assert_eq!(Locale::DEFAULT.flag(), None);
    /// ```
    pub fn flag(&self) -> Option<&'static str> {
        if !self.country.is_default() {
            return self.country.to_iso().map(|c| c.alpha2);
        }

        self.language.to_iso().and_then(|l| l.flag)
    }

    /// Whether this language and country are a valid combination in the
    /// [table](crate::locale::contains). A locale without a country is not.
    ///
    /// ```
    /// use koine::{Locale, Language, Country};
    ///
    /// assert!(Locale::EN_US.is_listed());
    /// assert!(!Locale::new(Language::ENG, Country::DEFAULT).is_listed());
    /// ```
    pub fn is_listed(&self) -> bool {
        match (self.language.to_id(), self.country.to_iso()) {
            (Some(language), Some(country)) => contains(language, country.alpha2),
            _ => false,
        }
    }

    /// The 64-bit integer representation: country in the high 32 bits,
    /// language in the low 32 bits, each its four raw bytes big-endian.
    ///
    /// ```
    /// use koine::Locale;
    ///
    /// assert_eq!(Locale::from_u64(Locale::EN_US.to_u64()), Locale::EN_US);
    /// assert_eq!(Locale::DEFAULT.to_u64(), 0);
    /// ```
    #[inline]
    pub const fn to_u64(self) -> u64 {
        let language = u32::from_be_bytes(self.language.to_raw()) as u64;
        let country = u32::from_be_bytes(self.country.to_raw()) as u64;
        (country << 32) | language
    }

    /// Rebuild a locale from its 64-bit integer representation, the inverse
    /// of [`Locale::to_u64`]. A component that is not in the tables becomes
    /// its `DEFAULT`.
    #[inline]
    pub const fn from_u64(value: u64) -> Self {
        Self {
            language: Language::new(&(value as u32).to_be_bytes()),
            country: Country::new(&((value >> 32) as u32).to_be_bytes()),
        }
    }

    /// Rebuild a locale from its 64-bit integer representation like
    /// [`Locale::from_u64`], but keep a component that is missing from the
    /// current tables as long as it is well formed: a lowercase language code
    /// and an uppercase country code of up to four ASCII letters padded with
    /// zero bytes, or zero. A code retired from the tables after it was stored
    /// therefore reads back intact. Returns `None` if either component is
    /// malformed.
    ///
    /// ```
    /// use koine::{Country, Language, Locale};
    ///
    /// assert_eq!(Locale::from_u64_lossless(Locale::EN_US.to_u64()), Some(Locale::EN_US));
    ///
    /// // "qqq" and "QQ" are well formed but in neither table.
    /// let value = 0x5151_0000_7171_7100;
    /// assert_eq!(Locale::from_u64(value), Locale::DEFAULT);
    /// let locale = Locale::from_u64_lossless(value).unwrap();
    /// assert_eq!(locale.to_u64(), value);
    /// assert_eq!(locale.to_string(), "qqq-QQ");
    ///
    /// // Not letters.
    /// assert_eq!(Locale::from_u64_lossless(0x3132_3300), None);
    /// ```
    #[inline]
    pub const fn from_u64_lossless(value: u64) -> Option<Self> {
        let Some(language) = Language::from_raw_shape((value as u32).to_be_bytes()) else {
            return None;
        };

        let Some(country) = Country::from_raw_shape(((value >> 32) as u32).to_be_bytes()) else {
            return None;
        };

        Some(Self { language, country })
    }

    /// Build from a `language[-country]` ISO code (case-insensitive), such as
    /// `"en-US"`, `"eng-US"` or `"en"`. An empty string or `"default"` maps
    /// to [`Locale::DEFAULT`]. Returns `None` for anything else, including a
    /// language or country part of `"default"`.
    ///
    /// Use this to read back a stored or serialized value, where
    /// `"default"` has to round-trip. For input that must name a real
    /// locale, such as user or browser input, use [`Locale::from_code`].
    ///
    /// ```
    /// use koine::Locale;
    ///
    /// assert_eq!(Locale::from_iso("en-us"), Some(Locale::EN_US));
    /// assert_eq!(Locale::from_iso("eng-US"), Some(Locale::EN_US));
    /// assert_eq!(Locale::from_iso("default"), Some(Locale::DEFAULT));
    /// assert_eq!(Locale::from_iso("en-default"), None);
    /// assert_eq!(Locale::from_iso("en-ZZ"), None);
    /// ```
    #[inline]
    pub fn from_iso(code: &str) -> Option<Self> {
        Self::parse(code).ok()
    }

    /// As [`Locale::from_iso`], saying which part was wrong on failure.
    fn parse(code: &str) -> Result<Self, ParseLocaleErr> {
        use crate::country::ParseCountryErrKind;
        use crate::language::ParseLanguageErrKind;

        const MALFORMED: ParseLocaleErr = ParseLocaleErr::new(ParseLocaleErrKind::Malformed);

        let code = code.trim();

        if code.is_empty() || code.eq_ignore_ascii_case("default") {
            return Ok(Self::DEFAULT);
        }

        let (language, country) = match code.split_once('-') {
            Some((language, country)) => (language, Some(country)),
            None => (code, None),
        };

        let language = match Language::parse(language) {
            Ok(language) if language.is_default() => return Err(MALFORMED),
            Ok(language) => language,
            Err(e) if e.kind() == ParseLanguageErrKind::Unknown => {
                return Err(ParseLocaleErr::new(ParseLocaleErrKind::UnknownLanguage));
            }
            Err(_) => return Err(MALFORMED),
        };

        let country = match country.map(Country::parse) {
            None => Country::DEFAULT,
            Some(Ok(country)) if country.is_default() => return Err(MALFORMED),
            Some(Ok(country)) => country,
            Some(Err(e)) if e.kind() == ParseCountryErrKind::Unknown => {
                return Err(ParseLocaleErr::new(ParseLocaleErrKind::UnknownCountry));
            }
            Some(Err(_)) => return Err(MALFORMED),
        };

        Ok(Self { language, country })
    }

    /// Build from a `language[-country]` ISO code (case-insensitive), like
    /// [`Locale::from_iso`], but never returns [`Locale::DEFAULT`]: an empty
    /// string, `"default"` and a language or country part of `"default"` all
    /// give `None`.
    ///
    /// Use this for input that must name a real locale, such as user or
    /// browser input.
    ///
    /// ```
    /// use koine::{Country, Language, Locale};
    ///
    /// assert_eq!(Locale::from_code("en-us"), Some(Locale::EN_US));
    /// assert_eq!(Locale::from_code("en"), Some(Locale::new(Language::ENG, Country::DEFAULT)));
    /// assert_eq!(Locale::from_code(""), None);
    /// assert_eq!(Locale::from_code("default"), None);
    /// assert_eq!(Locale::from_code("en-default"), None);
    /// assert_eq!(Locale::from_code("default-US"), None);
    /// assert_eq!(Locale::from_code("en-ZZ"), None);
    /// ```
    pub fn from_code(code: &str) -> Option<Self> {
        Self::from_iso(code).filter(|locale| !locale.is_default())
    }

    /// The English name of this locale for display: the language's
    /// [name](Language::name), followed by the country's
    /// [name](Country::name) in parentheses when one is set. A code missing
    /// from its table shows as the code, and [`Locale::DEFAULT`] as
    /// `"default"`, as in its string form.
    ///
    /// It writes straight to the formatter, so it needs no allocation.
    ///
    /// ```
    /// use koine::{Country, Language, Locale};
    ///
    /// assert_eq!(Locale::EN_US.display_name().to_string(), "English (United States)");
    /// assert_eq!(Locale::new(Language::ENG, Country::DEFAULT).display_name().to_string(), "English");
    /// assert_eq!(Locale::DEFAULT.display_name().to_string(), "default");
    /// ```
    #[inline]
    pub fn display_name(&self) -> impl fmt::Display + use<> {
        DisplayName(*self)
    }
}

/// The [`Locale::display_name`] of a locale.
struct DisplayName(Locale);

impl fmt::Display for DisplayName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Locale { language, country } = self.0;

        if language.is_default() {
            return f.write_str("default");
        }

        f.write_str(language.name().unwrap_or(language.as_str()))?;

        if !country.is_default() {
            f.write_str(" (")?;
            f.write_str(country.name().unwrap_or(country.as_str()))?;
            f.write_str(")")?;
        }

        Ok(())
    }
}

impl fmt::Display for Locale {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_default() {
            return f.write_str("default");
        }

        match self.language.to_tag() {
            Some(tag) => f.write_str(tag)?,
            None => f.write_str(self.language.as_str())?,
        }

        if !self.country.is_default() {
            write!(f, "-{}", self.country)?;
        }

        Ok(())
    }
}

/// Formats as `Locale(en-US)`, naming the type around the display form.
/// [`Locale::DEFAULT`] shows `Locale(default)`.
impl fmt::Debug for Locale {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Locale({self})")
    }
}

impl FromStr for Locale {
    type Err = ParseLocaleErr;

    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

#[cfg(feature = "serde")]
#[cfg_attr(koine_docsrs, doc(cfg(feature = "serde")))]
impl serde::Serialize for Locale {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.collect_str(self)
    }
}

#[cfg(feature = "serde")]
#[cfg_attr(koine_docsrs, doc(cfg(feature = "serde")))]
impl<'de> serde::Deserialize<'de> for Locale {
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct Visitor;

        impl serde::de::Visitor<'_> for Visitor {
            type Value = Locale;

            #[inline]
            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a 'language[-country]' locale code or 'default'")
            }

            #[inline]
            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                v.parse().map_err(serde::de::Error::custom)
            }
        }

        deserializer.deserialize_str(Visitor)
    }
}

/// musli `Encode` and `Decode` as the string form, against the musli-core
/// named `$musli` under `$feature`.
macro_rules! locale_musli {
    ($feature:literal, $musli:ident) => {
        #[cfg(feature = $feature)]
        #[cfg_attr(koine_docsrs, doc(cfg(feature = $feature)))]
        impl<M> $musli::Encode<M> for Locale {
            type Encode = Self;

            #[inline]
            fn encode<E>(&self, encoder: E) -> Result<(), E::Error>
            where
                E: $musli::Encoder<Mode = M>,
            {
                // Not `musli::display::encode`: it formats through
                // `Encoder::collect_string`, which allocates a buffer from the
                // context's allocator. The longest form is a 3-letter
                // language, `-` and a 2-letter country, so a stack buffer
                // does.
                let mut buf = Buf::default();
                fmt::write(&mut buf, format_args!("{self}")).map_err(|_| {
                    $musli::Context::message(encoder.cx(), "locale longer than expected")
                })?;
                encoder.encode_string(buf.as_str())
            }

            #[inline]
            fn as_encode(&self) -> &Self::Encode {
                self
            }
        }

        #[cfg(feature = $feature)]
        #[cfg_attr(koine_docsrs, doc(cfg(feature = $feature)))]
        impl<'de, M, A> $musli::Decode<'de, M, A> for Locale
        where
            A: $musli::Allocator,
        {
            #[inline]
            fn decode<D>(decoder: D) -> Result<Self, D::Error>
            where
                D: $musli::Decoder<'de, Mode = M, Allocator = A>,
            {
                $musli::display::decode(decoder)
            }
        }
    };
}

locale_musli!("musli02", musli_core02);

/// A fixed buffer to format a locale into without allocating.
#[cfg(feature = "musli02")]
#[derive(Default)]
struct Buf {
    bytes: [u8; 16],
    len: usize,
}

#[cfg(feature = "musli02")]
impl Buf {
    fn as_str(&self) -> &str {
        core::str::from_utf8(&self.bytes[..self.len]).unwrap_or_default()
    }
}

#[cfg(feature = "musli02")]
impl fmt::Write for Buf {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let end = self.len + s.len();
        let target = self.bytes.get_mut(self.len..end).ok_or(fmt::Error)?;
        target.copy_from_slice(s.as_bytes());
        self.len = end;
        Ok(())
    }
}

/// sqll `FromColumn` and `BindValue` against the sqll named `$sqll` under
/// `$feature`.
macro_rules! locale_sqll {
    ($feature:literal, $sqll:ident) => {
        /// Stored as the integer of [`Locale::to_u64`]; see
        /// [Storage in SQLite](crate#storage-in-sqlite).
        #[cfg(feature = $feature)]
        #[cfg_attr(koine_docsrs, doc(cfg(feature = $feature)))]
        impl $sqll::FromColumn<'_> for Locale {
            type Type = $sqll::ty::Integer;

            #[inline]
            fn from_column(
                stmt: &$sqll::Statement,
                index: $sqll::ty::Integer,
            ) -> $sqll::Result<Self> {
                let value = i64::from_column(stmt, index)?;

                // Only the shape is checked, so that a code missing from the current
                // tables still reads.
                match Locale::from_u64_lossless(value.cast_unsigned()) {
                    Some(locale) => Ok(locale),
                    None => Err($sqll::Error::new(
                        $sqll::Code::MISMATCH,
                        "invalid stored locale",
                    )),
                }
            }
        }

        /// Stored as the integer of [`Locale::to_u64`]; see
        /// [Storage in SQLite](crate#storage-in-sqlite).
        #[cfg(feature = $feature)]
        #[cfg_attr(koine_docsrs, doc(cfg(feature = $feature)))]
        impl $sqll::BindValue for Locale {
            #[inline]
            fn bind_value(
                &self,
                stmt: &mut $sqll::Statement,
                index: $sqll::Index,
            ) -> $sqll::Result<()> {
                self.to_u64().cast_signed().bind_value(stmt, index)
            }
        }
    };
}

locale_sqll!("sqll015", sqll015);

#[cfg(feature = "yew023")]
#[cfg_attr(koine_docsrs, doc(cfg(feature = "yew023")))]
impl yew023::html::IntoPropValue<yew023::virtual_dom::VNode> for Locale {
    #[inline]
    fn into_prop_value(self) -> yew023::virtual_dom::VNode {
        alloc::string::ToString::to_string(&self).into()
    }
}

/// The locale's string form, or `None` for [`Locale::DEFAULT`].
#[cfg(feature = "yew023")]
#[cfg_attr(koine_docsrs, doc(cfg(feature = "yew023")))]
impl yew023::html::IntoPropValue<Option<yew023::AttrValue>> for Locale {
    #[inline]
    fn into_prop_value(self) -> Option<yew023::AttrValue> {
        if self.is_default() {
            None
        } else {
            Some(alloc::string::ToString::to_string(&self).into())
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn debug_names_the_type() {
        let en_us = Locale::new(Language::ENG, Country::US);
        assert_eq!(alloc::format!("{en_us:?}"), "Locale(en-US)");
        assert_eq!(alloc::format!("{:?}", Locale::DEFAULT), "Locale(default)");
    }

    use alloc::string::ToString;

    use super::*;

    /// The table is sorted by language then country, which `by_language`'s
    /// binary search relies on.
    #[test]
    fn table_is_sorted_by_language_then_country() {
        assert!(ENTRIES.is_sorted_by_key(|e| (e.language, e.country)));
        assert!(ENTRIES.is_sorted_by_key(|e| (e.to_language(), e.to_country())));
    }

    #[test]
    fn by_language_matches_a_full_scan() {
        for language in crate::language::iter().map(|e| e.to_language()) {
            let found = by_language(language);
            let expected = ENTRIES.iter().filter(|e| e.to_language() == language);
            assert_eq!(found.len(), expected.clone().count(), "{language:?}");
            assert!(found.eq(expected), "{language:?}");
        }

        assert_eq!(by_language(Language::DEFAULT).len(), 0);
    }

    #[test]
    fn by_country_matches_a_full_scan() {
        for country in crate::country::iter().map(|e| e.to_country()) {
            let expected = ENTRIES.iter().filter(|e| e.country == country.as_str());
            assert!(by_country(country).eq(expected), "{country:?}");
        }

        assert_eq!(by_country(Country::DEFAULT).count(), 0);
    }

    #[test]
    fn every_entry_is_a_listed_locale() {
        for entry in iter() {
            let language = entry.to_language();
            let country = entry.to_country();
            assert!(!language.is_default(), "{entry:?}");
            assert!(!country.is_default(), "{entry:?}");
            assert_eq!(language, Language::new(entry.language.as_bytes()));
            assert_eq!(country, Country::new(entry.country.as_bytes()));

            let locale = entry.to_locale();
            assert_eq!(locale, Locale::new(language, country));
            assert!(locale.is_listed(), "{entry:?}");
            assert_eq!(locale.to_string().parse::<Locale>(), Ok(locale));
            assert_eq!(Locale::from_u64(locale.to_u64()), locale);
        }
    }

    #[test]
    fn parse_errors_say_which_part_failed() {
        use ParseLocaleErrKind::*;

        let cases = [
            ("zz", UnknownLanguage),
            ("zz-US", UnknownLanguage),
            ("zzz-US", UnknownLanguage),
            ("zz-USA", UnknownLanguage),
            ("en-ZZ", UnknownCountry),
            ("eng-zz", UnknownCountry),
            ("en_US", Malformed),
            ("en-", Malformed),
            ("-US", Malformed),
            ("en-default", Malformed),
            ("default-US", Malformed),
            ("en-USA", Malformed),
            ("en-US-x", Malformed),
            ("e", Malformed),
            ("english", Malformed),
            ("e1-US", Malformed),
        ];

        for (input, kind) in cases {
            let err = input.parse::<Locale>().unwrap_err();
            assert_eq!(err.kind(), kind, "{input}");
            assert_eq!(Locale::from_iso(input), None, "{input}");
        }
    }

    #[test]
    fn code_parse_errors_say_unknown_or_malformed() {
        use crate::country::ParseCountryErrKind;
        use crate::language::ParseLanguageErrKind;

        for (input, kind) in [
            ("zz", ParseLanguageErrKind::Unknown),
            ("zzz", ParseLanguageErrKind::Unknown),
            ("e", ParseLanguageErrKind::Malformed),
            ("engl", ParseLanguageErrKind::Malformed),
            ("e1", ParseLanguageErrKind::Malformed),
        ] {
            let err = input.parse::<Language>().unwrap_err();
            assert_eq!(err.kind(), kind, "{input}");
        }

        for (input, kind) in [
            ("ZZ", ParseCountryErrKind::Unknown),
            ("U", ParseCountryErrKind::Malformed),
            ("USA", ParseCountryErrKind::Malformed),
            ("U1", ParseCountryErrKind::Malformed),
        ] {
            let err = input.parse::<Country>().unwrap_err();
            assert_eq!(err.kind(), kind, "{input}");
        }
    }

    #[test]
    fn display_name_falls_back_to_codes() {
        // "qqq" and "QQ" are well formed but in neither table.
        let locale = Locale::from_u64_lossless(0x5151_0000_7171_7100).unwrap();
        assert_eq!(locale.to_string(), "qqq-QQ");
        assert_eq!(locale.display_name().to_string(), "qqq (QQ)");

        let locale = Locale::new(
            Language::new(b"swe"),
            Country::from_raw_unchecked(*b"QQ\0\0"),
        );
        assert_eq!(locale.display_name().to_string(), "Swedish (QQ)");
    }

    #[test]
    fn contains_rejects_long_keys() {
        assert!(!contains("english", "US"));
        assert!(!contains("eng", "USA"));
    }
}
