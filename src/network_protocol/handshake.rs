use protobuf::MessageField;

use near_primitives::{block::GenesisId, network::PeerId, version::ProtocolVersion};

use super::{
    edge::PartialEdgeInfo, peer::PeerChainInfo, proto, DynError, MessageType, NetworkError,
};

// *** Handshake ***

#[derive(Debug)]
#[cfg_attr(test, derive(PartialEq))]
pub struct Handshake {
    pub protocol_version: ProtocolVersion,
    pub oldest_supported_version: ProtocolVersion,
    pub sender_peer_id: PeerId,
    pub target_peer_id: PeerId,
    pub sender_listen_port: Option<u16>,
    pub sender_chain_info: PeerChainInfo,
    pub partial_edge_info: PartialEdgeInfo,
}

impl From<&Handshake> for MessageType {
    fn from(value: &Handshake) -> Self {
        Self::Handshake(proto::Handshake {
            protocol_version: value.protocol_version,
            oldest_supported_version: value.oldest_supported_version,
            sender_peer_id: MessageField::some((&value.sender_peer_id).into()),
            target_peer_id: MessageField::some((&value.target_peer_id).into()),
            // not used here but needs to be set because of the protocol requirement
            sender_listen_port: value.sender_listen_port.unwrap_or(0).into(),
            sender_chain_info: MessageField::some((&value.sender_chain_info).into()),
            partial_edge_info: MessageField::some((&value.partial_edge_info).into()),
            ..Default::default()
        })
    }
}

type ParseHandshakeError = DynError;

impl TryFrom<&proto::Handshake> for Handshake {
    type Error = ParseHandshakeError;

    fn try_from(value: &proto::Handshake) -> Result<Self, Self::Error> {
        Ok(Self {
            protocol_version: value.protocol_version,
            oldest_supported_version: value.oldest_supported_version,
            sender_peer_id: value
                .sender_peer_id
                .as_ref()
                .ok_or("sender_peer_id required")?
                .try_into()?,
            target_peer_id: value
                .target_peer_id
                .as_ref()
                .ok_or("target_peer_id required")?
                .try_into()?,
            sender_listen_port: {
                let port = u16::try_from(value.sender_listen_port)?;
                if port == 0 {
                    None
                } else {
                    Some(port)
                }
            },
            sender_chain_info: value
                .sender_chain_info
                .as_ref()
                .ok_or("sender_chain_info required")?
                .try_into()?,
            partial_edge_info: value
                .partial_edge_info
                .as_ref()
                .ok_or("sender_chain_info required")?
                .try_into()?,
        })
    }
}

// *** HandshakeFailure ***

#[derive(Debug)]
pub enum HandshakeFailure {
    ProtocolVersionMismatch {
        version: u32,
        oldest_supported_version: u32,
    },
    GenesisMismatch(GenesisId),
    InvalidTarget,
    UnknownReason,
    ParseHandshakeError(ParseHandshakeError),
}

impl From<&HandshakeFailure> for MessageType {
    fn from(value: &HandshakeFailure) -> Self {
        Self::HandshakeFailure(match value {
            HandshakeFailure::ProtocolVersionMismatch {
                version,
                oldest_supported_version,
            } => proto::HandshakeFailure {
                reason: proto::handshake_failure::Reason::ProtocolVersionMismatch.into(),
                version: *version,
                oldest_supported_version: *oldest_supported_version,
                ..proto::HandshakeFailure::default()
            },

            HandshakeFailure::GenesisMismatch(genesis_id) => proto::HandshakeFailure {
                reason: proto::handshake_failure::Reason::GenesisMismatch.into(),
                genesis_id: MessageField::some(genesis_id.into()),
                ..proto::HandshakeFailure::default()
            },

            HandshakeFailure::InvalidTarget => proto::HandshakeFailure {
                reason: proto::handshake_failure::Reason::InvalidTarget.into(),
                ..proto::HandshakeFailure::default()
            },

            // Panic because the error means it's a programming level error
            // (wrong usage of HandshakeFailure)
            x => panic!("Message can not be made from: {:#?}", x),
        })
    }
}

type ParseHandshakeFailureError = DynError;

impl TryFrom<&proto::HandshakeFailure> for HandshakeFailure {
    type Error = ParseHandshakeFailureError;

    fn try_from(value: &proto::HandshakeFailure) -> Result<Self, Self::Error> {
        Ok(match value.reason.enum_value_or_default() {
            proto::handshake_failure::Reason::ProtocolVersionMismatch => {
                HandshakeFailure::ProtocolVersionMismatch {
                    version: value.version,
                    oldest_supported_version: value.oldest_supported_version,
                }
            }

            proto::handshake_failure::Reason::GenesisMismatch => HandshakeFailure::GenesisMismatch(
                value
                    .genesis_id
                    .as_ref()
                    .ok_or("genesis_id required")?
                    .try_into()?,
            ),

            proto::handshake_failure::Reason::InvalidTarget => HandshakeFailure::InvalidTarget,

            proto::handshake_failure::Reason::UNKNOWN => HandshakeFailure::UnknownReason,
        })
    }
}

// *** HandshakeResponse ***

#[derive(Debug)]
pub struct HandshakeResponse(pub Handshake);

impl TryFrom<&proto::PeerMessage> for HandshakeResponse {
    type Error = NetworkError;

    fn try_from(value: &proto::PeerMessage) -> Result<Self, Self::Error> {
        match value
            .message_type
            .as_ref()
            .ok_or(NetworkError::InvalidResponse)?
        {
            MessageType::Handshake(handshake) => Ok(HandshakeResponse(
                handshake
                    .try_into()
                    .map_err(HandshakeFailure::ParseHandshakeError)
                    .map_err(NetworkError::HandshakeFailure)?,
            )),

            MessageType::HandshakeFailure(failure) => Err(failure.try_into().map_or(
                NetworkError::InvalidResponse,
                NetworkError::HandshakeFailure,
            )),

            _ => Err(NetworkError::UnexpectedResponse),
        }
    }
}
