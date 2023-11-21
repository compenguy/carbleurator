use anyhow::{Context, Result};
use log::{debug, trace, warn};

use btleplug::api::Manager as _;
use btleplug::api::Peripheral as _;
use btleplug::api::{Central, Characteristic, ScanFilter, WriteType};
use btleplug::platform::{Adapter, Manager, Peripheral};

use crate::errors::CarbleuratorError;
use crate::signaling::update_signal_progress;

pub(crate) struct BleSerial {
    characteristic_uuid: uuid::Uuid,
    name: String,
    peripheral: Option<Peripheral>,
    characteristic: Option<Characteristic>,
}

impl BleSerial {
    pub(crate) fn new(characteristic_uuid: uuid::Uuid, name: String) -> Self {
        debug!(
            "Filtering ble adapters for characteristics {:?}",
            characteristic_uuid
        );
        Self {
            characteristic_uuid,
            name,
            peripheral: None,
            characteristic: None,
        }
    }

    async fn get_central(&mut self) -> Result<Adapter> {
        update_signal_progress();

        debug!("Initializing bluetooth...");
        let manager = Manager::new().await?;

        update_signal_progress();

        debug!("Initializing BLE central...");
        let adapter = manager
            .adapters()
            .await?
            .into_iter()
            .next()
            .ok_or(CarbleuratorError::MissingBleAdapter)?;
        update_signal_progress();
        Ok(adapter)
    }

    async fn get_peripheral(&mut self) -> Result<Peripheral> {
        if self.peripheral.is_none() {
            let adapter = self.get_central().await?;
            update_signal_progress();

            debug!("Starting scan for BLE peripherals...");
            trace!("Using adapter {:?}", adapter.adapter_info().await);
            adapter
                .start_scan(ScanFilter {
                    //services: vec![self.characteristic_uuid],
                    services: vec![],
                })
                .await
                .with_context(|| "Failed to scan for new BLE peripherals".to_string())?;

            update_signal_progress();

            debug!("Waiting for devices to appear...");
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;

            update_signal_progress();
            let mut retries = 5;
            debug!("Iterating over discovered devices searching for a compatible peripheral...");
            while retries > 0 && self.peripheral.is_none() {
                let peripherals = adapter.peripherals().await?.into_iter();
                for peripheral in peripherals {
                    let local_name = peripheral
                        .properties()
                        .await?
                        .and_then(|props| props.local_name);
                    if local_name.is_none() {
                        continue;
                    }
                    debug!("\tperipheral {:?}", local_name);
                    if local_name
                        .as_ref()
                        .map(|name| name == &self.name)
                        .unwrap_or(false)
                    {
                        debug!("Found matching peripheral: {}", self.name);
                        self.peripheral = Some(peripheral);
                        break;
                    }
                }
                if self.peripheral.is_none() {
                    warn!("No compatible BLE peripherals found. Retrying...");
                    retries -= 1;
                    std::thread::sleep(std::time::Duration::from_secs(1));
                }
                update_signal_progress();
            }
        }

        let peripheral = self
            .peripheral
            .clone()
            .ok_or(CarbleuratorError::BleAdapterDiscoveryTimeout)?;
        //peripheral.discover_services().await?;
        trace!("BLE peripheral found ({:?})", peripheral);
        if !peripheral.is_connected().await? {
            debug!("Not connected to peripheral yet. Connecting...");
            peripheral.connect().await?;
        }
        debug!("BLE peripheral connected.");
        update_signal_progress();
        peripheral.discover_services().await?;
        Ok(peripheral)
    }

    pub(crate) async fn get_characteristic(&mut self) -> Result<Characteristic> {
        if self.characteristic.is_none() {
            let peripheral = self.get_peripheral().await?;
            debug!(
                "Searching for correct peripheral characteristic for communication ({})...",
                &self.characteristic_uuid
            );
            let characteristic = peripheral
                .characteristics()
                .into_iter()
                .inspect(|x| log::debug!("\tcharacteristic {}", x.uuid))
                .find(|x| x.uuid == self.characteristic_uuid)
                .ok_or(CarbleuratorError::BleAdapterMissingCharacteristic)?;
            self.characteristic = Some(characteristic);
        }
        self.characteristic
            .clone()
            .ok_or(CarbleuratorError::BleAdapterMissingCharacteristic)
            .map_err(|e| e.into())
    }

    pub(crate) async fn send_message(&mut self, message: &[u8]) -> Result<()> {
        let peripheral = self.get_peripheral().await?;
        let characteristic = self.get_characteristic().await?;

        peripheral
            .write(&characteristic, message, WriteType::WithoutResponse)
            .await
            .map_err(|e| e.into())
    }
}
