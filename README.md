# Simple Task Scheduler

Schedule repeated tasks to run asynchronously.

This is not a precise scheduler; tasks may not run (probably won't run) at the
exact interval you set.

Useful links:

* Code at [SourceHut](https://git.sr.ht/~rjframe/simple-scheduler) or at
  [Github](https://github.com/rjframe/simple-scheduler)
* [Issues/requests at SourceHut](https://todo.sr.ht/~rjframe/simple-scheduler)
* [Discussions at SourceHut](https://lists.sr.ht/~rjframe/public)

`simple_scheduler` uses the executor provided by the `futures` crate. If you
need or prefer another executor, I would accept PRs to optionally use it.

Why another task scheduler? When I took a look at what was available, everything
I found used cron expressions (which I don't want in a library API), did not run
the tasks asynchronously, and/or required tokio, which I wasn't using.

```rust
use simple_scheduler::{
    // Also exports chrono::DateTime<chrono::Utc> as DateTime.
    Duration, Time,
    Schedule, ScheduleAt, ScheduledTask, task,
};

use chrono::Utc;


fn main() {
    let periodic_task = ScheduledTask::new(
        ScheduleAt::Interval(Duration::seconds(4)),
        task!(async { println!("Do something every four seconds") })
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
```

I use this to run maintenance tasks on servers; I have arbitrarily set a minimum
`wake_interval` of 100 nanoseconds; I have not tested it at that rapid a period
or whether a lower limit makes sense.


## License

All library source code is licensed under the terms of the
[MPL 2.0 license](LICENSE.txt).

The source code of examples is licensed under the terms of the
[MIT license](examples/LICENSE.txt).


## Contributing

Patches and pull requests are welcome. For major features or breaking changes,
please open a ticket or start a discussion first so we can discuss what you
would like to do.


## Contact

- Email: code@ryanjframe.com
- Website: [www.ryanjframe.com](https://www.ryanjframe.com)
- diaspora*: rjframe@diasp.org


## Related Projects

* [clockwerk](https://crates.io/crates/clokwerk)
* [scheduling](https://crates.io/crates/scheduling)
* [tokio-cron-scheduler](https://crates.io/crates/tokio-cron-scheduler)
* [And many others](https://crates.io/search?page=1&per_page=10&q=task%20scheduler)
