use argon2::{Algorithm::Argon2d, Argon2, Params, PasswordHasher};
use bytes::{BufMut, Bytes, BytesMut};
use chacha20poly1305::{XChaCha20Poly1305, XNonce, aead::Aead};
use hmac::{Hmac, KeyInit, Mac};
use libp2p::{Multiaddr, PeerId};
use rand::Rng;
use serde::{Deserialize, Serialize};
use sha2::Sha256;

mod peer_id_serde {
    use super::*;
    use serde::{Deserializer, Serializer};

    pub fn serialize<S>(peer: &PeerId, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_bytes(&peer.to_bytes())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<PeerId, D::Error>
    where
        D: Deserializer<'de>,
    {
        let bytes = Vec::<u8>::deserialize(deserializer)?;

        PeerId::from_bytes(&bytes).map_err(serde::de::Error::custom)
    }
}
///An invite is a way to find another user on the web. It contains its address, Id, and metadata to check if the content is properly assigned, valid, and etc.
#[derive(Serialize, Deserialize, Debug)]
pub struct DirectInviteMetadata {
    pub address: Multiaddr,
    #[serde(with = "peer_id_serde")]
    pub peer: PeerId,
    pub timestamp: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DirectInviteRaw {
    metadata: Vec<u8>,
    ///Salt used to hash the contents of the metadata
    salt: [u8; 32],
    nonce: [u8; 24],
    signature: [u8; 32],
}

#[derive(Debug)]
pub struct DirectInvite {
    metadata: DirectInviteMetadata,
    ///Salt used to hash the contents of the metadata
    salt: [u8; 32],
    nonce: [u8; 24],
    signature: [u8; 32],
}

///The struct that is used locally to check if the connection of some
pub struct LocalDirectInvite {
    ///The address of the invite
    address: Multiaddr,
    ///The peer id of the user that generated this invite. Ideally, the user itself
    peer: PeerId,
    ///The maximum of usage of requests this invite can contain
    maximum_usages: u8,
    ///Until when this invite will be valid
    timestamp: u64,
}

impl DirectInviteMetadata {
    pub fn new(address: Multiaddr, peer: PeerId, timestamp: u64) -> Self {
        Self {
            address,
            peer,
            timestamp,
        }
    }
}

impl DirectInviteRaw {
    pub fn as_direct(self, password: &[u8]) -> color_eyre::Result<DirectInvite> {
        let crypto_key = {
            let mut key = [0; 32];
            let argon = Argon2::new(
                argon2::Algorithm::Argon2id,
                argon2::Version::V0x13,
                Params::default(),
            );
            argon.hash_password_into(password, &self.salt, &mut key)?;
            key
        };
        let decrypt_metadata = XChaCha20Poly1305::new_from_slice(&crypto_key)?
            .decrypt(&XNonce::try_from(self.nonce)?, self.metadata.as_ref())?;
        let metadata = postcard::from_bytes(&decrypt_metadata)?;
        Ok(DirectInvite {
            metadata,
            salt: self.salt,
            nonce: self.nonce,
            signature: self.signature,
        })
    }
}

impl DirectInvite {
    ///Retrieves cryptographic safe contents for using on a direct invite. Returns the salt and the nonce to sign the invite metadata.
    pub fn invite_crypto(private_key: &[u8; 32]) -> ([u8; 32], [u8; 24], Hmac<Sha256>) {
        let mut salt = [0; 32];
        let mut nonce = [0; 24];
        rand::rng().fill_bytes(&mut salt);
        rand::rng().fill_bytes(&mut nonce);
        let hmac = Hmac::new_from_slice(private_key).expect("Size should match properly to sha256");
        (salt, nonce, hmac)
    }

    ///Creates a new direct invite with salt, and signatures safely generated and the given `metadata`
    pub fn new(metadata: DirectInviteMetadata) -> Self {
        Self {
            metadata,
            salt: [0; 32],
            nonce: [0; 24],
            signature: [0; 32],
        }
    }

    pub fn metadata(&self) -> &DirectInviteMetadata {
        &self.metadata
    }

    ///Returns the raw representation of a direct invite to be sent across the network
    pub fn as_raw(
        self,
        private_key: &[u8; 32],
        password: &[u8],
    ) -> color_eyre::Result<DirectInviteRaw> {
        let (salt, nonce, mut hmac) = Self::invite_crypto(&private_key);

        let crypto_key = {
            let mut key = [0; 32];
            let argon = Argon2::new(
                argon2::Algorithm::Argon2id,
                argon2::Version::V0x13,
                Params::default(),
            );
            argon.hash_password_into(password, &salt, &mut key)?;
            key
        };

        let metadata = postcard::to_allocvec(&self.metadata)?;
        let encrypt_metadata = XChaCha20Poly1305::new_from_slice(&crypto_key)?
            .encrypt(&XNonce::try_from(nonce)?, metadata.as_ref())?;
        hmac.update(&encrypt_metadata);
        let signature = hmac.finalize();
        Ok(DirectInviteRaw {
            metadata: encrypt_metadata,
            salt,
            nonce,
            signature: *signature
                .as_bytes()
                .as_array()
                .expect("Signature should contain 32 bytes"),
        })
    }
}
