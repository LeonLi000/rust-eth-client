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
    let blocks_with_proofs: Vec<BlockWithProofs> = ["/Users/leon/dev/rust/leon/rust-eth-client/tests/src/eth_client/tests/data/3.json"]
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
            header:"f90218a0b495a1d7e6663152ae92708da4843337b958146015a2802f4193a410044698c9a06b17b938c6e4ef18b26ad81b9ca3515f27fd9c4e82aac56a1fd8eab288785e41945088d623ba0fcf0131e0897a91734a4d83596aa0a076ab0b899e8387436ff2658e2988f83cbf1af1590b9fe9feca3714f8d1824940a056e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421a056e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421b90100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000008503fe802ffe03821388808455ba4260a0476574682f76312e302e302d66633739643332642f6c696e75782f676f312e34a065e12eec23fe6555e6bcdb47aa25269ae106e5f16b54e1e92dcee25e1c8ad037882e9344e0cbde83ce".to_owned(),
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
