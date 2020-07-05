use thiserror::Error;

#[derive(Error, Debug)]
pub enum CarbleuratorError {
    #[error("USB not supported")]
    UsbNotSupportedError,
    #[error("USB device initialization error")]
    UsbDeviceInitializationError,
    #[error("USB initialization error")]
    UsbInitializationError(Box<dyn std::error::Error + Send + Sync>),
    #[error("No USB gamepads found")]
    MissingGamepad,
    #[error("No BLE adapters found")]
    MissingBleAdapter,
    #[error("BLE adapter discovery timeout")]
    BleAdapterDiscoveryTimeout,
    #[error("BLE adapter missing required characteristic")]
    BleAdapterMissingCharacteristic,
}

impl From<gilrs::Error> for CarbleuratorError {
    fn from(err: gilrs::Error) -> Self {
        match err {
            gilrs::Error::NotImplemented(_) => Self::UsbNotSupportedError,
            gilrs::Error::InvalidAxisToBtn => Self::UsbDeviceInitializationError,
            gilrs::Error::Other(e) => Self::UsbInitializationError(e),
        }
    }
}
