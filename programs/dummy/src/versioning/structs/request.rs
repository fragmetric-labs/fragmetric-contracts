use anchor_lang::prelude::*;
use fragmetric_util::request;

#[derive(AnchorSerialize, AnchorDeserialize)]
#[request(Request)]
pub enum InstructionRequest {
    V1(RequestV1),
}

#[derive(Default)]
pub struct Request {
    pub field1: u64,
    pub field2: String,
}

impl From<InstructionRequest> for Request {
    fn from(value: InstructionRequest) -> Self {
        match value {
            InstructionRequest::V1(req) => Self {
                field1: req.field1,
                field2: req.field2,
            },
        }
    }
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct RequestV1 {
    pub field1: u64,
    pub field2: String,
}
