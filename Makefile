schema:
	moleculec --language rust --schema-file contracts/eth-client/src/types/schemas/basic.mol > contracts/eth-client/src/types/generated/basic.rs
	moleculec --language rust --schema-file contracts/eth-client/src/types/schemas/cell_data.mol > contracts/eth-client/src/types/generated/cell_data.rs
	moleculec --language rust --schema-file contracts/eth-client/src/types/schemas/dags_merkle_roots.mol > contracts/eth-client/src/types/generated/dags_merkle_roots.rs
	moleculec --language rust --schema-file contracts/eth-client/src/types/schemas/witness.mol > contracts/eth-client/src/types/generated/witness.rs
	moleculec --language rust --schema-file contracts/eth-client/src/types/schemas/double_node_with_merkle_proof.mol > contracts/eth-client/src/types/generated/double_node_with_merkle_proof.rs
	cp contracts/eth-client/src/types/generated/*.rs tests/src/eth_client/types/generated

fmt:
	cd contracts/eth_client && cargo fmt --all
	cd contracts/eth_client && cargo fmt --all
	cd tests && cargo fmt --all

build:
	capsule build

test:
	capsule test

ci: fmt build test

.PHONY: fmt build test ci schema
