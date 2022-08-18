use std::fmt::{Display};

use dioxus::{core::to_owned, prelude::*};
use instant::*;

struct PomoTimer {
    work_duration: Duration,
    rest_duration: Duration,
    deadline: Instant,
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
        }
    }
    fn time_left(&self) -> Duration {
        let now = Instant::now();
        self.deadline.duration_since(now)
    }
}
impl Display for PomoTimer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let time_left = self.time_left();
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
    let time_label = use_state(&cx, || { " ".to_owned() });

    let _: &CoroutineHandle<()> = use_coroutine(&cx, {
        to_owned![timer];
        to_owned![time_label];
        |_| async move {
            loop {
                time_label.set(format!("{timer}"));
                async_std::task::sleep(Duration::from_secs(1)).await;
            }
        }
    });

    cx.render(rsx!(
        h1 { "{time_label}" }
    ))
}
