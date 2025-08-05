use serde::{Deserialize, Serialize};
use validator::Validate;
use chrono::Datelike;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct BackupMetadata {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub backup_type: String,
    pub status: String,
    pub file_path: Option<String>,
    pub file_size: Option<i64>,
    pub created_by_id: i32,
    pub created_by: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub error_message: Option<String>,
    pub compressed: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct BackupSchedule {
    pub id: i32,
    pub name: String,
    pub frequency: String,
    pub schedule_time: chrono::NaiveTime,
    pub schedule_day: Option<i32>,
    pub schedule_date: Option<i32>,
    pub retention_count: i32,
    pub enabled: bool,
    pub last_run_at: Option<chrono::DateTime<chrono::Utc>>,
    pub next_run_at: Option<chrono::DateTime<chrono::Utc>>,
    pub last_backup_id: Option<i32>,
    pub created_by_id: i32,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateBackupRequest {
    #[validate(length(min = 1, max = 255, message = "Name must be between 1 and 255 characters"))]
    pub name: Option<String>,
    #[validate(length(max = 1000, message = "Description must be less than 1000 characters"))]
    pub description: Option<String>,
    #[validate(custom(function = "validate_backup_type"))]
    pub backup_type: Option<String>,
    pub compress: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct RestoreBackupRequest {
    #[validate(range(min = 1, message = "Backup ID is required"))]
    pub backup_id: i32,
    pub drop_existing_tables: Option<bool>,
    pub create_pre_restore_backup: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreateScheduleRequest {
    #[validate(length(min = 1, max = 255, message = "Name is required"))]
    pub name: String,
    #[validate(custom(function = "validate_frequency"))]
    pub frequency: String,
    pub schedule_time: String, // HH:MM format
    pub schedule_day: Option<i32>, // 0-6 for weekly
    pub schedule_date: Option<i32>, // 1-31 for monthly
    #[validate(range(min = 1, max = 100, message = "Retention count must be between 1 and 100"))]
    pub retention_count: Option<i32>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct UpdateScheduleRequest {
    #[validate(range(min = 1, message = "Schedule ID is required"))]
    pub schedule_id: i32,
    #[validate(length(min = 1, max = 255, message = "Name is required"))]
    pub name: Option<String>,
    #[validate(custom(function = "validate_frequency"))]
    pub frequency: Option<String>,
    pub schedule_time: Option<String>,
    pub schedule_day: Option<i32>,
    pub schedule_date: Option<i32>,
    #[validate(range(min = 1, max = 100, message = "Retention count must be between 1 and 100"))]
    pub retention_count: Option<i32>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct DeleteBackupRequest {
    #[validate(range(min = 1, message = "Backup ID is required"))]
    pub backup_id: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupListResponse {
    pub backups: Vec<BackupMetadata>,
    pub total_count: i64,
    pub page: i32,
    pub limit: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupStatsResponse {
    pub total_backups: i64,
    pub storage_used: i64,
    pub storage_total: i64,
    pub last_backup_time: Option<chrono::DateTime<chrono::Utc>>,
    pub backup_status: String,
    pub avg_backup_size: Option<i64>,
    pub max_backup_size: Option<i64>,
    pub retention_policy: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupOperationStatus {
    pub operation_id: String,
    pub operation_type: String, // create, restore, delete
    pub status: String, // pending, running, completed, failed
    pub progress: Option<i32>, // 0-100
    pub description: Option<String>,
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub estimated_completion: Option<chrono::DateTime<chrono::Utc>>,
    pub error_message: Option<String>,
}

// Validation functions
fn validate_backup_type(backup_type: &str) -> Result<(), validator::ValidationError> {
    match backup_type {
        "full" | "data" | "schema" => Ok(()),
        _ => Err(validator::ValidationError::new("invalid_backup_type")),
    }
}

fn validate_frequency(frequency: &str) -> Result<(), validator::ValidationError> {
    match frequency {
        "daily" | "weekly" | "monthly" => Ok(()),
        _ => Err(validator::ValidationError::new("invalid_frequency")),
    }
}

impl CreateBackupRequest {
    pub fn get_backup_type(&self) -> String {
        self.backup_type
            .as_ref()
            .unwrap_or(&"full".to_string())
            .clone()
    }

    pub fn get_compress(&self) -> bool {
        self.compress.unwrap_or(true)
    }

    pub fn generate_name_if_empty(&mut self) {
        if self.name.is_none() || self.name.as_ref().unwrap().is_empty() {
            let now = chrono::Utc::now();
            let timestamp = now.format("%Y%m%d_%H%M%S");
            self.name = Some(format!("backup_{}", timestamp));
        }
    }
}

impl CreateScheduleRequest {
    pub fn get_retention_count(&self) -> i32 {
        self.retention_count.unwrap_or(7)
    }

    pub fn get_enabled(&self) -> bool {
        self.enabled.unwrap_or(true)
    }

    pub fn parse_schedule_time(&self) -> Result<chrono::NaiveTime, String> {
        chrono::NaiveTime::parse_from_str(&self.schedule_time, "%H:%M")
            .map_err(|_| "Invalid time format. Use HH:MM format.".to_string())
    }

    pub fn validate_schedule_constraints(&self) -> Result<(), String> {
        match self.frequency.as_str() {
            "weekly" => {
                if self.schedule_day.is_none() {
                    return Err("schedule_day is required for weekly frequency".to_string());
                }
                let day = self.schedule_day.unwrap();
                if !(0..=6).contains(&day) {
                    return Err("schedule_day must be between 0 (Sunday) and 6 (Saturday)".to_string());
                }
            }
            "monthly" => {
                if self.schedule_date.is_none() {
                    return Err("schedule_date is required for monthly frequency".to_string());
                }
                let date = self.schedule_date.unwrap();
                if !(1..=31).contains(&date) {
                    return Err("schedule_date must be between 1 and 31".to_string());
                }
            }
            "daily" => {
                // No additional constraints for daily
            }
            _ => return Err("Invalid frequency. Must be daily, weekly, or monthly".to_string()),
        }
        Ok(())
    }
}

impl BackupMetadata {
    pub fn is_completed(&self) -> bool {
        self.status == "completed"
    }

    pub fn is_running(&self) -> bool {
        matches!(self.status.as_str(), "pending" | "running")
    }

    pub fn is_failed(&self) -> bool {
        self.status == "failed"
    }

    pub fn get_file_size_mb(&self) -> Option<f64> {
        self.file_size.map(|size| size as f64 / 1024.0 / 1024.0)
    }

    pub fn get_duration(&self) -> Option<chrono::Duration> {
        if let (Some(completed), created) = (self.completed_at, self.created_at) {
            Some(completed - created)
        } else {
            None
        }
    }
}

impl BackupSchedule {
    pub fn calculate_next_run(&self) -> Option<chrono::DateTime<chrono::Utc>> {
        if !self.enabled {
            return None;
        }

        let now = chrono::Utc::now();
        let today = now.date_naive();
        
        match self.frequency.as_str() {
            "daily" => {
                let next_run_date = if now.time() > self.schedule_time {
                    today + chrono::Duration::days(1)
                } else {
                    today
                };
                Some(next_run_date.and_time(self.schedule_time).and_utc())
            }
            "weekly" => {
                if let Some(target_day) = self.schedule_day {
                    let current_weekday = today.weekday().num_days_from_sunday() as i32;
                    let days_until_target = if target_day > current_weekday {
                        target_day - current_weekday
                    } else if target_day < current_weekday {
                        7 - (current_weekday - target_day)
                    } else {
                        // Same day - check if time has passed
                        if now.time() > self.schedule_time {
                            7 // Next week
                        } else {
                            0 // Today
                        }
                    };
                    
                    let next_run_date = today + chrono::Duration::days(days_until_target as i64);
                    Some(next_run_date.and_time(self.schedule_time).and_utc())
                } else {
                    None
                }
            }
            "monthly" => {
                if let Some(target_date) = self.schedule_date {
                    let current_day = today.day() as i32;
                    let next_run_date = if target_date > current_day || 
                        (target_date == current_day && now.time() <= self.schedule_time) {
                        // This month
                        today.with_day(target_date as u32).unwrap_or(today)
                    } else {
                        // Next month
                        let next_month = if today.month() == 12 {
                            today.with_year(today.year() + 1).and_then(|d| d.with_month(1))
                        } else {
                            today.with_month(today.month() + 1)
                        };
                        
                        next_month
                            .and_then(|d| d.with_day(target_date as u32))
                            .unwrap_or(today + chrono::Duration::days(30))
                    };
                    
                    Some(next_run_date.and_time(self.schedule_time).and_utc())
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    pub fn is_due(&self) -> bool {
        if let Some(next_run) = self.next_run_at {
            chrono::Utc::now() >= next_run
        } else {
            false
        }
    }
}