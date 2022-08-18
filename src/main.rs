use dioxus::{core::to_owned, prelude::*};
use instant::*;
use std::{fmt::Display, ops::Add};

#[derive(Clone, Copy)]
enum TimerState {
    Working,
    Resting,
    Paused(Instant),
    Inactive,
}

#[derive(Clone, Copy)]
struct PomoTimer {
    work_duration: Duration,
    rest_duration: Duration,
    deadline: Instant,
    state: TimerState,
}
impl PomoTimer {
    fn new(work_duration: Duration, rest_duration: Duration) -> Self {
        let deadline = match Instant::now().checked_add(work_duration) {
            Some(t) => t,
            None => Instant::now(),
        };
        PomoTimer {
            work_duration,
            rest_duration,
            deadline,
            state: TimerState::Working,
        }
    }

    fn time_left(&self) -> Duration {
        let now = Instant::now();
        self.deadline.duration_since(now)
    }

    fn pause(&mut self) {
        match self.state {
            TimerState::Paused(_) => return,
            TimerState::Inactive => return,
            _ => {}
        }
        self.state = TimerState::Paused(Instant::now());
    }

    fn resume(&mut self) {
        match self.state {
            TimerState::Paused(paused_at) => {
                self.deadline += Instant::now()
                    .checked_duration_since(paused_at)
                    .unwrap_or(Duration::from_secs(0));
            }
            TimerState::Inactive => {
                self.deadline = Instant::now() + self.work_duration;
            },
            TimerState::Working => {
                return
            }
            TimerState::Resting => {
                return
            },
        };

        self.state = TimerState::Working;
    }
}
impl Display for PomoTimer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let time_left = match self.state {
            TimerState::Working => self.time_left(),
            TimerState::Resting => self.time_left(),
            TimerState::Paused(paused_at) => self.deadline.duration_since(paused_at),
            TimerState::Inactive => Duration::from_secs(0),
        };

        let minutes_left = time_left.as_secs() / 60;
        let secs_left = time_left.as_secs() % 60;
        write!(f, "{}:{}", minutes_left, secs_left)
    }
}

fn main() {
    dioxus::web::launch(app);
}

fn app(cx: Scope) -> Element {
    let timer = use_state(&cx, || {
        PomoTimer::new(Duration::from_secs(60 * 25), Duration::from_secs(60 * 5))
    });

    let _: &CoroutineHandle<()> = use_coroutine(&cx, {
        to_owned![timer];
        |_| async move {
            loop {
                timer.needs_update();
                async_std::task::sleep(Duration::from_secs(1)).await;
            }
        }
    });

    cx.render(rsx!(
        h1 { "{timer}" }
        
        button { 
            onclick: |evt| timer.make_mut().pause(),
            "Pause"
        }
        button { 
            onclick: |evt| timer.make_mut().resume(),
            "Resume"
        }
    ))
}
