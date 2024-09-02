use fragmetric_util::*;

// This forces `impl From<InstructionRequest> for Request`
#[request(Request)]
pub enum InstructionRequest {
    V1(RequestV1),
    V2(RequestV2),
}

pub struct Request {
    field1: u64,
    field2: String,
}

pub struct RequestV1 {
    field1: u64,
}

pub struct RequestV2 {
    field1: u64,
    field2: u64,
}

impl From<InstructionRequest> for Request {
    fn from(value: InstructionRequest) -> Self {
        match value {
            InstructionRequest::V1(RequestV1 { field1 }) => Self {
                field1,
                field2: Default::default(),
            },
            InstructionRequest::V2(RequestV2 { field1, field2 }) => Self {
                field1,
                field2: field2.to_string(),
            },
        }
    }
}

#[test]
fn test_request_conversion() {
    let old_request = InstructionRequest::V1(RequestV1 { field1: 32 });
    let internal_request: Request = old_request.into();
    assert_eq!(internal_request.field1, 32);
    assert_eq!(internal_request.field2, "");
}
