use std::{fs, io, net, str::FromStr};

use serde::Deserialize;

use clap::Parser;

use tokio::time::Duration;

use near_crypto::PublicKey;

use near_primitives::{block::GenesisId, hash::CryptoHash, network::PeerId};

mod connection;
mod network_protocol;

use connection::Connection;
use network_protocol::Handshake;

const DEFAULT_LISTEN_PORT: u16 = 24567;

#[derive(Deserialize)]
struct NodeKey {
    public_key: String,
}

#[derive(clap::Parser)]
struct Args {
    /// Network address of the node to connect (address:port)
    #[clap(short = 'n', long, default_value = "127.0.0.1:24567")]
    node_addr: String,

    /// Connection timeout (in seconds)
    #[clap(short = 't', long, default_value = "1")]
    connection_timeout: u64,

    /// Optional blockchain ID of the genesis for the handshake request - "localnet",
    /// "testnet", "mainnet" etc. (if provided, then "genesis_hash" must be also provided).
    #[clap(short = 'c', long, requires = "genesis_hash", verbatim_doc_comment)]
    genesis_chain_id: Option<String>,

    /// Optional hash of the genesis for the handshake request (if not provided, the genesis
    /// will be requested from the node by sending a preliminary handshake request with an
    /// empty genesis and then sending the second handshake request with the proper genesis
    /// value). Requires that "genesis_chain_id" is also provided.
    #[clap(short = 'g', long, requires = "genesis_chain_id", verbatim_doc_comment)]
    genesis_hash: Option<String>,

    // Height of the head for the handshake request
    #[clap(short = 'b', long, default_value = "0")]
    head_height: u64,
}

async fn run(args: Args) -> Result<Handshake, String> {
    let node_key: NodeKey = {
        let node_key_file_path = format!(
            "{}/.near/node_key.json",
            std::env::var("HOME").map_err(|_| {
                "HOME environment variable not set (required to read node key config file)"
            })?
        );

        serde_json::from_reader(io::BufReader::new(
            fs::File::open(&node_key_file_path).map_err(|_| {
                format!(
                    "Error opening node key config file: {}",
                    &node_key_file_path
                )
            })?,
        ))
        .map_err(|_| {
            format!(
                "Error parsing node key config file: {}",
                &node_key_file_path
            )
        })?
    };

    let peer_id = PeerId::new(PublicKey::from_str(&node_key.public_key).map_err(|_| {
        format!(
            "Error parsing public_key value from node key config file: {}",
            &node_key.public_key
        )
    })?);

    let genesis_id = match args.genesis_hash {
        Some(hash) => Some(GenesisId {
            chain_id: args
                .genesis_chain_id
                .ok_or("genesis_chain_id command line arg not provided")?,
            hash: CryptoHash::from_str(&hash).map_err(|_| {
                format!(
                    "Error parsing hash value from genesis_hash command line arg: {}",
                    &hash
                )
            })?,
        }),
        None => None,
    };

    let node_addr: net::SocketAddr = args.node_addr.parse().map_err(|_| {
        format!(
            "Error parsing network address from node_addr command line arg: {}",
            &args.node_addr
        )
    })?;

    let (_connection, handshake) = Connection::connect(
        node_addr,
        peer_id,
        DEFAULT_LISTEN_PORT,
        Duration::from_secs(args.connection_timeout),
        genesis_id,
        args.head_height,
    )
    .await
    .map_err(|e| format!("Error establishing connection to node: {:#?}", e))?;

    Ok(handshake)
}

#[tokio::main]
async fn main() {
    match run(Args::parse()).await {
        Ok(handshake) => println!(
            "Handshake performed successfully, response from the node: {:#?}",
            handshake
        ),
        Err(e) => println!("{}", e),
    }
}
