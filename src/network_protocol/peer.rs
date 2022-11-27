use protobuf::MessageField;

use borsh::{BorshDeserialize, BorshSerialize};

use super::{proto, DynError};

use near_primitives::{
    block::GenesisId,
    hash::CryptoHash,
    network::PeerId,
    types::{BlockHeight, ShardId},
};

// *** CryptoHash ***

impl From<&CryptoHash> for proto::CryptoHash {
    fn from(value: &CryptoHash) -> Self {
        Self {
            hash: value.0.into(),
            ..Default::default()
        }
    }
}

type ParseCryptoHashError = DynError;

impl TryFrom<&proto::CryptoHash> for CryptoHash {
    type Error = ParseCryptoHashError;

    fn try_from(value: &proto::CryptoHash) -> Result<Self, Self::Error> {
        CryptoHash::try_from(&value.hash[..])
    }
}

// *** PeerId ***

impl From<&PeerId> for proto::PublicKey {
    fn from(value: &PeerId) -> Self {
        Self {
            borsh: value.try_to_vec().unwrap(),
            ..Self::default()
        }
    }
}

pub type ParsePeerIdError = borsh::maybestd::io::Error;

impl TryFrom<&proto::PublicKey> for PeerId {
    type Error = ParsePeerIdError;

    fn try_from(value: &proto::PublicKey) -> Result<Self, Self::Error> {
        Self::try_from_slice(&value.borsh)
    }
}

// *** GenesisId ***

impl From<&GenesisId> for proto::GenesisId {
    fn from(value: &GenesisId) -> Self {
        Self {
            chain_id: value.chain_id.clone(),
            hash: MessageField::some((&value.hash).into()),
            ..Self::default()
        }
    }
}

type ParseGenesisIdError = DynError;

impl TryFrom<&proto::GenesisId> for GenesisId {
    type Error = ParseGenesisIdError;

    fn try_from(value: &proto::GenesisId) -> Result<Self, Self::Error> {
        Ok(Self {
            chain_id: value.chain_id.clone(),
            hash: value.hash.as_ref().ok_or("hash required")?.try_into()?,
        })
    }
}

// *** PeerChainInfo ***

#[derive(Debug, Default)]
#[cfg_attr(test, derive(PartialEq))]
pub struct PeerChainInfo {
    pub genesis_id: GenesisId,
    pub height: BlockHeight,
    pub tracked_shards: Vec<ShardId>,
    pub archival: bool,
}

impl From<&PeerChainInfo> for proto::PeerChainInfo {
    fn from(value: &PeerChainInfo) -> Self {
        Self {
            genesis_id: MessageField::some((&value.genesis_id).into()),
            height: value.height,
            tracked_shards: value.tracked_shards.clone(),
            archival: value.archival,
            ..Self::default()
        }
    }
}

type ParsePeerChainInfoError = DynError;

impl TryFrom<&proto::PeerChainInfo> for PeerChainInfo {
    type Error = ParsePeerChainInfoError;

    fn try_from(value: &proto::PeerChainInfo) -> Result<Self, Self::Error> {
        Ok(Self {
            genesis_id: value
                .genesis_id
                .as_ref()
                .ok_or("genesis_id required")?
                .try_into()?,
            height: value.height,
            tracked_shards: value.tracked_shards.clone(),
            archival: value.archival,
        })
    }
}
