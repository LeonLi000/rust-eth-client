use crate::types::{Error, CellDataView, witness::WitnessReader, basic::{ChainReader, Uint64}, dags_merkle_roots::DagsMerkleRootsReader, double_node_with_merkle_proof::DoubleNodeWithMerkleProofReader};
use crate::helper::{*, DoubleNodeWithMerkleProof};
use alloc::{vec, vec::Vec};
use ckb_std::{
    ckb_constants::Source,
    debug,
    high_level::{load_cell_data, load_witness_args, QueryIter},
};
use molecule::prelude::{Reader, Builder, Entity};
use eth_spv_lib::eth_types::*;
use crate::types::basic::{BytesVec, HeaderInfo, HeaderInfoReader};
use rlp::Encodable;

pub const HEADER_CACHE_LIMIT: usize = 500;

#[derive(Debug)]
pub struct CellDataTuple(Option<CellDataView>, Option<CellDataView>);

pub fn verify() -> Result<(), Error> {
    let input_data = get_data(Source::GroupInput)?.expect("should not happen");
    let output_data = get_data(Source::GroupOutput)?.expect("should not happen");
    verify_data(&input_data, &output_data)?;
    debug!("verify data finish");
    verify_witness(&input_data, &output_data)?;
    Ok(())
}

fn verify_data(
    input_data: &CellDataView,
    output_data: &CellDataView,
) -> Result<(), Error> {
    if input_data.user_lockscript.as_ref() != output_data.user_lockscript.as_ref()
    {
        return Err(Error::InvalidDataChange);
    }
    Ok(())
}


/// ensure transfer happen on XChain by verifying the spv proof
fn verify_witness(input: &CellDataView, output: &CellDataView) -> Result<(), Error> {
    let witness_args = load_witness_args(0, Source::GroupInput)?.input_type();
    if witness_args.is_none() {
        return Err(Error::InvalidWitness);
    }
    let witness_args = witness_args.to_opt().unwrap().raw_data();
    // debug!("witness_args parsed: {:?}", &witness_args);
    if WitnessReader::verify(&witness_args, false).is_err() {
        return Err(Error::InvalidWitness);
    }
    let witness = WitnessReader::new_unchecked(&witness_args);
    // parse header
    let header_raw = witness.header().raw_data();
    // let header: BlockHeader = rlp::decode(header_raw.to_vec().as_slice()).unwrap();

    // check input && output data
    let header = verify_input_output_data(input, output, header_raw)?;
    // parse merkle proof
    let mut proofs = vec![];
    for i in 0..witness.merkle_proof().len() {
        let proof_raw = witness.merkle_proof().get_unchecked(i).raw_data();
        let proof = parse_proof(proof_raw)?;
        proofs.push(proof);
    }
    // parse dep data
    let merkle_root = parse_dep_data(witness, header.number)?;
    if !verify_header(&header, Option::None, merkle_root, &proofs) {
        return Err(Error::InvalidMerkleProofData);
    }
    Ok(())
}

fn to_u64(data: &Uint64) -> u64 {
    let mut res = [0u8; 8];
    res.copy_from_slice(data.as_slice());
    u64::from_be_bytes(res)
}

fn verify_input_output_data(input: &CellDataView, output: &CellDataView, header_raw: &[u8]) -> Result<BlockHeader, Error> {
    let header: BlockHeader = rlp::decode(header_raw.to_vec().as_slice()).unwrap();
    debug!("header after decode is {:?}", header);

    if ChainReader::verify(&input.headers, false).is_err() {
        return Err(Error::InvalidCellData);
    }
    let chain_input_reader = ChainReader::new_unchecked(&input.headers);
    let main_input_reader = chain_input_reader.main();
    debug!("main_input len: {:?}", main_input_reader.len());
    let uncle_input_reader = chain_input_reader.uncle();
    if ChainReader::verify(&output.headers, false).is_err() {
        return Err(Error::InvalidCellData);
    }
    let chain_output_reader = ChainReader::new_unchecked(&output.headers);
    let main_output_reader = chain_output_reader.main();
    let uncle_output_reader = chain_output_reader.uncle();
    debug!("main_output len: {:?}", main_output_reader.len());
    // let main_tail = main_output_reader.get_unchecked(main_output_reader.len()-1).raw_data();
    // header is on main chain.
    let tail_info_input = main_input_reader.get_unchecked(main_input_reader.len() - 1).raw_data();
    if HeaderInfoReader::verify(&tail_info_input, false).is_err() {
        return Err(Error::InvalidCellData);
    }
    let tail_info_input_reader = HeaderInfoReader::new_unchecked(tail_info_input);
    let tail_header_input = tail_info_input_reader.header().raw_data();

    let tail_info_output = main_output_reader.get_unchecked(main_output_reader.len() - 1).raw_data();
    if HeaderInfoReader::verify(&tail_info_output, false).is_err() {
        return Err(Error::InvalidCellData);
    }
    debug!("tail_info_output: {:?}", tail_info_output);
    let tail_info_output_reader = HeaderInfoReader::new_unchecked(tail_info_output);
    let tail_header_output = tail_info_output_reader.header().raw_data();
    // header is on main chain.
    if tail_header_output == header_raw {
        debug!("header is on main chain");
        let tail_input: BlockHeader = rlp::decode(tail_header_input.to_vec().as_slice()).unwrap();
        debug!("header parent hash: {:?} tail_input hash: {:?}", header.parent_hash.0, tail_input.hash.unwrap().0);
        // if header.parent_hash == tail_input.hash => the chain is not reorg.
        // else do reorg.
        if tail_input.hash.unwrap().0 == header.parent_hash.0 {
            debug!("the chain is not reorg.");
            let prev_difficult: Uint64 = tail_info_input_reader.total_difficulty().to_entity();
            let left: Uint64 = tail_info_output_reader.total_difficulty().to_entity();
            let right: Uint64 = header.difficulty.0.as_u64().into();
            debug!("left u64: {} right u64 {}", to_u64(&left), to_u64(&right).checked_add(to_u64(&prev_difficult)).unwrap());
            assert_eq!(to_u64(&left), to_u64(&right).checked_add(to_u64(&prev_difficult)).unwrap());

            if main_output_reader.len() > HEADER_CACHE_LIMIT {
                return Err(Error::InvalidCellData);
            }
            if main_input_reader.len() == main_output_reader.len() && main_output_reader.len() == HEADER_CACHE_LIMIT {
                let mut input_data = vec![];
                for i in 1..main_input_reader.len() {
                    input_data.push(main_input_reader.get_unchecked(i).raw_data())
                }
                let mut output_data = vec![];
                for i in 0..main_output_reader.len()-1 {
                    output_data.push(main_output_reader.get_unchecked(i).raw_data())
                }
                assert_eq!(input_data, output_data);
            } else if main_input_reader.len() < main_output_reader.len(){
                debug!("the main input len < output len");
                let mut input_data = vec![];
                for i in 0..main_input_reader.len() {
                    input_data.push(main_input_reader.get_unchecked(i).raw_data())
                }
                let mut output_data = vec![];
                for i in 0..main_output_reader.len()-1 {
                    output_data.push(main_output_reader.get_unchecked(i).raw_data())
                }
                assert_eq!(input_data, output_data);
            } else {
                return Err(Error::InvalidCellData);
            }
        } else {
            debug!("the chain is reorg.");

        }
    }


    // header is on uncle chain will cause chain org.
    // let uncle_tail = uncle_output_reader.get_unchecked(uncle_input_reader.len()-1).raw_data();
    // if main_input_reader.len() {
    //
    // }

    // assert_eq!(header_ , header_raw);

    // debug!("difficulty: {:?}",header_info_reader.total_difficulty().raw_data().into());
    // let mut data = [0u8; 8];
    // data.copy_from_slice(Uint64::from_slice(&header_info_reader.total_difficulty().raw_data()).expect(""));


    // if main_output_reader.len() > main_input_reader.len() {
    //     assert_eq!(main_output_reader.get_unchecked(main_output_reader.len() - 1).raw_data(), header_raw);
    //     let mut input_data = vec![];
    //     for i in 0..main_input_reader.len() {
    //         input_data.push(main_input_reader.get_unchecked(i).raw_data())
    //     }
    //     input_data.push(header_raw);
    //     let mut output_data = vec![];
    //     for i in 0..main_output_reader.len() {
    //         output_data.push(main_output_reader.get_unchecked(i).raw_data())
    //     }
    //     assert_eq!(input_data, output_data);
    // } else if uncle_output_reader.len() > uncle_input_reader.len() {
    //     assert_eq!(uncle_output_reader.get_unchecked(uncle_output_reader.len() - 1).raw_data(), header_raw);
    //     let mut input_data = vec![];
    //     for i in 0..uncle_input_reader.len() {
    //         input_data.push(uncle_input_reader.get_unchecked(i).raw_data())
    //     }
    //     input_data.push(header_raw);
    //     let mut output_data = vec![];
    //     for i in 0..uncle_output_reader.len() {
    //         output_data.push(uncle_output_reader.get_unchecked(i).raw_data())
    //     }
    //     assert_eq!(input_data, output_data);
    // } else {
    //     return Err(Error::InvalidCellData);
    // }
    Ok(header)
}

fn parse_proof(proof_raw: &[u8]) -> Result<DoubleNodeWithMerkleProof, Error> {
    if DoubleNodeWithMerkleProofReader::verify(&proof_raw, false).is_err() {
        return Err(Error::InvalidWitness);
    }
    let merkle_proof = DoubleNodeWithMerkleProofReader::new_unchecked(proof_raw);
    let mut dag_nodes = vec![];
    for i in 0..merkle_proof.dag_nodes().len() {
        let mut node = [0u8; 64];
        node.copy_from_slice(merkle_proof.dag_nodes().get_unchecked(i).raw_data());
        dag_nodes.push(H512(node.into()));
    }
    let mut proofs = vec![];
    for i in 0..merkle_proof.proof().len() {
        let mut proof = [0u8; 16];
        proof.copy_from_slice(merkle_proof.proof().get_unchecked(i).raw_data());
        proofs.push(H128(proof.into()));
    }
    Ok(DoubleNodeWithMerkleProof::new(
        dag_nodes,
        proofs,
    ))
}

fn parse_dep_data(witness: WitnessReader, number: u64) -> Result<H128, Error> {
    let cell_dep_index_list = witness.cell_dep_index_list().raw_data();
    if cell_dep_index_list.len() != 1 {
        return Err(Error::InvalidWitness);
    }
    let dep_data = load_cell_data(cell_dep_index_list[0].into(), Source::CellDep)?;
    // debug!("dep data is {:?}", &dep_data);
    if DagsMerkleRootsReader::verify(&dep_data, false).is_err() {
        return Err(Error::DagsMerkleRootsDataInvalid);
    }
    let dags_reader = DagsMerkleRootsReader::new_unchecked(&dep_data);
    let idx: usize = (number / 30000) as usize;
    let merkle_root_tmp = dags_reader.dags_merkle_roots().get_unchecked(idx).raw_data();
    let mut merkle_root = [0u8; 16];
    merkle_root.copy_from_slice(merkle_root_tmp);
    Ok(H128(merkle_root.into()))
}

fn get_data(source: Source) -> Result<Option<CellDataView>, Error> {
    let data_list = QueryIter::new(load_cell_data, source).collect::<Vec<Vec<u8>>>();
    match data_list.len() {
        0 => Ok(None),
        1 => Ok(Some(CellDataView::from_slice(
            data_list[0].as_slice(),
        )?)),
        _ => Err(Error::TxInvalid),
    }
}
