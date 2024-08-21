use anchor_lang::prelude::*;

#[constant]
pub const ADMIN_PUBKEY: Pubkey = pubkey!("91zBeWL8kHBaMtaVrHwWsck1UacDKvje82QQ3HE2k8mJ");
#[constant]
pub const FRAGSOL_MINT_ADDRESS: Pubkey = pubkey!("FRAGsJAbW4cHk2DYhtAWohV6MUMauJHCFtT1vGvRwnXN");

#[cfg(feature = "mainnet")]
pub const BSOL_STAKE_POOL_ADDRESS: Pubkey = pubkey!("stk9ApL5HeVAwPLr3TLhDXdZS8ptVu7zp6ov8HFDuMi");
#[cfg(not(feature = "mainnet"))]
pub const BSOL_STAKE_POOL_ADDRESS: Pubkey = pubkey!("azFVdHtAJN8BX3sbGAYkXvtdjdrT5U6rj9rovvUFos9");

pub const MSOL_STAKE_POOL_ADDRESS: Pubkey = pubkey!("8szGkuLTAux9XMgZ2vtY39jVSowEcpBfFfD8hXSEqdGC");
