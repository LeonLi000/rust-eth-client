use super::*;
use crate::eth_client::types::{
    generated::{basic::BytesVec, Chain}
};
use helper::run_test_case;
use types::*;
use rlp;
use eth_spv_lib::eth_types::*;
use molecule::prelude::{Entity, Builder};

const MAX_CYCLES: u64 = 10_000_000;


#[test]
fn test_add_block_2() {
    let blocks_with_proofs: Vec<BlockWithProofs> = ["../tests/src/eth_client/tests/data/height-2.json"]
        .iter()
        .map(|filename| read_block((&filename).to_string()))
        .collect();
    let block_with_proof_2 = blocks_with_proofs.get(0).expect("error");
    let (output_data_raw,_) = create_data(block_with_proof_2, 0);
    let output_data = create_cell_data(output_data_raw);
    let witness = Witness {
        cell_dep_index_list: vec![0],
        header: block_with_proof_2.header_rlp.0.clone(),
        merkle_proof: block_with_proof_2.to_double_node_with_merkle_proof_vec(),
    };
    let case = generate_correct_case( Option::None, output_data.as_bytes(), witness);
    run_test_case(case);
}

#[test]
fn test_add_block_3() {
    let blocks_with_proofs: Vec<BlockWithProofs> = ["../tests/src/eth_client/tests/data/height-2.json","../tests/src/eth_client/tests/data/height-3.json"]
        .iter()
        .map(|filename| read_block((&filename).to_string()))
        .collect();
    let block_with_proof_2 = blocks_with_proofs.get(0).expect("error");
    let block_with_proof_3 = blocks_with_proofs.get(1).expect("error");

    let (input_data_raw, input_difficulty) = create_data(block_with_proof_2, 0);
    let input_data = create_cell_data(input_data_raw.clone());
    let mut output_data_raw = input_data_raw.clone();
    let (output_data_raw_temp,_) = create_data(block_with_proof_3, input_difficulty);
    for i in 0..output_data_raw_temp.clone().len() {
        output_data_raw.push(output_data_raw_temp[i].clone());
    }
    let output_data = create_cell_data(output_data_raw);
    let witness = Witness {
        cell_dep_index_list: vec![0],
        header: block_with_proof_3.header_rlp.0.clone(),
        merkle_proof: block_with_proof_3.to_double_node_with_merkle_proof_vec(),
    };
    let case = generate_correct_case(Option::Some(input_data.as_bytes()), output_data.as_bytes(), witness);
    run_test_case(case);
}

fn generate_correct_case(input: Option<molecule::bytes::Bytes>, output:molecule::bytes::Bytes, witness_:Witness) -> TestCase {
    TestCase {
        input_capacity: 100000,
        output_capacity: 100000,
        input_data: input,
        output_data: output,
        witness: witness_,
        cell_deps_data: read_roots_collection_raw(),
        expect_return_code: 0,
    }
}

fn create_data(block_with_proof: &BlockWithProofs, pre_block_difficulty: u64) -> (Vec<Bytes>, u64) {
    let header: BlockHeader = rlp::decode(block_with_proof.header_rlp.0.as_slice()).unwrap();
    let header_info = basic::HeaderInfo::new_builder().header(basic::Bytes::from(block_with_proof.header_rlp.0.clone()))
        .total_difficulty(header.difficulty.0.as_u64().checked_add(pre_block_difficulty).unwrap().into())
        .hash(basic::Byte32::from_slice(header.hash.unwrap().0.as_bytes()).unwrap() )
        .build();
    (vec![header_info.as_slice().to_vec().into()], header.difficulty.0.as_u64())
}

fn create_cell_data(data: Vec<basic::Bytes>) -> CellData {
    CellData::new_builder()
        .headers(Chain::new_builder().main(BytesVec::new_builder().set(data).build()).build())
        .build()
}
