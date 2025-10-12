// Task runner service for background job execution

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock, Semaphore};
use tokio::time::{sleep, Duration};
use tracing::{debug, error, info, instrument, warn};
use uuid::Uuid;

use crate::services::automation::{JobExecution, JobStatus};
use crate::services::database::DatabaseService;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub name: String,
    pub task_type: TaskType,
    pub priority: TaskPriority,
    pub payload: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub scheduled_at: Option<DateTime<Utc>>,
    pub max_retries: u32,
    pub retry_count: u32,
    pub timeout_seconds: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TaskType {
    DocketUpdate,
    DocumentGeneration,
    ExportData,
    EFilingSubmission,
    CacheCleanup,
    HealthCheck,
    NotificationSend,
    DataSync,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum TaskPriority {
    Low = 1,
    Medium = 2,
    High = 3,
    Critical = 4,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResult {
    pub task_id: String,
    pub status: TaskStatus,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub output: Option<serde_json::Value>,
    pub error_message: Option<String>,
    pub retry_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TaskStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
    Retrying,
}

pub struct TaskRunner {
    database: Arc<DatabaseService>,
    task_queue: Arc<RwLock<Vec<Task>>>,
    running_tasks: Arc<RwLock<HashMap<String, TaskResult>>>,
    completed_tasks: Arc<RwLock<HashMap<String, TaskResult>>>,
    task_sender: mpsc::UnboundedSender<Task>,
    task_receiver: Arc<RwLock<Option<mpsc::UnboundedReceiver<Task>>>>,
    semaphore: Arc<Semaphore>,
    max_concurrent_tasks: usize,
}

impl TaskRunner {
    pub fn new(database: Arc<DatabaseService>, max_concurrent_tasks: usize) -> Self {
        let (task_sender, task_receiver) = mpsc::unbounded_channel();
        
        Self {
            database,
            task_queue: Arc::new(RwLock::new(Vec::new())),
            running_tasks: Arc::new(RwLock::new(HashMap::new())),
            completed_tasks: Arc::new(RwLock::new(HashMap::new())),
            task_sender,
            task_receiver: Arc::new(RwLock::new(Some(task_receiver))),
            semaphore: Arc::new(Semaphore::new(max_concurrent_tasks)),
            max_concurrent_tasks,
        }
    }

    #[instrument(skip(self))]
    pub async fn start(&self) -> Result<()> {
        info!("Starting task runner with {} max concurrent tasks", self.max_concurrent_tasks);

        let mut receiver = self.task_receiver.write().await.take()
            .context("Task receiver already taken")?;

        let database = self.database.clone();
        let running_tasks = self.running_tasks.clone();
        let completed_tasks = self.completed_tasks.clone();
        let semaphore = self.semaphore.clone();

        tokio::spawn(async move {
            while let Some(task) = receiver.recv().await {
                let permit = semaphore.clone().acquire_owned().await.unwrap();
                let database_clone = database.clone();
                let running_tasks_clone = running_tasks.clone();
                let completed_tasks_clone = completed_tasks.clone();
                let task_clone = task.clone();

                tokio::spawn(async move {
                    let _permit = permit; // Keep permit alive for task duration
                    
                    let result = Self::execute_task(task_clone, database_clone, running_tasks_clone).await;
                    
                    // Move task from running to completed
                    if let Some(task_result) = running_tasks_clone.write().await.remove(&result.task_id) {
                        completed_tasks_clone.write().await.insert(result.task_id.clone(), result);
                    }
                });
            }
        });

        info!("Task runner started successfully");
        Ok(())
    }

    #[instrument(skip(self, task))]
    pub async fn submit_task(&self, task: Task) -> Result<String> {
        debug!("Submitting task: {} ({})", task.name, task.id);
        
        self.task_sender.send(task.clone())
            .context("Failed to submit task to queue")?;
        
        debug!("Task submitted successfully: {}", task.id);
        Ok(task.id.clone())
    }

    pub async fn create_task(
        &self,
        name: String,
        task_type: TaskType,
        priority: TaskPriority,
        payload: serde_json::Value,
        scheduled_at: Option<DateTime<Utc>>,
    ) -> Task {
        Task {
            id: Uuid::new_v4().to_string(),
            name,
            task_type,
            priority,
            payload,
            created_at: Utc::now(),
            scheduled_at,
            max_retries: 3,
            retry_count: 0,
            timeout_seconds: 300, // 5 minutes default
        }
    }

    async fn execute_task(
        task: Task,
        database: Arc<DatabaseService>,
        running_tasks: Arc<RwLock<HashMap<String, TaskResult>>>,
    ) -> TaskResult {
        let mut result = TaskResult {
            task_id: task.id.clone(),
            status: TaskStatus::Running,
            started_at: Utc::now(),
            completed_at: None,
            output: None,
            error_message: None,
            retry_count: task.retry_count,
        };

        // Add to running tasks
        running_tasks.write().await.insert(task.id.clone(), result.clone());

        info!("Executing task: {} ({})", task.name, task.id);

        // Execute the task based on type
        let execution_result = match task.task_type {
            TaskType::DocketUpdate => Self::execute_docket_update(&task, &database).await,
            TaskType::DocumentGeneration => Self::execute_document_generation(&task, &database).await,
            TaskType::ExportData => Self::execute_export_data(&task, &database).await,
            TaskType::EFilingSubmission => Self::execute_efiling_submission(&task, &database).await,
            TaskType::CacheCleanup => Self::execute_cache_cleanup(&task, &database).await,
            TaskType::HealthCheck => Self::execute_health_check(&task, &database).await,
            TaskType::NotificationSend => Self::execute_notification_send(&task, &database).await,
            TaskType::DataSync => Self::execute_data_sync(&task, &database).await,
        };

        // Update result based on execution outcome
        result.completed_at = Some(Utc::now());
        
        match execution_result {
            Ok(output) => {
                result.status = TaskStatus::Completed;
                result.output = output;
                info!("Task completed successfully: {} ({})", task.name, task.id);
            }
            Err(e) => {
                result.status = TaskStatus::Failed;
                result.error_message = Some(e.to_string());
                error!("Task failed: {} ({}) - {}", task.name, task.id, e);
            }
        }

        result
    }

    async fn execute_docket_update(
        task: &Task,
        _database: &DatabaseService,
    ) -> Result<Option<serde_json::Value>> {
        debug!("Executing docket update task: {}", task.id);
        
        // TODO: Implement docket update logic
        // This would fetch latest docket information and update cache
        
        Ok(Some(serde_json::json!({
            "dockets_updated": 0,
            "completed_at": Utc::now()
        })))
    }

    async fn execute_document_generation(
        task: &Task,
        _database: &DatabaseService,
    ) -> Result<Option<serde_json::Value>> {
        debug!("Executing document generation task: {}", task.id);
        
        // TODO: Implement document generation logic
        // This would generate documents based on templates and data
        
        Ok(Some(serde_json::json!({
            "documents_generated": 0,
            "completed_at": Utc::now()
        })))
    }

    async fn execute_export_data(
        task: &Task,
        _database: &DatabaseService,
    ) -> Result<Option<serde_json::Value>> {
        debug!("Executing export data task: {}", task.id);
        
        // TODO: Implement data export logic
        // This would export data in various formats
        
        Ok(Some(serde_json::json!({
            "files_exported": 0,
            "completed_at": Utc::now()
        })))
    }

    async fn execute_efiling_submission(
        task: &Task,
        _database: &DatabaseService,
    ) -> Result<Option<serde_json::Value>> {
        debug!("Executing e-filing submission task: {}", task.id);
        
        // TODO: Implement e-filing submission logic
        // This would submit documents to court e-filing systems
        
        Ok(Some(serde_json::json!({
            "submissions_processed": 0,
            "completed_at": Utc::now()
        })))
    }

    async fn execute_cache_cleanup(
        task: &Task,
        database: &DatabaseService,
    ) -> Result<Option<serde_json::Value>> {
        debug!("Executing cache cleanup task: {}", task.id);
        
        let cleaned_count = database.cleanup_expired_cache().await?;
        
        Ok(Some(serde_json::json!({
            "cleaned_entries": cleaned_count,
            "completed_at": Utc::now()
        })))
    }

    async fn execute_health_check(
        task: &Task,
        database: &DatabaseService,
    ) -> Result<Option<serde_json::Value>> {
        debug!("Executing health check task: {}", task.id);
        
        database.health_check().await?;
        let stats = database.get_database_stats().await?;
        
        Ok(Some(serde_json::json!({
            "database_healthy": true,
            "database_stats": stats,
            "completed_at": Utc::now()
        })))
    }

    async fn execute_notification_send(
        task: &Task,
        _database: &DatabaseService,
    ) -> Result<Option<serde_json::Value>> {
        debug!("Executing notification send task: {}", task.id);
        
        // TODO: Implement notification sending logic
        // This would send notifications via various channels
        
        Ok(Some(serde_json::json!({
            "notifications_sent": 0,
            "completed_at": Utc::now()
        })))
    }

    async fn execute_data_sync(
        task: &Task,
        _database: &DatabaseService,
    ) -> Result<Option<serde_json::Value>> {
        debug!("Executing data sync task: {}", task.id);
        
        // TODO: Implement data synchronization logic
        // This would sync data with external providers
        
        Ok(Some(serde_json::json!({
            "records_synced": 0,
            "completed_at": Utc::now()
        })))
    }

    pub async fn get_task_status(&self, task_id: &str) -> Option<TaskResult> {
        // Check running tasks first
        if let Some(result) = self.running_tasks.read().await.get(task_id) {
            return Some(result.clone());
        }
        
        // Check completed tasks
        self.completed_tasks.read().await.get(task_id).cloned()
    }

    pub async fn get_running_tasks(&self) -> Vec<TaskResult> {
        self.running_tasks.read().await.values().cloned().collect()
    }

    pub async fn get_completed_tasks(&self, limit: Option<usize>) -> Vec<TaskResult> {
        let completed = self.completed_tasks.read().await;
        let mut tasks: Vec<TaskResult> = completed.values().cloned().collect();
        
        // Sort by completion time (most recent first)
        tasks.sort_by(|a, b| {
            b.completed_at.unwrap_or(b.started_at)
                .cmp(&a.completed_at.unwrap_or(a.started_at))
        });
        
        if let Some(limit) = limit {
            tasks.truncate(limit);
        }
        
        tasks
    }

    pub async fn cancel_task(&self, task_id: &str) -> Result<()> {
        if let Some(mut result) = self.running_tasks.write().await.remove(task_id) {
            result.status = TaskStatus::Cancelled;
            result.completed_at = Some(Utc::now());
            self.completed_tasks.write().await.insert(task_id.to_string(), result);
            info!("Cancelled task: {}", task_id);
        }
        Ok(())
    }

    pub async fn get_queue_stats(&self) -> serde_json::Value {
        let running_count = self.running_tasks.read().await.len();
        let completed_count = self.completed_tasks.read().await.len();
        let available_permits = self.semaphore.available_permits();
        
        serde_json::json!({
            "running_tasks": running_count,
            "completed_tasks": completed_count,
            "available_slots": available_permits,
            "max_concurrent": self.max_concurrent_tasks,
            "timestamp": Utc::now()
        })
    }
}
