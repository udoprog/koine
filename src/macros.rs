/// Define a code type stored as up to four ASCII letters padded with `0`,
/// where the all-zero value is the `DEFAULT` sentinel.
///
/// The defining module provides `parse`, which `FromStr`, serde and musli
/// decoding parse through, and a `Debug` implementation.
#[allow(unused_macros)]
macro_rules! define_code {
    (
        $(#[$meta:meta])*
        pub struct $name:ident;
        $(#[$err_meta:meta])*
        pub struct $err:ident;
        $(#[$kind_meta:meta])*
        pub enum $kind:ident {
            $(#[$unknown_meta:meta])*
            Unknown($unknown_msg:literal),
            $(#[$malformed_meta:meta])*
            Malformed($malformed_msg:literal),
        }
        $(#[$new_meta:meta])*
        const fn new valid by $valid:path;
        case $to_case:ident, $is_case:ident;
        expecting $expecting:literal;
        stored error $stored_msg:literal;
    ) => {
        $(#[$meta])*
        ///
        /// The in-memory representation is the code's bytes directly. The
        /// only place the bytes are turned into an integer is the `sqll014`
        /// conversion, which pins the byte order so the stored value is the
        /// same whatever the host's endianness.
        #[derive(Default, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
        pub struct $name([u8; 4]);

        $(#[$err_meta])*
        #[derive(Debug, Clone, PartialEq, Eq)]
        #[non_exhaustive]
        pub struct $err {
            kind: $kind,
        }

        $(#[$kind_meta])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        #[non_exhaustive]
        pub enum $kind {
            $(#[$unknown_meta])*
            Unknown,
            $(#[$malformed_meta])*
            Malformed,
        }

        impl $err {
            /// Which way the input was wrong.
            #[inline]
            pub const fn kind(&self) -> $kind {
                self.kind
            }

            #[inline]
            const fn unknown() -> Self {
                Self { kind: $kind::Unknown }
            }

            #[inline]
            const fn malformed() -> Self {
                Self { kind: $kind::Malformed }
            }
        }

        impl ::core::fmt::Display for $err {
            #[inline]
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.write_str(match self.kind {
                    $kind::Unknown => $unknown_msg,
                    $kind::Malformed => $malformed_msg,
                })
            }
        }

        impl ::core::error::Error for $err {}

        impl $name {
            /// The sentinel for "no value": never a real code. It displays as
            /// `"default"`, and parses back from `"default"` or an empty
            /// string.
            pub const DEFAULT: $name = $name([0; 4]);

            $(#[$new_meta])*
            #[inline]
            pub const fn new(bytes: &[u8]) -> Self {
                if bytes.len() > 4 {
                    return Self::DEFAULT;
                }

                let mut out = [0u8; 4];
                let mut n = 0;

                while n < bytes.len() {
                    out[n] = bytes[n].$to_case();
                    n += 1;
                }

                let mut test = out.as_slice();

                while let [prefix @ .., 0] = test {
                    test = prefix;
                }

                if !$valid(test) {
                    return Self::DEFAULT;
                }

                Self(out)
            }

            /// Build from the raw representation checking only its shape:
            /// letters in the stored case followed by zero padding, or all
            /// zero for `DEFAULT`. A code missing from the current tables is
            /// kept.
            #[allow(dead_code)]
            #[inline]
            pub(crate) const fn from_raw_shape(bytes: [u8; 4]) -> Option<Self> {
                let mut n = 0;

                while n < bytes.len() && bytes[n] != 0 {
                    // `as_raw_code` relies on the letters being ASCII.
                    if !bytes[n].$is_case() {
                        return None;
                    }

                    n += 1;
                }

                while n < bytes.len() {
                    if bytes[n] != 0 {
                        return None;
                    }

                    n += 1;
                }

                Some(Self(bytes))
            }

            /// Build from raw bytes known to be a valid code, as the tables
            /// hold, without checking them.
            #[inline]
            #[allow(dead_code)]
            pub(crate) const fn from_raw_unchecked(bytes: [u8; 4]) -> Self {
                Self(bytes)
            }

            /// The raw byte-wise representation: the code's ASCII letters
            /// padded with `0`, all zero for `DEFAULT`.
            #[inline]
            pub const fn to_raw(&self) -> [u8; 4] {
                self.0
            }

            /// This code, or `other` if this is `DEFAULT`.
            #[inline]
            pub fn or(self, other: Self) -> Self {
                if self.is_default() { other } else { self }
            }

            /// This code, or `DEFAULT` if `f` returns `false` for it.
            #[inline]
            pub fn filter(self, f: impl FnOnce(Self) -> bool) -> Self {
                if f(self) { self } else { Self::DEFAULT }
            }

            /// The code as a string, `"default"` for `DEFAULT`.
            #[inline]
            pub fn as_str(&self) -> &str {
                if self.is_default() {
                    return "default";
                }

                self.as_raw_code()
            }

            #[inline]
            fn as_raw_code(&self) -> &str {
                let end = self.0.iter().position(|&b| b == 0).unwrap_or(self.0.len());

                // SAFETY: Every constructor only stores ASCII letters and
                // zero padding.
                unsafe { ::core::str::from_utf8_unchecked(&self.0[..end]) }
            }

            /// Whether this is the `DEFAULT` sentinel.
            #[inline]
            pub const fn is_default(self) -> bool {
                matches!(self.0, [0, 0, 0, 0])
            }

            /// Copy `input` into `buf` with normalized case, or `None` if it
            /// is too long or not alphabetic.
            fn normalize<'a>(buf: &'a mut [u8; 4], input: &str) -> Option<&'a str> {
                let bytes = input.as_bytes();

                if bytes.len() > buf.len() {
                    return None;
                }

                for (b, o) in bytes.iter().zip(buf.iter_mut()) {
                    if !b.is_ascii_alphabetic() {
                        return None;
                    }

                    *o = b.$to_case();
                }

                ::core::str::from_utf8(&buf[..bytes.len()]).ok()
            }
        }

        impl ::core::fmt::Display for $name {
            #[inline]
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.write_str(self.as_str())
            }
        }

        impl ::core::str::FromStr for $name {
            type Err = $err;

            #[inline]
            fn from_str(s: &str) -> Result<Self, Self::Err> {
                Self::parse(s)
            }
        }

        #[cfg(feature = "serde")]
        #[cfg_attr(koine_docsrs, doc(cfg(feature = "serde")))]
        impl ::serde::Serialize for $name {
            #[inline]
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: ::serde::Serializer,
            {
                serializer.serialize_str(self.as_str())
            }
        }

        #[cfg(feature = "serde")]
        #[cfg_attr(koine_docsrs, doc(cfg(feature = "serde")))]
        impl<'de> ::serde::Deserialize<'de> for $name {
            #[inline]
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: ::serde::Deserializer<'de>,
            {
                struct Visitor;

                impl ::serde::de::Visitor<'_> for Visitor {
                    type Value = $name;

                    #[inline]
                    fn expecting(
                        &self,
                        formatter: &mut ::core::fmt::Formatter,
                    ) -> ::core::fmt::Result {
                        formatter.write_str($expecting)
                    }

                    #[inline]
                    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
                    where
                        E: ::serde::de::Error,
                    {
                        v.parse().map_err(::serde::de::Error::custom)
                    }
                }

                deserializer.deserialize_str(Visitor)
            }
        }

        #[cfg(feature = "musli01")]
        #[cfg_attr(koine_docsrs, doc(cfg(feature = "musli01")))]
        impl<M> ::musli_core01::Encode<M> for $name {
            type Encode = Self;

            #[inline]
            fn encode<E>(&self, encoder: E) -> Result<(), E::Error>
            where
                E: ::musli_core01::Encoder<Mode = M>,
            {
                encoder.encode_string(self.as_str())
            }

            #[inline]
            fn as_encode(&self) -> &Self::Encode {
                self
            }
        }

        #[cfg(feature = "musli01")]
        #[cfg_attr(koine_docsrs, doc(cfg(feature = "musli01")))]
        impl<'de, M, A> ::musli_core01::Decode<'de, M, A> for $name
        where
            A: ::musli_core01::Allocator,
        {
            #[inline]
            fn decode<D>(decoder: D) -> Result<Self, D::Error>
            where
                D: ::musli_core01::Decoder<'de, Mode = M, Allocator = A>,
            {
                use ::musli_core01::Context as _;
                let cx = decoder.cx();
                decoder.decode_unsized(|s: &str| s.parse::<$name>().map_err(cx.map()))
            }
        }

        #[cfg(feature = "sqll014")]
        #[cfg_attr(koine_docsrs, doc(cfg(feature = "sqll014")))]
        impl ::sqll014::FromColumn<'_> for $name {
            type Type = ::sqll014::ty::Integer;

            #[inline]
            fn from_column(
                stmt: &::sqll014::Statement,
                index: ::sqll014::ty::Integer,
            ) -> ::sqll014::Result<Self> {
                let value = i64::from_column(stmt, index)?;

                // Only the shape is checked, so that a code missing from the
                // current tables still reads.
                if let Ok(value) = u32::try_from(value)
                    && let Some(code) = $name::from_raw_shape(value.to_be_bytes())
                {
                    return Ok(code);
                }

                Err(::sqll014::Error::new(::sqll014::Code::MISMATCH, $stored_msg))
            }
        }

        #[cfg(feature = "sqll014")]
        #[cfg_attr(koine_docsrs, doc(cfg(feature = "sqll014")))]
        impl ::sqll014::BindValue for $name {
            #[inline]
            fn bind_value(
                &self,
                stmt: &mut ::sqll014::Statement,
                index: ::sqll014::Index,
            ) -> ::sqll014::Result<()> {
                i64::from(u32::from_be_bytes(self.0)).bind_value(stmt, index)
            }
        }

        #[cfg(feature = "yew023")]
        #[cfg_attr(koine_docsrs, doc(cfg(feature = "yew023")))]
        impl ::yew023::html::IntoPropValue<::yew023::virtual_dom::VNode> for $name {
            #[inline]
            fn into_prop_value(self) -> ::yew023::virtual_dom::VNode {
                ::alloc::string::ToString::to_string(&self).into()
            }
        }
    };
}
