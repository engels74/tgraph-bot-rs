//! Scheduler Service - Core scheduling system using tokio-cron-scheduler
//! 
//! This module provides the foundation for automated graph generation and posting
//! with configurable cron-based schedules.

use std::sync::Arc;
use anyhow::{Result, Context};
use tokio::sync::{RwLock, Mutex};
use tokio_cron_scheduler::{JobScheduler, Job};
use tracing::{info, warn, debug};
use uuid::Uuid;
use std::collections::HashMap;

/// Type alias for job identifiers (same as tokio-cron-scheduler's JobId)
pub type JobId = Uuid;

/// Core scheduler service that manages cron-based job scheduling
pub struct SchedulerService {
    /// The underlying tokio-cron-scheduler instance
    scheduler: Arc<Mutex<JobScheduler>>,
    /// Map of job IDs to their metadata for tracking
    jobs: Arc<RwLock<HashMap<JobId, JobMetadata>>>,
    /// Whether the scheduler is currently running
    is_running: Arc<RwLock<bool>>,
}

/// Metadata for tracking scheduled jobs
#[derive(Debug, Clone)]
pub struct JobMetadata {
    /// Unique identifier for the job
    pub id: JobId,
    /// Human-readable name for the job
    pub name: String,
    /// Cron expression used for scheduling
    pub cron_expression: String,
    /// Optional description of what the job does
    pub description: Option<String>,
    /// Whether the job is currently enabled
    pub enabled: bool,
}

impl SchedulerService {
    /// Create a new scheduler service instance
    /// 
    /// # Returns
    /// 
    /// A new `SchedulerService` instance ready to be started
    pub async fn new() -> Result<Self> {
        info!("Creating new scheduler service");
        
        let scheduler = JobScheduler::new()
            .await
            .context("Failed to create JobScheduler")?;
        
        debug!("JobScheduler created successfully");
        
        Ok(SchedulerService {
            scheduler: Arc::new(Mutex::new(scheduler)),
            jobs: Arc::new(RwLock::new(HashMap::new())),
            is_running: Arc::new(RwLock::new(false)),
        })
    }
    
    /// Start the scheduler service
    /// 
    /// This will begin processing all scheduled jobs according to their cron expressions.
    /// 
    /// # Errors
    /// 
    /// Returns an error if the scheduler fails to start or is already running.
    pub async fn start(&self) -> Result<()> {
        let mut is_running = self.is_running.write().await;
        
        if *is_running {
            warn!("Scheduler is already running");
            return Ok(());
        }
        
        info!("Starting scheduler service");
        
        let scheduler = self.scheduler.lock().await;
        scheduler.start()
            .await
            .context("Failed to start scheduler")?;
        
        *is_running = true;
        info!("Scheduler service started successfully");
        
        Ok(())
    }
    
    /// Stop the scheduler service gracefully
    /// 
    /// This will stop processing new jobs and wait for running jobs to complete.
    /// 
    /// # Errors
    /// 
    /// Returns an error if the scheduler fails to stop cleanly.
    pub async fn stop(&self) -> Result<()> {
        let mut is_running = self.is_running.write().await;
        
        if !*is_running {
            warn!("Scheduler is not running");
            return Ok(());
        }
        
        info!("Stopping scheduler service");
        
        let mut scheduler = self.scheduler.lock().await;
        scheduler.shutdown()
            .await
            .context("Failed to shutdown scheduler")?;
        
        *is_running = false;
        info!("Scheduler service stopped successfully");
        
        Ok(())
    }
    
    /// Check if the scheduler is currently running
    pub async fn is_running(&self) -> bool {
        *self.is_running.read().await
    }
    
    /// Add a new scheduled job with a cron expression
    /// 
    /// # Arguments
    /// 
    /// * `name` - Human-readable name for the job
    /// * `cron_expression` - Valid cron expression (e.g., "0 0 * * *" for daily at midnight)
    /// * `description` - Optional description of what the job does
    /// * `job_fn` - Async function to execute when the job runs
    /// 
    /// # Returns
    /// 
    /// The JobId of the newly created job
    /// 
    /// # Errors
    /// 
    /// Returns an error if the cron expression is invalid or the job cannot be added.
    pub async fn add_job<F, Fut>(
        &self,
        name: String,
        cron_expression: String,
        description: Option<String>,
        job_fn: F,
    ) -> Result<JobId>
    where
        F: Fn() -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = ()> + Send + 'static,
    {
        info!("Adding new job: {} with cron: {}", name, cron_expression);
        
        // Validate cron expression by creating a job
        let job = Job::new_async(cron_expression.as_str(), move |_uuid, _scheduler| {
            let job_fn = job_fn();
            Box::pin(async move {
                job_fn.await;
            })
        })
        .context("Invalid cron expression or job creation failed")?;
        
        let scheduler = self.scheduler.lock().await;
        let job_id = scheduler.add(job)
            .await
            .context("Failed to add job to scheduler")?;
        
        // Store job metadata
        let metadata = JobMetadata {
            id: job_id,
            name: name.clone(),
            cron_expression,
            description,
            enabled: true,
        };
        
        let mut jobs = self.jobs.write().await;
        jobs.insert(job_id, metadata);
        
        info!("Successfully added job: {} with ID: {:?}", name, job_id);
        
        Ok(job_id)
    }
    
    /// Remove a scheduled job by its ID
    /// 
    /// # Arguments
    /// 
    /// * `job_id` - The JobId of the job to remove
    /// 
    /// # Errors
    /// 
    /// Returns an error if the job doesn't exist or cannot be removed.
    pub async fn remove_job(&self, job_id: JobId) -> Result<()> {
        info!("Removing job with ID: {:?}", job_id);
        
        // Get job name for logging before removal
        let job_name = {
            let jobs = self.jobs.read().await;
            jobs.get(&job_id)
                .map(|metadata| metadata.name.clone())
                .unwrap_or_else(|| format!("Unknown job {:?}", job_id))
        };
        
        let scheduler = self.scheduler.lock().await;
        scheduler.remove(&job_id)
            .await
            .context("Failed to remove job from scheduler")?;
        
        // Remove from our metadata tracking
        let mut jobs = self.jobs.write().await;
        jobs.remove(&job_id);
        
        info!("Successfully removed job: {}", job_name);
        
        Ok(())
    }
    
    /// List all currently scheduled jobs
    /// 
    /// # Returns
    /// 
    /// A vector of JobMetadata for all jobs
    pub async fn list_jobs(&self) -> Vec<JobMetadata> {
        let jobs = self.jobs.read().await;
        jobs.values().cloned().collect()
    }
    
    /// Get metadata for a specific job
    /// 
    /// # Arguments
    /// 
    /// * `job_id` - The JobId to look up
    /// 
    /// # Returns
    /// 
    /// The JobMetadata if found, None otherwise
    pub async fn get_job(&self, job_id: JobId) -> Option<JobMetadata> {
        let jobs = self.jobs.read().await;
        jobs.get(&job_id).cloned()
    }
    
    /// Get the number of currently scheduled jobs
    pub async fn job_count(&self) -> usize {
        let jobs = self.jobs.read().await;
        jobs.len()
    }
}

/// Helper function to validate a cron expression without creating a job
/// 
/// # Arguments
/// 
/// * `cron_expression` - The cron expression to validate
/// 
/// # Returns
/// 
/// True if the expression is valid, false otherwise
pub fn validate_cron_expression(cron_expression: &str) -> bool {
    // Try to create a dummy job to validate the expression
    Job::new(cron_expression, |_uuid, _scheduler| {
        // Empty job function for validation
    }).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    
    #[tokio::test]
    async fn test_scheduler_creation() {
        let scheduler = SchedulerService::new().await;
        assert!(scheduler.is_ok());
        
        let scheduler = scheduler.unwrap();
        assert!(!scheduler.is_running().await);
    }
    
    #[tokio::test]
    async fn test_scheduler_start_stop() {
        let scheduler = SchedulerService::new().await.unwrap();
        
        // Test starting
        assert!(scheduler.start().await.is_ok());
        assert!(scheduler.is_running().await);
        
        // Test stopping
        assert!(scheduler.stop().await.is_ok());
        assert!(!scheduler.is_running().await);
    }
    
    #[tokio::test]
    async fn test_add_and_remove_job() {
        let scheduler = SchedulerService::new().await.unwrap();
        
        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = counter.clone();
        
        // Add a job that runs every second (for testing)
        let job_id = scheduler.add_job(
            "test_job".to_string(),
            "* * * * * *".to_string(), // Every second
            Some("Test job description".to_string()),
            move || {
                let counter = counter_clone.clone();
                async move {
                    counter.fetch_add(1, Ordering::SeqCst);
                }
            }
        ).await;
        
        assert!(job_id.is_ok());
        let job_id = job_id.unwrap();
        
        // Verify job was added
        assert_eq!(scheduler.job_count().await, 1);
        
        let job_metadata = scheduler.get_job(job_id).await;
        assert!(job_metadata.is_some());
        
        let metadata = job_metadata.unwrap();
        assert_eq!(metadata.name, "test_job");
        assert_eq!(metadata.cron_expression, "* * * * * *");
        assert_eq!(metadata.description, Some("Test job description".to_string()));
        
        // Remove the job
        assert!(scheduler.remove_job(job_id).await.is_ok());
        assert_eq!(scheduler.job_count().await, 0);
    }
    
    #[tokio::test]
    async fn test_list_jobs() {
        let scheduler = SchedulerService::new().await.unwrap();
        
        // Add multiple jobs
        let _job1 = scheduler.add_job(
            "job1".to_string(),
            "0 0 0 * * *".to_string(),
            None,
            || async {}
        ).await.unwrap();
        
        let _job2 = scheduler.add_job(
            "job2".to_string(),
            "0 0 12 * * *".to_string(),
            Some("Job 2 description".to_string()),
            || async {}
        ).await.unwrap();
        
        let jobs = scheduler.list_jobs().await;
        assert_eq!(jobs.len(), 2);
        
        // Verify job names are present
        let job_names: Vec<&String> = jobs.iter().map(|j| &j.name).collect();
        assert!(job_names.contains(&&"job1".to_string()));
        assert!(job_names.contains(&&"job2".to_string()));
    }
    
    #[test]
    fn test_validate_cron_expression() {
        // Valid expressions (6-field format: second minute hour day month weekday)
        assert!(validate_cron_expression("0 0 0 * * *"));     // Daily at midnight
        assert!(validate_cron_expression("0 0 */2 * * *"));   // Every 2 hours
        assert!(validate_cron_expression("* * * * * *"));     // Every second
        assert!(validate_cron_expression("1/5 * * * * *"));   // Every 5 seconds
        
        // Invalid expressions
        assert!(!validate_cron_expression("invalid"));
        assert!(!validate_cron_expression("60 0 0 * * *"));   // Invalid second
        assert!(!validate_cron_expression("0 60 0 * * *"));   // Invalid minute
        assert!(!validate_cron_expression("0 0 25 * * *"));   // Invalid hour
    }
    
    #[tokio::test]
    async fn test_invalid_cron_expression() {
        let scheduler = SchedulerService::new().await.unwrap();
        
        let result = scheduler.add_job(
            "invalid_job".to_string(),
            "invalid_cron".to_string(),
            None,
            || async {}
        ).await;
        
        assert!(result.is_err());
    }
} 