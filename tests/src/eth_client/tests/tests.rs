use super::*;
use futures::future::join_all;
use crate::eth_client::types::{
    generated::{basic::BytesVec, Chain}
};
use helper::run_test_case;
use types::*;
use rlp;
use eth_spv_lib::eth_types::*;
use molecule::prelude::{Entity, Builder};
use rlp::RlpStream;
use web3::futures::Future;
use web3::types::{Block, H256};
use lazy_static::lazy_static;
use hex;

const MAX_CYCLES: u64 = 10_000_000;


lazy_static! {
    static ref WEB3RS: web3::Web3<web3::transports::Http> = {
        let (eloop, transport) = web3::transports::Http::new(
            "https://mainnet.infura.io/v3/9c7178cede9f4a8a84a151d058bd609c",
        )
        .unwrap();
        eloop.into_remote();
        web3::Web3::new(transport)
    };
}

#[test]
fn test_add_block_2() {
    let blocks_with_proofs: Vec<BlockWithProofs> = ["../tests/src/eth_client/tests/data/height-2.json"]
        .iter()
        .map(|filename| read_block((&filename).to_string()))
        .collect();
    let block_with_proof_2 = blocks_with_proofs.get(0).expect("error");
    let (output_data_raw,_) = create_data(block_with_proof_2, 0);
    let output_data = create_cell_data(vec![output_data_raw], Option::None);
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
    let input_data = create_cell_data(vec![input_data_raw.clone()],Option::None);

    let (output_data_raw,_) = create_data(block_with_proof_3, input_difficulty);
    let output_data_vec = vec![input_data_raw.clone(), output_data_raw.clone()];
    let output_data = create_cell_data(output_data_vec, Option::None);
    let witness = Witness {
        cell_dep_index_list: vec![0],
        header: block_with_proof_3.header_rlp.0.clone(),
        merkle_proof: block_with_proof_3.to_double_node_with_merkle_proof_vec(),
    };
    let case = generate_correct_case(Option::Some(input_data.as_bytes()), output_data.as_bytes(), witness);
    run_test_case(case);
}

#[test]
fn test_add_block_68_69() {
    let blocks_with_proofs: Vec<BlockWithProofs> = ["../tests/src/eth_client/tests/data/height-10913468.json","../tests/src/eth_client/tests/data/height-10913469-1.json"]
        .iter()
        .map(|filename| read_block((&filename).to_string()))
        .collect();
    let block_with_proof_68 = blocks_with_proofs.get(0).expect("error");
    let block_with_proof_69 = blocks_with_proofs.get(1).expect("error");

    let (input_data_raw, input_difficulty) = create_data(block_with_proof_68, 0);
    let input_data = create_cell_data(vec![input_data_raw.clone()], Option::None);

    let (output_data_raw,_) = create_data(block_with_proof_69, input_difficulty);
    let output_data_vec = vec![input_data_raw.clone(), output_data_raw.clone()];
    let output_data = create_cell_data(output_data_vec,Option::None);
    let witness = Witness {
        cell_dep_index_list: vec![0],
        header: block_with_proof_69.header_rlp.0.clone(),
        merkle_proof: block_with_proof_69.to_double_node_with_merkle_proof_vec(),
    };
    let case = generate_correct_case(Option::Some(input_data.as_bytes()), output_data.as_bytes(), witness);
    run_test_case(case);
}

#[test]
fn test_add_block_69_reorg() {
    let blocks_with_proofs: Vec<BlockWithProofs> = ["../tests/src/eth_client/tests/data/height-10913468.json","../tests/src/eth_client/tests/data/height-10913469-1.json", "../tests/src/eth_client/tests/data/height-10913469.json"]
        .iter()
        .map(|filename| read_block((&filename).to_string()))
        .collect();
    let block_with_proof_68 = blocks_with_proofs.get(0).expect("error");
    let block_with_proof_69_1 = blocks_with_proofs.get(1).expect("error");
    let block_with_proof_69 = blocks_with_proofs.get(2).expect("error");

    let (input_data_raw_68, input_difficulty_68) = create_data(block_with_proof_68, 0);
    let (input_data_raw_69_1, _) = create_data(block_with_proof_69_1, input_difficulty_68);
    let input_data_vec = vec![input_data_raw_68.clone(), input_data_raw_69_1.clone()];
    let input_data = create_cell_data(input_data_vec, Option::None);

    let (output_data_raw_temp,_) = create_data(block_with_proof_69, input_difficulty_68);
    let output_data_vec = vec![input_data_raw_68.clone(), output_data_raw_temp.clone()];
    let output_data = create_cell_data(output_data_vec, Option::Some(vec![input_data_raw_69_1]));
    let witness = Witness {
        cell_dep_index_list: vec![0],
        header: block_with_proof_69.header_rlp.0.clone(),
        merkle_proof: block_with_proof_69.to_double_node_with_merkle_proof_vec(),
    };
    let case = generate_correct_case(Option::Some(input_data.as_bytes()), output_data.as_bytes(), witness);
    run_test_case(case);
}

#[test]
fn test_add_block_69_without_reorg() {
    let blocks_with_proofs: Vec<BlockWithProofs> = ["../tests/src/eth_client/tests/data/height-10913468.json","../tests/src/eth_client/tests/data/height-10913469-1.json", "../tests/src/eth_client/tests/data/height-10913469.json"]
        .iter()
        .map(|filename| read_block((&filename).to_string()))
        .collect();
    let block_with_proof_68 = blocks_with_proofs.get(0).expect("error");
    let block_with_proof_69_1 = blocks_with_proofs.get(1).expect("error");
    let block_with_proof_69 = blocks_with_proofs.get(2).expect("error");

    let (input_data_raw_68, input_difficulty_68) = create_data(block_with_proof_68, 0);
    let (input_data_raw_69, _) = create_data(block_with_proof_69, input_difficulty_68);
    let input_data_vec = vec![input_data_raw_68.clone(), input_data_raw_69.clone()];
    let input_data = create_cell_data(input_data_vec, Option::None);

    let (output_data_raw_temp,_) = create_data(block_with_proof_69_1, 0);
    let output_data_vec = vec![input_data_raw_68.clone(), input_data_raw_69.clone()];
    let output_data = create_cell_data(output_data_vec, Option::Some(vec![output_data_raw_temp.clone()]));
    let witness = Witness {
        cell_dep_index_list: vec![0],
        header: block_with_proof_69_1.header_rlp.0.clone(),
        merkle_proof: block_with_proof_69_1.to_double_node_with_merkle_proof_vec(),
    };
    let case = generate_correct_case(Option::Some(input_data.as_bytes()), output_data.as_bytes(), witness);
    run_test_case(case);
}

#[test]
fn test_add_block_37_38() {
    let blocks_with_proofs: Vec<BlockWithProofs> = ["../tests/src/eth_client/tests/data/height-10917837.json","../tests/src/eth_client/tests/data/height-10917838-1.json", "../tests/src/eth_client/tests/data/height-10917838.json"]
        .iter()
        .map(|filename| read_block((&filename).to_string()))
        .collect();
    let block_with_proof_37 = blocks_with_proofs.get(0).expect("error");
    let block_with_proof_38_1 = blocks_with_proofs.get(1).expect("error");
    let block_with_proof_38 = blocks_with_proofs.get(2).expect("error");

    let (input_data_raw_37, input_difficulty_37) = create_data(block_with_proof_37, 0);
    let (input_data_raw_38_1, _) = create_data(block_with_proof_38_1, input_difficulty_37);
    let input_data_vec = vec![input_data_raw_37.clone(), input_data_raw_38_1.clone()];
    let input_data = create_cell_data(input_data_vec, Option::None);

    let (output_data_raw_temp,_) = create_data(block_with_proof_38, input_difficulty_37);
    let output_data_vec = vec![input_data_raw_37.clone(), output_data_raw_temp.clone()];
    let output_data = create_cell_data(output_data_vec, Option::Some(vec![input_data_raw_38_1]));
    let witness = Witness {
        cell_dep_index_list: vec![0],
        header: block_with_proof_38.header_rlp.0.clone(),
        merkle_proof: block_with_proof_38.to_double_node_with_merkle_proof_vec(),
    };
    let case = generate_correct_case(Option::Some(input_data.as_bytes()), output_data.as_bytes(), witness);
    run_test_case(case);
}

#[test]
fn test_add_block_37_38_39() {
    let blocks_with_proofs: Vec<BlockWithProofs> =
        [
            "../tests/src/eth_client/tests/data/height-10917837.json"
            ,"../tests/src/eth_client/tests/data/height-10917838-1.json"
            , "../tests/src/eth_client/tests/data/height-10917838.json"
            , "../tests/src/eth_client/tests/data/height-10917839-1.json"
        ]
        .iter()
        .map(|filename| read_block((&filename).to_string()))
        .collect();
    let block_with_proof_37 = blocks_with_proofs.get(0).expect("error");
    let block_with_proof_38_1 = blocks_with_proofs.get(1).expect("error");
    let block_with_proof_38 = blocks_with_proofs.get(2).expect("error");
    let block_with_proof_39_1 = blocks_with_proofs.get(3).expect("error");

    let (input_data_raw_37, input_difficulty_37) = create_data(block_with_proof_37, 0);
    let (input_data_raw_38_1, _) = create_data(block_with_proof_38_1, input_difficulty_37);
    let (input_data_raw_38, input_difficulty_38) = create_data(block_with_proof_38, input_difficulty_37);
    let input_data_vec = vec![input_data_raw_37.clone(), input_data_raw_38.clone()];
    let input_data = create_cell_data(input_data_vec, Option::Some(vec![input_data_raw_38_1.clone()]));

    let (output_data_raw_39_1,_) = create_data(block_with_proof_39_1, input_difficulty_38);
    let output_data_vec = vec![input_data_raw_37.clone(), input_data_raw_38.clone(), output_data_raw_39_1.clone()];
    let output_data = create_cell_data(output_data_vec, Option::Some(vec![input_data_raw_38_1.clone()]));
    let witness = Witness {
        cell_dep_index_list: vec![0],
        header: block_with_proof_39_1.header_rlp.0.clone(),
        merkle_proof: block_with_proof_39_1.to_double_node_with_merkle_proof_vec(),
    };
    let case = generate_correct_case(Option::Some(input_data.as_bytes()), output_data.as_bytes(), witness);
    run_test_case(case);
}

#[test]
fn test_add_block_37_38_39_reorg() {
    let blocks_with_proofs: Vec<BlockWithProofs> =
        [
            "../tests/src/eth_client/tests/data/height-10917837.json"
            ,"../tests/src/eth_client/tests/data/height-10917838-1.json"
            , "../tests/src/eth_client/tests/data/height-10917838.json"
            , "../tests/src/eth_client/tests/data/height-10917839-1.json"
            ,"../tests/src/eth_client/tests/data/height-10917839.json"
        ]
            .iter()
            .map(|filename| read_block((&filename).to_string()))
            .collect();
    let block_with_proof_37 = blocks_with_proofs.get(0).expect("error");
    let block_with_proof_38_1 = blocks_with_proofs.get(1).expect("error");
    let block_with_proof_38 = blocks_with_proofs.get(2).expect("error");
    let block_with_proof_39_1 = blocks_with_proofs.get(3).expect("error");
    let block_with_proof_39 = blocks_with_proofs.get(4).expect("error");

    let (input_data_raw_37, input_difficulty_37) = create_data(block_with_proof_37, 0);
    let (input_data_raw_38_1, _) = create_data(block_with_proof_38_1, input_difficulty_37);
    let (input_data_raw_38, input_difficulty_38) = create_data(block_with_proof_38, input_difficulty_37);
    let (input_data_raw_39_1, _) = create_data(block_with_proof_39_1, input_difficulty_38);
    let input_data_vec = vec![input_data_raw_37.clone(), input_data_raw_38.clone(), input_data_raw_39_1.clone()];
    let input_data = create_cell_data(input_data_vec, Option::Some(vec![input_data_raw_38_1.clone()]));

    let (output_data_raw_39,_) = create_data(block_with_proof_39, input_difficulty_38);
    let output_data_vec = vec![input_data_raw_37.clone(), input_data_raw_38.clone(), output_data_raw_39.clone()];
    let output_data = create_cell_data(output_data_vec, Option::Some(vec![input_data_raw_38_1.clone(), input_data_raw_39_1.clone()]));
    let witness = Witness {
        cell_dep_index_list: vec![0],
        header: block_with_proof_39.header_rlp.0.clone(),
        merkle_proof: block_with_proof_39.to_double_node_with_merkle_proof_vec(),
    };
    let case = generate_correct_case(Option::Some(input_data.as_bytes()), output_data.as_bytes(), witness);
    run_test_case(case);
}

#[test]
fn test_get_block() {
    get_blocks(&WEB3RS, 1,2);
}

fn get_blocks(
    web3rust: &web3::Web3<web3::transports::Http>,
    start: usize,
    stop: usize,
) -> (Vec<Vec<u8>>, Vec<H256>) {
    let mut data = [0u8; 32];
    data.copy_from_slice(hex::decode("8c75abd8ed0bd8ae98382fec6c082301b777929c9ae1021700cc344d6ef02780").expect("error").as_slice());
    let hash = H256(data.into());
    println!("hash: {:?}", hash);
    let futures = (start..stop)
        .map(|i| web3rust.eth().block((i as u64).into()))
        // .map(|i| web3rust.eth().block(BlockId::Hash(hash)))
        .collect::<Vec<_>>();
    let block_headers = join_all(futures).wait().unwrap();

    let mut blocks: Vec<Vec<u8>> = vec![];
    let mut hashes: Vec<H256> = vec![];
    for block_header in block_headers {
        let mut stream = RlpStream::new();
        rlp_append(&block_header.clone().unwrap(), &mut stream);
        blocks.push(stream.out());
        hashes.push(H256(block_header.clone().unwrap().hash.unwrap().0.into()));
    }
    for i in 0..blocks.len() {
        println!("header rlp: {:?}",  hex::encode(blocks[i].clone()));
    }

    (blocks, hashes)
}

// Wish to avoid this code and use web3+rlp libraries directly
fn rlp_append<TX>(header: &Block<TX>, stream: &mut RlpStream) {
    stream.begin_list(15);
    stream.append(&header.parent_hash);
    stream.append(&header.uncles_hash);
    stream.append(&header.author);
    stream.append(&header.state_root);
    stream.append(&header.transactions_root);
    stream.append(&header.receipts_root);
    stream.append(&header.logs_bloom);
    stream.append(&header.difficulty);
    stream.append(&header.number.unwrap());
    stream.append(&header.gas_limit);
    stream.append(&header.gas_used);
    stream.append(&header.timestamp);
    stream.append(&header.extra_data.0);
    stream.append(&header.mix_hash.unwrap());
    stream.append(&header.nonce.unwrap());
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

fn create_data(block_with_proof: &BlockWithProofs, pre_block_difficulty: u64) -> (Bytes, u64) {
    let header: BlockHeader = rlp::decode(block_with_proof.header_rlp.0.as_slice()).unwrap();
    let header_info = basic::HeaderInfo::new_builder().header(basic::Bytes::from(block_with_proof.header_rlp.0.clone()))
        .total_difficulty(header.difficulty.0.as_u64().checked_add(pre_block_difficulty).unwrap().into())
        .hash(basic::Byte32::from_slice(header.hash.unwrap().0.as_bytes()).unwrap() )
        .build();
    (header_info.as_slice().to_vec().into(), header.difficulty.0.as_u64().checked_add(pre_block_difficulty).unwrap())
}

fn create_cell_data(data: Vec<basic::Bytes>, uncle: Option<Vec<basic::Bytes>>) -> CellData {
    match uncle {
        Some(t) => {
            CellData::new_builder()
                .headers(Chain::new_builder()
                    .main(BytesVec::new_builder().set(data).build())
                    .uncle(BytesVec::new_builder().set(t).build()).build())
                .build()
        },
        None =>{
            CellData::new_builder()
                .headers(Chain::new_builder().main(BytesVec::new_builder().set(data).build()).build())
                .build()
        },
    }

}


