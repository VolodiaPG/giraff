use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use futures::Future;
use model::SlaId;
use std::collections::VecDeque;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::Mutex;
use tokio_cron_scheduler::{Job, JobScheduler};
use uom::si::f64::Time;
use uom::si::time::second;

pub struct Cron {
    scheduler:            JobScheduler,
    periodic_task_period: Time,
    pub tasks:            Arc<Mutex<VecDeque<TaskEntry>>>,
}
pub struct UnprovisionFunction {
    pub sla:  SlaId,
    pub node: String, // k8s node
}
pub enum Task {
    UnprovisionFunction(UnprovisionFunction),
}

pub struct TaskEntry {
    pub task:       Task,
    pub created_at: DateTime<Utc>,
}

impl Cron {
    pub async fn new(periodic_task_period: Time) -> Result<Self> {
        let scheduler = JobScheduler::new()
            .await
            .context("Failed to create the cron job scheduler")?;
        scheduler.start().await.context("Failed to start the scheduler")?;

        Ok(Self {
            scheduler,
            periodic_task_period,
            tasks: Arc::new(Mutex::new(VecDeque::new())),
        })
    }

    // Add job to be executed at the period configured at the creation of the
    // instance
    pub async fn add_periodic<T>(&self, callback: T) -> Result<()>
    where
        T: 'static,
        T: Fn() -> Pin<Box<dyn Future<Output = ()> + Send>> + Send + Sync,
    {
        let job = Job::new_async(
            format!(
                "1/{} * * * * *",
                self.periodic_task_period.get::<second>()
            )
            .as_str(),
            move |_, _| callback(),
        )?;
        self.scheduler
            .add(job)
            .await
            .context("Failed to add a job to the scheduler")?;
        Ok(())
    }

    // Add a job to execute in <duration> time
    pub async fn add_oneshot<T>(
        &self,
        duration: Time,
        task: Task,
        callback: T,
    ) -> Result<()>
    where
        T: 'static,
        T: Fn() -> Pin<Box<dyn Future<Output = ()> + Send>> + Send + Sync,
    {
        let created_at = Utc::now();
        let entry = TaskEntry { task, created_at };
        let duration =
            std::time::Duration::from_secs_f64(duration.get::<second>());
        let tasks = self.tasks.clone();
        let callback = Arc::new(callback);
        let job = Job::new_one_shot_at_instant_async(
            Instant::now() + duration,
            move |_, _| {
                let tasks = tasks.clone();
                let callback = callback.clone();
                Box::pin(async move {
                    tasks.lock().await.pop_front();
                    callback().await
                })
            },
        )?;
        self.scheduler
            .add(job)
            .await
            .context("Failed to add a job to the scheduler")?;
        self.tasks.lock().await.push_back(entry);
        Ok(())
    }
}
