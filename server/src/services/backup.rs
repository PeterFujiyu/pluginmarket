use anyhow::{anyhow, Result};
use sqlx::PgPool;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::{
    models::{
        BackupMetadata, BackupSchedule, CreateBackupRequest, RestoreBackupRequest,
        CreateScheduleRequest, UpdateScheduleRequest, DeleteBackupRequest,
        BackupListResponse, BackupStatsResponse, BackupOperationStatus,
        AdminPaginationQuery,
    },
    utils::config::Config,
};

pub struct BackupService {
    pool: PgPool,
    config: Arc<Config>,
    // Track running operations
    operations: Arc<RwLock<HashMap<String, BackupOperationStatus>>>,
    // Prevent concurrent operations
    operation_lock: Arc<Mutex<()>>,
}

impl BackupService {
    pub fn new(pool: PgPool, config: Arc<Config>) -> Self {
        Self {
            pool,
            config,
            operations: Arc::new(RwLock::new(HashMap::new())),
            operation_lock: Arc::new(Mutex::new(())),
        }
    }

    /// Get backup statistics
    pub async fn get_backup_stats(&self) -> Result<BackupStatsResponse> {
        let stats = sqlx::query!(
            r#"
            SELECT 
                COUNT(*) as total_backups,
                COALESCE(SUM(file_size), 0) as "storage_used!: i64",
                MAX(created_at) as last_backup_time,
                COALESCE(AVG(file_size), 0) as "avg_backup_size!: i64",
                COALESCE(MAX(file_size), 0) as "max_backup_size!: i64"
            FROM backup_metadata 
            WHERE status = 'completed'
            "#
        )
        .fetch_one(&self.pool)
        .await?;

        // Calculate storage total (10GB default)
        let storage_total = 10 * 1024 * 1024 * 1024i64; // 10GB

        // Determine backup status
        let backup_status = if stats.last_backup_time.is_some() {
            let last_backup = stats.last_backup_time.unwrap();
            let hours_since = chrono::Utc::now()
                .signed_duration_since(last_backup)
                .num_hours();
            
            if hours_since < 24 {
                "success"
            } else if hours_since < 72 {
                "warning"
            } else {
                "error"
            }
        } else {
            "warning"
        };

        // Get retention policy from first schedule or default
        let retention_policy = sqlx::query!(
            "SELECT retention_count FROM backup_schedules LIMIT 1"
        )
        .fetch_optional(&self.pool)
        .await?
        .map(|r| r.retention_count)
        .unwrap_or(30);

        Ok(BackupStatsResponse {
            total_backups: stats.total_backups.unwrap_or(0),
            storage_used: stats.storage_used,
            storage_total,
            last_backup_time: stats.last_backup_time,
            backup_status: backup_status.to_string(),
            avg_backup_size: Some(stats.avg_backup_size),
            max_backup_size: Some(stats.max_backup_size),
            retention_policy,
        })
    }

    /// List backups with pagination and filtering
    pub async fn list_backups(
        &self,
        pagination: AdminPaginationQuery,
        filter: Option<String>,
    ) -> Result<BackupListResponse> {
        let page = pagination.page.unwrap_or(1).max(1);
        let limit = pagination.limit.unwrap_or(10).clamp(1, 100);
        let offset = (page - 1) * limit;

        let (where_clause, params) = match filter.as_deref() {
            Some("manual") => ("WHERE backup_type = 'full' AND created_by_id IS NOT NULL", vec![]),
            Some("scheduled") => ("WHERE backup_type = 'full' AND created_by_id IS NULL", vec![]),
            Some("today") => {
                let today = chrono::Utc::now().date_naive();
                (
                    "WHERE DATE(created_at) = $3",
                    vec![today.to_string()],
                )
            }
            Some("week") => {
                let week_ago = chrono::Utc::now() - chrono::Duration::weeks(1);
                (
                    "WHERE created_at >= $3",
                    vec![week_ago.to_rfc3339()],
                )
            }
            _ => ("", vec![]),
        };

        // Get total count
        let count_query = format!(
            "SELECT COUNT(*) as count FROM backup_metadata {}",
            where_clause
        );
        let total_count = if params.is_empty() {
            sqlx::query_scalar::<_, i64>(&count_query)
                .fetch_one(&self.pool)
                .await?
        } else {
            sqlx::query_scalar::<_, i64>(&count_query)
                .bind(&params[0])
                .fetch_one(&self.pool)
                .await?
        };

        // Get backups
        let query = format!(
            r#"
            SELECT 
                id, name, description, backup_type, status, file_path, file_size,
                created_by_id, created_by, created_at, completed_at, error_message, compressed
            FROM backup_metadata 
            {} 
            ORDER BY created_at DESC 
            LIMIT $1 OFFSET $2
            "#,
            where_clause
        );

        let backups = if params.is_empty() {
            sqlx::query_as::<_, BackupMetadata>(&query)
                .bind(limit)
                .bind(offset)
                .fetch_all(&self.pool)
                .await?
        } else {
            sqlx::query_as::<_, BackupMetadata>(&query)
                .bind(limit)
                .bind(offset)
                .bind(&params[0])
                .fetch_all(&self.pool)
                .await?
        };

        Ok(BackupListResponse {
            backups,
            total_count,
            page,
            limit,
        })
    }

    /// Create a new backup
    pub async fn create_backup(
        &self,
        admin_id: i32,
        admin_email: &str,
        mut request: CreateBackupRequest,
        _ip_address: Option<std::net::IpAddr>,
    ) -> Result<String> {
        let _lock = self.operation_lock.lock().await;

        // Generate name if not provided
        request.generate_name_if_empty();
        let backup_name = request.name.as_ref().unwrap();

        // Check for existing running backups
        let running_count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM backup_metadata WHERE status IN ('pending', 'running')"
        )
        .fetch_one(&self.pool)
        .await?;

        if running_count > 0 {
            return Err(anyhow!("Another backup operation is already in progress"));
        }

        // Create backup record
        let backup_id = sqlx::query_scalar::<_, i32>(
            r#"
            INSERT INTO backup_metadata (
                name, description, backup_type, status, created_by_id, created_by, compressed
            ) VALUES ($1, $2, $3, 'pending', $4, $5, $6)
            RETURNING id
            "#
        )
        .bind(backup_name)
        .bind(&request.description)
        .bind(&request.get_backup_type())
        .bind(admin_id)
        .bind(admin_email)
        .bind(request.get_compress())
        .fetch_one(&self.pool)
        .await?;

        let operation_id = Uuid::new_v4().to_string();

        // Add to operations tracking
        {
            let mut operations = self.operations.write().await;
            operations.insert(
                operation_id.clone(),
                BackupOperationStatus {
                    operation_id: operation_id.clone(),
                    operation_type: "create".to_string(),
                    status: "pending".to_string(),
                    progress: Some(0),
                    description: Some(format!("Creating backup: {}", backup_name)),
                    started_at: chrono::Utc::now(),
                    estimated_completion: Some(chrono::Utc::now() + chrono::Duration::minutes(5)),
                    error_message: None,
                },
            );
        }

        // Start backup process asynchronously
        let pool = self.pool.clone();
        let config = self.config.clone();
        let operations = self.operations.clone();
        let backup_name = backup_name.clone();
        let backup_type = request.get_backup_type();
        let compress = request.get_compress();
        let op_id_for_spawn = operation_id.clone();

        tokio::spawn(async move {
            if let Err(e) = Self::execute_backup(
                pool,
                config,
                operations,
                op_id_for_spawn,
                backup_id,
                backup_name,
                backup_type,
                compress,
            ).await {
                error!("Backup operation failed: {}", e);
            }
        });

        Ok(operation_id)
    }

    /// Execute the actual backup process
    async fn execute_backup(
        pool: PgPool,
        config: Arc<Config>,
        operations: Arc<RwLock<HashMap<String, BackupOperationStatus>>>,
        operation_id: String,
        backup_id: i32,
        backup_name: String,
        backup_type: String,
        compress: bool,
    ) -> Result<()> {
        // Update status to running
        {
            let mut ops = operations.write().await;
            if let Some(op) = ops.get_mut(&operation_id) {
                op.status = "running".to_string();
                op.progress = Some(10);
            }
        }

        sqlx::query!(
            "UPDATE backup_metadata SET status = 'running' WHERE id = $1",
            backup_id
        )
        .execute(&pool)
        .await?;

        // Create backup directory if it doesn't exist
        let backup_dir = Path::new(&config.storage.upload_path).join("backups");
        tokio::fs::create_dir_all(&backup_dir).await?;

        let file_extension = if compress { ".sql.gz" } else { ".sql" };
        let backup_filename = format!("{}_{}{}", backup_name, chrono::Utc::now().format("%Y%m%d_%H%M%S"), file_extension);
        let backup_path = backup_dir.join(&backup_filename);

        // Update progress
        {
            let mut ops = operations.write().await;
            if let Some(op) = ops.get_mut(&operation_id) {
                op.progress = Some(30);
                op.description = Some("Dumping database...".to_string());
            }
        }

        // Execute pg_dump
        let database_url = &config.database.url;
        let mut pg_dump_cmd = tokio::process::Command::new("pg_dump");
        
        // Parse database URL to get connection parameters
        let url = url::Url::parse(database_url)?;
        let host = url.host_str().unwrap_or("localhost");
        let port = url.port().unwrap_or(5432);
        let username = url.username();
        let password = url.password().unwrap_or("");
        let database = url.path().trim_start_matches('/');

        pg_dump_cmd
            .arg("--host").arg(host)
            .arg("--port").arg(port.to_string())
            .arg("--username").arg(username)
            .arg("--no-password")
            .arg("--verbose")
            .env("PGPASSWORD", password);

        match backup_type.as_str() {
            "data" => {
                pg_dump_cmd.arg("--data-only");
            }
            "schema" => {
                pg_dump_cmd.arg("--schema-only");
            }
            _ => {
                // Full backup is default
            }
        }

        pg_dump_cmd.arg(database);

        let output = if compress {
            // We cannot directly pass a `tokio::process::ChildStdout` into
            // another command's stdin. Instead, we must use `tokio::io::copy`
            // to asynchronously pipe the data between the two processes.
            let mut pg_dump_child = pg_dump_cmd
                .stdout(std::process::Stdio::piped())
                .spawn()?;
            
            let mut gzip_child = tokio::process::Command::new("gzip")
                .stdin(std::process::Stdio::piped())
                .stdout(std::process::Stdio::piped())
                .spawn()?;

            // Get the asynchronous handles for piping
            let mut pg_dump_stdout = pg_dump_child.stdout.take().ok_or_else(|| anyhow!("Failed to get pg_dump stdout handle"))?;
            let mut gzip_stdin = gzip_child.stdin.take().ok_or_else(|| anyhow!("Failed to get gzip stdin handle"))?;

            // Asynchronously copy data from pg_dump's stdout to gzip's stdin.
            // This is crucial to avoid deadlocks. The `copy` function handles the
            // data transfer efficiently and closes the writer when done.
            tokio::io::copy(&mut pg_dump_stdout, &mut gzip_stdin).await?;

            // Wait for both processes to finish. We need to collect the output from gzip.
            // The `pg_dump` output is already handled by the `copy` operation.
            let (pg_dump_result, gzip_output_result) = tokio::join!(
                pg_dump_child.wait(),
                gzip_child.wait_with_output(),
            );
            
            // Check for errors from both processes
            let pg_dump_status = pg_dump_result?;
            if !pg_dump_status.success() {
                // `pg_dump` has failed, so we can't proceed.
                return Err(anyhow!("pg_dump failed during compression"));
            }

            let gzip_output = gzip_output_result?;
            if !gzip_output.status.success() {
                return Err(anyhow!("gzip failed: {}", String::from_utf8_lossy(&gzip_output.stderr)));
            }

            // Write the compressed output to the file
            tokio::fs::write(&backup_path, &gzip_output.stdout).await?;

            gzip_output // Return the gzip output for a consistent code path
        } else {
            pg_dump_cmd.output().await?
        };

        // This block handles errors for both compressed and uncompressed paths
        // in a unified way.
        if !output.status.success() {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            
            // Update operation as failed
            {
                let mut ops = operations.write().await;
                if let Some(op) = ops.get_mut(&operation_id) {
                    op.status = "failed".to_string();
                    op.error_message = Some(error_msg.to_string());
                }
            }

            sqlx::query!(
                "UPDATE backup_metadata SET status = 'failed', error_message = $1 WHERE id = $2",
                error_msg.to_string(),
                backup_id
            )
            .execute(&pool)
            .await?;

            return Err(anyhow!("Backup operation failed: {}", error_msg));
        }

        // Get file size
        let file_metadata = tokio::fs::metadata(&backup_path).await?;
        let file_size = file_metadata.len() as i64;

        // Update progress
        {
            let mut ops = operations.write().await;
            if let Some(op) = ops.get_mut(&operation_id) {
                op.progress = Some(90);
                op.description = Some("Finalizing backup...".to_string());
            }
        }

        // Update backup record as completed
        sqlx::query!(
            r#"
            UPDATE backup_metadata 
            SET status = 'completed', file_path = $1, file_size = $2, completed_at = NOW()
            WHERE id = $3
            "#,
            backup_path.to_string_lossy().to_string(),
            file_size,
            backup_id
        )
        .execute(&pool)
        .await?;

        // Complete operation
        {
            let mut ops = operations.write().await;
            if let Some(op) = ops.get_mut(&operation_id) {
                op.status = "completed".to_string();
                op.progress = Some(100);
                op.description = Some("Backup completed successfully".to_string());
            }
        }

        info!("Backup completed successfully: {} ({}MB)", backup_name, file_size / 1024 / 1024);

        // Log the backup creation
        sqlx::query!(
            r#"
            INSERT INTO system_logs (log_level, component, message, details)
            VALUES ('INFO', 'backup', 'Backup created successfully', $1)
            "#,
            serde_json::json!({
                "backup_id": backup_id,
                "backup_name": backup_name,
                "file_size": file_size,
                "backup_type": backup_type,
                "compressed": compress
            })
        )
        .execute(&pool)
        .await?;

        Ok(())
    }

    /// Get operation status
    pub async fn get_operation_status(&self, operation_id: &str) -> Option<BackupOperationStatus> {
        let operations = self.operations.read().await;
        operations.get(operation_id).cloned()
    }

    /// List all current operations
    pub async fn list_operations(&self) -> Vec<BackupOperationStatus> {
        let operations = self.operations.read().await;
        operations.values().cloned().collect()
    }

    /// Download backup file
    pub async fn get_backup_file_path(&self, backup_id: i32) -> Result<String> {
        let backup = sqlx::query_as::<_, BackupMetadata>(
            "SELECT * FROM backup_metadata WHERE id = $1 AND status = 'completed'"
        )
        .bind(backup_id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| anyhow!("Backup not found or not completed"))?;

        backup.file_path
            .ok_or_else(|| anyhow!("Backup file path not available"))
    }

    /// Delete backup
    pub async fn delete_backup(
        &self,
        admin_id: i32,
        request: DeleteBackupRequest,
        _ip_address: Option<std::net::IpAddr>,
    ) -> Result<()> {
        let backup = sqlx::query_as::<_, BackupMetadata>(
            "SELECT * FROM backup_metadata WHERE id = $1"
        )
        .bind(request.backup_id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| anyhow!("Backup not found"))?;

        // Don't allow deletion of running backups
        if backup.is_running() {
            return Err(anyhow!("Cannot delete a running backup"));
        }

        // Delete file if it exists
        if let Some(file_path) = &backup.file_path {
            if Path::new(file_path).exists() {
                if let Err(e) = tokio::fs::remove_file(file_path).await {
                    warn!("Failed to delete backup file {}: {}", file_path, e);
                }
            }
        }

        // Delete database record
        sqlx::query!(
            "DELETE FROM backup_metadata WHERE id = $1",
            request.backup_id
        )
        .execute(&self.pool)
        .await?;

        // Log the deletion
        sqlx::query!(
            r#"
            INSERT INTO system_logs (log_level, component, message, details, user_id, ip_address)
            VALUES ('INFO', 'backup', 'Backup deleted', $1, $2, $3)
            "#,
            serde_json::json!({
                "backup_id": backup.id,
                "backup_name": backup.name,
                "deleted_by": admin_id
            }),
            admin_id,
            _ip_address.map(|ip| ip.to_string())
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Restore backup (placeholder implementation)
    pub async fn restore_backup(
        &self,
        admin_id: i32,
        request: RestoreBackupRequest,
        _ip_address: Option<std::net::IpAddr>,
    ) -> Result<String> {
        let _lock = self.operation_lock.lock().await;

        let backup = sqlx::query_as::<_, BackupMetadata>(
            "SELECT * FROM backup_metadata WHERE id = $1 AND status = 'completed'"
        )
        .bind(request.backup_id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| anyhow!("Backup not found or not completed"))?;

        let operation_id = Uuid::new_v4().to_string();

        // Add to operations tracking
        {
            let mut operations = self.operations.write().await;
            operations.insert(
                operation_id.clone(),
                BackupOperationStatus {
                    operation_id: operation_id.clone(),
                    operation_type: "restore".to_string(),
                    status: "pending".to_string(),
                    progress: Some(0),
                    description: Some(format!("Restoring backup: {}", backup.name)),
                    started_at: chrono::Utc::now(),
                    estimated_completion: Some(chrono::Utc::now() + chrono::Duration::minutes(10)),
                    error_message: None,
                },
            );
        }

        // Log the restore attempt
        sqlx::query!(
            r#"
            INSERT INTO system_logs (log_level, component, message, details, user_id, ip_address)
            VALUES ('WARN', 'backup', 'Database restore initiated', $1, $2, $3)
            "#,
            serde_json::json!({
                "backup_id": backup.id,
                "backup_name": backup.name,
                "restore_options": {
                    "drop_existing_tables": request.drop_existing_tables.unwrap_or(false),
                    "create_pre_restore_backup": request.create_pre_restore_backup.unwrap_or(true)
                }
            }),
            admin_id,
            _ip_address.map(|ip| ip.to_string())
        )
        .execute(&self.pool)
        .await?;

        // Note: In a production system, you would implement the actual restore logic here
        // This is a simplified implementation that just marks the operation as completed
        warn!("Backup restore is not fully implemented - this is a placeholder");

        Ok(operation_id)
    }

    /// Schedule management methods
    pub async fn list_schedules(&self) -> Result<Vec<BackupSchedule>> {
        let schedules = sqlx::query_as::<_, BackupSchedule>(
            "SELECT * FROM backup_schedules ORDER BY created_at DESC"
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(schedules)
    }

    pub async fn create_schedule(
        &self,
        admin_id: i32,
        request: CreateScheduleRequest,
    ) -> Result<i32> {
        // Validate schedule constraints
        request.validate_schedule_constraints()
            .map_err(|e| anyhow!("Schedule validation failed: {}", e))?;
        
        let schedule_time = request.parse_schedule_time()
            .map_err(|e| anyhow!("Failed to parse schedule time: {}", e))?;
        
        let schedule_id = sqlx::query_scalar::<_, i32>(
            r#"
            INSERT INTO backup_schedules (
                name, frequency, schedule_time, schedule_day, schedule_date,
                retention_count, enabled, created_by_id
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING id
            "#
        )
        .bind(&request.name)
        .bind(&request.frequency)
        .bind(schedule_time)
        .bind(request.schedule_day)
        .bind(request.schedule_date)
        .bind(request.get_retention_count())
        .bind(request.get_enabled())
        .bind(admin_id)
        .fetch_one(&self.pool)
        .await?;

        // Calculate and update next run time
        if let Ok(schedule) = self.get_schedule(schedule_id).await {
            if let Some(next_run) = schedule.calculate_next_run() {
                sqlx::query!(
                    "UPDATE backup_schedules SET next_run_at = $1 WHERE id = $2",
                    next_run,
                    schedule_id
                )
                .execute(&self.pool)
                .await?;
            }
        }

        Ok(schedule_id)
    }

    pub async fn get_schedule(&self, schedule_id: i32) -> Result<BackupSchedule> {
        let schedule = sqlx::query_as::<_, BackupSchedule>(
            "SELECT * FROM backup_schedules WHERE id = $1"
        )
        .bind(schedule_id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| anyhow!("Schedule not found"))?;

        Ok(schedule)
    }
}
