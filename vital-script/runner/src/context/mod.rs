pub use vital_script_primitives::traits::context::Context as ContextT;

mod env;
mod input;
mod runner;

pub use env::EnvContext;
pub use input::InputResourcesContext;
pub use runner::RunnerContext;

pub struct Context {
    env: EnvContext,
    input_resources: InputResourcesContext,
    runner: RunnerContext,
}

impl ContextT for Context {
    type Env = EnvContext;
    type InputResource = InputResourcesContext;
    type Runner = RunnerContext;

    fn env(&mut self) -> &mut Self::Env {
        &mut self.env
    }

    fn input_resource(&mut self) -> &mut Self::InputResource {
        &mut self.input_resources
    }

    fn runner(&mut self) -> &mut Self::Runner {
        &mut self.runner
    }
}
