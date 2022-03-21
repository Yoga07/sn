// Copyright 2022 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

//! Module providing keys, keypairs, and signatures.
//!
//! The easiest way to get a `PublicKey` is to create a random `Keypair` first through one of the
//! `new` functions. A `PublicKey` can't be generated by itself; it must always be derived from a
//! secret key.

use super::super::{utils, Error, Result};
use super::super::{Keypair, Signature};

use hex_fmt::HexFmt;
use serde::{Deserialize, Serialize};
use signature::Verifier;
use std::{
    cmp::Ordering,
    convert::TryInto,
    fmt::{self, Debug, Display, Formatter, LowerHex, UpperHex},
    hash::{Hash, Hasher},
};
use xor_name::{XorName, XOR_NAME_LEN};

/// Wrapper for different public key types.
#[derive(Clone, Copy, Eq, PartialEq, Serialize, Deserialize, custom_debug::Debug)]
pub enum PublicKey {
    /// Ed25519 public key.
    Ed25519(#[debug(with = "Self::fmt_ed25519")] ed25519_dalek::PublicKey),
    /// BLS public key.
    Bls(bls::PublicKey),
    /// BLS public key share.
    BlsShare(bls::PublicKeyShare),
}

impl PublicKey {
    /// Construct and ed25519 public key from
    /// a hex-encoded string.
    ///
    /// It is often useful
    /// to parse such raw strings in user-facing
    /// apps like CLI
    pub fn ed25519_from_hex(hex: &str) -> Result<Self> {
        let bytes = hex::decode(hex).map_err(|err| {
            Error::FailedToParse(format!(
                "Couldn't parse ed25519 public key bytes from hex: {}",
                err
            ))
        })?;
        let pk = ed25519_dalek::PublicKey::from_bytes(bytes.as_ref()).map_err(|err| {
            Error::FailedToParse(format!(
                "Couldn't parse ed25519 public key from bytes: {}",
                err
            ))
        })?;
        Ok(Self::from(pk))
    }

    /// Construct and ed25519 public key from
    /// a hex-encoded string.
    ///
    /// It is often useful
    /// to parse such raw strings in user-facing
    /// apps like CLI
    pub fn bls_from_hex(hex: &str) -> Result<Self> {
        let bytes = hex::decode(hex).map_err(|err| {
            Error::FailedToParse(format!(
                "Couldn't parse BLS public key bytes from hex: {}",
                err
            ))
        })?;
        let bytes_fixed_len: [u8; bls::PK_SIZE] = bytes.as_slice().try_into()
            .map_err(|_| Error::FailedToParse(format!(
                "Couldn't parse BLS public key bytes from hex. The provided string must represent exactly {} bytes.",
                bls::PK_SIZE
            )))?;
        let pk = bls::PublicKey::from_bytes(bytes_fixed_len).map_err(|err| {
            Error::FailedToParse(format!(
                "Couldn't parse BLS public key from fixed-length byte array: {}",
                err
            ))
        })?;
        Ok(Self::from(pk))
    }

    /// Returns the bytes of the underlying public key
    pub fn to_bytes(self) -> Vec<u8> {
        match self {
            PublicKey::Ed25519(pub_key) => pub_key.to_bytes().into(),
            PublicKey::Bls(pub_key) => pub_key.to_bytes().into(),
            PublicKey::BlsShare(pub_key) => pub_key.to_bytes().into(),
        }
    }

    /// Returns the ed25519 key, if applicable.
    pub fn ed25519(&self) -> Option<ed25519_dalek::PublicKey> {
        if let Self::Ed25519(key) = self {
            Some(*key)
        } else {
            None
        }
    }

    /// Returns the BLS key, if applicable.
    pub fn bls(&self) -> Option<bls::PublicKey> {
        if let Self::Bls(key) = self {
            Some(*key)
        } else {
            None
        }
    }

    /// Returns the BLS key share, if applicable.
    pub fn bls_share(&self) -> Option<bls::PublicKeyShare> {
        if let Self::BlsShare(key) = self {
            Some(*key)
        } else {
            None
        }
    }

    /// Returns `Ok(())` if `signature` matches the message and `Err(Error::InvalidSignature)`
    /// otherwise.
    pub fn verify<T: AsRef<[u8]>>(&self, signature: &Signature, data: T) -> Result<()> {
        let is_valid = match (self, signature) {
            (Self::Ed25519(pub_key), Signature::Ed25519(sig)) => {
                pub_key.verify(data.as_ref(), sig).is_ok()
            }
            (Self::Bls(pub_key), Signature::Bls(sig)) => pub_key.verify(sig, data),
            (Self::BlsShare(pub_key), Signature::BlsShare(sig)) => pub_key.verify(&sig.share, data),
            _ => return Err(Error::SigningKeyTypeMismatch),
        };
        if is_valid {
            Ok(())
        } else {
            Err(Error::InvalidSignature)
        }
    }

    /// Returns the `PublicKey` serialised and encoded in z-base-32.
    pub fn encode_to_zbase32(&self) -> Result<String> {
        utils::encode(&self)
    }

    /// Creates from z-base-32 encoded string.
    pub fn decode_from_zbase32<I: AsRef<str>>(encoded: I) -> Result<Self> {
        utils::decode(encoded)
    }

    // ed25519_dalek::PublicKey has overly verbose debug output, so we provide our own
    pub(crate) fn fmt_ed25519(pk: &ed25519_dalek::PublicKey, f: &mut Formatter) -> fmt::Result {
        write!(f, "PublicKey({:0.10})", HexFmt(pk.as_bytes()))
    }
}

#[allow(clippy::derive_hash_xor_eq)]
impl Hash for PublicKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        utils::serialise(&self).unwrap_or_default().hash(state)
    }
}

impl Ord for PublicKey {
    fn cmp(&self, other: &PublicKey) -> Ordering {
        utils::serialise(&self)
            .unwrap_or_default()
            .cmp(&utils::serialise(other).unwrap_or_default())
    }
}

impl PartialOrd for PublicKey {
    fn partial_cmp(&self, other: &PublicKey) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl From<PublicKey> for XorName {
    fn from(public_key: PublicKey) -> Self {
        let bytes = match public_key {
            PublicKey::Ed25519(pub_key) => {
                return XorName(pub_key.to_bytes());
            }
            PublicKey::Bls(pub_key) => pub_key.to_bytes(),
            PublicKey::BlsShare(pub_key) => pub_key.to_bytes(),
        };
        let mut xor_name: XorName = xor_name::rand::random();
        xor_name.0.clone_from_slice(&bytes[..XOR_NAME_LEN]);
        xor_name
    }
}

impl From<ed25519_dalek::PublicKey> for PublicKey {
    fn from(public_key: ed25519_dalek::PublicKey) -> Self {
        Self::Ed25519(public_key)
    }
}

impl From<bls::PublicKey> for PublicKey {
    fn from(public_key: bls::PublicKey) -> Self {
        Self::Bls(public_key)
    }
}

impl From<bls::PublicKeyShare> for PublicKey {
    fn from(public_key: bls::PublicKeyShare) -> Self {
        Self::BlsShare(public_key)
    }
}

impl From<&Keypair> for PublicKey {
    fn from(keypair: &Keypair) -> Self {
        keypair.public_key()
    }
}

impl Display for PublicKey {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        Debug::fmt(self, formatter)
    }
}

impl LowerHex for PublicKey {
    /// Useful for displaying public key in user-facing apps
    /// E.g. in cli and in human-readable messaging like for sn_authd
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", hex::encode(self.to_bytes()))
    }
}

impl UpperHex for PublicKey {
    /// Useful for displaying public key in user-facing apps
    /// E.g. in cli and in human-readable messaging like for sn_authd
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", hex::encode_upper(self.to_bytes()))
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::utils;
    use super::*;
    use bls::{self};

    fn gen_keypairs() -> Vec<Keypair> {
        let mut rng = rand::thread_rng();
        let bls_secret_key = bls::SecretKeySet::random(1, &mut rng);
        vec![
            Keypair::new_ed25519(&mut rng),
            Keypair::new_bls_share(
                0,
                bls_secret_key.secret_key_share(0),
                bls_secret_key.public_keys(),
            ),
        ]
    }

    pub(crate) fn gen_keys() -> Vec<PublicKey> {
        gen_keypairs().iter().map(PublicKey::from).collect()
    }

    #[test]
    fn zbase32_encode_decode_public_key() -> Result<()> {
        let keys = gen_keys();

        for key in keys {
            assert_eq!(
                key,
                PublicKey::decode_from_zbase32(&key.encode_to_zbase32()?)?
            );
        }

        Ok(())
    }

    // Test serialising and deserialising public keys.
    #[test]
    fn serialisation_public_key() -> Result<()> {
        let keys = gen_keys();

        for key in keys {
            let encoded = utils::serialise(&key)?;
            let decoded: PublicKey = utils::deserialise(&encoded)?;

            assert_eq!(decoded, key);
        }

        Ok(())
    }
}
