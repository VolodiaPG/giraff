use tokio_cron_scheduler::{Job, JobScheduler};

pub fn cron_init() {
    let sched = JobScheduler::new().unwrap();

    // TODO option to configure ?
    sched
        .add(Job::new("1/5 * * * * *", |_, _| ping()).unwrap())
        .unwrap();

    sched.start().unwrap();
}

fn ping() {
    println!("ping");
}
