use anyhow::{Context, Result};
use futures::Future;
use std::pin::Pin;
use std::time::Instant;
use tokio_cron_scheduler::{Job, JobScheduler};
use uom::si::f64::Time;
use uom::si::time::second;

pub struct Cron {
    scheduler:            JobScheduler,
    periodic_task_period: Time,
}

impl Cron {
    pub async fn new(periodic_task_period: Time) -> Result<Self> {
        let scheduler = JobScheduler::new()
            .await
            .context("Failed to create the cron job scheduler")?;
        scheduler.start().await.context("Failed to start the scheduler")?;

        Ok(Self { scheduler, periodic_task_period })
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
        callback: T,
    ) -> Result<()>
    where
        T: 'static,
        T: Fn() -> Pin<Box<dyn Future<Output = ()> + Send>> + Send + Sync,
    {
        let duration =
            std::time::Duration::from_secs_f64(duration.get::<second>());
        let job = Job::new_one_shot_at_instant_async(
            Instant::now() + duration,
            move |_, _| callback(),
        )?;
        self.scheduler
            .add(job)
            .await
            .context("Failed to add a job to the scheduler")?;
        Ok(())
    }
}
