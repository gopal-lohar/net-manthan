use anyhow::Result;
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::str::FromStr;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DownloadStatus {
    Pending,
    Downloading,
    Paused,
    Completed,
    Failed,
}

impl From<DownloadStatus> for String {
    fn from(status: DownloadStatus) -> Self {
        match status {
            DownloadStatus::Pending => "pending".to_string(),
            DownloadStatus::Downloading => "downloading".to_string(),
            DownloadStatus::Paused => "paused".to_string(),
            DownloadStatus::Completed => "completed".to_string(),
            DownloadStatus::Failed => "failed".to_string(),
        }
    }
}

impl FromStr for DownloadStatus {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "pending" => Ok(DownloadStatus::Pending),
            "downloading" => Ok(DownloadStatus::Downloading),
            "paused" => Ok(DownloadStatus::Paused),
            "completed" => Ok(DownloadStatus::Completed),
            "failed" => Ok(DownloadStatus::Failed),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Download {
    pub id: Option<i64>,
    pub url: String,
    pub filename: String,
    pub mime_type: Option<String>,
    pub total_size: u64,
    pub status: DownloadStatus,
}

#[derive(Debug, Clone)]
pub struct DownloadPart {
    pub id: Option<i64>,
    pub download_id: i64,
    pub url: String,
    pub start_range: u64,
    pub end_range: u64,
    pub completed: bool,
}

pub struct DatabaseManager {
    conn: Connection,
}

impl DatabaseManager {
    pub fn new(db_path: &Path) -> Result<Self> {
        let conn = Connection::open(db_path)?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS downloads (
                id INTEGER PRIMARY KEY,
                url TEXT NOT NULL,
                filename TEXT NOT NULL,
                mime_type TEXT,
                total_size INTEGER,
                status TEXT
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS download_parts (
                id INTEGER PRIMARY KEY,
                download_id INTEGER NOT NULL,
                url TEXT NOT NULL,
                start_range INTEGER,
                end_range INTEGER,
                completed BOOLEAN,
                FOREIGN KEY(download_id) REFERENCES downloads(id)
            )",
            [],
        )?;

        Ok(DatabaseManager { conn })
    }

    pub fn insert_download(&self, download: &Download) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO downloads (url, filename, mime_type, total_size, status) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                download.url,
                download.filename,
                download.mime_type,
                download.total_size,
                String::from(download.status.clone())
            ]
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn insert_download_part(&self, part: &DownloadPart) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO download_parts (download_id, url, start_range, end_range, completed) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                part.download_id,
                part.url,
                part.start_range,
                part.end_range,
                part.completed
            ]
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn update_download_status(&self, download_id: i64, status: DownloadStatus) -> Result<()> {
        self.conn.execute(
            "UPDATE downloads SET status = ?1 WHERE id = ?2",
            params![String::from(status), download_id],
        )?;
        Ok(())
    }

    pub fn update_download_part(&self, part_id: i64, completed: bool) -> Result<()> {
        self.conn.execute(
            "UPDATE download_parts SET completed = ?1 WHERE id = ?2",
            params![completed, part_id],
        )?;
        Ok(())
    }

    pub fn get_download_by_id(&self, download_id: i64) -> Result<Option<Download>> {
        let download = self.conn.query_row(
            "SELECT id, url, filename, mime_type, total_size, status FROM downloads WHERE id = ?1",
            params![download_id],
            |row| {
                Ok(Download {
                    id: Some(row.get(0)?),
                    url: row.get(1)?,
                    filename: row.get(2)?,
                    mime_type: row.get(3)?,
                    total_size: row.get(4)?,
                    status: DownloadStatus::from_str(&row.get::<_, String>(5)?).unwrap_or(DownloadStatus::Pending),
                })
            }
        ).optional()?;

        Ok(download)
    }

    pub fn get_download_parts(&self, download_id: i64) -> Result<Vec<DownloadPart>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, download_id, url, start_range, end_range, completed FROM download_parts WHERE download_id = ?1"
        )?;

        let parts = stmt
            .query_map(params![download_id], |row| {
                Ok(DownloadPart {
                    id: Some(row.get(0)?),
                    download_id: row.get(1)?,
                    url: row.get(2)?,
                    start_range: row.get(3)?,
                    end_range: row.get(4)?,
                    completed: row.get(5)?,
                })
            })?
            .collect::<Result<Vec<DownloadPart>, _>>()?;

        Ok(parts)
    }

    pub fn get_all_downloads(&self) -> Result<Vec<Download>> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, url, filename, mime_type, total_size, status FROM downloads")?;

        let downloads = stmt
            .query_map([], |row| {
                Ok(Download {
                    id: Some(row.get(0)?),
                    url: row.get(1)?,
                    filename: row.get(2)?,
                    mime_type: row.get(3)?,
                    total_size: row.get(4)?,
                    status: DownloadStatus::from_str(&row.get::<_, String>(5)?)
                        .unwrap_or(DownloadStatus::Pending),
                })
            })?
            .collect::<Result<Vec<Download>, _>>()?;

        Ok(downloads)
    }
}

// Optional: Add tests to validate database operations
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_download_crud_operations() -> Result<()> {
        let dir = tempdir()?;
        let db_path = dir.path().join("downloads.db");

        let db_manager = DatabaseManager::new(&db_path)?;

        // Test inserting a download
        let download = Download {
            id: None,
            url: "http://example.com/file.zip".to_string(),
            filename: "file.zip".to_string(),
            mime_type: Some("application/zip".to_string()),
            total_size: 1024,
            status: DownloadStatus::Pending,
        };

        let download_id = db_manager.insert_download(&download)?;

        // Retrieve and verify the download
        let retrieved_download = db_manager.get_download_by_id(download_id)?;
        assert!(retrieved_download.is_some());

        // Test updating download status
        db_manager.update_download_status(download_id, DownloadStatus::Downloading)?;

        Ok(())
    }
}
