use super::*;
use crate::eth_client::types::{
    generated::{basic::BytesVec, Chain}
};
use helper::run_test_case;
use types::*;
use ckb_testtool::{builtin::ALWAYS_SUCCESS, context::Context};
use rlp;
use eth_spv_lib::eth_types::*;
use molecule::prelude::{Entity, Builder};

const MAX_CYCLES: u64 = 10_000_000;

fn generate_correct_case(input: molecule::bytes::Bytes, output:molecule::bytes::Bytes, witness_:Witness) -> TestCase {
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

#[test]
fn test_basic() {
    let mut context = Context::default();
    let always_success_out_point = context.deploy_cell(ALWAYS_SUCCESS.clone());
    let user_lock_script = context
        .build_script(&always_success_out_point, Default::default())
        .expect("script");
    let user_lock_script =
        basic::Script::from_slice(user_lock_script.as_slice()).unwrap();

    let blocks_with_proofs: Vec<BlockWithProofs> = ["../tests/src/eth_client/tests/data/height-2.json","../tests/src/eth_client/tests/data/height-3.json"]
        .iter()
        .map(|filename| read_block((&filename).to_string()))
        .collect();
    let block_with_proof_2 = blocks_with_proofs.get(0).expect("error");
    let block_with_proof_3 = blocks_with_proofs.get(1).expect("error");

    let header_input: BlockHeader = rlp::decode(block_with_proof_2.header_rlp.0.as_slice()).unwrap();
    let header_info_input = basic::HeaderInfo::new_builder().header(basic::Bytes::from(block_with_proof_2.header_rlp.0.clone()))
        .total_difficulty(Default::default())
        .hash(basic::Byte32::from_slice(header_input.hash.unwrap().0.as_bytes()).unwrap() )
        .build();
    let input_main_data = vec![header_info_input.as_slice().to_vec().into()];
    let input_data = create_cell_data(input_main_data, user_lock_script.clone());

    let header_output: BlockHeader = rlp::decode(block_with_proof_3.header_rlp.0.as_slice()).unwrap();
    let header_info_output = basic::HeaderInfo::new_builder().header(basic::Bytes::from(block_with_proof_3.header_rlp.0.clone()))
        .total_difficulty(header_output.difficulty.0.as_u64().into())
        .hash(basic::Byte32::from_slice(header_output.hash.unwrap().0.as_bytes()).unwrap() )
        .build();
    let output_main_data = vec![header_info_input.as_slice().to_vec().into(), header_info_output.as_slice().to_vec().into()];
    let output_data = create_cell_data(output_main_data, user_lock_script.clone());
    let witness = Witness {
        cell_dep_index_list: vec![0],
        header: block_with_proof_3.header_rlp.0.clone(),
        merkle_proof: block_with_proof_3.to_double_node_with_merkle_proof_vec(),
    };
    let case = generate_correct_case(input_data.as_bytes(), output_data.as_bytes(), witness);
    run_test_case(case);
}

fn create_cell_data(data: Vec<basic::Bytes>, script: basic::Script) -> CellData {
    CellData::new_builder()
        .headers(Chain::new_builder().main(BytesVec::new_builder().set(data).build()).build())
        .user_lockscript(script)
        .build()
}
