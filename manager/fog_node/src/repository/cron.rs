use futures::Future;
use std::fmt::Debug;
use std::pin::Pin;
use std::time::Instant;
use tokio_cron_scheduler::{Job, JobScheduler, JobSchedulerError};
use uom::si::f64::Time;
use uom::si::time::second;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    JobScheduler(#[from] JobSchedulerError),
}

pub struct Cron {
    scheduler:            JobScheduler,
    periodic_task_period: Time,
}

impl Cron {
    pub async fn new(periodic_task_period: Time) -> Result<Self, Error> {
        let scheduler = JobScheduler::new().await?;
        scheduler.start().await?;

        Ok(Self { scheduler, periodic_task_period })
    }

    pub async fn add_periodic<T>(&self, callback: T) -> Result<(), Error>
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
        self.scheduler.add(job).await?;
        Ok(())
    }

    pub async fn add_oneshot<T>(
        &self,
        duration: Time,
        callback: T,
    ) -> Result<(), Error>
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
        self.scheduler.add(job).await?;
        Ok(())
    }
}
