#[doc(hidden)]
#[cfg(feature = "devnet")]
mod __devnet {
    anchor_lang::declare_program!(fragmetric_devnet);
}

#[doc(hidden)]
#[cfg(not(feature = "devnet"))]
mod __mainnet {
    anchor_lang::declare_program!(fragmetric);
}

#[cfg(feature = "devnet")]
pub use __devnet::fragmetric_devnet as fragmetric;
#[cfg(not(feature = "devnet"))]
pub use __mainnet::fragmetric;
