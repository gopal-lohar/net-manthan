use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, OptionalExtension, Row};
use std::path::{Path, PathBuf};
use uuid::Uuid;

/// Represents a download in the database
#[derive(Debug, Clone)]
pub struct Download {
    pub download_id: String,
    pub filename: String,
    pub path: String,
    pub referrer: Option<String>,
    pub download_link: String,
    pub resumable: bool,
    pub total_size: u64,
    pub size_downloaded: u64,
    pub average_speed: u64,
    pub date_added: DateTime<Utc>,
    pub date_finished: Option<DateTime<Utc>>,
    pub active_time: i64, // Stored as seconds
    pub parts: Vec<DownloadPart>,
}

/// Represents a part of a download in the database
#[derive(Debug, Clone)]
pub struct DownloadPart {
    pub download_id: String,
    pub part_id: String,
    pub start_bytes: u64,
    pub end_bytes: u64,
    pub total_bytes: u64,
    pub bytes_downloaded: u64,
}

// connecting to the database
pub fn connect_to_database(db_path: &PathBuf) -> Result<DatabaseManager> {
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent)
            .context("download file doesn't exist and cannot be created")?;
    }

    match DatabaseManager::new(db_path) {
        Ok(db_manager) => Ok(db_manager),
        Err(e) => Err(e),
    }
}

/// Manages database operations for the download manager
pub struct DatabaseManager {
    conn: Connection,
}

impl DatabaseManager {
    /// Creates a new DatabaseManager and initializes the database tables if they don't exist
    pub fn new<P: AsRef<Path>>(db_path: P) -> Result<Self> {
        let conn = Connection::open(db_path).context("Failed to open database connection")?;

        let manager = DatabaseManager { conn };
        manager.initialize_tables()?;

        Ok(manager)
    }

    /// Initializes the database tables if they don't exist
    fn initialize_tables(&self) -> Result<()> {
        self.conn
            .execute(
                "CREATE TABLE IF NOT EXISTS downloads (
                download_id TEXT PRIMARY KEY,
                filename TEXT NOT NULL,
                path TEXT NOT NULL,
                referrer TEXT,
                download_link TEXT NOT NULL,
                resumable BOOLEAN NOT NULL,
                total_size INTEGER NOT NULL,
                size_downloaded INTEGER NOT NULL,
                average_speed INTEGER NOT NULL,
                date_added TEXT NOT NULL,
                date_finished TEXT,
                active_time INTEGER NOT NULL
            )",
                [],
            )
            .context("Failed to create downloads table")?;

        self.conn
            .execute(
                "CREATE TABLE IF NOT EXISTS download_parts (
                download_id TEXT NOT NULL,
                part_id TEXT PRIMARY KEY,
                start_bytes INTEGER NOT NULL,
                end_bytes INTEGER NOT NULL,
                total_bytes INTEGER NOT NULL,
                bytes_downloaded INTEGER NOT NULL,
                FOREIGN KEY (download_id) REFERENCES downloads (download_id) ON DELETE CASCADE
            )",
                [],
            )
            .context("Failed to create download_parts table")?;

        // Create index for faster lookup of parts by download_id
        self.conn
            .execute(
                "CREATE INDEX IF NOT EXISTS idx_parts_download_id ON download_parts (download_id)",
                [],
            )
            .context("Failed to create index on download_parts")?;

        Ok(())
    }

    /// Inserts a new download into the database
    pub fn insert_download(&mut self, download: &mut Download) -> Result<()> {
        // Generate a UUID if not provided
        if download.download_id.is_empty() {
            // Fixed: Use Uuid::new_v4() correctly based on version
            download.download_id = Uuid::new_v4().to_string();
        }

        let tx = self
            .conn
            .transaction()
            .context("Failed to begin transaction")?;

        tx.execute(
            "INSERT INTO downloads (
                download_id, filename, path, referrer, download_link, resumable,
                total_size, size_downloaded, average_speed, date_added, date_finished, active_time
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            params![
                download.download_id,
                download.filename,
                download.path,
                download.referrer,
                download.download_link,
                download.resumable,
                download.total_size,
                download.size_downloaded,
                download.average_speed,
                download.date_added.to_rfc3339(),
                download.date_finished.map(|dt| dt.to_rfc3339()),
                download.active_time
            ],
        )
        .context("Failed to insert download")?;

        // Insert download parts
        for part in &mut download.parts {
            if part.part_id.is_empty() {
                // Fixed: Use Uuid::new_v4() correctly
                part.part_id = Uuid::new_v4().to_string();
            }
            part.download_id = download.download_id.clone();

            tx.execute(
                "INSERT INTO download_parts (
                    download_id, part_id, start_bytes, end_bytes, total_bytes, bytes_downloaded
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![
                    part.download_id,
                    part.part_id,
                    part.start_bytes,
                    part.end_bytes,
                    part.total_bytes,
                    part.bytes_downloaded
                ],
            )
            .context("Failed to insert download part")?;
        }

        tx.commit().context("Failed to commit transaction")?;
        Ok(())
    }

    /// Retrieves a download from the database by its ID, including its parts
    pub fn get_download(&self, download_id: &str) -> Result<Option<Download>> {
        let download = self
            .conn
            .query_row(
                "SELECT
                download_id, filename, path, referrer, download_link, resumable,
                total_size, size_downloaded, average_speed, date_added, date_finished, active_time
             FROM downloads WHERE download_id = ?1",
                [download_id],
                |row| self.row_to_download(row),
            )
            .optional()
            .context("Failed to query download")?;

        if let Some(mut download) = download {
            download.parts = self.get_download_parts(download_id)?;
            Ok(Some(download))
        } else {
            Ok(None)
        }
    }

    /// Retrieves all downloads from the database
    pub fn get_all_downloads(&self) -> Result<Vec<Download>> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT
                download_id, filename, path, referrer, download_link, resumable,
                total_size, size_downloaded, average_speed, date_added, date_finished, active_time
             FROM downloads",
            )
            .context("Failed to prepare statement")?;

        let download_iter = stmt
            .query_map([], |row| self.row_to_download(row))
            .context("Failed to query downloads")?;

        let mut downloads = Vec::new();
        for download_result in download_iter {
            let mut download = download_result.context("Failed to read download")?;
            download.parts = self.get_download_parts(&download.download_id)?;
            downloads.push(download);
        }

        Ok(downloads)
    }

    /// Retrieves all parts for a given download
    pub fn get_download_parts(&self, download_id: &str) -> Result<Vec<DownloadPart>> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT
                download_id, part_id, start_bytes, end_bytes, total_bytes, bytes_downloaded
             FROM download_parts WHERE download_id = ?1",
            )
            .context("Failed to prepare statement")?;

        let part_iter = stmt
            .query_map([download_id], |row| {
                Ok(DownloadPart {
                    download_id: row.get(0)?,
                    part_id: row.get(1)?,
                    start_bytes: row.get(2)?,
                    end_bytes: row.get(3)?,
                    total_bytes: row.get(4)?,
                    bytes_downloaded: row.get(5)?,
                })
            })
            .context("Failed to query download parts")?;

        let mut parts = Vec::new();
        for part_result in part_iter {
            parts.push(part_result.context("Failed to read download part")?);
        }

        Ok(parts)
    }

    /// Updates an existing download in the database
    pub fn update_download(&mut self, download: &Download) -> Result<()> {
        // Fixed: Changed signature to &mut self to allow mutable transaction
        let tx = self
            .conn
            .transaction()
            .context("Failed to begin transaction")?;

        tx.execute(
            "UPDATE downloads SET
                filename = ?2,
                path = ?3,
                referrer = ?4,
                download_link = ?5,
                resumable = ?6,
                total_size = ?7,
                size_downloaded = ?8,
                average_speed = ?9,
                date_added = ?10,
                date_finished = ?11,
                active_time = ?12
             WHERE download_id = ?1",
            params![
                download.download_id,
                download.filename,
                download.path,
                download.referrer,
                download.download_link,
                download.resumable,
                download.total_size,
                download.size_downloaded,
                download.average_speed,
                download.date_added.to_rfc3339(),
                download.date_finished.map(|dt| dt.to_rfc3339()),
                download.active_time
            ],
        )
        .context("Failed to update download")?;

        tx.commit().context("Failed to commit transaction")?;
        Ok(())
    }

    /// Updates a download part in the database
    pub fn update_download_part(&self, part: &DownloadPart) -> Result<()> {
        self.conn
            .execute(
                "UPDATE download_parts SET
                start_bytes = ?3,
                end_bytes = ?4,
                total_bytes = ?5,
                bytes_downloaded = ?6
             WHERE download_id = ?1 AND part_id = ?2",
                params![
                    part.download_id,
                    part.part_id,
                    part.start_bytes,
                    part.end_bytes,
                    part.total_bytes,
                    part.bytes_downloaded
                ],
            )
            .context("Failed to update download part")?;

        Ok(())
    }

    /// Deletes a download and all its parts from the database
    pub fn delete_download(&self, download_id: &str) -> Result<()> {
        // Due to the foreign key constraint with ON DELETE CASCADE,
        // deleting the download will automatically delete its parts
        self.conn
            .execute(
                "DELETE FROM downloads WHERE download_id = ?1",
                [download_id],
            )
            .context("Failed to delete download")?;

        Ok(())
    }

    /// Creates a new download part for an existing download
    pub fn create_download_part(&self, part: &mut DownloadPart) -> Result<()> {
        if part.part_id.is_empty() {
            // Fixed: Use Uuid::new_v4() correctly
            part.part_id = Uuid::new_v4().to_string();
        }

        self.conn
            .execute(
                "INSERT INTO download_parts (
                download_id, part_id, start_bytes, end_bytes, total_bytes, bytes_downloaded
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![
                    part.download_id,
                    part.part_id,
                    part.start_bytes,
                    part.end_bytes,
                    part.total_bytes,
                    part.bytes_downloaded
                ],
            )
            .context("Failed to insert download part")?;

        Ok(())
    }

    /// Gets a specific download part by its ID
    pub fn get_download_part(&self, part_id: &str) -> Result<Option<DownloadPart>> {
        self.conn
            .query_row(
                "SELECT
                download_id, part_id, start_bytes, end_bytes, total_bytes, bytes_downloaded
             FROM download_parts WHERE part_id = ?1",
                [part_id],
                |row| {
                    Ok(DownloadPart {
                        download_id: row.get(0)?,
                        part_id: row.get(1)?,
                        start_bytes: row.get(2)?,
                        end_bytes: row.get(3)?,
                        total_bytes: row.get(4)?,
                        bytes_downloaded: row.get(5)?,
                    })
                },
            )
            .optional()
            .context("Failed to query download part")
    }

    /// Deletes a specific download part
    pub fn delete_download_part(&self, part_id: &str) -> Result<()> {
        self.conn
            .execute("DELETE FROM download_parts WHERE part_id = ?1", [part_id])
            .context("Failed to delete download part")?;

        Ok(())
    }

    /// Helper method to convert a database row to a Download struct
    fn row_to_download(&self, row: &Row) -> rusqlite::Result<Download> {
        let date_added_str: String = row.get(9)?;
        let date_added = DateTime::parse_from_rfc3339(&date_added_str)
            .map(|dt| dt.with_timezone(&Utc))
            .map_err(|e| {
                rusqlite::Error::FromSqlConversionFailure(
                    9,
                    rusqlite::types::Type::Text,
                    Box::new(e),
                )
            })?;

        let date_finished_str: Option<String> = row.get(10)?;
        let date_finished = match date_finished_str {
            Some(dt_str) => Some(
                DateTime::parse_from_rfc3339(&dt_str)
                    .map(|dt| dt.with_timezone(&Utc))
                    .map_err(|e| {
                        rusqlite::Error::FromSqlConversionFailure(
                            10,
                            rusqlite::types::Type::Text,
                            Box::new(e),
                        )
                    })?,
            ),
            None => None,
        };

        Ok(Download {
            download_id: row.get(0)?,
            filename: row.get(1)?,
            path: row.get(2)?,
            referrer: row.get(3)?,
            download_link: row.get(4)?,
            resumable: row.get(5)?,
            total_size: row.get(6)?,
            size_downloaded: row.get(7)?,
            average_speed: row.get(8)?,
            date_added,
            date_finished,
            active_time: row.get(11)?,
            parts: Vec::new(), // Parts will be loaded separately
        })
    }

    /// Gets incomplete downloads for resuming after program restart
    pub fn get_incomplete_downloads(&self) -> Result<Vec<Download>> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT
                download_id, filename, path, referrer, download_link, resumable,
                total_size, size_downloaded, average_speed, date_added, date_finished, active_time
             FROM downloads WHERE size_downloaded < total_size AND date_finished IS NULL",
            )
            .context("Failed to prepare statement")?;

        let download_iter = stmt
            .query_map([], |row| self.row_to_download(row))
            .context("Failed to query incomplete downloads")?;

        let mut downloads = Vec::new();
        for download_result in download_iter {
            let mut download = download_result.context("Failed to read download")?;
            download.parts = self.get_download_parts(&download.download_id)?;
            downloads.push(download);
        }

        Ok(downloads)
    }

    /// Gets completed downloads within a date range
    pub fn get_completed_downloads(
        &self,
        from_date: &DateTime<Utc>,
        to_date: &DateTime<Utc>,
    ) -> Result<Vec<Download>> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT
                download_id, filename, path, referrer, download_link, resumable,
                total_size, size_downloaded, average_speed, date_added, date_finished, active_time
             FROM downloads
             WHERE date_finished IS NOT NULL
               AND date_finished BETWEEN ?1 AND ?2",
            )
            .context("Failed to prepare statement")?;

        let download_iter = stmt
            .query_map(
                params![from_date.to_rfc3339(), to_date.to_rfc3339()],
                |row| self.row_to_download(row),
            )
            .context("Failed to query completed downloads")?;

        let mut downloads = Vec::new();
        for download_result in download_iter {
            let mut download = download_result.context("Failed to read download")?;
            download.parts = self.get_download_parts(&download.download_id)?;
            downloads.push(download);
        }

        Ok(downloads)
    }

    /// Updates the progress of a download part
    pub fn update_part_progress(&self, part_id: &str, bytes_downloaded: u64) -> Result<()> {
        self.conn
            .execute(
                "UPDATE download_parts SET bytes_downloaded = ?2 WHERE part_id = ?1",
                params![part_id, bytes_downloaded],
            )
            .context("Failed to update part progress")?;

        Ok(())
    }

    /// Calculates and updates the total progress of a download based on its parts
    pub fn update_download_progress(&self, download_id: &str) -> Result<u64> {
        let parts = self.get_download_parts(download_id)?;
        let total_downloaded: u64 = parts.iter().map(|p| p.bytes_downloaded).sum();

        self.conn
            .execute(
                "UPDATE downloads SET size_downloaded = ?2 WHERE download_id = ?1",
                params![download_id, total_downloaded],
            )
            .context("Failed to update download progress")?;

        Ok(total_downloaded)
    }

    /// Marks a download as complete
    pub fn mark_download_complete(&self, download_id: &str) -> Result<()> {
        let now = Utc::now();

        self.conn
            .execute(
                "UPDATE downloads SET
                date_finished = ?2,
                size_downloaded = total_size
             WHERE download_id = ?1",
                params![download_id, now.to_rfc3339()],
            )
            .context("Failed to mark download as complete")?;

        Ok(())
    }

    /// Updates the active time of a download
    pub fn update_active_time(&self, download_id: &str, additional_seconds: i64) -> Result<()> {
        self.conn
            .execute(
                "UPDATE downloads SET active_time = active_time + ?2 WHERE download_id = ?1",
                params![download_id, additional_seconds],
            )
            .context("Failed to update active time")?;

        Ok(())
    }

    /// Gets download statistics
    pub fn get_download_stats(&self) -> Result<DownloadStats> {
        let total_downloads: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM downloads", [], |row| row.get(0))
            .context("Failed to count downloads")?;

        let completed_downloads: i64 = self
            .conn
            .query_row(
                "SELECT COUNT(*) FROM downloads WHERE size_downloaded >= total_size",
                [],
                |row| row.get(0),
            )
            .context("Failed to count completed downloads")?;

        let active_downloads: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM downloads WHERE size_downloaded < total_size AND date_finished IS NULL",
            [],
            |row| row.get(0),
        ).context("Failed to count active downloads")?;

        // Fixed: Properly handle NULL result for SUM
        let total_downloaded_bytes: u64 = self
            .conn
            .query_row(
                "SELECT COALESCE(SUM(size_downloaded), 0) FROM downloads",
                [],
                |row| row.get(0),
            )
            .context("Failed to sum downloaded bytes")?;

        Ok(DownloadStats {
            total_downloads: total_downloads as u64,
            completed_downloads: completed_downloads as u64,
            active_downloads: active_downloads as u64,
            total_downloaded_bytes,
        })
    }
}

/// Statistics about downloads in the database
#[derive(Debug, Clone)]
pub struct DownloadStats {
    pub total_downloads: u64,
    pub completed_downloads: u64,
    pub active_downloads: u64,
    pub total_downloaded_bytes: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn create_test_download() -> Download {
        Download {
            download_id: String::new(), // Will be auto-generated
            filename: "test-file.zip".to_string(),
            path: "/downloads".to_string(),
            referrer: Some("https://example.com".to_string()),
            download_link: "https://example.com/files/test-file.zip".to_string(),
            resumable: true,
            total_size: 1000000,
            size_downloaded: 0,
            average_speed: 0,
            date_added: Utc::now(),
            date_finished: None,
            active_time: 0,
            parts: vec![
                DownloadPart {
                    download_id: String::new(), // Will be filled by insert_download
                    part_id: String::new(),     // Will be auto-generated
                    start_bytes: 0,
                    end_bytes: 499999,
                    total_bytes: 500000,
                    bytes_downloaded: 0,
                },
                DownloadPart {
                    download_id: String::new(), // Will be filled by insert_download
                    part_id: String::new(),     // Will be auto-generated
                    start_bytes: 500000,
                    end_bytes: 999999,
                    total_bytes: 500000,
                    bytes_downloaded: 0,
                },
            ],
        }
    }

    #[test]
    fn test_database_operations() -> Result<()> {
        let dir = tempdir()?;
        let db_path = dir.path().join("test.db");

        // Create database manager
        let mut db_manager = DatabaseManager::new(&db_path)?;

        // Test insert
        let mut download = create_test_download();
        db_manager.insert_download(&mut download)?;
        assert!(!download.download_id.is_empty());

        // Test get
        let retrieved = db_manager.get_download(&download.download_id)?.unwrap();
        assert_eq!(retrieved.filename, download.filename);
        assert_eq!(retrieved.parts.len(), 2);

        // Test update
        let mut updated = retrieved.clone();
        updated.size_downloaded = 500000;
        updated.average_speed = 1024;
        db_manager.update_download(&updated)?;

        // Test part update
        let mut part = updated.parts[0].clone();
        part.bytes_downloaded = 250000;
        db_manager.update_download_part(&part)?;

        // Test progress update
        db_manager.update_download_progress(&download.download_id)?;
        let after_progress = db_manager.get_download(&download.download_id)?.unwrap();
        assert_eq!(after_progress.size_downloaded, 250000);

        // Test get all downloads
        let all_downloads = db_manager.get_all_downloads()?;
        assert_eq!(all_downloads.len(), 1);

        // Test mark as complete
        db_manager.mark_download_complete(&download.download_id)?;
        let completed = db_manager.get_download(&download.download_id)?.unwrap();
        assert!(completed.date_finished.is_some());
        assert_eq!(completed.size_downloaded, completed.total_size);

        // Test stats
        let stats = db_manager.get_download_stats()?;
        assert_eq!(stats.total_downloads, 1);
        assert_eq!(stats.completed_downloads, 1);

        // Test delete
        db_manager.delete_download(&download.download_id)?;
        let after_delete = db_manager.get_download(&download.download_id)?;
        assert!(after_delete.is_none());

        Ok(())
    }
}
