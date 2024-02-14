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
            bail!("the output is already asserted");
        }

        self.outputs.insert(index);

        Ok(())
    }

    fn try_mint(&mut self) -> Result<()> {
        if self.had_mint {
            bail!("each tx can only have one mint");
        }

        self.had_mint = true;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use vital_script_ops::instruction::{
        assert_input::InstructionInputAssert, assert_output::InstructionOutputAssert, Instruction,
    };
    use vital_script_primitives::{names::Name, resources::Resource};

    use super::*;
    use crate::mock::*;

    #[test]
    fn test_assert_input_can_work() {
        let test_name = "test1";
        let test = Name::try_from(test_name).expect("name format");

        let env_interface = EnvMock::new();
        let mut ctx = TestCtx::new(&env_interface);
        ctx.mint_name(test_name);

        let outpoint1 = ctx.get_name_outpoint(test_name).expect("should mint");

        TestCtx::new(&env_interface)
            .with_instructions(vec![
                Instruction::Input(InstructionInputAssert { index: 1, resource: test.into() }),
                Instruction::Output(InstructionOutputAssert { indexs: vec![0] }),
                Instruction::move_to(0, test),
            ])
            .with_ops()
            .with_input(outpoint1)
            .with_output(2000)
            .run()
            .expect("transfer name failed");
    }

    #[test]
    fn test_assert_input_two_times_will_failed() {
        let test_name = "test1";
        let test = Name::try_from(test_name).expect("name format");

        let env_interface = EnvMock::new();
        let mut ctx = TestCtx::new(&env_interface);
        ctx.mint_name(test_name);

        let outpoint1 = ctx.get_name_outpoint(test_name).expect("should mint");

        let res = TestCtx::new(&env_interface)
            .with_instructions(vec![
                Instruction::Input(InstructionInputAssert { index: 1, resource: test.into() }),
                Instruction::Input(InstructionInputAssert { index: 1, resource: test.into() }),
                Instruction::Output(InstructionOutputAssert { indexs: vec![0] }),
                Instruction::move_to(0, test),
            ])
            .with_ops()
            .with_input(outpoint1)
            .with_output(2000)
            .run();

        assert_eq!(
            res.err()
                .expect("should failed by assert input two times")
                .root_cause()
                .to_string(),
            "the input is already asserted"
        );
    }

    #[test]
    fn test_assert_output_two_times_will_failed() {
        let test_name = "test1";
        let test = Name::try_from(test_name).expect("name format");

        let env_interface = EnvMock::new();
        let mut ctx = TestCtx::new(&env_interface);
        ctx.mint_name(test_name);

        let outpoint1 = ctx.get_name_outpoint(test_name).expect("should mint");

        let res = TestCtx::new(&env_interface)
            .with_instructions(vec![
                Instruction::Input(InstructionInputAssert { index: 1, resource: test.into() }),
                Instruction::Output(InstructionOutputAssert { indexs: vec![0] }),
                Instruction::Output(InstructionOutputAssert { indexs: vec![0] }),
                Instruction::move_to(0, test),
            ])
            .with_ops()
            .with_input(outpoint1)
            .with_output(2000)
            .run();

        assert_eq!(
            res.err()
                .expect("should failed by assert output two times")
                .root_cause()
                .to_string(),
            "the output is already asserted"
        );
    }

    fn test_mints_in_one_tx_res(mints: &[Resource]) -> Result<()> {
        let env_interface = EnvMock::new();

        let outputs = mints.iter().enumerate().map(|(idx, _)| idx as u8).collect::<Vec<_>>();
        let mut ins = Vec::with_capacity(mints.len() + 1);
        ins.push(Instruction::Output(InstructionOutputAssert { indexs: outputs }));
        for (idx, mint) in mints.iter().enumerate() {
            ins.push(Instruction::mint(idx as u8, mint.resource_type()));
        }

        let mut ctx = TestCtx::new(&env_interface).with_instructions(ins).with_ops();

        for _ in 0..mints.len() {
            ctx = ctx.with_output(2000);
        }

        ctx.run()?;

        Ok(())
    }

    #[test]
    fn test_mint_two_times_in_a_tx_will_failed() {
        const ERROR_STR: &str = "each tx can only have one mint";

        let res = test_mints_in_one_tx_res(&[
            Name::must_from("test1").into(),
            Name::must_from("test2").into(),
        ]);
        assert_eq!(
            res.err().expect("should failed by mint two times").root_cause().to_string(),
            ERROR_STR
        );

        let res = test_mints_in_one_tx_res(&[
            Name::must_from("test111111").into(),
            Name::must_from("test2").into(),
        ]);
        assert_eq!(
            res.err().expect("should failed by mint two times").root_cause().to_string(),
            ERROR_STR
        );

        let res = test_mints_in_one_tx_res(&[
            Name::must_from("test111111").into(),
            Name::must_from("test2").into(),
            Name::must_from("test3").into(),
        ]);
        assert_eq!(
            res.err().expect("should failed by mint two times").root_cause().to_string(),
            ERROR_STR
        );
    }
}
