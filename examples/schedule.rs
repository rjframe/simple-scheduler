use simple_scheduler::{
    Duration, Time,
    Schedule, ScheduleAt, ScheduledTask, task,
};

use chrono::Utc;


fn main() {
    let periodic_task = ScheduledTask::new(
        ScheduleAt::Interval(Duration::seconds(4)),
        task!(async { println!("You'll see me every four seconds") })
    ).unwrap();

    let onetime_task = ScheduledTask::new(
        ScheduleAt::DateTime(Utc::now() + Duration::seconds(15)),
        task!(task2())
    ).unwrap();

    let daily_task = ScheduledTask::new(
        ScheduleAt::Daily(Time::from_hms(01, 23, 45)),
        task!(async { println!("Hello"); })
    ).unwrap();

    let schedule = Schedule::builder()
        .tasks([
            periodic_task,
            onetime_task,
            daily_task,
        ])
        .wake_interval(Duration::seconds(1)).unwrap()
        .build();

    schedule.run();

    // run() exits immediately, so we need to wait a bit.
    std::thread::sleep(Duration::seconds(30).to_std().unwrap());
}

async fn task2() {
    println!("Function executed");
}
