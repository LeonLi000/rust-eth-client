use ckb_std::ckb_types::bytes::Bytes;
use super::error::Error;
use super::generated::cell_data::CellDataReader;
use core::result::Result;
use molecule::prelude::*;
use crate::types::basic::BytesVec;


#[derive(Debug)]
pub struct CellDataView {
    pub user_lockscript: Bytes,
    pub headers: Bytes,
}

impl CellDataView {
    pub fn from_slice(slice: &[u8]) -> Result<CellDataView, Error> {
        CellDataReader::verify(slice, false).map_err(|_| Error::Encoding)?;
        let data_reader = CellDataReader::new_unchecked(slice);
        let headers = data_reader.headers().to_entity().as_bytes();
        let user_lockscript = data_reader.user_lockscript().to_entity().as_bytes();
        Ok(CellDataView {
            headers,
            user_lockscript,
        })
    }
}
