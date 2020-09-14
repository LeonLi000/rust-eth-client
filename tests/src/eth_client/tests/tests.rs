use super::*;
// use ckb_tool::ckb_types::{bytes::Bytes, core::TransactionBuilder, packed::*, prelude::*};
// use crate::*;
use helper::run_test_case;
use types::*;
use ckb_testtool::{builtin::ALWAYS_SUCCESS, context::Context};

const MAX_CYCLES: u64 = 10_000_000;

fn generate_correct_case() -> TestCase {
    let mut context = Context::default();
    let always_success_out_point = context.deploy_cell(ALWAYS_SUCCESS.clone());
    let user_lockscript = context
        .build_script(&always_success_out_point, Default::default())
        .expect("script");
    let blocks_with_proofs: Vec<BlockWithProofs> = ["/Users/leon/dev/rust/leon/rust-eth-client/tests/src/eth_client/tests/data/2.json"]
        .iter()
        .map(|filename| read_block((&filename).to_string()))
        .collect();
    let block_with_proof = blocks_with_proofs.get(0).expect("error");
    let case = TestCase {
        input_capacity: 100000,
        output_capacity: 100000,
        cell_data: CellDataTest {
            user_lockscript: user_lockscript.clone(),
            headers:Default::default(),
        },
        witness: Witness {
            cell_dep_index_list: vec![0],
            header:"f90218a088e96d4537bea4d9c05d12549907b32561d3bf31f45aae734cdc119f13406cb6a01dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d4934794dd2f1e6e498202e86d8f5442af596580a4f03c2ca04943d941637411107494da9ec8bc04359d731bfd08b72b4d0edcbd4cd2ecb341a056e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421a056e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421b90100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000008503ff00100002821388808455ba4241a0476574682f76312e302e302d30636463373634372f6c696e75782f676f312e34a02f0790c5aa31ab94195e1f6443d645af5b75c46c04fbf9911711198a0ce8fdda88b853fa261a86aa9e".to_owned(),
            merkle_proof: block_with_proof.to_double_node_with_merkle_proof_vec(),
        },
        cell_deps_data: read_roots_collection_raw(),
        expect_return_code: 0,
    };
    case
}

#[test]
fn test_basic() {
    let case = generate_correct_case();
    run_test_case(case);
}
