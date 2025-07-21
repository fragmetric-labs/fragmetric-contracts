#![allow(dead_code, unused_imports)]

#[cfg(not(any(feature = "devnet", feature = "mainnet")))]
mod local;
#[cfg(not(any(feature = "devnet", feature = "mainnet")))]
pub use local::*;

#[cfg(feature = "devnet")]
mod devnet;
#[cfg(feature = "devnet")]
pub use devnet::*;

#[cfg(feature = "mainnet")]
mod mainnet;
#[cfg(feature = "mainnet")]
pub use mainnet::*;
