# Tool to perform a network handshake for NEAR blockchain node

The node and the handshake tool both are cross-platform and can work on Linux, Mac OS, or Windows.
The following usage instruction assumes that Linux is used, but all steps can be reproduced on other OSs
similarly.

## Compiling and running a local NEAR blockchain validator node instance

1. Install prerequisites

- Rust (please follow these instructions: https://docs.near.org/docs/tutorials/contracts/intro-to-rust#3-step-rust-installation).
- Git

2. Install dependencies:

```
sudo apt update
sudo apt install -y git binutils-dev cmake gcc g++ protobuf-compiler libssl-dev pkg-config clang llvm
```

3. Clone NEAR core repository (it's better to do that in another directory - not in the root of the handshake tool repository):

```
git clone https://github.com/near/nearcore.git
cd nearcore
```

4. Checkout the supported version and build ***neard*** binary:

```
git checkout crates-0.15.0
make neard
```

5. Initialize working directory:

```
./target/release/neard --home ~/.near init --chain-id localnet
```
(run this command from the root of the NEAR core repository).

6. Ensure that ***config.json***, ***node_key.json***, ***validator_key.json***, and ***genesis.json*** configuration files are created in ***~/.near*** directory.

7. Run the node and ensure it's started successfully (see log outputs in your console).

```
./target/release/neard --home ~/.near run
```
(run this command from the root of the NEAR core repository).

Additional information about NEAR validator node building and usage can be found here:

https://near-nodes.io/validator/compile-and-run-a-node
https://near-nodes.io/validator/running-a-node

## Compiling and running the handshake tool

1. Open a new terminal window and change the current directory to the root of the handshake tool repository.

2. Build the handshake tool:

```
cargo build
```

3. Run tests:

```
cargo test
```

4. Assuming that NEAR blockchain validator node is running (see this doc above), run the handshake tool with default arguments:

```
cargo run
```

It should connect to the local instance of the validator node, perform the handshake and display the resulting
handshake response returned from the node.

It's possible to run the handshake tool with custom options using command line arguments. Run the following command to see
a description of the arguments:

```
cargo run -- -h
```

