use alloc::vec::Vec;
use anyhow::{bail, Result};
use vital_script_primitives::{
    names::Name,
    resources::{self, Resource, Tag, VRC20, VRC721},
    traits::context::InputResourcesContext as InputResourcesContextT,
    H256, U256,
};

use crate::TARGET;

const VEC_CAP_SIZE: usize = 8;

#[derive(Clone)]
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

    fn cost(&mut self, resource: &Resource) -> Result<()> {
        self.inputs.cost(resource)
    }

    fn get_uncosted_vrc20(&self, name: Tag) -> Option<Resource> {
        log::debug!(target: TARGET, "get_uncosted_vrc20 {:?}", name);
        for vrc20 in &self.inputs.vrc20s {
            log::debug!(target: TARGET, "vrc20 {:?}", vrc20);
            if !vrc20.is_costed() && vrc20.name == name {
                if vrc20.amount.is_zero() {
                    return None
                } else {
                    let alive = vrc20.amount.saturating_sub(vrc20.costed);

                    return Some(Resource::VRC20(VRC20::new(name, alive)));
                }
            }
        }

        None
    }

    fn all(&self) -> &[u8] {
        &self.inputs_indexs
    }

    fn uncosted(&self) -> Vec<(u8, Resource)> {
        self.inputs.uncosted_inputs()
    }
}

#[derive(Clone)]
pub struct NameInput {
    index: u8,
    costed: bool,
    name: Name,
}

#[derive(Debug, Clone)]
pub struct VRC20Input {
    index: u8,
    amount: U256,
}

#[derive(Debug, Clone)]
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

#[derive(Clone)]
pub struct VRC721Input {
    index: u8,
    costed: bool,
    hash: H256,
}

#[derive(Clone)]
pub struct VRC721Inputs {
    name: Tag,
    inputs: Vec<VRC721Input>,
}

impl VRC721Inputs {
    pub fn cost(&mut self, hash: H256) -> Result<()> {
        for i in self.inputs.iter_mut() {
            if i.hash == hash {
                if i.costed {
                    bail!("had already costed");
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

#[derive(Clone)]
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
        log::debug!(target: TARGET, "push input name: {} {}", index, name);

        self.names.push(NameInput { index, name, costed: false })
    }

    pub fn push_vrc20(&mut self, index: u8, name: Tag, amount: U256) {
        log::debug!(target: TARGET, "push input vrc20: {} {} {}", index, name, amount);

        for vrc20 in self.vrc20s.iter_mut() {
            if vrc20.name == name {
                vrc20.amount = vrc20.amount.saturating_add(amount);
                vrc20.inputs.push(VRC20Input { index, amount });
                return;
            }
        }

        let mut inputs = Vec::with_capacity(VEC_CAP_SIZE);
        inputs.push(VRC20Input { index, amount });

        self.vrc20s.push(VRC20Inputs { name, amount, costed: U256::zero(), inputs });
    }

    pub fn push_vrc721(&mut self, index: u8, name: Tag, hash: H256) {
        log::debug!(target: TARGET, "push input vrc721: {} {} {}", index, name, hash);

        let new = VRC721Input { index, costed: false, hash };

        for vrc721s in self.vrc721s.iter_mut() {
            if vrc721s.name == name {
                vrc721s.inputs.push(new);
                return;
            }
        }

        let mut inputs = Vec::with_capacity(VEC_CAP_SIZE);
        inputs.push(new);

        self.vrc721s.push(VRC721Inputs { name, inputs })
    }

    /// If all input resources had been costed.
    #[allow(dead_code)]
    pub fn is_costed(&self) -> bool {
        self.vrc20s.iter().all(|v| v.is_costed())
            && self.vrc721s.iter().all(|v| v.is_costed())
            && self.names.iter().all(|v| v.costed)
    }

    /// Cost the resources from input
    pub fn cost(&mut self, resource: &Resource) -> Result<()> {
        match resource {
            Resource::Name(n) => self.cost_name(n),
            Resource::VRC20(v) => self.cost_vrc20(v),
            Resource::VRC721(v) => self.cost_vrc721(v),
        }
    }

    pub fn cost_vrc20(&mut self, resource: &resources::VRC20) -> Result<()> {
        log::debug!(target: TARGET, "cost_vrc20: {}", resource);

        for v in self.vrc20s.iter_mut() {
            if v.name == resource.name {
                return v.cost(resource.amount);
            }
        }

        bail!("not found res in inputs")
    }

    pub fn cost_vrc721(&mut self, resource: &resources::VRC721) -> Result<()> {
        log::debug!(target: TARGET, "cost_vrc721: {}", resource);

        for v in self.vrc721s.iter_mut() {
            if v.name == resource.name {
                return v.cost(resource.hash);
            }
        }

        bail!("not found res in inputs")
    }

    pub fn cost_name(&mut self, resource: &resources::Name) -> Result<()> {
        log::debug!(target: TARGET, "cost_name: {}", resource);

        for v in self.names.iter_mut() {
            if &v.name == resource {
                if v.costed {
                    bail!("had already costed")
                } else {
                    v.costed = true;
                    return Ok(())
                }
            }
        }

        bail!("not found name {} res in inputs", resource)
    }

    /// Return the uncosted resource with the input index.
    /// This resources will be put into the space.
    /// The cost rule will be first-indexed first-cost.
    pub fn uncosted_inputs(&self) -> Vec<(u8, Resource)> {
        let mut res = Vec::with_capacity(8);

        for vrc20 in self.vrc20s.iter() {
            log::debug!(target: TARGET, "the input vrc20: {:?}", vrc20);
            if !vrc20.is_costed() {
                // we calculate the uncosted amount, the cost is first-indexed first-cost,
                // so we need from the lastest one.
                let mut uncosted_amount = vrc20.amount.saturating_sub(vrc20.costed);

                log::debug!(target: TARGET, "the input vrc20: {} {:?}", vrc20.name.to_string(), uncosted_amount);

                let iter = (0..vrc20.inputs.len()).rev();

                for input_index in iter {
                    let input = &vrc20.inputs[input_index];

                    log::debug!(target: TARGET, "process input vrc20: {} {} {:?}", input_index, uncosted_amount, input.amount);
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mock::*;

    #[test]
    fn test_input_resources_context_should_work() {
        let mut ctx = InputResourcesContext::new(8);

        let resources = vec![
            Name::must_from("abc").into(),
            Resource::vrc20("abc", 10000.into()).expect("vrc20"),
            Resource::vrc721("abc", H256::random()).expect("vrc721"),
        ];

        assert!(ctx.push(0, resources[0].clone()).is_ok(), "the push name should ok");
        assert!(ctx.push(1, resources[1].clone()).is_ok(), "the push vrc20 should ok");
        assert!(ctx.push(2, resources[2].clone()).is_ok(), "the push vrc721 should ok");

        assert_eq!(ctx.inputs_indexs, vec![0, 1, 2], "the inputs index should eq");
        assert_eq!(ctx.all(), vec![0, 1, 2], "the all inputs index should eq");

        let mut uncosted = ctx.uncosted();
        uncosted.sort();

        let mut expect = resources
            .iter()
            .enumerate()
            .map(|(i, r)| (i as u8, r.clone()))
            .collect::<Vec<_>>();
        expect.sort();

        assert_eq!(uncosted, expect, "the inputs index should eq");
    }

    #[test]
    fn test_input_resources_context_vrc20_should_work() {
        let mut ctx = InputResourcesContext::new(8);

        let resources = vec![
            Resource::vrc20("abc", 10000.into()).expect("vrc20"),
            Resource::vrc20("abc", 10000.into()).expect("vrc20"),
            Resource::vrc20("abc", 10000.into()).expect("vrc20"),
            Resource::vrc20("abcdefgh", 10000.into()).expect("vrc20"),
            Resource::vrc20("abcdefgh", 20000.into()).expect("vrc20"),
            Resource::vrc20("abcdefgh", 30000.into()).expect("vrc20"),
            Resource::vrc20("abcdefgh", 40000.into()).expect("vrc20"),
        ];

        for (i, r) in resources.iter().enumerate() {
            assert!(ctx.push(i as u8, r.clone()).is_ok(), "the push vrc20 should ok");
        }

        assert_eq!(ctx.inputs.vrc20s.len(), 2, "the inputs should merged");
        assert_eq!(ctx.inputs.vrc20s[0].name.to_string(), "abc");
        assert_eq!(ctx.inputs.vrc20s[0].amount, U256::from(30000));
        assert_eq!(ctx.inputs.vrc20s[0].inputs.len(), 3);
        assert_eq!(ctx.inputs.vrc20s[1].name.to_string(), "abcdefgh");
        assert_eq!(ctx.inputs.vrc20s[1].amount, U256::from(100000));
        assert_eq!(ctx.inputs.vrc20s[1].inputs.len(), 4);
    }

    #[test]
    fn test_input_resources_context_vrc20_cost_should_work() {
        let mut ctx = InputResourcesContext::new(8);

        let resources = vec![
            Resource::vrc20("abc", 10000.into()).expect("vrc20"),
            Resource::vrc20("abc", 10000.into()).expect("vrc20"),
            Resource::vrc20("abc", 10000.into()).expect("vrc20"),
            Resource::vrc20("abcdefgh", 10000.into()).expect("vrc20"),
            Resource::vrc20("abcdefgh", 20000.into()).expect("vrc20"),
            Resource::vrc20("abcdefgh", 30000.into()).expect("vrc20"),
            Resource::vrc20("abcdefgh", 40000.into()).expect("vrc20"),
        ];

        for (i, r) in resources.iter().enumerate() {
            assert!(ctx.push(i as u8, r.clone()).is_ok(), "the push vrc20 should ok");
        }

        assert_err_str(
            ctx.cost(&Resource::vrc20("abcd", 10000.into()).expect("vrc20")),
            "not found res in inputs",
            "cost no pushed",
        );

        assert_err_str(
            ctx.cost(&Resource::vrc20("abc", 40000.into()).expect("vrc20")),
            "not enough inputs",
            "cost too much",
        );

        // cost first one
        {
            assert!(ctx.cost(&Resource::vrc20("abc", 8000.into()).expect("vrc20")).is_ok());
            let uncosted =
                ctx.get_uncosted_vrc20(Name::must_from("abc")).expect("should not costed all");
            assert_eq!(uncosted, Resource::vrc20("abc", 22000.into()).expect("vrc20"));

            assert_eq!(ctx.inputs.vrc20s[0].costed, U256::from(8000));

            let mut uncosted = ctx.uncosted();
            uncosted.sort();

            assert_eq!(
                uncosted,
                vec![
                    Resource::vrc20("abc", 2000.into()).expect("vrc20"),
                    Resource::vrc20("abc", 10000.into()).expect("vrc20"),
                    Resource::vrc20("abc", 10000.into()).expect("vrc20"),
                    Resource::vrc20("abcdefgh", 10000.into()).expect("vrc20"),
                    Resource::vrc20("abcdefgh", 20000.into()).expect("vrc20"),
                    Resource::vrc20("abcdefgh", 30000.into()).expect("vrc20"),
                    Resource::vrc20("abcdefgh", 40000.into()).expect("vrc20"),
                ]
                .into_iter()
                .enumerate()
                .map(|(i, r)| (i as u8, r))
                .collect::<Vec<_>>()
            );
        }

        {
            assert!(ctx.cost(&Resource::vrc20("abc", 2000.into()).expect("vrc20")).is_ok());
            let uncosted =
                ctx.get_uncosted_vrc20(Name::must_from("abc")).expect("should not costed all");
            assert_eq!(uncosted, Resource::vrc20("abc", 20000.into()).expect("vrc20"));

            assert_eq!(ctx.inputs.vrc20s[0].costed, U256::from(10000));

            let mut uncosted = ctx.uncosted();
            uncosted.sort();

            // first in, first costed
            assert_eq!(
                uncosted,
                vec![
                    (1, Resource::vrc20("abc", 10000.into()).expect("vrc20")),
                    (2, Resource::vrc20("abc", 10000.into()).expect("vrc20")),
                    (3, Resource::vrc20("abcdefgh", 10000.into()).expect("vrc20")),
                    (4, Resource::vrc20("abcdefgh", 20000.into()).expect("vrc20")),
                    (5, Resource::vrc20("abcdefgh", 30000.into()).expect("vrc20")),
                    (6, Resource::vrc20("abcdefgh", 40000.into()).expect("vrc20")),
                ]
            );
        }

        {
            assert!(ctx.cost(&Resource::vrc20("abc", 14000.into()).expect("vrc20")).is_ok());
            let uncosted =
                ctx.get_uncosted_vrc20(Name::must_from("abc")).expect("should not costed all");
            assert_eq!(uncosted, Resource::vrc20("abc", 6000.into()).expect("vrc20"));

            assert_eq!(ctx.inputs.vrc20s[0].costed, U256::from(24000));

            let mut uncosted = ctx.uncosted();
            uncosted.sort();

            // first in, first costed
            assert_eq!(
                uncosted,
                vec![
                    (2, Resource::vrc20("abc", 6000.into()).expect("vrc20")),
                    (3, Resource::vrc20("abcdefgh", 10000.into()).expect("vrc20")),
                    (4, Resource::vrc20("abcdefgh", 20000.into()).expect("vrc20")),
                    (5, Resource::vrc20("abcdefgh", 30000.into()).expect("vrc20")),
                    (6, Resource::vrc20("abcdefgh", 40000.into()).expect("vrc20")),
                ]
            );
        }

        {
            assert!(ctx.cost(&Resource::vrc20("abc", 6000.into()).expect("vrc20")).is_ok());
            let uncosted = ctx.get_uncosted_vrc20(Name::must_from("abc"));
            assert_eq!(uncosted, None);

            assert_eq!(ctx.inputs.vrc20s[0].costed, U256::from(30000));

            let mut uncosted = ctx.uncosted();
            uncosted.sort();

            // first in, first costed
            assert_eq!(
                uncosted,
                vec![
                    (3, Resource::vrc20("abcdefgh", 10000.into()).expect("vrc20")),
                    (4, Resource::vrc20("abcdefgh", 20000.into()).expect("vrc20")),
                    (5, Resource::vrc20("abcdefgh", 30000.into()).expect("vrc20")),
                    (6, Resource::vrc20("abcdefgh", 40000.into()).expect("vrc20")),
                ]
            );
        }

        {
            assert!(ctx.cost(&Resource::vrc20("abcdefgh", 100000.into()).expect("vrc20")).is_ok());
            let uncosted = ctx.get_uncosted_vrc20(Name::must_from("abcdefgh"));
            assert_eq!(uncosted, None);

            assert_eq!(ctx.inputs.vrc20s[1].costed, U256::from(100000));

            let mut uncosted = ctx.uncosted();
            uncosted.sort();

            // first in, first costed
            assert_eq!(uncosted, vec![]);
        }
    }

    #[test]
    fn test_input_resources_context_vrc721_cost_should_work() {
        let mut ctx = InputResourcesContext::new(8);

        let resources = vec![
            Resource::vrc721("abc", H256::random()).expect("vrc721"),
            Resource::vrc721("abc", H256::random()).expect("vrc721"),
            Resource::vrc721("abc", H256::random()).expect("vrc721"),
            Resource::vrc721("abcdefgh", H256::random()).expect("vrc721"),
        ];

        for (i, r) in resources.iter().enumerate() {
            assert!(ctx.push(i as u8, r.clone()).is_ok(), "the push vrc20 should ok");
        }

        assert_err_str(
            ctx.cost(
                &Resource::vrc721("abcd", resources[0].as_vrc721().expect("721").hash)
                    .expect("vrc20"),
            ),
            "not found res in inputs",
            "cost no pushed",
        );

        assert_err_str(
            ctx.cost(&Resource::vrc721("abc", H256::zero()).expect("vrc20")),
            "not enough inputs",
            "cost no pushed at some name",
        );

        {
            assert!(ctx.cost(&resources[0]).is_ok());
            assert!(ctx.inputs.vrc721s[0].inputs[0].costed);

            let mut uncosted = ctx.uncosted();
            uncosted.sort();

            assert_eq!(
                uncosted,
                vec![
                    (1, resources[1].clone()),
                    (2, resources[2].clone()),
                    (3, resources[3].clone())
                ]
            );
        }

        assert_err_str(ctx.cost(&resources[0]), "had already costed", "cost vrc721 had costed");
    }

    #[test]
    fn test_input_resources_context_name_cost_should_work() {
        let mut ctx = InputResourcesContext::new(8);

        let resources: Vec<Resource> = vec![
            Name::must_from("abc").into(),
            Name::must_from("abcd").into(),
            Name::must_from("abcde").into(),
            Name::must_from("abcdef").into(),
        ];

        for (i, r) in resources.iter().enumerate() {
            assert!(ctx.push(i as u8, r.clone()).is_ok(), "the push name should ok");
        }

        assert_err_str(
            ctx.cost(&Name::must_from("aaa").into()),
            "not found name aaa res in inputs",
            "cost no pushed",
        );

        {
            assert!(ctx.cost(&resources[0]).is_ok());
            assert!(ctx.inputs.names[0].costed);

            let mut uncosted = ctx.uncosted();
            uncosted.sort();

            assert_eq!(
                uncosted,
                vec![
                    (1, resources[1].clone()),
                    (2, resources[2].clone()),
                    (3, resources[3].clone())
                ]
            );
        }

        assert_err_str(ctx.cost(&resources[0]), "had already costed", "cost name had costed");

        {
            assert!(ctx.cost(&resources[2]).is_ok());
            assert!(ctx.inputs.names[2].costed);

            let mut uncosted = ctx.uncosted();
            uncosted.sort();

            assert_eq!(uncosted, vec![(1, resources[1].clone()), (3, resources[3].clone())]);
        }
    }
}
