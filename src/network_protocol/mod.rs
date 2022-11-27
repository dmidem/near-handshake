mod _proto {
    include!(concat!(env!("OUT_DIR"), "/proto/mod.rs"));
}

use _proto::network as proto;

pub use proto::PeerMessage;

pub(crate) use proto::peer_message::Message_type as MessageType;

mod edge;
mod handshake;
mod peer;

pub use edge::PartialEdgeInfo;
pub use handshake::{Handshake, HandshakeFailure, HandshakeResponse};
pub use peer::PeerChainInfo;

#[derive(Debug)]
pub enum NetworkError {
    IO(std::io::Error),
    InvalidResponse,
    UnexpectedResponse,
    HandshakeFailure(HandshakeFailure),
}

type DynError = Box<dyn std::error::Error + Send + Sync>;

impl<T> From<T> for proto::PeerMessage
where
    MessageType: From<T>,
{
    fn from(value: T) -> Self {
        Self {
            message_type: Some(value.into()),
            ..Default::default()
        }
    }
}
