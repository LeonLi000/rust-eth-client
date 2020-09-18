use crate::eth_client::types::{
    generated::{basic,basic::BytesVec, witness, dags_merkle_roots, double_node_with_merkle_proof}
};
use crate::*;
use super::types::*;
use ckb_testtool::{builtin::ALWAYS_SUCCESS, context::Context};
use ckb_tool::ckb_types::{bytes::Bytes, core::TransactionBuilder, packed::*, prelude::*};
use molecule::prelude::*;
use std::convert::TryInto;

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
    let input_cell_out_point = match case.input_data {
        Some(_) => context.create_cell(
            CellOutput::new_builder()
                .capacity(case.input_capacity.pack())
                .lock(always_success_lockscript.clone())
                .type_(Some(typescript.clone()).pack())
                .build(),
            case.input_data.unwrap(),
        ),
        None => context.create_cell(
            CellOutput::new_builder()
                .capacity(case.input_capacity.pack())
                .lock(always_success_lockscript.clone())
                .build(),
            Bytes::new(),
        ),
    };
    let input_cell = CellInput::new_builder()
        .previous_output(input_cell_out_point)
        .build();
    // let inputs = vec![input_cell];
    let outputs = vec![CellOutput::new_builder()
        .capacity(case.output_capacity.pack())
        .type_(Some(typescript.clone()).pack())
        .lock(always_success_lockscript.clone())
        .build()];
    let outputs_data = vec![case.output_data];
    let proof_vec = case.witness.merkle_proof;
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
        merkle_proofs.push(p);
    }
    let mut proofs = vec![];
    for i in 0..merkle_proofs.len() {
        proofs.push(basic::Bytes::from(merkle_proofs[i].as_slice().to_vec()));
    }
    let witness_data = witness::Witness::new_builder()
        .header(case.witness.header.into())
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
        .input(input_cell)
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
