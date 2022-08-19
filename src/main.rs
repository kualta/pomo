use async_std::task::sleep;
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
        match self.deadline.checked_duration_since(now) {
            Some(time_left) => time_left,
            None => Duration::from_secs(0),
        }
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
            }
            TimerState::Working => return,
            TimerState::Resting => return,
        };

        self.state = TimerState::Working;
    }

    fn update(&mut self) {
        let time_left = self.time_left();
        if !time_left.is_zero() {
            return;
        }
        self.deadline = match self.state {
            TimerState::Working => {
                self.state = TimerState::Resting;
                Instant::now() + self.rest_duration
            }
            TimerState::Resting => {
                self.state = TimerState::Working;
                Instant::now() + self.work_duration
            }
            _ => return,
        }
    }
}
impl Display for PomoTimer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let time_left = match self.state {
            TimerState::Working => self.time_left(),
            TimerState::Resting => self.time_left(),
            TimerState::Paused(paused_at) => {
                match self.deadline.checked_duration_since(paused_at) {
                    Some(duration) => duration,
                    None => Duration::from_secs(0),
                }
            }
            TimerState::Inactive => Duration::from_secs(0),
        };

        let minutes_left = time_left.as_secs() / 60;
        let secs_left = time_left.as_secs() % 60;
        write!(f, "{}:{}", minutes_left, secs_left)
    }
}

fn App(cx: Scope) -> Element {
    cx.render(rsx! (
        body {
            Timer {}
        }
    ))
}

fn Timer(cx: Scope) -> Element {
    let timer = use_state(&cx, || {
        PomoTimer::new(Duration::from_secs(60 * 25), Duration::from_secs(60 * 5))
    });

    fixed_update(cx, timer);

    cx.render(rsx! (
        div {
            head { link { rel: "stylesheet", href: "https://unpkg.com/tailwindcss@^2.0/dist/tailwind.min.css" } }
            body {
                class: "flex justify-center items-center h-screen bg-gradient-to-bl from-pink-300 via-purple-300 to-indigo-400",
                div { 
                    class: "w-96 items-center",
                    h1 { 
                        class: "font-extrabold font-sans text-transparent text-8xl 
                                bg-clip-text bg-gradient-to-r from-purple-400 to-pink-600",
                        "{timer}" 
                    }
                    br {}
                    button { 
                        class: "w-1/3 text-gray-500 hover:text-gray-700 border border-gray-800 focus:outline-none 
                                font-medium rounded-lg text-sm px-5 py-2.5 text-center 
                                mr-2 mb-2 dark:border-gray-600 dark:text-gray-400 
                                dark:hover:text-white dark:hover:bg-gray-600 dark:focus:ring-gray-800",
                        onclick: |_| timer.make_mut().pause(), 
                        "Pause" 
                    }
                    button { 
                        class: "w-1/3 text-purple-500 hover:text-purple-700 border 
                                border-purple-500 focus:outline-none 
                                font-medium rounded-lg text-sm px-5 py-2.5 text-center 
                                mr-2 mb-2 dark:border-purple-400 dark:text-purple-400 
                                dark:hover:text-white dark:hover:bg-purple-500 dark:focus:ring-purple-900",
                        onclick: |_| timer.make_mut().resume(), 
                        "Resume" 
                    }
                }
            }
        }
    ))
}

fn fixed_update(cx: Scope, timer: &UseState<PomoTimer>) {
    use_coroutine(&cx, {
        to_owned![timer];
        |_: UnboundedReceiver<()>| async move {
            loop {
                timer.make_mut().update();
                sleep(Duration::from_secs(1)).await;
            }
        }
    });
}

fn main() {
    dioxus::web::launch(App);
}
