use anchor_lang::prelude::*;
use fragmetric_util::request;

#[derive(AnchorSerialize, AnchorDeserialize)]
#[request(Request)]
pub enum InstructionRequest {
    V1(RequestV1),
    V2(RequestV2),
}

#[derive(Default)]
pub struct Request {
    pub field1: u64,
    pub field2: u32,
    pub field3: String,
    pub field4: bool,
}

impl From<InstructionRequest> for Request {
    fn from(value: InstructionRequest) -> Self {
        match value {
            InstructionRequest::V1(req) => Self {
                field1: req.field1,
                field3: req.field2,
                ..Default::default()
            },
            InstructionRequest::V2(req) => Self {
                field1: req.field1,
                field2: req.field2,
                field3: req.field3,
                field4: req.field4,
            },
        }
    }
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct RequestV1 {
    pub field1: u64,
    pub field2: String,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct RequestV2 {
    pub field1: u64,
    pub field2: u32,
    pub field3: String,
    pub field4: bool,
}
