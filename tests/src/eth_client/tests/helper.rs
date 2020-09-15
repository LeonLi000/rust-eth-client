use crate::eth_client::types::{
    generated::{basic,basic::{BytesVec}, witness, dags_merkle_roots, double_node_with_merkle_proof}
};
use crate::*;
use super::{types::*, cell_data::CellData};
use ckb_testtool::{builtin::ALWAYS_SUCCESS, context::Context};
use ckb_tool::ckb_types::{bytes::Bytes, core::TransactionBuilder, packed::*, prelude::*};
use molecule::prelude::*;
use std::convert::TryInto;
use crate::eth_client::types::generated::Chain;

pub const MAX_CYCLES: u64 = 100_000_000_000;
pub const PLEDGE: u64 = 10000;
pub const XT_CELL_CAPACITY: u64 = 200;

pub fn run_test_case(case: TestCase) {
    let mut context = Context::default();
    let typescript_bin: Bytes = Loader::default().load_binary("eth-client");
    let typescript_out_point = context.deploy_cell(typescript_bin);
    let always_success_out_point = context.deploy_cell(ALWAYS_SUCCESS.clone());

    // prepare scripts
    let typescript = context
        .build_script(&typescript_out_point, Default::default())
        .expect("script");
    let typescript_dep = CellDep::new_builder()
        .out_point(typescript_out_point)
        .build();
    let always_success_lockscript = context
        .build_script(&always_success_out_point, Default::default())
        .expect("script");
    let always_success_lockscript_dep = CellDep::new_builder()
        .out_point(always_success_out_point)
        .build();
    let lock_hash: [u8; 32] = always_success_lockscript.calc_script_hash().unpack();
    // let lock_hash = [0u8; 32];
    dbg!(hex::encode(lock_hash));

    let user_lockscript =
        basic::Script::from_slice(case.cell_data.user_lockscript.as_slice()).unwrap();

    let input_main_data = vec![basic::Bytes::from(hex::decode("f90218a088e96d4537bea4d9c05d12549907b32561d3bf31f45aae734cdc119f13406cb6a01dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d4934794dd2f1e6e498202e86d8f5442af596580a4f03c2ca04943d941637411107494da9ec8bc04359d731bfd08b72b4d0edcbd4cd2ecb341a056e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421a056e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421b90100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000008503ff00100002821388808455ba4241a0476574682f76312e302e302d30636463373634372f6c696e75782f676f312e34a02f0790c5aa31ab94195e1f6443d645af5b75c46c04fbf9911711198a0ce8fdda88b853fa261a86aa9e").expect(""))];
    let input_data = CellData::new_builder()
        .headers(Chain::new_builder().main(BytesVec::new_builder().set(input_main_data).build()).build())
        .user_lockscript(user_lockscript.clone())
        .build();
    let output_main_data =vec![basic::Bytes::from(hex::decode("f90218a088e96d4537bea4d9c05d12549907b32561d3bf31f45aae734cdc119f13406cb6a01dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d4934794dd2f1e6e498202e86d8f5442af596580a4f03c2ca04943d941637411107494da9ec8bc04359d731bfd08b72b4d0edcbd4cd2ecb341a056e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421a056e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421b90100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000008503ff00100002821388808455ba4241a0476574682f76312e302e302d30636463373634372f6c696e75782f676f312e34a02f0790c5aa31ab94195e1f6443d645af5b75c46c04fbf9911711198a0ce8fdda88b853fa261a86aa9e").expect("")),
                               basic::Bytes::from(hex::decode("f90218a0b495a1d7e6663152ae92708da4843337b958146015a2802f4193a410044698c9a06b17b938c6e4ef18b26ad81b9ca3515f27fd9c4e82aac56a1fd8eab288785e41945088d623ba0fcf0131e0897a91734a4d83596aa0a076ab0b899e8387436ff2658e2988f83cbf1af1590b9fe9feca3714f8d1824940a056e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421a056e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421b90100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000008503fe802ffe03821388808455ba4260a0476574682f76312e302e302d66633739643332642f6c696e75782f676f312e34a065e12eec23fe6555e6bcdb47aa25269ae106e5f16b54e1e92dcee25e1c8ad037882e9344e0cbde83ce").expect(""))];
    let output_data = CellData::new_builder()
        .headers(Chain::new_builder().main(BytesVec::new_builder().set(output_main_data).build()).build())
        .user_lockscript(user_lockscript.clone())
        .build();

    let input_cell_out_point = context.create_cell(
        CellOutput::new_builder()
            .capacity(case.input_capacity.pack())
            .lock(always_success_lockscript.clone())
            .type_(Some(typescript.clone()).pack())
            .build(),
        input_data.as_bytes(),
    );
    let input_cell = CellInput::new_builder()
        .previous_output(input_cell_out_point)
        .build();
    let inputs = vec![input_cell];
    let outputs = vec![CellOutput::new_builder()
        .capacity(case.output_capacity.pack())
        .type_(Some(typescript.clone()).pack())
        .lock(always_success_lockscript.clone())
        .build()];
    let outputs_data = vec![output_data.as_bytes()];
    let proof_vec = case.witness.merkle_proof;
    dbg!("proof_vec: {:?}", &proof_vec[0]);
    let mut proof_json_vec = vec![];
    for i in 0..proof_vec.len() {
        let dag_nodes = &proof_vec[i].dag_nodes;
        let mut dag_nodes_string = vec![];
        for j in 0..dag_nodes.len() {
            dag_nodes_string.push(hex::encode(dag_nodes[j].0));
        }
        let proof = &proof_vec[i].proof;
        let mut proof_string = vec![];
        for j in 0..proof.len() {
            proof_string.push(hex::encode(proof[j].0));
        }
        proof_json_vec.push(DoubleNodeWithMerkleProofJson{
            dag_nodes: dag_nodes_string,
            proof: proof_string,
        })
    }
    let mut merkle_proofs: Vec<double_node_with_merkle_proof::DoubleNodeWithMerkleProof> = vec![];
    for i in 0..proof_json_vec.len() {
        let proof = &proof_json_vec[i];


        let p:double_node_with_merkle_proof::DoubleNodeWithMerkleProof = (*proof).clone().try_into().unwrap();
        if i == 0 {
            dbg!("proof: {:?}", proof);
            dbg!("p: {:?}", &p.as_reader().proof().len());
        }
        merkle_proofs.push(p);
    }
    let mut proofs = vec![];
    for i in 0..merkle_proofs.len() {
        proofs.push(basic::Bytes::from(merkle_proofs[i].as_slice().to_vec()));
    }
    let witness_data = witness::Witness::new_builder()
        .header(hex::decode(case.witness.header).expect("error").into())
        .merkle_proof(BytesVec::new_builder().set(proofs).build())
        .cell_dep_index_list(case.witness.cell_dep_index_list.into())
        .build();
    let witness = WitnessArgs::new_builder()
        .input_type(Some(witness_data.as_bytes()).pack())
        .build();
    let dep_data_raw= case.cell_deps_data;
    let mut dag_root = vec![];
    for i in 0..dep_data_raw.dag_merkle_roots.len() {
        dag_root.push(hex::encode(&dep_data_raw.dag_merkle_roots[i].0).clone());
    }
    let dep_data_string = RootsCollectionJson{
        dag_merkle_roots: dag_root,
    };
    let dep_data: dags_merkle_roots::DagsMerkleRoots = dep_data_string.try_into().unwrap();
    let data_out_point = context.deploy_cell(dep_data.as_bytes());
    let data_dep = CellDep::new_builder().out_point(data_out_point).build();

    let tx = TransactionBuilder::default()
        .inputs(inputs)
        .outputs(outputs)
        .outputs_data(outputs_data.pack())
        .cell_dep(data_dep)
        .cell_dep(typescript_dep)
        .cell_dep(always_success_lockscript_dep)
        .witness(witness.as_bytes().pack())
        .build();

    let res = context.verify_tx(&tx, MAX_CYCLES);
    dbg!(&res);
    match res {
        Ok(_cycles) => assert_eq!(case.expect_return_code, 0),
        Err(err) => assert!(check_err(err, case.expect_return_code)),
    }
}

pub fn check_err(err: ckb_tool::ckb_error::Error, code: i8) -> bool {
    let get = format!("{}", err);
    let expected = format!("Script(ValidationFailure({}))", code);
    dbg!(&get, &expected);
    get == expected
}
