//! The utils

use std::io::Read;

use bytes::{Buf, Bytes};

pub struct Reader<'a> {
    datas: &'a mut Bytes,
}

impl<'a> Reader<'a> {
    pub fn new(datas: &'a mut Bytes) -> Self {
        Self { datas }
    }
}

impl<'a> parity_scale_codec::Input for Reader<'a> {
    fn remaining_len(
        &mut self,
    ) -> core::prelude::v1::Result<Option<usize>, parity_scale_codec::Error> {
        Ok(Some(self.datas.remaining()))
    }

    fn read(
        &mut self,
        into: &mut [u8],
    ) -> core::prelude::v1::Result<(), parity_scale_codec::Error> {
        self.datas
            .reader()
            .read(into)
            .map_err(|_err| parity_scale_codec::Error::from("io"))?;
        Ok(())
    }
}
