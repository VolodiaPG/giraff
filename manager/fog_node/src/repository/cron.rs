use futures::Future;
use std::fmt::Debug;
use std::pin::Pin;
use tokio_cron_scheduler::{Job, JobScheduler, JobSchedulerError};
use uom::si::f64::Time;
use uom::si::time::second;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    JobScheduler(#[from] JobSchedulerError),
}

pub type CronFn =
    Box<dyn Fn() -> Pin<Box<dyn Future<Output = ()> + Send>> + Send + Sync>;

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

    pub async fn register_periodic(
        &self,
        callback: CronFn,
    ) -> Result<(), Error> {
        let toto = move |_, _| callback();
        let job = Job::new_async(
            format!(
                "1/{} * * * * *",
                self.periodic_task_period.get::<second>()
            )
            .as_str(),
            toto,
        )?;
        self.scheduler.add(job).await?;
        Ok(())
    }
}
