use alloc::vec::Vec;
use anyhow::{bail, Result};
use vital_script_primitives::{
    names::Name,
    resources::{self, Resource, Tag, VRC20, VRC721},
    traits::context::InputResourcesContext as InputResourcesContextT,
    H256, U256,
};

const VEC_CAP_SIZE: usize = 8;

const INPUT_MAX: usize = 64;
const OUTPUT_MAX: usize = 64;

pub struct InputResourcesContext {
    inputs: InputResources,
    inputs_indexs: Vec<u8>,
}

impl InputResourcesContext {
    pub fn new(cap: usize) -> Self {
        Self { inputs: InputResources::new(cap), inputs_indexs: Vec::with_capacity(cap) }
    }
}

impl InputResourcesContextT for InputResourcesContext {
    fn push(&mut self, input_index: u8, resource: Resource) -> Result<()> {
        match resource {
            Resource::Name(name) => self.inputs.push_name(input_index, name),
            Resource::VRC20(v) => self.inputs.push_vrc20(input_index, v.name, v.amount),
            Resource::VRC721(v) => self.inputs.push_vrc721(input_index, v.name, v.hash),
        }

        self.inputs_indexs.push(input_index);

        Ok(())
    }

    fn cost(&mut self, resource: Resource) -> Result<()> {
        match resource {
            Resource::Name(name) => self.inputs.cost_name(name),
            Resource::VRC20(v) => self.inputs.cost_vrc20(v),
            Resource::VRC721(v) => self.inputs.cost_vrc721(v),
        }
    }

    fn all(&self) -> &[u8] {
        &self.inputs_indexs
    }

    fn uncosted(&self) -> Vec<(u8, Resource)> {
        self.inputs.uncosted_inputs()
    }
}

pub struct NameInput {
    index: u8,
    costed: bool,
    name: Name,
}

pub struct VRC20Input {
    index: u8,
    amount: U256,
}

pub struct VRC20Inputs {
    name: Tag,
    amount: U256,
    costed: U256,
    inputs: Vec<VRC20Input>,
}

impl VRC20Inputs {
    pub fn cost(&mut self, amount: U256) -> Result<()> {
        if self.amount < amount + self.costed {
            bail!("not enough inputs");
        }

        self.costed += amount;

        Ok(())
    }

    pub fn is_costed(&self) -> bool {
        self.amount == self.costed
    }
}

pub struct VRC721Input {
    index: u8,
    costed: bool,
    hash: H256,
}

pub struct VRC721Inputs {
    name: Tag,
    inputs: Vec<VRC721Input>,
}

impl VRC721Inputs {
    pub fn cost(&mut self, hash: H256) -> Result<()> {
        for i in self.inputs.iter_mut() {
            if i.hash == hash {
                if i.costed {
                    bail!("had already cost");
                } else {
                    i.costed = true;
                    return Ok(())
                }
            }
        }

        bail!("not enough inputs")
    }

    pub fn is_costed(&self) -> bool {
        self.inputs.iter().all(|i| i.costed)
    }
}

pub struct InputResources {
    names: Vec<NameInput>,
    vrc20s: Vec<VRC20Inputs>,
    vrc721s: Vec<VRC721Inputs>,
}

impl InputResources {
    pub fn new(cap: usize) -> Self {
        Self {
            names: Vec::with_capacity(cap),
            vrc20s: Vec::with_capacity(cap),
            vrc721s: Vec::with_capacity(cap),
        }
    }

    pub fn push_name(&mut self, index: u8, name: Tag) {
        self.names.push(NameInput { index, name, costed: false })
    }

    pub fn push_vrc20(&mut self, index: u8, name: Tag, amount: U256) {
        for vrc20 in self.vrc20s.iter_mut() {
            if vrc20.name == name {
                vrc20.amount = vrc20.amount.saturating_add(amount);
                vrc20.inputs.push(VRC20Input { index, amount });
                return;
            }
        }

        self.vrc20s.push(VRC20Inputs {
            name,
            amount,
            costed: U256::zero(),
            inputs: Vec::with_capacity(VEC_CAP_SIZE),
        })
    }

    pub fn push_vrc721(&mut self, index: u8, name: Tag, hash: H256) {
        for vrc721s in self.vrc721s.iter_mut() {
            if vrc721s.name == name {
                vrc721s.inputs.push(VRC721Input { index, costed: false, hash });
                return;
            }
        }

        self.vrc721s
            .push(VRC721Inputs { name, inputs: Vec::with_capacity(VEC_CAP_SIZE) })
    }

    /// If all input resources had been costed.
    pub fn is_costed(&self) -> bool {
        self.vrc20s.iter().all(|v| v.is_costed())
            && self.vrc721s.iter().all(|v| v.is_costed())
            && self.names.iter().all(|v| v.costed)
    }

    /// Cost the resources from input
    pub fn cost(&mut self, resource: Resource) -> Result<()> {
        match resource {
            Resource::Name(n) => self.cost_name(n),
            Resource::VRC20(v) => self.cost_vrc20(v),
            Resource::VRC721(v) => self.cost_vrc721(v),
        }
    }

    pub fn cost_vrc20(&mut self, resource: resources::VRC20) -> Result<()> {
        for v in self.vrc20s.iter_mut() {
            if v.name == resource.name {
                return v.cost(resource.amount);
            }
        }

        bail!("no found res in inputs")
    }

    pub fn cost_vrc721(&mut self, resource: resources::VRC721) -> Result<()> {
        for v in self.vrc721s.iter_mut() {
            if v.name == resource.name {
                return v.cost(resource.hash);
            }
        }

        bail!("not found res in inputs")
    }

    pub fn cost_name(&mut self, resource: resources::Name) -> Result<()> {
        for v in self.names.iter_mut() {
            if v.name == resource {
                if v.costed {
                    bail!("had already costed")
                } else {
                    v.costed = true;
                    return Ok(())
                }
            }
        }

        bail!("not found res in inputs")
    }

    /// Return the uncosted resource with the input index.
    /// This resources will be put into the space.
    /// The cost rule will be first-indexed first-cost.
    pub fn uncosted_inputs(&self) -> Vec<(u8, Resource)> {
        let mut res = Vec::with_capacity(8);

        for vrc20 in self.vrc20s.iter() {
            if !vrc20.is_costed() {
                // we calculate the uncosted amount, the cost is first-indexed first-cost,
                // so we need from the lastest one.
                let mut uncosted_amount = vrc20.amount.saturating_sub(vrc20.costed);
                for input_index in vrc20.inputs.len() - 1..=0 {
                    let input = &vrc20.inputs[input_index];
                    if uncosted_amount <= input.amount {
                        // in this input, is all
                        res.push((
                            input.index,
                            Resource::VRC20(VRC20::new(vrc20.name, uncosted_amount)),
                        ));
                        break;
                    } else {
                        // this input is uncosted.
                        uncosted_amount = uncosted_amount.saturating_sub(input.amount);
                        res.push((
                            input.index,
                            Resource::VRC20(VRC20::new(vrc20.name, input.amount)),
                        ));
                    };
                }
            }
        }

        for vrc721 in self.vrc721s.iter() {
            if !vrc721.is_costed() {
                for input in vrc721.inputs.iter() {
                    if !input.costed {
                        res.push((
                            input.index,
                            Resource::VRC721(VRC721::new(vrc721.name, input.hash)),
                        ))
                    }
                }
            }
        }

        for name in self.names.iter() {
            if !name.costed {
                res.push((name.index, Resource::Name(name.name)));
            }
        }

        res
    }
}
