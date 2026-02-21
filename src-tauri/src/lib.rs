use btleplug::api::{Central, Manager as _, Peripheral, WriteType};
use btleplug::platform::Manager;
use std::time::Duration;
use tokio::time;
use uuid::Uuid;

const WRITE_UUID: Uuid = Uuid::from_u128(0x0000ff02_0000_1000_8000_00805f9b34fb);
const PRINTER_MAC: &str = "B9:73:BC:71:FA:43";

#[tauri::command]
async fn test_autopilot() -> Result<String, String> {
    println!("[AUTOPILOT] Starting systematic hardware test...");
    
    let manager = Manager::new().await.map_err(|e| e.to_string())?;
    let adapters = manager.adapters().await.map_err(|e| e.to_string())?;
    let central = adapters.into_iter().next().ok_or("No adapter")?;

    // 1. Connection attempt
    let peripherals = central.peripherals().await.map_err(|e| e.to_string())?;
    let p = peripherals.iter().find(|p| p.address().to_string() == PRINTER_MAC)
        .ok_or("Printer not in cache. Ensure it's ON.")?;

    if !p.is_connected().await.unwrap_or(false) {
        p.connect().await.map_err(|e| e.to_string())?;
        time::sleep(Duration::from_secs(2)).await;
    }
    
    p.discover_services().await.map_err(|e| e.to_string())?;
    let cmd_char = p.characteristics().into_iter().find(|c| c.uuid == WRITE_UUID)
        .ok_or("Write char not found")?;

    // 2. Protocol Variations
    let variations = vec![
        ("Standard ESC/POS", vec![0x1b, 0x40, 0x1b, 0x64, 0x02]),
        ("Phomemo Magic", vec![0x1f, 0x11, 0x02, 0x04, 0x1d, 0x0c]),
        ("Simple Feed", vec![0x1d, 0x0c]),
    ];

    for (name, data) in variations {
        println!("[AUTOPILOT] Trying variation: {}", name);
        match p.write(&cmd_char, &data, WriteType::WithResponse).await {
            Ok(_) => return Ok(format!("SUCCESS! Printer accepted: {}", name)),
            Err(e) => println!("[AUTOPILOT] {} failed: {}", name, e),
        }
        time::sleep(Duration::from_millis(500)).await;
    }

    Err("All protocol variations failed to receive acknowledgement".to_string())
}

#[tauri::command]
async fn print_native(data: Vec<u8>) -> Result<String, String> {
    println!("[Print] Starting native print job. Total bytes: {}", data.len());
    
    let manager = Manager::new().await.map_err(|e| e.to_string())?;
    let adapters = manager.adapters().await.map_err(|e| e.to_string())?;
    let central = adapters.into_iter().next().ok_or("No Bluetooth adapter found")?;
    
    let peripherals = central.peripherals().await.map_err(|e| e.to_string())?;
    let p = peripherals.iter().find(|p| p.address().to_string() == PRINTER_MAC)
        .ok_or("Printer not found. Ensure it's ON and in range.")?;
    
    println!("[Print] Found printer: {}", p.address());
    
    if !p.is_connected().await.unwrap_or(false) {
        println!("[Print] Connecting...");
        p.connect().await.map_err(|e| e.to_string())?;
        time::sleep(Duration::from_millis(500)).await;
    }
    
    println!("[Print] Discovering services...");
    p.discover_services().await.map_err(|e| e.to_string())?;
    time::sleep(Duration::from_millis(100)).await;
    
    let cmd_char = p.characteristics().into_iter().find(|c| c.uuid == WRITE_UUID)
        .ok_or("Write characteristic not found")?;
    
    println!("[Print] Streaming {} bytes...", data.len());
    
    let header: Vec<u8> = data.iter().take(8).cloned().collect();
    let footer: Vec<u8> = data.iter().rev().take(3).cloned().collect::<Vec<_>>().into_iter().rev().collect();
    let pixel_data: Vec<u8> = data.iter().skip(8).take(data.len() - 11).cloned().collect();
    
    println!("[Print] Header: {:02x?}", header);
    p.write(&cmd_char, &header, WriteType::WithResponse).await
        .map_err(|e| format!("Header write failed: {}", e))?;
    time::sleep(Duration::from_millis(50)).await;
    
    println!("[Print] Pixel data: {} bytes", pixel_data.len());
    let chunk_size = 128;
    for (i, chunk) in pixel_data.chunks(chunk_size).enumerate() {
        p.write(&cmd_char, chunk, WriteType::WithoutResponse).await
            .map_err(|e| format!("Pixel chunk {} write failed: {}", i, e))?;
        if i % 10 == 0 {
            time::sleep(Duration::from_millis(5)).await;
        }
    }
    
    println!("[Print] Footer: {:02x?}", footer);
    p.write(&cmd_char, &footer, WriteType::WithResponse).await
        .map_err(|e| format!("Footer write failed: {}", e))?;
    
    println!("[Print] Done!");
    Ok("Printed successfully".to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  tauri::Builder::default()
    .invoke_handler(tauri::generate_handler![print_native, test_autopilot])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
