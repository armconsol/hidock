// Device management commands

use crate::commands::AppState;
use crate::db::types::{Device, DeviceStatus};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BindDeviceRequest {
    pub id: String,
    pub name: String,
}

/// List all devices
#[tauri::command]
pub async fn list_devices(state: State<'_, AppState>) -> Result<Vec<Device>, String> {
    let db = state
        .db
        .lock()
        .map_err(|e| format!("Database lock error: {}", e))?;

    db.list_devices()
        .map_err(|e| format!("Failed to list devices: {}", e))
}

/// Get a device by ID
#[tauri::command]
pub async fn get_device(
    device_id: String,
    state: State<'_, AppState>,
) -> Result<Option<Device>, String> {
    let db = state
        .db
        .lock()
        .map_err(|e| format!("Database lock error: {}", e))?;

    db.get_device(&device_id)
        .map_err(|e| format!("Failed to get device: {}", e))
}

/// Bind a new device
#[tauri::command]
pub async fn bind_device(
    request: BindDeviceRequest,
    state: State<'_, AppState>,
) -> Result<Device, String> {
    let db = state
        .db
        .lock()
        .map_err(|e| format!("Database lock error: {}", e))?;

    // Check if device already exists
    if let Some(_existing) = db
        .get_device(&request.id)
        .map_err(|e| format!("Failed to check device existence: {}", e))?
    {
        return Err(format!("Device with ID {} is already bound", request.id));
    }

    let now = Utc::now();
    let device = Device {
        id: request.id,
        name: request.name,
        status: DeviceStatus::Disconnected,
        last_sync: None,
        created_at: now,
    };

    db.insert_device(&device)
        .map_err(|e| format!("Failed to bind device: {}", e))
}

/// Unbind a device
#[tauri::command]
pub async fn unbind_device(device_id: String, state: State<'_, AppState>) -> Result<(), String> {
    let db = state
        .db
        .lock()
        .map_err(|e| format!("Database lock error: {}", e))?;

    db.delete_device(&device_id)
        .map_err(|e| format!("Failed to unbind device: {}", e))
}

/// Update device status
#[tauri::command]
pub async fn update_device_status(
    device_id: String,
    status: String,
    state: State<'_, AppState>,
) -> Result<Device, String> {
    let db = state
        .db
        .lock()
        .map_err(|e| format!("Database lock error: {}", e))?;

    let device_status =
        DeviceStatus::from_str(&status).map_err(|e| format!("Invalid device status: {}", e))?;

    db.update_device_status(&device_id, device_status)
        .map_err(|e| format!("Failed to update device status: {}", e))
}

/// Update device last sync time
#[tauri::command]
pub async fn update_device_last_sync(
    device_id: String,
    state: State<'_, AppState>,
) -> Result<Device, String> {
    let db = state
        .db
        .lock()
        .map_err(|e| format!("Database lock error: {}", e))?;

    db.update_device_last_sync(&device_id)
        .map_err(|e| format!("Failed to update device last sync: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Database;

    #[test]
    fn test_device_operations() {
        let db = Database::new_in_memory().expect("Failed to create test database");

        // Bind a device
        let device = Device {
            id: "HIDOC-001".to_string(),
            name: "My HiDoc P1".to_string(),
            status: DeviceStatus::Disconnected,
            last_sync: None,
            created_at: Utc::now(),
        };

        db.insert_device(&device).unwrap();

        // Get device
        let retrieved = db.get_device("HIDOC-001").unwrap().unwrap();
        assert_eq!(retrieved.id, "HIDOC-001");
        assert_eq!(retrieved.name, "My HiDoc P1");
        assert_eq!(retrieved.status, DeviceStatus::Disconnected);

        // List devices
        let devices = db.list_devices().unwrap();
        assert_eq!(devices.len(), 1);
        assert_eq!(devices[0].id, "HIDOC-001");
    }

    #[test]
    fn test_bind_duplicate_device() {
        let db = Database::new_in_memory().expect("Failed to create test database");

        let device = Device {
            id: "HIDOC-001".to_string(),
            name: "My HiDoc P1".to_string(),
            status: DeviceStatus::Disconnected,
            last_sync: None,
            created_at: Utc::now(),
        };

        // First bind should succeed
        db.insert_device(&device).unwrap();

        // Second bind should fail
        let result = db.insert_device(&device);
        assert!(result.is_err());
    }

    #[test]
    fn test_unbind_device() {
        let db = Database::new_in_memory().expect("Failed to create test database");

        // Bind a device
        let device = Device {
            id: "HIDOC-001".to_string(),
            name: "My HiDoc P1".to_string(),
            status: DeviceStatus::Disconnected,
            last_sync: None,
            created_at: Utc::now(),
        };
        db.insert_device(&device).unwrap();

        // Unbind the device
        db.delete_device("HIDOC-001").unwrap();

        // Verify device is gone
        let devices = db.list_devices().unwrap();
        assert_eq!(devices.len(), 0);
    }

    #[test]
    fn test_update_device_status() {
        let db = Database::new_in_memory().expect("Failed to create test database");

        // Bind a device
        let device = Device {
            id: "HIDOC-001".to_string(),
            name: "My HiDoc P1".to_string(),
            status: DeviceStatus::Disconnected,
            last_sync: None,
            created_at: Utc::now(),
        };
        db.insert_device(&device).unwrap();

        // Update status to connected
        db.update_device_status("HIDOC-001", DeviceStatus::Connected)
            .unwrap();

        let updated = db.get_device("HIDOC-001").unwrap().unwrap();
        assert_eq!(updated.status, DeviceStatus::Connected);

        // Update status to disconnected
        db.update_device_status("HIDOC-001", DeviceStatus::Disconnected)
            .unwrap();

        let updated = db.get_device("HIDOC-001").unwrap().unwrap();
        assert_eq!(updated.status, DeviceStatus::Disconnected);
    }

    #[test]
    fn test_update_device_last_sync() {
        let db = Database::new_in_memory().expect("Failed to create test database");

        // Bind a device
        let device = Device {
            id: "HIDOC-001".to_string(),
            name: "My HiDoc P1".to_string(),
            status: DeviceStatus::Disconnected,
            last_sync: None,
            created_at: Utc::now(),
        };
        db.insert_device(&device).unwrap();
        assert!(device.last_sync.is_none());

        // Update last sync
        db.update_device_last_sync("HIDOC-001").unwrap();

        let updated = db.get_device("HIDOC-001").unwrap().unwrap();
        assert!(updated.last_sync.is_some());
    }
}
