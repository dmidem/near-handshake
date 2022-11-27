use std::net::SocketAddr;

use tokio::{io::BufReader, net::TcpStream, time};

use near_primitives::{
    block::GenesisId, network::PeerId, types::BlockHeight, version::PROTOCOL_VERSION,
};

use crate::network_protocol::{Handshake, NetworkError};

use super::Connection;

impl Connection<BufReader<TcpStream>> {
    pub async fn connect(
        addr: SocketAddr,
        peer_id: PeerId,
        sender_listen_port: u16,
        timeout: time::Duration,

        genesis_id: Option<GenesisId>,
        head_height: BlockHeight,
    ) -> Result<(Self, Handshake), NetworkError> {
        const BUF_READER_SIZE: usize = 1024;

        let stream = BufReader::with_capacity(
            BUF_READER_SIZE,
            time::timeout(timeout, TcpStream::connect(addr))
                .await
                .map_err(|e| NetworkError::IO(e.into()))?
                .map_err(NetworkError::IO)?,
        );

        let mut connection = Self::new(stream, peer_id, sender_listen_port, timeout);

        let handshake = connection
            .handshake_with_optional_genesis(PROTOCOL_VERSION, genesis_id, head_height)
            .await?
            .0;

        Ok((connection, handshake))
    }
}
