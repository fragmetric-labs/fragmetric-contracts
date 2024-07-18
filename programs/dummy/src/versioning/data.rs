use anchor_lang::prelude::*;

use super::*;

impl DataV2 {
    pub fn update(&mut self, request: Request) -> Result<()> {
        self.field1 = request.field1;
        self.field2 = request.field2;
        self.field3 = request.field3;
        self.field4 = request.field4;
        Ok(())
    }
}
