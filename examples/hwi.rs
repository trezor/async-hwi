use async_hwi::HWI;

#[cfg(feature = "specter")]
use async_hwi::specter::{Specter, SpecterSimulator};

#[cfg(feature = "ledger")]
use async_hwi::ledger::{Ledger, LedgerSimulator};

#[cfg(feature = "trezor")]
use async_hwi::trezor::{Trezor, TrezorSimulator};

#[tokio::main]
pub async fn main() {
    let list = list_hardware_wallets().await;
    eprintln!(
        "{} device{} connected",
        list.len(),
        if list.len() > 1 { "s" } else { "" }
    );

    for hw in list {
        eprintln!(
            "{} (fingerprint: {}, version: {})",
            hw.device_kind(),
            hw.get_master_fingerprint().await.unwrap(),
            hw.get_version()
                .await
                .map(|v| v.to_string())
                .unwrap_or("unknown".to_string()),
        );
    }
}

pub async fn list_hardware_wallets() -> Vec<Box<dyn HWI + Send>> {
    let mut hws = Vec::new();

    #[cfg(feature = "specter")]
    if let Ok(device) = SpecterSimulator::try_connect().await {
        hws.push(device.into());
    }

    #[cfg(feature = "specter")]
    if let Ok(device) = Specter::try_connect_serial().await {
        hws.push(device.into());
    }

    #[cfg(feature = "ledger")]
    if let Ok(device) = LedgerSimulator::try_connect().await {
        hws.push(device.into());
    }

    #[cfg(feature = "ledger")]
    if let Ok(device) = Ledger::try_connect_hid() {
        hws.push(device.into());
    }

    #[cfg(feature = "trezor")]
    if let Ok(device) = TrezorSimulator::try_connect().await {
        hws.push(device.into());
    }

    #[cfg(feature = "trezor")]
    if let Ok(device) = Trezor::try_connect_usb().await {
        hws.push(device.into());
    }

    hws
}
