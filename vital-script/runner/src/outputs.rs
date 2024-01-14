//! The output index with it 's resource.

use alloc::vec::Vec;

use anyhow::{bail, Result};

use vital_script_primitives::resources::Resource;

#[inline]
fn u8_to_pos(i: u8, c: u8) -> Vec<u8> {
    let mut res = Vec::new();

    for pos in 0..8 {
        let mask = 1 << pos;
        if (mask & i) != 0 {
            res.push(8 * c + pos);
        }
    }

    res
}

pub struct OutputWithResource {
    pub index: u8,
    pub resource: Option<Resource>,
}

pub struct OutputResources {
    outputs: Vec<OutputWithResource>,
}

impl OutputResources {
    pub fn new(cap: usize) -> Self {
        Self { outputs: Vec::with_capacity(cap) }
    }

    pub fn declare_output(&mut self, index: u8) {
        self.outputs.push(OutputWithResource { index, resource: None })
    }

    pub fn declare_output_flag16(&mut self, flag: [u8; 2]) {
        [u8_to_pos(flag[0], 0), u8_to_pos(flag[1], 1)]
            .concat()
            .into_iter()
            .for_each(|index| self.declare_output(index));
    }

    pub fn declare_output_flag32(&mut self, flag: [u8; 4]) {
        [0_u8, 1, 2, 3]
            .map(|c| u8_to_pos(flag[c as usize], c))
            .concat()
            .into_iter()
            .for_each(|index| self.declare_output(index));
    }

    pub fn move_resource_to(&mut self, index: u8, resource: Resource) -> Result<()> {
        for output in self.outputs.iter_mut() {
            if output.index == index {
                if let Some(res) = output.resource.as_mut() {
                    // will check if the resource type is eq.
                    res.merge(&resource)?;
                } else {
                    output.resource = Some(resource);
                }

                return Ok(());
            }
        }

        self.outputs.push(OutputWithResource { index, resource: Some(resource) });

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;

    #[test]
    fn test_u8_to_pos() {
        assert!(u8_to_pos(0, 0).is_empty());
        assert_eq!(u8_to_pos(1, 0), vec![0]);
        assert_eq!(u8_to_pos(2, 0), vec![1]);
        assert_eq!(u8_to_pos(3, 0), vec![0, 1]);
        assert_eq!(u8_to_pos(4, 0), vec![2]);
        assert_eq!(u8_to_pos(5, 0), vec![0, 2]);
        assert_eq!(u8_to_pos(6, 0), vec![1, 2]);
        assert_eq!(u8_to_pos(7, 0), vec![0, 1, 2]);

        assert_eq!(u8_to_pos(10, 0), vec![1, 3]);
        assert_eq!(u8_to_pos(24, 0), vec![3, 4]);
        assert_eq!(u8_to_pos(25, 0), vec![0, 3, 4]);
        assert_eq!(u8_to_pos(153, 0), vec![0, 3, 4, 7]);
        assert_eq!(u8_to_pos(240, 0), vec![4, 5, 6, 7]);

        assert_eq!(u8_to_pos(0xf0, 0), vec![4, 5, 6, 7]);
        assert_eq!(u8_to_pos(0x0f, 0), vec![0, 1, 2, 3]);
        assert_eq!(u8_to_pos(0xff, 0), vec![0, 1, 2, 3, 4, 5, 6, 7]);

        assert_eq!(u8_to_pos(0xf0, 1), vec![12, 13, 14, 15]);
        assert_eq!(u8_to_pos(0x0f, 1), vec![8, 9, 10, 11]);
        assert_eq!(u8_to_pos(0xff, 1), vec![8, 9, 10, 11, 12, 13, 14, 15]);
    }
}
