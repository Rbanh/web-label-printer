use btleplug::api::{Central, Manager as _, Peripheral, ScanFilter, WriteType};
use btleplug::platform::Manager;
use std::time::Duration;
use tokio::time;
use uuid::Uuid;

const WRITE_UUID: Uuid = Uuid::from_u128(0x0000ff02_0000_1000_8000_00805f9b34fb);
const SERVICE_UUID: Uuid = Uuid::from_u128(0x0000ff00_0000_1000_8000_00805f9b34fb);
const PRINTER_MAC: &str = "B9:73:BC:71:FA:43";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("--- STANDALONE BLE DEBUGGER ---");
    let manager = Manager::new().await?;
    let adapters = manager.adapters().await?;
    let central = adapters.into_iter().next().ok_or("No adapter")?;

    println!("Starting targeted scan to identify BLE type...");
    let filter = ScanFilter { services: vec![SERVICE_UUID] };
    central.start_scan(filter).await?;
    time::sleep(Duration::from_secs(3)).await;

    let peripherals = central.peripherals().await?;
    let p = peripherals.iter().find(|p| p.address().to_string() == PRINTER_MAC)
        .ok_or("Printer not found after scan")?;

    println!("Attempting BLE Connection...");
    p.connect().await?;
    
    println!("SUCCESS! Handshake established.");
    p.discover_services().await?;
    
    let chars = p.characteristics();
    let cmd_char = chars.iter().find(|c| c.uuid == WRITE_UUID).ok_or("No FF02")?;

    println!("Sending test line feed...");
    p.write(cmd_char, &[0x1d, 0x0c], WriteType::WithResponse).await?;
    println!("FEED COMMAND ACKNOWLEDGED BY PRINTER!");

    p.disconnect().await?;
    Ok(())
}
