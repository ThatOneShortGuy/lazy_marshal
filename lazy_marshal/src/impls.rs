use std::{collections::HashMap, fmt::Debug, hash::Hash, marker::PhantomData, u64};

use crate::{
    error::MarshalError,
    traits::{Marshal, UnMarshal},
    utils::readn_to_vec,
    Either,
};

impl Marshal for bool {
    #[inline]
    fn marshal(self) -> impl Iterator<Item = u8> {
        let d: u8 = match self {
            true => 1,
            false => 0,
        };
        d.marshal()
    }
}

impl UnMarshal for bool {
    #[inline]
    fn unmarshal(data: &mut impl Iterator<Item = u8>) -> Result<Self, MarshalError> {
        Ok(match u8::unmarshal(data)? {
            0 => false,
            1 => true,
            b => Err(MarshalError::InvalidData(format!(
                "Found '{b}' when unmarshalling a bool. Should be either 0 (false) or 1 (true)"
            )))?,
        })
    }
}

macro_rules! primative_nums {
    ($ty:ident) => {
        impl Marshal for $ty {
            #[inline]
            fn marshal(self) -> impl Iterator<Item = u8> {
                self.to_le_bytes().into_iter()
            }
        }

        impl UnMarshal for $ty {
            fn unmarshal(data: &mut impl Iterator<Item = u8>) -> Result<Self, MarshalError> {
                const TY_SIZE: usize = std::mem::size_of::<$ty>();
                #[allow(invalid_value)]
                let mut d: [u8; TY_SIZE] = unsafe { std::mem::MaybeUninit::uninit().assume_init() };
                for i in 0..TY_SIZE {
                    d[i] = match data.next() {
                        Some(val) => val,
                        None => Err(MarshalError::InvalidSizedDecode(i))?,
                    }
                }
                Ok(Self::from_le_bytes(d))
            }
        }
    };
    ($ty:ident, $cast:ident) => {
        impl Marshal for $ty {
            #[inline]
            fn marshal(self) -> impl Iterator<Item = u8> {
                (self as $cast).to_le_bytes().into_iter()
            }
        }

        impl UnMarshal for $ty {
            fn unmarshal(data: &mut impl Iterator<Item = u8>) -> Result<Self, MarshalError> {
                let d = match readn_to_vec(data, std::mem::size_of::<Self>()) {
                    Ok(a) => a,
                    Err(e) => Err(MarshalError::InvalidSizedDecode(e))?,
                };
                Ok($cast::from_le_bytes(d.try_into().unwrap())
                    .try_into()
                    .map_err(|_| MarshalError::InvalidDecode)?)
            }
        }
    };
}

primative_nums!(usize, u64);
primative_nums!(u8);
primative_nums!(u16);
primative_nums!(u32);
primative_nums!(u64);
primative_nums!(u128);
primative_nums!(isize, i64);
primative_nums!(i8);
primative_nums!(i16);
primative_nums!(i32);
primative_nums!(i64);
primative_nums!(i128);
primative_nums!(f32);
primative_nums!(f64);
primative_nums!(char, u32);

impl Marshal for &str {
    fn marshal(self) -> impl Iterator<Item = u8> {
        let d = self.as_bytes();
        d.len().marshal().chain(d.into_iter().cloned())
    }
}

impl Marshal for String {
    fn marshal(self) -> impl Iterator<Item = u8> {
        let d = self.into_bytes();
        d.len().marshal().chain(d.into_iter())
    }
}

impl<T: Marshal> Marshal for PhantomData<T> {
    fn marshal(self) -> impl Iterator<Item = u8> {
        std::iter::empty()
    }
}

impl UnMarshal for String {
    fn unmarshal(data: &mut impl Iterator<Item = u8>) -> Result<Self, MarshalError> {
        let len = usize::unmarshal(data)?;
        let d = match readn_to_vec(data, len) {
            Ok(v) => v,
            Err(e) => Err(MarshalError::InvalidSizedDecode(e))?,
        };
        Ok(String::from_utf8(d)?)
    }
}

impl<T: Marshal + Clone> Marshal for &[T] {
    fn marshal(self) -> impl Iterator<Item = u8> {
        let len = self.len();
        let d = self.into_iter().cloned().map(|v| v.marshal()).flatten();
        len.marshal().chain(d)
    }
}

impl<T: Marshal> Marshal for Vec<T> {
    fn marshal(self) -> impl Iterator<Item = u8> {
        let len = self.len();
        let d = self.into_iter().map(|v| v.marshal()).flatten();
        len.marshal().chain(d)
    }
}

impl<T: UnMarshal> UnMarshal for Vec<T> {
    fn unmarshal(data: &mut impl Iterator<Item = u8>) -> Result<Self, MarshalError> {
        let len = usize::unmarshal(data)?;
        let val = (0..len)
            .map(|_| T::unmarshal(data))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(val)
    }
}

impl<K, V> Marshal for HashMap<K, V>
where
    K: Marshal,
    V: Marshal,
{
    fn marshal(self) -> impl Iterator<Item = u8> {
        let len = self.len();
        let data = self
            .into_iter()
            .map(|(k, v)| k.marshal().chain(v.marshal()))
            .flatten();
        len.marshal().chain(data)
    }
}

impl<K, V> UnMarshal for HashMap<K, V>
where
    K: UnMarshal + Hash + Eq,
    V: UnMarshal + Debug,
{
    fn unmarshal(data: &mut impl Iterator<Item = u8>) -> Result<Self, MarshalError> {
        let len = usize::unmarshal(data)?;
        let mut val = Self::with_capacity(len);
        for _ in 0..len {
            let key = K::unmarshal(data)?;
            let value = V::unmarshal(data)?;
            match val.insert(key, value) {
                Some(a) => Err(MarshalError::InvalidData(format!(
                    "Duplicate Key while decoding HashMap: {a:#?}"
                )))?,
                None => (),
            };
        }
        Ok(val)
    }
}

impl<T: Marshal> Marshal for Option<T> {
    fn marshal(self) -> impl Iterator<Item = u8> {
        match self {
            Some(v) => Either::Left(1u8.marshal().chain(v.marshal())),
            None => Either::Right(0u8.marshal().chain(std::iter::empty())),
        }
    }
}

impl<T: UnMarshal> UnMarshal for Option<T> {
    fn unmarshal(data: &mut impl Iterator<Item = u8>) -> Result<Self, MarshalError> {
        let variant = u8::unmarshal(data)?;

        Ok(match variant {
            0 => None,
            1 => Some(T::unmarshal(data)?),
            other => Err(MarshalError::InvalidData(format!(
                "Found '{other}' when unmarshalling the option. Should be either 0 or 1"
            )))?,
        })
    }
}

#[cfg(feature = "tuples")]
mod tuples {
    use super::{Marshal, MarshalError, UnMarshal};

    macro_rules! tuple_marshal_inner {
        ($self:ident) => {};
        ($self:ident, $first:tt, $($rest:tt,)+) => {
            $self.$first.marshal().chain(tuple_marshal_inner!($self, $($rest,)+))
        };
        ($self:ident, $first:tt,) => {
            $self.$first.marshal()
        };
    }

    macro_rules! tuple_impl {
        ($([$($n:tt),+])*) => {
            $(
                paste::item! {
                    #[cfg_attr(docsrs, doc(hidden))]
                    impl<$([<T $n>]),+> Marshal for ($([<T $n>],)+)
                    where
                    $([<T $n>]: Marshal,)+
                    {
                        fn marshal(self) -> impl Iterator<Item = u8> {
                            tuple_marshal_inner!(self, $($n,)*)
                        }
                    }
                }
                paste::item! {
                    #[cfg_attr(docsrs, doc(hidden))]
                    impl<$([<T $n>]),+> UnMarshal for ($([<T $n>],)+)
                    where
                    $([<T $n>]: UnMarshal,)+
                    {
                        fn unmarshal(data: &mut impl Iterator<Item = u8>) -> Result<Self, MarshalError> {
                            Ok(($(
                                [<T $n>]::unmarshal(data)?,
                            )+))
                        }
                    }
                }
            )*
        };
    }

    tuple_impl!(
        [0]
        [0, 1]
        [0, 1, 2]
        [0, 1, 2, 3]
        [0, 1, 2, 3, 4]
        [0, 1, 2, 3, 4, 5]
        [0, 1, 2, 3, 4, 5, 6]
        [0, 1, 2, 3, 4, 5, 6, 7]
    );
}

#[cfg(test)]
mod tests;
