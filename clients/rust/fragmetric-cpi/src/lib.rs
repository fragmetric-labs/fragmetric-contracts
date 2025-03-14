#[doc(hidden)]
#[cfg(feature = "devnet")]
mod __devnet {
    anchor_lang::declare_program!(fragmetric_restaking_devnet);
}

#[doc(hidden)]
#[cfg(not(feature = "devnet"))]
mod __mainnet {
    anchor_lang::declare_program!(fragmetric_restaking);
}

#[cfg(feature = "devnet")]
pub use __devnet::fragmetric_restaking_devnet as fragmetric_restaking;
#[cfg(not(feature = "devnet"))]
pub use __mainnet::fragmetric_restaking;
