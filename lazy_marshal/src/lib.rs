mod error;
mod impls;
mod traits;
mod utils;
pub use error::MarshalError;
pub use traits::*;

pub mod prelude {
    pub use crate::error::*;
    pub use crate::traits::*;

    #[cfg(feature = "derive")]
    pub use lazy_marshal_derive::{Marshal, UnMarshal};
}
