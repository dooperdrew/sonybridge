use async_trait::async_trait;
use sonybridge_core::transport::{Transport, TransportError};
use std::fmt::Debug;
use std::sync::Mutex;
use thiserror::Error;
use windows::core::{GUID, HSTRING};
use windows::Devices::Bluetooth::{BluetoothDevice, BluetoothLEDevice};
use windows::Devices::Enumeration::DeviceInformation;
use windows::Networking::Sockets::StreamSocket;
use windows::Storage::Streams::{DataReader, DataWriter, InputStreamOptions};

/// Sony MDR Protocol v1 Service UUID (WH-1000XM3, WH-1000XM4)
pub const SONY_MDR_V1_UUID: GUID = GUID::from_u128(0x96cc203e_5068_46ad_b32d_396ceebe3928);
/// Sony MDR Protocol v2 Service UUID (WH-1000XM5, WF-1000XM5)
pub const SONY_MDR_V2_UUID: GUID = GUID::from_u128(0x956c7b26_d49a_4ba8_b03f_b17d393cb6e2);
/// Standard Bluetooth Serial Port Profile (SPP) UUID
pub const SPP_SERVICE_UUID: GUID = GUID::from_u128(0x00001101_0000_1000_8000_00805f9b34fb);

#[derive(Error, Debug)]
pub enum BluetoothError {
    #[error("Device not found: {0}")]
    DeviceNotFound(String),
    #[error("RFCOMM service not found or access denied: {0}")]
    ServiceNotFound(String),
    #[error("Socket connection failed: {0}")]
    SocketError(String),
    #[error("WinRT COM error: {0}")]
    WinRt(#[from] windows::core::Error),
}

#[derive(Debug, Clone)]
pub struct DiscoveredDevice {
    pub id: String,
    pub name: String,
    pub mac_address: u64,
    pub is_connected: bool,
    pub rfcomm_services: Vec<String>,
}

pub struct WindowsBluetoothManager;

impl WindowsBluetoothManager {
    /// Enumerate paired Bluetooth devices and filter for Sony headphone devices.
    pub async fn discover_paired_sony_devices() -> Result<Vec<DiscoveredDevice>, BluetoothError> {
        let selector = BluetoothDevice::GetDeviceSelectorFromPairingState(true)?;
        let devices = DeviceInformation::FindAllAsyncAqsFilter(&selector)?.get()?;

        let mut results = Vec::new();
        for device_info in devices {
            let name = device_info.Name()?.to_string();
            let id = device_info.Id()?.to_string();

            let lower_name = name.to_lowercase();
            if lower_name.contains("wh-1000")
                || lower_name.contains("wf-1000")
                || lower_name.contains("sony")
                || lower_name.contains("mdr-")
            {
                if let Ok(bt_device_op) = BluetoothDevice::FromIdAsync(&HSTRING::from(&id)) {
                    if let Ok(bt_device) = bt_device_op.get() {
                        let mac = bt_device.BluetoothAddress().unwrap_or(0);
                        let is_connected = bt_device
                            .ConnectionStatus()
                            .map(|s| s.0 == 1) // 1 = Connected
                            .unwrap_or(false);

                        let mut rfcomm_services = Vec::new();
                        if let Ok(services_op) = bt_device.GetRfcommServicesAsync() {
                            if let Ok(services_res) = services_op.get() {
                                if let Ok(services) = services_res.Services() {
                                    for svc in services {
                                        if let Ok(svc_id) = svc.ServiceId() {
                                            if let Ok(uuid_str) = svc_id.AsString() {
                                                rfcomm_services.push(uuid_str.to_string());
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        results.push(DiscoveredDevice {
                            id,
                            name,
                            mac_address: mac,
                            is_connected,
                            rfcomm_services,
                        });
                    }
                }
            }
        }
        Ok(results)
    }

    /// Enumerate paired Bluetooth LE devices and filter for Sony headphone devices.
    pub async fn discover_paired_sony_ble_devices() -> Result<Vec<String>, BluetoothError> {
        let selector = BluetoothLEDevice::GetDeviceSelectorFromPairingState(true)?;
        let devices = DeviceInformation::FindAllAsyncAqsFilter(&selector)?.get()?;

        let mut results = Vec::new();
        for device_info in devices {
            let name = device_info.Name()?.to_string();
            let lower_name = name.to_lowercase();
            if lower_name.contains("wh-1000")
                || lower_name.contains("wf-1000")
                || lower_name.contains("sony")
                || lower_name.contains("mdr-")
            {
                results.push(name);
            }
        }
        Ok(results)
    }

    /// Comprehensive connection diagnostic check inspecting Classic Bluetooth, BLE, and Audio endpoints.
    pub async fn run_connection_diagnostics() -> ConnectionDiagnostics {
        let classic_devices = Self::discover_paired_sony_devices()
            .await
            .unwrap_or_default();
        let ble_devices = Self::discover_paired_sony_ble_devices()
            .await
            .unwrap_or_default();
        let audio_endpoints =
            crate::audio::AudioEndpointSanitizer::enumerate_render_endpoints().unwrap_or_default();
        let hands_free_active = crate::audio::AudioEndpointSanitizer::is_hands_free_active();

        let classic_active = classic_devices.iter().find(|d| d.is_connected);
        let ble_found = ble_devices.first().cloned();
        let default_audio = audio_endpoints
            .iter()
            .find(|e| e.is_default)
            .map(|e| e.name.clone());

        if let Some(active) = classic_active {
            ConnectionDiagnostics {
                is_connected: true,
                status: "connected".to_string(),
                device_name: Some(active.name.clone()),
                mac_address: Some(format!("{:012X}", active.mac_address)),
                classic_paired: true,
                ble_paired: ble_found.is_some(),
                ble_device_name: ble_found,
                audio_endpoints_count: audio_endpoints.len(),
                hands_free_active,
                default_audio_endpoint: default_audio,
                battery_percent: None,
                message: format!(
                    "Connected to {} via Classic Bluetooth & RFCOMM.",
                    active.name
                ),
                troubleshooting_steps: Vec::new(),
            }
        } else if !classic_devices.is_empty() {
            let first = &classic_devices[0];
            ConnectionDiagnostics {
                is_connected: false,
                status: "disconnected".to_string(),
                device_name: Some(first.name.clone()),
                mac_address: Some(format!("{:012X}", first.mac_address)),
                classic_paired: true,
                ble_paired: ble_found.is_some(),
                ble_device_name: ble_found,
                audio_endpoints_count: audio_endpoints.len(),
                hands_free_active,
                default_audio_endpoint: default_audio,
                battery_percent: None,
                message: format!(
                    "'{}' is paired in Windows but currently disconnected or powered off.",
                    first.name
                ),
                troubleshooting_steps: vec![
                    "Turn on your headphones (press Power once).".to_string(),
                    "Ensure your headphones are within Bluetooth range.".to_string(),
                    "Click 'Reconnect' in SonyBridge once powered on.".to_string(),
                ],
            }
        } else if let Some(ble_name) = ble_found {
            ConnectionDiagnostics {
                is_connected: false,
                status: "ble_only".to_string(),
                device_name: None,
                mac_address: None,
                classic_paired: false,
                ble_paired: true,
                ble_device_name: Some(ble_name.clone()),
                audio_endpoints_count: audio_endpoints.len(),
                hands_free_active,
                default_audio_endpoint: default_audio,
                battery_percent: None,
                message: format!(
                    "Found '{ble_name}' (Bluetooth Low Energy beacon only). Audio & MDR controls require Classic Bluetooth pairing."
                ),
                troubleshooting_steps: vec![
                    "Turn headphones completely OFF.".to_string(),
                    "Press and hold the Power button for 7 SECONDS until you hear 'Bluetooth pairing'.".to_string(),
                    "In Windows Settings -> Bluetooth & devices -> Add device -> Bluetooth, select 'WH-1000XM3' (headphone icon, NOT LE_).".to_string(),
                    "Once paired, return to SonyBridge and click 'Reconnect'.".to_string(),
                ],
            }
        } else {
            ConnectionDiagnostics {
                is_connected: false,
                status: "not_paired".to_string(),
                device_name: None,
                mac_address: None,
                classic_paired: false,
                ble_paired: false,
                ble_device_name: None,
                audio_endpoints_count: audio_endpoints.len(),
                hands_free_active,
                default_audio_endpoint: default_audio,
                battery_percent: None,
                message: "No Sony Bluetooth headphones detected in Windows.".to_string(),
                troubleshooting_steps: vec![
                    "Ensure your PC has Bluetooth enabled.".to_string(),
                    "Put your headphones into pairing mode by holding the Power button for 7 seconds until 'Bluetooth pairing' is announced.".to_string(),
                    "Open Windows Settings -> Bluetooth & devices -> Add device -> Bluetooth, and pair your headphones.".to_string(),
                    "Click 'Reconnect' once paired.".to_string(),
                ],
            }
        }
    }
}

/// Comprehensive connection diagnostic telemetry for UI presentation and troubleshooting.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ConnectionDiagnostics {
    pub is_connected: bool,
    pub status: String,
    pub device_name: Option<String>,
    pub mac_address: Option<String>,
    pub classic_paired: bool,
    pub ble_paired: bool,
    pub ble_device_name: Option<String>,
    pub audio_endpoints_count: usize,
    pub hands_free_active: bool,
    pub default_audio_endpoint: Option<String>,
    pub battery_percent: Option<u8>,
    pub message: String,
    pub troubleshooting_steps: Vec<String>,
}

struct ActiveSocket {
    _socket: StreamSocket,
    writer: DataWriter,
    reader: DataReader,
}

/// Real WinRT RFCOMM StreamSocket Transport implementation for Sony MDR headphones.
pub struct WinRtRfcommTransport {
    device_name: String,
    mac_address: Option<u64>,
    connected: bool,
    active_socket: Mutex<Option<ActiveSocket>>,
}

impl WinRtRfcommTransport {
    pub fn new(device_name: impl Into<String>) -> Self {
        Self {
            device_name: device_name.into(),
            mac_address: None,
            connected: false,
            active_socket: Mutex::new(None),
        }
    }

    pub fn with_mac(device_name: impl Into<String>, mac_address: u64) -> Self {
        Self {
            device_name: device_name.into(),
            mac_address: Some(mac_address),
            connected: false,
            active_socket: Mutex::new(None),
        }
    }

    /// Connect to the headphone's Sony MDR RFCOMM service via WinRT StreamSocket.
    pub async fn connect(&mut self) -> Result<(), BluetoothError> {
        tracing::info!(device = %self.device_name, "Connecting via WinRT RFCOMM transport");

        let bt_device = if let Some(mac) = self.mac_address {
            BluetoothDevice::FromBluetoothAddressAsync(mac)?.get()?
        } else {
            let discovered = WindowsBluetoothManager::discover_paired_sony_devices().await?;
            let first = discovered.into_iter().next().ok_or_else(|| {
                BluetoothError::DeviceNotFound(format!(
                    "No paired Sony device matching '{}'",
                    self.device_name
                ))
            })?;
            self.mac_address = Some(first.mac_address);
            BluetoothDevice::FromBluetoothAddressAsync(first.mac_address)?.get()?
        };

        let services_result = bt_device.GetRfcommServicesAsync()?.get()?;
        let services = services_result.Services()?;

        let mut target_service = None;
        for svc in &services {
            if let Ok(svc_id) = svc.ServiceId() {
                if let Ok(uuid) = svc_id.Uuid() {
                    if uuid == SONY_MDR_V1_UUID || uuid == SONY_MDR_V2_UUID {
                        target_service = Some(svc);
                        break;
                    } else if uuid == SPP_SERVICE_UUID && target_service.is_none() {
                        target_service = Some(svc);
                    }
                }
            }
        }

        // Fallback to first available RFCOMM service if specific UUID wasn't matched
        if target_service.is_none() {
            target_service = services.into_iter().next();
        }

        let service = target_service.ok_or_else(|| {
            BluetoothError::ServiceNotFound(
                "No RFCOMM services exposed by headphone (ensure device is powered on)".to_string(),
            )
        })?;

        let host_name = service.ConnectionHostName()?;
        let service_name = service.ConnectionServiceName()?;

        let socket = StreamSocket::new()?;
        socket
            .ConnectAsync(&host_name, &service_name)?
            .get()
            .map_err(|e| BluetoothError::SocketError(e.to_string()))?;

        let writer = DataWriter::CreateDataWriter(&socket.OutputStream()?)?;
        let reader = DataReader::CreateDataReader(&socket.InputStream()?)?;
        reader.SetInputStreamOptions(InputStreamOptions::Partial)?;

        if let Ok(mut guard) = self.active_socket.lock() {
            *guard = Some(ActiveSocket {
                _socket: socket,
                writer,
                reader,
            });
        }

        self.connected = true;
        tracing::info!(device = %self.device_name, "RFCOMM control channel established");
        Ok(())
    }
}

#[async_trait]
impl Transport for WinRtRfcommTransport {
    async fn send(&mut self, data: &[u8]) -> Result<(), TransportError> {
        if !self.connected {
            return Err(TransportError::NotConnected);
        }

        let guard = self
            .active_socket
            .lock()
            .map_err(|e| TransportError::Io(e.to_string()))?;

        if let Some(sock) = guard.as_ref() {
            sock.writer
                .WriteBytes(data)
                .map_err(|e| TransportError::Io(e.to_string()))?;
            sock.writer
                .StoreAsync()
                .map_err(|e| TransportError::Io(e.to_string()))?
                .get()
                .map_err(|e| TransportError::Io(e.to_string()))?;
            tracing::debug!(len = data.len(), "Sent MDR bytes via WinRT RFCOMM");
            Ok(())
        } else {
            Err(TransportError::NotConnected)
        }
    }

    async fn receive(&mut self, buf: &mut [u8]) -> Result<usize, TransportError> {
        if !self.connected {
            return Err(TransportError::NotConnected);
        }

        let guard = self
            .active_socket
            .lock()
            .map_err(|e| TransportError::Io(e.to_string()))?;

        if let Some(sock) = guard.as_ref() {
            let loaded = sock
                .reader
                .LoadAsync(buf.len() as u32)
                .map_err(|e| TransportError::Io(e.to_string()))?
                .get()
                .map_err(|e| TransportError::Io(e.to_string()))? as usize;

            if loaded == 0 {
                return Ok(0);
            }

            let to_read = loaded.min(buf.len());
            sock.reader
                .ReadBytes(&mut buf[..to_read])
                .map_err(|e| TransportError::Io(e.to_string()))?;
            Ok(to_read)
        } else {
            Err(TransportError::NotConnected)
        }
    }

    fn is_connected(&self) -> bool {
        self.connected
    }

    async fn disconnect(&mut self) -> Result<(), TransportError> {
        if let Ok(mut guard) = self.active_socket.lock() {
            *guard = None;
        }
        self.connected = false;
        Ok(())
    }
}
