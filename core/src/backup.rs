//! Backup and restore functionality for settings and statistics

use serde::{Deserialize, Serialize};
use std::time::SystemTime;

/// Backup data structure
#[derive(Debug, Serialize, Deserialize)]
pub struct BackupData {
    /// Backup version for compatibility
    pub version: u32,
    /// Timestamp when backup was created
    pub created_at: SystemTime,
    /// App configuration
    pub config: crate::Config,
    /// Custom filter rules
    pub custom_rules: Vec<String>,
    /// Statistics snapshot
    pub statistics: StatisticsBackup,
}

/// Statistics data for backup
#[derive(Debug, Serialize, Deserialize)]
pub struct StatisticsBackup {
    pub blocked_count: u64,
    pub allowed_count: u64,
    pub data_saved: u64,
    pub top_domains: Vec<DomainBackup>,
}

/// Domain statistics for backup
#[derive(Debug, Serialize, Deserialize)]
pub struct DomainBackup {
    pub domain: String,
    pub count: u64,
    pub data_saved: u64,
}

impl BackupData {
    /// Current backup format version
    pub const CURRENT_VERSION: u32 = 1;

    /// Create a new backup
    pub fn create(
        config: crate::Config,
        custom_rules: Vec<String>,
        statistics: &crate::Statistics,
    ) -> Self {
        BackupData {
            version: Self::CURRENT_VERSION,
            created_at: SystemTime::now(),
            config,
            custom_rules,
            statistics: StatisticsBackup {
                blocked_count: statistics.get_blocked_count(),
                allowed_count: statistics.get_allowed_count(),
                data_saved: statistics.get_data_saved(),
                top_domains: statistics
                    .top_blocked_domains(50)
                    .into_iter()
                    .map(|stats| DomainBackup {
                        domain: stats.domain,
                        count: stats.count,
                        data_saved: stats.data_saved,
                    })
                    .collect(),
            },
        }
    }

    /// Export backup to JSON string
    pub fn to_json(&self) -> Result<String, Box<dyn std::error::Error>> {
        Ok(serde_json::to_string_pretty(self)?)
    }

    /// Import backup from JSON string
    pub fn from_json(json: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let backup: BackupData = serde_json::from_str(json)?;

        // Validate version compatibility
        if backup.version > Self::CURRENT_VERSION {
            return Err("Backup version is too new".into());
        }

        Ok(backup)
    }

    /// Validate backup data
    pub fn validate(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Check version
        if self.version == 0 || self.version > Self::CURRENT_VERSION {
            return Err("Invalid backup version".into());
        }

        // Check timestamp is not in future
        if let Ok(duration) = self.created_at.duration_since(SystemTime::now()) {
            if duration.as_secs() > 0 {
                return Err("Backup timestamp is in the future".into());
            }
        }

        // Validate config
        if self.config.max_memory_mb == 0 {
            return Err("Invalid configuration in backup".into());
        }

        Ok(())
    }
}

/// Backup manager for handling backup operations
pub struct BackupManager {
    backup_dir: Option<std::path::PathBuf>,
}

impl BackupManager {
    /// Create a new backup manager
    pub fn new(backup_dir: Option<std::path::PathBuf>) -> Self {
        BackupManager { backup_dir }
    }

    /// Save backup to file
    pub fn save_backup(
        &self,
        backup: &BackupData,
        filename: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let backup_dir = self
            .backup_dir
            .as_ref()
            .ok_or("No backup directory configured")?;

        std::fs::create_dir_all(backup_dir)?;

        let backup_path = backup_dir.join(filename);
        let json = backup.to_json()?;
        std::fs::write(backup_path, json)?;

        Ok(())
    }

    /// Load backup from file
    pub fn load_backup(&self, filename: &str) -> Result<BackupData, Box<dyn std::error::Error>> {
        let backup_dir = self
            .backup_dir
            .as_ref()
            .ok_or("No backup directory configured")?;

        let backup_path = backup_dir.join(filename);
        let json = std::fs::read_to_string(backup_path)?;
        let backup = BackupData::from_json(&json)?;

        backup.validate()?;

        Ok(backup)
    }

    /// List available backups
    pub fn list_backups(&self) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let backup_dir = self
            .backup_dir
            .as_ref()
            .ok_or("No backup directory configured")?;

        let mut backups = Vec::new();

        if backup_dir.exists() {
            for entry in std::fs::read_dir(backup_dir)? {
                let entry = entry?;
                let path = entry.path();

                if path.extension().and_then(|s| s.to_str()) == Some("json") {
                    if let Some(filename) = path.file_name().and_then(|s| s.to_str()) {
                        backups.push(filename.to_string());
                    }
                }
            }
        }

        // Sort by filename (which should include timestamp)
        backups.sort();
        backups.reverse(); // Most recent first

        Ok(backups)
    }

    /// Create automatic backup with timestamp
    pub fn create_auto_backup(
        &self,
        config: crate::Config,
        custom_rules: Vec<String>,
        statistics: &crate::Statistics,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let backup = BackupData::create(config, custom_rules, statistics);

        // Generate filename with timestamp
        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
        let filename = format!("adblock_backup_{}.json", timestamp);

        self.save_backup(&backup, &filename)?;

        Ok(filename)
    }
}
