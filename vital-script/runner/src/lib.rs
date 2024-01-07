//! A Runner for vital scripts
//!
//! It will run a vital script then call the impl callback.
//!
//! A Runner need depend the env trait which mainly contains the resource interface.

use anyhow::Result;

pub mod traits;

pub struct Runner {}

impl Runner {
    pub fn run(&self) -> Result<()> {
        // 1. pre check to input and output

        // 1.1 build the input resources

        // 1.2 check ouput resources

        // 2. run opcodes, cost input resources, call env traits.

        // 3. post check

        // 3.1 check all input resources all cost, if not, just set them to shadow space.

        // 3.2 check the output resources

        todo!()
    }
}
