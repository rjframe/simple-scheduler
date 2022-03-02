/* This Source Code Form is subject to the terms of the Mozilla Public
   License, v. 2.0. If a copy of the MPL was not distributed with this
   file, You can obtain one at http://mozilla.org/MPL/2.0/.
*/

//! Simple scheduler for periodic tasks.
//!
//! Scheduled tasks are run asynchronously.
//!
//! This is not a precise scheduler; tasks may run at the exact interval or time
//! specified.
//!
//! # Examples
//!
//! ```
//! use simple_scheduler::{
//!     // `Time` is a chrono::naive::NaiveTime.
//!     // Also exports chrono::DateTime<chrono::Utc> as DateTime.
//!     Duration, Time,
//!     Schedule, ScheduleAt, ScheduledTask, task,
//! };
//!
//! use chrono::Utc;
//!
//!
//! fn main() {
//!     let periodic_task = ScheduledTask::new(
//!         ScheduleAt::Interval(Duration::seconds(4)),
//!         task!(async { println!("Approximately four seconds have passed.") })
//!     ).unwrap();
//!
//!     let onetime_task = ScheduledTask::new(
//!         ScheduleAt::DateTime(Utc::now() + Duration::seconds(15)),
//!         task!(task2())
//!     ).unwrap();
//!
//!     let daily_task = ScheduledTask::new(
//!         ScheduleAt::Daily(Time::from_hms(01, 23, 45)),
//!         task!(async { println!("Hello"); })
//!     ).unwrap();
//!
//!     let schedule = Schedule::builder()
//!         .tasks([
//!             periodic_task,
//!             onetime_task,
//!             daily_task,
//!         ])
//!         .wake_interval(Duration::seconds(1)).unwrap()
//!         .build();
//!
//!     schedule.run();
//!
//!     // run() exits immediately, so we need to wait a bit.
//!     std::thread::sleep(Duration::seconds(30).to_std().unwrap());
//! }
//!
//! async fn task2() {
//!     println!("Function executed");
//! }
//! ```

use std::pin::Pin;

pub use chrono::{
    naive::NaiveTime as Time,
    Duration,
};

use futures::future::{self, Future};

// TODO: Non-str error types.
// TODO: I may want to wrap chrono::Duration to prevent creation of negative
// durations.

/// Create a task from an async code block.
#[macro_export]
macro_rules! task {
    ($task:expr) => { Box::new(move || Box::pin($task)) };
}

// TODO: Return a Result type from tasks?
type Task = Box<dyn Fn() -> Pin<Box<dyn Future<Output = ()>>> + Send + 'static>;
pub type DateTime = chrono::DateTime<chrono::Utc>;

/// The execution schedule for a particular task.
pub enum ScheduleAt {
    /// Run a task at a regular interval.
    ///
    /// The first run will be immediately upon calling [Schedule::run], then
    /// will repeat at the period set here.
    ///
    /// The interval must be non-negative. A task with a duration of `0` will
    /// run on every wake timer.
    Interval(Duration),
    /// Run a one-time task at the given date and time.
    DateTime(DateTime),
    /// Run a task every day at the specified time.
    Daily(chrono::naive::NaiveTime),
    // TODO: Weekly
}

/// A task to run on a schedule.
pub struct ScheduledTask {
    schedule: ScheduleAt,
    task: Task,
    last_run: Option<DateTime>,
}

impl ScheduledTask {
    /// Create a new scheduled task.
    ///
    /// If the schedule includes a [Duration] less than `0`, returns an error.
    pub fn new(schedule: ScheduleAt, task: Task) -> Result<Self, &'static str> {
        if let ScheduleAt::Interval(dur) = schedule {
            if dur < Duration::zero() {
                return Err("Schedule interval must not be negative");
            }
        }

        Ok(Self {
            schedule,
            task,
            last_run: None,
        })
    }
}

/// Builder object for a [Schedule].
pub struct ScheduleBuilder {
    tasks: Vec<ScheduledTask>,
    wake_interval: Duration,
}

impl Default for ScheduleBuilder {
    fn default() -> Self {
        Self {
            tasks: vec![],
            wake_interval: Duration::minutes(1),
        }
    }
}

impl ScheduleBuilder {
    /// Add a task to the schedule.
    pub fn task(mut self, task: ScheduledTask) -> Self {
        self.tasks.push(task);
        self
    }

    /// Add multiple tasks to the schedule.
    pub fn tasks(mut self, tasks: impl Into<Vec<ScheduledTask>>) -> Self {
        self.tasks.append(&mut tasks.into());
        self
    }

    /// Set the interval at which to check for tasks to run.
    ///
    /// The default interval is 1 minute. The minimum is 100 nanoseconds.
    pub fn wake_interval(mut self, interval: Duration)
    -> Result<Self, &'static str> {
        if interval < Duration::nanoseconds(100) {
            Err("Wake interval must be at least 100 nanoseconds")
        } else {
            self.wake_interval = interval;
            Ok(self)
        }
    }

    pub fn build(self) -> Schedule {
        Schedule {
            tasks: self.tasks,
            wake_interval: self.wake_interval,
        }
    }
}

/// A scheduler to run tasks.
///
/// This is not a precise scheduler; tasks may run later than scheduled.
pub struct Schedule {
    // TODO: With a large number of tasks, probably better to use
    // FuturesUnordered. Would we notice the extra overhead for a small number
    // of tasks? Probably not.
    tasks: Vec<ScheduledTask>,
    wake_interval: Duration,
}

impl Schedule {
    pub fn builder() -> ScheduleBuilder {
        ScheduleBuilder::default()
    }

    /// Run the scheduled tasks at their configured times.
    pub fn run(mut self) {
        std::thread::spawn(move || {
            loop {
                self.run_tasks();
                // Unwrap safety: wake_interval cannot be created as negative.
                std::thread::sleep(self.wake_interval.to_std().unwrap());
            }
        });
    }

    fn run_tasks(&mut self) {
        let mut run_tasks = vec![];
        let now = chrono::Utc::now();

        for task in self.tasks.iter_mut() {
            match task.schedule {
                ScheduleAt::Interval(dur) => {
                    // If we haven't run the task yet, run it now.
                    let last_run = task.last_run.unwrap_or_else(|| now - dur);

                    if last_run + dur <= now {
                        task.last_run = Some(now);
                        run_tasks.push((task.task)());
                    }
                },
                ScheduleAt::DateTime(dt) => {
                    // We're assuming that if the clock is set backwards, it's
                    // better not to run the task again.
                    // TODO: Remove the task after execution?
                    if task.last_run.is_none() && now >= dt {
                        task.last_run = Some(now);
                        run_tasks.push((task.task)());
                    }
                },
                ScheduleAt::Daily(time) => {
                    // `task.last_run` tracks daily runs for this schedule type.
                    if now.time() < time {
                        task.last_run = None;
                    }

                    if task.last_run.is_none() && now.time() >= time {
                        task.last_run = Some(now);
                        run_tasks.push((task.task)());
                    }
                }
            }
        }

        if ! run_tasks.is_empty() {
            futures::executor::block_on(async {
                future::join_all(run_tasks).await
            });
        }
    }
}
