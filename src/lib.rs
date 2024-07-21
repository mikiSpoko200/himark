
#[macro_export]
macro_rules! denmark {
    ($ty:ty as $($traits:path),+ $(,)?) => {
        $(impl $traits for $ty { })+
    };
    // ($ty:ty as $( $($segments:ident ::)? $traits:ident),+ $(,)?) => {
    //     $(impl $($segments:ident ::)? $traits for $ty { })+
    // };
}

#[cfg(feature = "attrs")]
extern crate himark_proc;

#[cfg(feature = "attrs")]
pub use himark_proc::{mark, marker};
