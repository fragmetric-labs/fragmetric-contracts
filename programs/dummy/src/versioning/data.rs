use anchor_lang::prelude::*;

use super::*;

impl DataV1 {
    pub fn update(&mut self, request: Request) -> Result<()> {
        self.field1 = request.field1;
        self.field2 = request.field2;
        Ok(())
    }
}
