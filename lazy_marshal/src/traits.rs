use crate::error::MarshalError;

pub trait Marshal: Sized {
    /// Marshal the object into an iterator of bytes
    /// ```
    /// use lazy_marshal::prelude::*;
    ///
    /// let i: Vec<u8> = 260u32.marshal().collect();
    /// assert!(i == vec![0, 0, 1, 4]);
    /// ```
    fn marshal(self) -> impl Iterator<Item = u8>;
}

pub trait UnMarshal: Sized {
    /// Unmarshal an iterator of bytes into `Self`
    ///
    /// # Errors
    /// It errors when the data doesn't make sense or can't be decoded to a meaningful output
    ///
    /// ```
    /// use lazy_marshal::prelude::*;
    ///
    /// let mut i = "Hello, World!".marshal();
    /// let decoded = String::unmarshal(&mut i).unwrap();
    ///
    /// assert_eq!("Hello, World!".to_string(), decoded);
    /// ```
    fn unmarshal(data: &mut impl Iterator<Item = u8>) -> Result<Self, MarshalError>;
}

/// Inspired by Rayon's [Either](https://crates.io/crates/either) crate
/// Just the minimal amount needed to make things work for this use case
pub(crate) enum Either<L, R> {
    Left(L),
    Right(R),
}

impl<L, R> Iterator for Either<L, R>
where
    L: Iterator,
    R: Iterator<Item = L::Item>,
{
    type Item = L::Item;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Either::Left(iter) => iter.next(),
            Either::Right(iter) => iter.next(),
        }
    }
}

/// Used as a wrapper when the [`Marshal::marshal()`] types differ on return values.
/// This tends to happen frequently when marshalling enums, so having a `Box<dyn Iterator>`
/// can cover all varients of the enum.
///
/// **Sample Enum**
/// ```
/// enum MaybeU32 {
///     None,
///     Some(u32),
///     More(u32, u32),
/// }
/// ```
/// **Will fail to compile**
/// ```compile_fail
/// # use lazy_marshal::prelude::*;
/// # enum MaybeU32 {
/// #     None,
/// #     Some(u32),
/// #     More(u32, u32),
/// # }
///
/// impl Marshal for MaybeU32 {
///     fn marshal(self) -> impl Iterator<Item = u8> {
///         match self {
///             MaybeU32::None => 0u8.marshal(),
///             MaybeU32::Some(val) => 1u8.marshal().chain(val.marshal()),
///             MaybeU32::More(val1, val2) => 2u8.marshal().chain(val1.marshal()).chain(val2.marshal()),
///         }
///     }
/// }
/// ```
///
/// **Compiles successfully**
/// ```
/// # use lazy_marshal::prelude::*;
/// # enum MaybeU32 {
/// #     None,
/// #     Some(u32),
/// #     More(u32, u32),
/// # }
///
/// impl Marshal for MaybeU32 {
///     fn marshal(self) -> impl Iterator<Item = u8> {
///         match self {
///             MaybeU32::None => MarshalIterator(Box::new(0u8.marshal())),
///             MaybeU32::Some(val) => MarshalIterator(Box::new(1u8.marshal().chain(val.marshal()))),
///             MaybeU32::More(val1, val2) => MarshalIterator(Box::new(2u8.marshal().chain(val1.marshal()).chain(val2.marshal()))),
///         }
///     }
/// }
/// ```
pub struct MarshalIterator(pub Box<dyn Iterator<Item = u8>>);

impl Iterator for MarshalIterator {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}
