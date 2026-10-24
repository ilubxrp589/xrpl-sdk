//! Typed flags: one type per transaction type and per ledger entry type,
//! so a flag of one type cannot be set on another. Each is a set of bits
//! with named constants, combined with `|`, written as the `u32` `rippled`
//! writes (`#[serde(transparent)]`), and keeping every bit, named or not.
//!
//! ```
//! use xrpl_core::flags::Flags;
//! use xrpl_core::transaction::models::PaymentFlags;
//!
//! let flags = PaymentFlags::PARTIAL_PAYMENT | PaymentFlags::NO_RIPPLE_DIRECT;
//! assert!(flags.contains(PaymentFlags::PARTIAL_PAYMENT));
//! assert_eq!(flags.bits(), 0x0003_0000);
//! assert_eq!(format!("{flags:?}"), "PaymentFlags(NO_RIPPLE_DIRECT | PARTIAL_PAYMENT)");
//! ```

/// What every flags type offers. Implemented by the flags types this crate
/// generates (`PaymentFlags`, `AccountRootFlags`, …).
///
/// ```
/// use xrpl_core::flags::Flags;
/// use xrpl_core::transaction::models::PaymentFlags;
///
/// /// Any flags type, as hex and a count of the bits it does not name.
/// fn describe<F: Flags>(flags: F) -> String {
///     format!("{:#010X} ({} unknown)", flags.bits(), flags.unknown_bits().count_ones())
/// }
///
/// let flags = PaymentFlags::from_bits(0x0002_0000 | 0x0100_0000);
/// assert!(flags.contains(PaymentFlags::PARTIAL_PAYMENT));
/// assert!(PaymentFlags::ALL.contains(PaymentFlags::PARTIAL_PAYMENT));
/// assert_eq!(describe(flags), "0x01020000 (1 unknown)");
/// ```
pub trait Flags: Copy + Default + Eq + core::fmt::Debug + Into<u32> {
    /// Every flag this version names.
    const ALL: Self;

    /// The flags of `bits`, every bit kept (named or not).
    ///
    /// ```
    /// use xrpl_core::flags::Flags;
    /// use xrpl_core::transaction::models::PaymentFlags;
    ///
    /// // A Payment's `Flags` as rippled writes it.
    /// let flags = PaymentFlags::from_bits(131072);
    /// assert_eq!(flags, PaymentFlags::PARTIAL_PAYMENT);
    /// ```
    fn from_bits(bits: u32) -> Self;

    /// The flags as the `u32` `rippled` writes.
    ///
    /// ```
    /// use xrpl_core::flags::Flags;
    /// use xrpl_core::transaction::models::PaymentFlags;
    ///
    /// let flags = PaymentFlags::PARTIAL_PAYMENT | PaymentFlags::LIMIT_QUALITY;
    /// assert_eq!(flags.bits(), 0x0006_0000);
    /// ```
    fn bits(self) -> u32;

    /// Whether every flag of `other` is set.
    ///
    /// ```
    /// use xrpl_core::flags::Flags;
    /// use xrpl_core::transaction::models::PaymentFlags;
    ///
    /// let flags = PaymentFlags::PARTIAL_PAYMENT | PaymentFlags::NO_RIPPLE_DIRECT;
    /// assert!(flags.contains(PaymentFlags::PARTIAL_PAYMENT));
    /// assert!(!flags.contains(PaymentFlags::PARTIAL_PAYMENT | PaymentFlags::LIMIT_QUALITY));
    /// ```
    fn contains(self, other: Self) -> bool {
        self.bits() & other.bits() == other.bits()
    }

    /// Whether no flag is set.
    ///
    /// ```
    /// use xrpl_core::flags::Flags;
    /// use xrpl_core::transaction::models::PaymentFlags;
    ///
    /// assert!(PaymentFlags::default().is_empty());
    /// assert!(!PaymentFlags::PARTIAL_PAYMENT.is_empty());
    /// ```
    fn is_empty(self) -> bool {
        self.bits() == 0
    }

    /// The bits that are set but that this version does not name (a flag of
    /// a newer `rippled`, or one set by mistake).
    ///
    /// ```
    /// use xrpl_core::flags::Flags;
    /// use xrpl_core::transaction::models::PaymentFlags;
    ///
    /// assert_eq!(PaymentFlags::PARTIAL_PAYMENT.unknown_bits(), 0);
    /// // Bit 24 is no Payment flag.
    /// assert_eq!(PaymentFlags::from_bits(0x0102_0000).unknown_bits(), 0x0100_0000);
    /// ```
    fn unknown_bits(self) -> u32 {
        self.bits() & !Self::ALL.bits()
    }
}

/// Define a flags type: `flags_type!(/// docs\n Name { /// doc\n FLAG = 0x…, … });`
/// (The generated types need `std`; without it the macro is unused.)
#[cfg_attr(not(feature = "std"), allow(unused_macros))]
macro_rules! flags_type {
    (
        $(#[$meta:meta])*
        $name:ident { $( $(#[$doc:meta])* $flag:ident = $value:expr ),* $(,)? }
    ) => {
        $(#[$meta])*
        #[derive(Clone, Copy, Default, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
        #[serde(transparent)]
        pub struct $name(u32);

        impl $name {
            $( $(#[$doc])* pub const $flag: Self = Self($value); )*

            /// No flags.
            ///
            #[doc = concat!(
                "```\n",
                "# use xrpl_core::flags::Flags;\n",
                "# use xrpl_core::transaction::models::*;\n",
                "# use xrpl_core::types::ledger_object::*;\n",
                "let flags = ", stringify!($name), "::empty();\n",
                "assert!(flags.is_empty());\n",
                "assert_eq!(flags, ", stringify!($name), "::default());\n",
                "assert_eq!(u32::from(flags), 0);\n",
                "```",
            )]
            pub const fn empty() -> Self {
                Self(0)
            }

            /// The named flags, as (name, bits).
            const NAMED: &'static [(&'static str, u32)] = &[$((stringify!($flag), $value)),*];
        }

        impl $crate::flags::Flags for $name {
            const ALL: Self = Self(0 $(| $value)*);
            fn from_bits(bits: u32) -> Self {
                Self(bits)
            }
            fn bits(self) -> u32 {
                self.0
            }
        }

        impl From<$name> for u32 {
            fn from(flags: $name) -> u32 {
                flags.0
            }
        }

        impl core::ops::BitOr for $name {
            type Output = Self;
            fn bitor(self, other: Self) -> Self {
                Self(self.0 | other.0)
            }
        }

        impl core::ops::BitOrAssign for $name {
            fn bitor_assign(&mut self, other: Self) {
                self.0 |= other.0;
            }
        }

        impl core::ops::BitAnd for $name {
            type Output = Self;
            fn bitand(self, other: Self) -> Self {
                Self(self.0 & other.0)
            }
        }

        impl core::ops::Sub for $name {
            type Output = Self;
            /// The flags of `self` without those of `other`.
            fn sub(self, other: Self) -> Self {
                Self(self.0 & !other.0)
            }
        }

        impl core::fmt::Debug for $name {
            /// The names of the flags set, and any other bits in hex.
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                write!(f, concat!(stringify!($name), "("))?;
                let mut rest = self.0;
                let mut first = true;
                for (name, bits) in Self::NAMED {
                    if *bits != 0 && self.0 & bits == *bits {
                        if !first {
                            f.write_str(" | ")?;
                        }
                        f.write_str(name)?;
                        first = false;
                        rest &= !bits;
                    }
                }
                if rest != 0 {
                    if !first {
                        f.write_str(" | ")?;
                    }
                    write!(f, "{rest:#010X}")?;
                } else if first {
                    f.write_str("empty")?;
                }
                f.write_str(")")
            }
        }
    };
}

#[cfg_attr(not(feature = "std"), allow(unused_imports))]
pub(crate) use flags_type;
