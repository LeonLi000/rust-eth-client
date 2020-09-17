use ckb_std::ckb_types::bytes::Bytes;
use super::error::Error;
use super::generated::cell_data::CellDataReader;
use core::result::Result;
use molecule::prelude::*;


#[derive(Debug)]
pub struct CellDataView {
    pub headers: Bytes,
}

impl CellDataView {
    pub fn from_slice(slice: &[u8]) -> Result<CellDataView, Error> {
        CellDataReader::verify(slice, false).map_err(|_| Error::Encoding)?;
        let data_reader = CellDataReader::new_unchecked(slice);
        let headers = data_reader.headers().to_entity().as_bytes();
        Ok(CellDataView {
            headers,
        })
    }
}
