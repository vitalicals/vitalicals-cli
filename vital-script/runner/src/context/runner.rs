use alloc::collections::BTreeSet;
use anyhow::{bail, Result};
use vital_script_primitives::traits::context::RunnerContext as RunnerContextT;

#[derive(Default, Clone)]
pub struct RunnerContext {
    inputs: BTreeSet<u8>,
    outputs: BTreeSet<u8>,
    had_mint: bool,
}

impl RunnerContext {
    pub fn new() -> Self {
        Self::default()
    }
}

impl RunnerContextT for RunnerContext {
    fn is_output_available(&self, index: u8) -> bool {
        self.outputs.contains(&index)
    }

    fn try_assert_input(&mut self, index: u8) -> Result<()> {
        if self.inputs.contains(&index) {
            bail!("the input is already asserted");
        }

        self.inputs.insert(index);

        Ok(())
    }

    fn try_assert_output(&mut self, index: u8) -> Result<()> {
        if self.outputs.contains(&index) {
            bail!("the input is already asserted");
        }

        self.outputs.insert(index);

        Ok(())
    }

    fn try_mint(&mut self) -> Result<()> {
        if self.had_mint{
            bail!("each tx can only have one mint");
        }

        self.had_mint = true;

        Ok(())
    }
}
