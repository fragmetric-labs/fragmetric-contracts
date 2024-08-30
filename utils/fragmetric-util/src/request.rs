#[cfg(feature = "derive")]
pub use fragmetric_util_derive::request;

#[doc(hidden)]
pub mod __private {
    /// Trait derived by macro.
    pub trait __IntoRequest<T>: Into<T> {}
}
