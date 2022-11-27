use borsh::{BorshDeserialize, BorshSerialize};

use near_crypto::{SecretKey, Signature};

use near_primitives::{hash::CryptoHash, network::PeerId};

use super::proto;

#[derive(BorshSerialize, BorshDeserialize, Debug, Default)]
#[cfg_attr(test, derive(PartialEq))]
pub struct PartialEdgeInfo {
    pub nonce: u64,
    pub signature: Signature,
}

impl PartialEdgeInfo {
    fn build_hash(peer0: &PeerId, peer1: &PeerId, nonce: u64) -> CryptoHash {
        CryptoHash::hash_borsh(&(peer0, peer1, nonce))
    }

    pub fn new(peer0: &PeerId, peer1: &PeerId, nonce: u64, secret_key: &SecretKey) -> Self {
        let hash = if peer0 < peer1 {
            Self::build_hash(peer0, peer1, nonce)
        } else {
            Self::build_hash(peer1, peer0, nonce)
        };

        Self {
            nonce,
            signature: secret_key.sign(hash.as_ref()),
        }
    }
}

impl From<&PartialEdgeInfo> for proto::PartialEdgeInfo {
    fn from(value: &PartialEdgeInfo) -> Self {
        Self {
            borsh: value.try_to_vec().unwrap(),
            ..Self::default()
        }
    }
}

pub type ParsePartialEdgeInfoError = borsh::maybestd::io::Error;

impl TryFrom<&proto::PartialEdgeInfo> for PartialEdgeInfo {
    type Error = ParsePartialEdgeInfoError;

    fn try_from(value: &proto::PartialEdgeInfo) -> Result<Self, Self::Error> {
        Self::try_from_slice(&value.borsh)
    }
}
