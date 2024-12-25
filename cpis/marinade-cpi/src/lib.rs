#[doc(hidden)]
mod __private {
    anchor_lang::declare_program!(marinade);
}

pub use __private::marinade::accounts as state;
pub use __private::marinade::accounts::*;
pub use __private::marinade::types::*;
pub use __private::marinade::*;
