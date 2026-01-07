//! # Function Scheduler

use std::collections::HashMap;
use std::sync::RwLock;

use chrono::{DateTime, Utc};
use uuid::Uuid;

use super::errors::{FunctionError, FunctionResult};

/// A scheduled job
#[derive(Debug, Clone)]
pub struct ScheduledJob {
    /// Job ID
    pub id: Uuid,

    /// Function name
    pub function_name: String,

    /// Cron expression
    pub cron: String,

    /// Last run time
    pub last_run: Option<DateTime<Utc>>,

    /// Next scheduled run
    pub next_run: Option<DateTime<Utc>>,

    /// Whether the job is enabled
    pub enabled: bool,
}

impl ScheduledJob {
    /// Create a new scheduled job
    pub fn new(function_name: String, cron: String) -> FunctionResult<Self> {
        // Basic cron validation (stubbed - would use cron parser in production)
        if cron.split_whitespace().count() < 5 {
            return Err(FunctionError::InvalidCron(cron));
        }

        Ok(Self {
            id: Uuid::new_v4(),
            function_name,
            cron,
            last_run: None,
            next_run: Some(Utc::now()), // Stub: would calculate from cron
            enabled: true,
        })
    }

    /// Mark as run
    pub fn mark_run(&mut self) {
        self.last_run = Some(Utc::now());
        // In production, would calculate next run from cron
        self.next_run = Some(Utc::now() + chrono::Duration::hours(1));
    }
}

/// Job scheduler
#[derive(Debug, Default)]
pub struct Scheduler {
    /// Jobs by ID
    jobs: RwLock<HashMap<Uuid, ScheduledJob>>,

    /// Jobs by function name
    by_function: RwLock<HashMap<String, Uuid>>,
}

impl Scheduler {
    /// Create a new scheduler
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a scheduled job
    pub fn schedule(&self, job: ScheduledJob) -> FunctionResult<Uuid> {
        let id = job.id;
        let function_name = job.function_name.clone();

        {
            let mut jobs = self
                .jobs
                .write()
                .map_err(|_| FunctionError::Internal("Lock poisoned".into()))?;
            jobs.insert(id, job);
        }

        {
            let mut by_function = self
                .by_function
                .write()
                .map_err(|_| FunctionError::Internal("Lock poisoned".into()))?;
            by_function.insert(function_name, id);
        }

        Ok(id)
    }

    /// Cancel a job
    pub fn cancel(&self, job_id: Uuid) -> FunctionResult<()> {
        let function_name = {
            let mut jobs = self
                .jobs
                .write()
                .map_err(|_| FunctionError::Internal("Lock poisoned".into()))?;
            jobs.remove(&job_id).map(|j| j.function_name)
        };

        if let Some(name) = function_name {
            let mut by_function = self
                .by_function
                .write()
                .map_err(|_| FunctionError::Internal("Lock poisoned".into()))?;
            by_function.remove(&name);
        }

        Ok(())
    }

    /// Get jobs that are due to run
    pub fn get_due_jobs(&self) -> Vec<ScheduledJob> {
        let now = Utc::now();

        self.jobs
            .read()
            .map(|jobs| {
                jobs.values()
                    .filter(|j| j.enabled && j.next_run.map(|t| t <= now).unwrap_or(false))
                    .cloned()
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Mark a job as run
    pub fn mark_run(&self, job_id: Uuid) -> FunctionResult<()> {
        let mut jobs = self
            .jobs
            .write()
            .map_err(|_| FunctionError::Internal("Lock poisoned".into()))?;

        if let Some(job) = jobs.get_mut(&job_id) {
            job.mark_run();
        }

        Ok(())
    }

    /// Get job count
    pub fn len(&self) -> usize {
        self.jobs.read().map(|j| j.len()).unwrap_or(0)
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scheduled_job_creation() {
        let job = ScheduledJob::new("cleanup".to_string(), "0 0 * * *".to_string()).unwrap();

        assert_eq!(job.function_name, "cleanup");
        assert!(job.enabled);
    }

    #[test]
    fn test_invalid_cron() {
        let result = ScheduledJob::new("test".to_string(), "invalid".to_string());

        assert!(result.is_err());
    }

    #[test]
    fn test_scheduler() {
        let scheduler = Scheduler::new();

        let job = ScheduledJob::new("daily_task".to_string(), "0 0 * * *".to_string()).unwrap();

        let id = scheduler.schedule(job).unwrap();
        assert_eq!(scheduler.len(), 1);

        scheduler.cancel(id).unwrap();
        assert_eq!(scheduler.len(), 0);
    }

    #[test]
    fn test_get_due_jobs() {
        let scheduler = Scheduler::new();

        let job = ScheduledJob::new("immediate".to_string(), "* * * * *".to_string()).unwrap();

        scheduler.schedule(job).unwrap();

        let due = scheduler.get_due_jobs();
        assert_eq!(due.len(), 1);
    }
}
