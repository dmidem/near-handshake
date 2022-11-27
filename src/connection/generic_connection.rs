use std::io;

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    time,
};

use protobuf::Message;

use near_crypto::{KeyType, SecretKey};

use near_primitives::{
    block::GenesisId, network::PeerId, types::BlockHeight, version::ProtocolVersion,
};

use crate::network_protocol::{
    Handshake, HandshakeFailure, HandshakeResponse, NetworkError, PartialEdgeInfo, PeerChainInfo,
    PeerMessage,
};

pub struct Connection<Stream>
where
    Stream: AsyncReadExt + AsyncWriteExt + std::marker::Unpin,
{
    pub(super) stream: Stream,
    peer_id: PeerId,
    sender_listen_port: u16,
    timeout: time::Duration,

    secret_key: SecretKey,
    my_peer_id: PeerId,
}

impl<Stream> Connection<Stream>
where
    Stream: AsyncReadExt + AsyncWriteExt + std::marker::Unpin,
{
    pub(super) fn new(
        stream: Stream,
        peer_id: PeerId,
        sender_listen_port: u16,
        timeout: time::Duration,
    ) -> Self {
        let secret_key = SecretKey::from_random(KeyType::ED25519);
        let my_peer_id = PeerId::new(secret_key.public_key());

        Self {
            stream,
            peer_id,
            sender_listen_port,
            timeout,

            secret_key,
            my_peer_id,
        }
    }

    pub(super) async fn handshake_with_optional_genesis(
        &mut self,
        protocol_version: ProtocolVersion,
        genesis_id: Option<GenesisId>,
        head_height: BlockHeight,
    ) -> Result<HandshakeResponse, NetworkError> {
        // If genesis_id passed as a function argument is None, do the handshake with
        // a default (empty) genesis, to get it from the node as GenesisMismatch error
        // payload
        let genesis_id = match genesis_id {
            None => match self
                .handshake(protocol_version, Default::default(), head_height)
                .await
            {
                Err(NetworkError::HandshakeFailure(HandshakeFailure::GenesisMismatch(
                    genesis_id,
                ))) => Ok(genesis_id),
                Err(e) => Err(e),
                Ok(HandshakeResponse(_)) => Err(NetworkError::UnexpectedResponse)?,
            },
            Some(genesis_id) => Ok(genesis_id),
        }?;

        self.handshake(protocol_version, genesis_id, head_height)
            .await
    }

    pub(super) fn create_handshake(
        &mut self,
        protocol_version: ProtocolVersion,
        genesis_id: GenesisId,
        head_height: BlockHeight,
    ) -> Handshake {
        let sender_peer_id = self.my_peer_id.clone();
        let target_peer_id = self.peer_id.clone();
        let secret_key = self.secret_key.clone();

        let sender_chain_info = PeerChainInfo {
            genesis_id,
            height: head_height,
            archival: false,
            ..Default::default()
        };

        let partial_edge_info =
            PartialEdgeInfo::new(&sender_peer_id, &target_peer_id, 1, &secret_key);

        Handshake {
            protocol_version,
            oldest_supported_version: protocol_version - 2,
            sender_peer_id: self.my_peer_id.clone(),
            target_peer_id: self.peer_id.clone(),
            sender_listen_port: Some(self.sender_listen_port),
            sender_chain_info,
            partial_edge_info,
        }
    }

    pub(super) async fn handshake(
        &mut self,
        protocol_version: ProtocolVersion,
        genesis_id: GenesisId,
        head_height: BlockHeight,
    ) -> Result<HandshakeResponse, NetworkError> {
        let request = (&self.create_handshake(protocol_version, genesis_id, head_height)).into();

        self.write_message(request)
            .await
            .map_err(NetworkError::IO)?;

        (&self
            .read_message_with_timeout()
            .await
            .map_err(NetworkError::IO)?)
            .try_into()
    }

    pub(super) async fn write_message(&mut self, msg: PeerMessage) -> io::Result<()> {
        let msg_data = msg.write_to_bytes()?;
        let data = [&(msg_data.len() as u32).to_le_bytes(), msg_data.as_slice()].concat();
        self.stream.write_all(&data).await
    }

    pub(super) async fn read_message(&mut self) -> io::Result<PeerMessage> {
        let mut prefix_data = [0; 4];
        self.stream.read_exact(&mut prefix_data).await?;

        let msg_len = u32::from_le_bytes(prefix_data) as usize;

        let mut msg_data = vec![0u8; msg_len];
        self.stream.read_exact(&mut msg_data).await?;

        PeerMessage::parse_from_bytes(&msg_data).map_err(|_| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("error parsing message (length: {})", msg_len),
            )
        })
    }

    pub(super) async fn read_message_with_timeout(&mut self) -> io::Result<PeerMessage> {
        time::timeout(self.timeout, self.read_message()).await?
    }
}
