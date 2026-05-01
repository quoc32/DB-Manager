use std::process::Command;
use serde::Serialize;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn get_running_dbs() -> Vec<String> {
    // Danh sách các tên tiến trình (process) đặc trưng của các DB
    let db_processes = vec![
        ("MySQL", "mysqld.exe"),
        ("PostgreSQL", "postgres.exe"),
        ("Redis", "redis-server.exe"),
        ("MongoDB", "mongod.exe"),
        ("SQL Server", "sqlservr.exe"),
    ];

    // Chạy lệnh tasklist để lấy danh sách tiến trình đang chạy trên Windows
    let output = Command::new("tasklist")
        .arg("/NH") // Loại bỏ tiêu đề (No Header)
        .arg("/FO") // Định dạng đầu ra
        .arg("CSV") // Xuất dạng CSV để dễ xử lý
        .output();

    let mut running_dbs = Vec::new();

    if let Ok(out) = output {
        let stdout = String::from_utf8_lossy(&out.stdout);
        
        for (db_name, process_name) in db_processes {
            if stdout.to_lowercase().contains(&process_name.to_lowercase()) {
                running_dbs.push(db_name.to_string());
            }
        }
    }

    running_dbs
}

#[derive(Serialize)]
struct DbStatus {
    name: String,
    status: String, // "Running" hoặc "Stopped" hoặc "Not Installed"
}

#[tauri::command]
fn get_all_db_status() -> Vec<DbStatus> {
    // Danh sách các Service Name có khả năng tồn tại
    let check_list = vec![
        ("MySQL", vec!["MySQL80", "MySQL57", "mysql"]),
        ("MongoDB", vec!["MongoDB"]),
        ("MSSQL", vec!["MSSQL$SQLEXPRESS", "MSSQL$MSSQLSERVER"]),
        ("PostgreSQL", vec!["postgresql-x64-18", "postgres"]),
    ];

    let mut results = Vec::new();

    for (display_name, service_names) in check_list {
        let mut final_status = "Not Installed".to_string();

        for s_name in service_names {
            let output = Command::new("sc")
                .arg("query")
                .arg(s_name)
                .output();

            if let Ok(out) = output {
                let stdout = String::from_utf8_lossy(&out.stdout);
                if stdout.contains("RUNNING") {
                    final_status = "Running".to_string();
                    break; 
                } else if stdout.contains("STOPPED") || stdout.contains("PAUSED") || stdout.contains("START_PENDING") {
                    final_status = "Stopped".to_string();
                    break;
                }
                // Nếu stdout chứa mã lỗi 1060, nó sẽ tiếp tục vòng lặp để thử tên khác
            }
        }

        results.push(DbStatus {
            name: display_name.to_string(),
            status: final_status,
        });
    }

    results
}

// Hàm hỗ trợ tìm tên service thực tế trong máy
fn find_actual_service_name(pattern: &str) -> Option<String> {
    let output = Command::new("powershell")
        .arg("-Command")
        .arg(format!("Get-Service -Name '{}' | Select-Object -ExpandProperty Name", pattern))
        .output()
        .ok()?;
    
    let name = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if name.is_empty() { None } else { Some(name) }
}

#[tauri::command]
fn control_service(name: String, action: String) -> Result<String, String> {
    let service_name = match name.as_str() {
        "MySQL" => find_actual_service_name("MySQL*").unwrap_or("MySQL80".to_string()),
        "SQL Server" => find_actual_service_name("MSSQL$*").unwrap_or("MSSQL$SQLEXPRESS".to_string()),
        "PostgreSQL" => find_actual_service_name("postgresql*").unwrap_or("postgresql-x64-18".to_string()),
        "MongoDB" => "MongoDB".to_string(),
        _ => return Err("Dịch vụ không xác định".into()),
    };

    // Chạy lệnh điều khiển với service_name vừa tìm được
    let output = Command::new("net")
        .arg(&action)
        .arg(&service_name)
        .output();

    match output {
        Ok(out) => {
            if out.status.success() {
                Ok(format!("Đã {} thành công {}", &action, &name))
            } else {
                let err = String::from_utf8_lossy(&out.stderr);
                Err(err.to_string())
            }
        },
        Err(_) => Err("Không thể thực thi lệnh hệ thống".into()),
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            get_all_db_status,
            control_service
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
