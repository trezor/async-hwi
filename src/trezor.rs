use std::fmt::Debug;
use std::str::FromStr;

use bitcoin::{
    consensus::encode,
    util::bip32::{DerivationPath, ExtendedPubKey, Fingerprint},
    util::psbt::PartiallySignedTransaction as Psbt,
};

use super::{DeviceKind, Error as HWIError, HWI};
use async_trait::async_trait;

#[derive(Debug)]
pub struct Trezor<T: Transport> {
    transport: T,
    kind: DeviceKind,
}

impl<T: Transport> Trezor<T> {
    pub async fn fingerprint(&self) -> Result<Fingerprint, TrezorError> {
        const RES: &str = "00000000";
        Fingerprint::from_str(&RES).map_err(|e| TrezorError::Device(e.to_string()))
    }

    pub async fn get_extended_pubkey(
        &self,
        _path: &DerivationPath,
    ) -> Result<ExtendedPubKey, TrezorError> {
        const RES: &str = "";
        ExtendedPubKey::from_str(&RES).map_err(|e| TrezorError::Device(e.to_string()))
    }

    /// If the descriptor contains master public keys but doesn't contain wildcard derivations,
    /// the default derivation /{0,1}/* will be added to all extended keys in the descriptor.
    /// If at least one of the xpubs has a wildcard derivation the descriptor will not be changed.
    /// /** is an equivalent of /{0,1}/*.
    pub async fn add_wallet(&self, _name: &str, _policy: &str) -> Result<(), TrezorError> {
        Ok(())
    }

    pub async fn sign(&self, _psbt: &Psbt) -> Result<Psbt, TrezorError> {
        const RES: &[u8] = "".as_bytes();
        encode::deserialize(&RES).map_err(|e| TrezorError::Device(e.to_string()))
    }
}

#[async_trait]
impl<T: Transport + Sync + Send> HWI for Trezor<T> {
    fn device_kind(&self) -> DeviceKind {
        self.kind
    }

    async fn get_version(&self) -> Result<super::Version, HWIError> {
        Err(HWIError::UnimplementedMethod)
    }

    async fn is_connected(&self) -> Result<(), HWIError> {
        self.fingerprint().await?;
        Ok(())
    }

    async fn get_master_fingerprint(&self) -> Result<Fingerprint, HWIError> {
        Ok(self.fingerprint().await?)
    }

    async fn get_extended_pubkey(
        &self,
        path: &DerivationPath,
        _display: bool,
    ) -> Result<ExtendedPubKey, HWIError> {
        Ok(self.get_extended_pubkey(path).await?)
    }

    async fn register_wallet(
        &self,
        name: &str,
        policy: &str,
    ) -> Result<Option<[u8; 32]>, HWIError> {
        self.add_wallet(name, policy).await?;
        Ok(None)
    }

    async fn sign_tx(&self, _psbt: &mut Psbt) -> Result<(), HWIError> {
        Ok(())
    }
}

#[async_trait]
pub trait Transport: Debug {
    async fn request(&self, req: &str) -> Result<String, TrezorError>;
}

#[derive(Debug)]
pub struct TcpTransport;

#[async_trait]
impl Transport for TcpTransport {
    async fn request(&self, _req: &str) -> Result<String, TrezorError> {
        return Ok(String::from(""));
    }
}

pub type TrezorSimulator = Trezor<TcpTransport>;

impl TrezorSimulator {
    pub async fn try_connect() -> Result<Self, HWIError> {
        let s = TrezorSimulator {
            transport: TcpTransport {},
            kind: DeviceKind::TrezorSimulator,
        };
        s.is_connected().await?;
        Ok(s)
    }
}

impl Trezor<UsbTransport> {
    pub async fn try_connect_usb() -> Result<Self, HWIError> {
        let s = Trezor {
            transport: UsbTransport {},
            kind: DeviceKind::Trezor,
        };
        s.is_connected().await?;
        Ok(s)
    }
}

#[derive(Debug)]
pub struct UsbTransport;
impl UsbTransport {
    pub const TREZOR_VID: u16 = 0x1209;
    pub const TREZOR_PID: u16 = 0x53C1;
}

#[async_trait]
impl Transport for UsbTransport {
    async fn request(&self, _req: &str) -> Result<String, TrezorError> {
        return Ok(String::from(""));
    }
}

#[derive(Debug)]
pub enum TrezorError {
    DeviceNotFound,
    DeviceDidNotSign,
    Device(String),
}

impl std::fmt::Display for TrezorError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::DeviceNotFound => write!(f, "Trezor not found"),
            Self::DeviceDidNotSign => write!(f, "Trezor did not sign the psbt"),
            Self::Device(e) => write!(f, "Trezor error: {}", e),
        }
    }
}

impl From<TrezorError> for HWIError {
    fn from(e: TrezorError) -> HWIError {
        match e {
            TrezorError::DeviceNotFound => HWIError::DeviceNotFound,
            TrezorError::DeviceDidNotSign => HWIError::DeviceDidNotSign,
            TrezorError::Device(e) => HWIError::Device(e),
        }
    }
}
