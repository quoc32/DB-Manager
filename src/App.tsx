import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";

interface DbStatus {
  name: string;
  status: string;
}

function App() {
  const [dbList, setDbList] = useState<DbStatus[]>([]);
  const [loading, setLoading] = useState<string | null>(null); // Lưu tên DB đang được xử lý

  const refreshStatus = async () => {
    try {
      const list = await invoke<DbStatus[]>("get_all_db_status");
      setDbList(list);
    } catch (err) {
      console.error("Failed to fetch status:", err);
    }
  };

  const handleAction = async (name: string, currentStatus: string) => {
    const action = currentStatus === "Running" ? "stop" : "start";
    setLoading(name); // Hiển thị trạng thái đang xử lý cho DB này
    
    try {
      // Gọi hàm control_service bạn vừa viết bên Rust
      await invoke("control_service", { name, action });
      // Sau khi lệnh chạy xong, đợi 1 chút rồi refresh lại trạng thái
      setTimeout(refreshStatus, 1500); 
    } catch (err) {
      alert(`Lỗi: ${err}`);
    } finally {
      setLoading(null);
    }
  };

  useEffect(() => {
    refreshStatus();
  }, []);

  return (
    <main className="container">
      <div className="header">
        <h2>Database Control Center</h2>
        <button className="refresh-btn" onClick={refreshStatus}>
          🔄 Làm mới
        </button>
      </div>

      <div className="db-grid">
        {dbList.map((db) => (
          <div key={db.name} className="db-card">
            <div className="db-info">
              <div className={`status-indicator ${db.status.toLowerCase()}`}></div>
              <strong>{db.name}</strong>
            </div>
            
            <div className="db-actions">
              <span className={`status-text ${db.status.toLowerCase()}`}>
                {db.status}
              </span>
              
              {db.status !== "Not Installed" && (
                <button 
                  className={`action-btn ${db.status === "Running" ? "stop" : "start"}`}
                  onClick={() => handleAction(db.name, db.status)}
                  disabled={loading === db.name}
                >
                  {loading === db.name ? "..." : (db.status === "Running" ? "Stop" : "Start")}
                </button>
              )}
            </div>
          </div>
        ))}
        {dbList.length === 0 && <p>Đang quét hệ thống...</p>}
      </div>
    </main>
  );
}

export default App;