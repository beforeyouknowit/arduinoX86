use serialport::SerialPortInfo;

#[derive(Default)]
pub struct SerialManager {
    ports: Vec<SerialPortInfo>,
}

impl SerialManager {
    pub fn new() -> Self {
        SerialManager {
            ports: serialport::available_ports().unwrap_or_else(|_| vec![]),
        }
    }

    pub fn refresh(&mut self) {
        self.ports = serialport::available_ports().unwrap_or_else(|_| vec![]);
    }

    pub fn port_names(&self) -> Vec<String> {
        self.ports
            .iter()
            .map(|port| match &port.port_type {
                serialport::SerialPortType::UsbPort(info) => {
                    format!(
                        "{} ({})",
                        port.port_name,
                        info.product.as_deref().unwrap_or("Unknown USB Device")
                    )
                }
                serialport::SerialPortType::BluetoothPort => {
                    format!("{} (Bluetooth)", port.port_name)
                }
                _ => port.port_name.clone(),
            })
            .collect()
    }

    pub fn ports(&self) -> &[SerialPortInfo] {
        &self.ports
    }
}
