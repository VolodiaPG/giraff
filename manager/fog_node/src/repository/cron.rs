use anyhow::Result;
use chrono::{DateTime, Utc};
use futures::Future;
use model::SlaId;
use num_traits::ToPrimitive;
use std::collections::VecDeque;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use uom::si::f64::Time;
use uom::si::time::{millisecond, second};

pub struct Cron {
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
        Ok(Self {
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
        let interval_duration = Duration::from_secs(
            self.periodic_task_period
                .clone()
                .get::<second>()
                .ceil()
                .to_u64()
                .unwrap(),
        );
        let mut interval = tokio::time::interval(interval_duration);

        tokio::spawn(async move {
            loop {
                interval.tick().await;
                tokio::spawn(callback());
            }
        });
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
        let tasks = self.tasks.clone();
        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(
                duration.get::<millisecond>().ceil().to_u64().unwrap(),
            ))
            .await;

            tasks.lock().await.pop_front();
            callback().await
        });

        let entry = TaskEntry { task, created_at: Utc::now() };
        self.tasks.lock().await.push_back(entry);
        Ok(())
    }
}
