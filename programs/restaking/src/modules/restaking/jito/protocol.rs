use anchor_lang::prelude::*;

pub struct JitoRestakingProtocol {}
impl JitoRestakingProtocol {
    pub const PROGRAM_ID: Pubkey = pubkey!("34X2uqBhEGiWHu43RDEMwrMqXF4CpCPEZNaKdAaUS9jx");
    pub const CONFIG_ADDRESS: Pubkey = pubkey!("Cx2tQmB4RdCQADK8dGzt9sbXDpQ9o2pcjuhKnN42NxbK");
    pub const VAULT_ADDRESS: Pubkey = pubkey!("8bCy6TWfxc7H2ib61ijR1LzGynZNuVspdeUNra9AS9Lg");
    pub const VRT_MINT_ADDRESS: Pubkey = pubkey!("5w2JCmAbBdSRv1y8igM3YNjvnGdfYUeYuVmtw9fU5TXZ");
    pub const DEPOSITOR: Pubkey = pubkey!("93HjmKouPN1giyvzu6xkw1oMqpPCzkPx6EhGELPiSYzx");
    pub const DEPOSITOR_SUPPORTED_TOKEN_ACCOUNT: Pubkey = pubkey!("VGmCK8xrGXxk6yLvUaKUKhgVqLyWZ8aLjV46SZFK4zM");
    pub const DEPOSITOR_VRT_TOKEN_ACCOUNT: Pubkey = pubkey!("414CcD3G9deiXZd5gwgwNtQtB41EwhEiQBfE3KFBVJWz");
    pub const VAULT_RECEIPT_TOKEN_FEE_ACCOUNT: Pubkey = pubkey!("8KZz3scnFXQ8J66hr8CWRTfgo2R21LiuzLY5n9Mv7E9u");
    pub const VAULT_LST_MINT: Pubkey = pubkey!("J1shGRurZVzL2DqZNzfSU8s44H2B94kw5YyyckuszG1N");
}