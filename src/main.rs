#![allow(unused, dead_code, non_snake_case)]

use async_std::task::sleep;
use dioxus::{core::to_owned, prelude::*};
use instant::*;
use std::fmt::Display;
use web_sys::HtmlAudioElement;

const PUBLIC_URL: &str = "/Pomodoro/";

#[derive(Clone, Copy)]
enum TimerState {
    Working,
    Resting,
    Paused(Instant),
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
            Some(time) => time,
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
        match self.deadline.checked_duration_since(Instant::now()) {
            Some(time_left) => time_left,
            None => Duration::from_secs(0),
        }
    }

    fn pause(&mut self) {
        if let TimerState::Paused(_) = self.state { 
            return; 
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
            _ => return,
        };

        self.state = TimerState::Working;
    }

    fn update(&mut self) {
        if self.time_left().is_zero() {
            self.flip();
        }
    }

    /// Flips the state of this [`PomoTimer`] and extends the deadline 
    fn flip(&mut self) {
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
        };
        self.ring();
    }

    fn ring(&self) {
        let bell_path = PUBLIC_URL.to_owned() + "assets/bell.mp3";
        HtmlAudioElement::new_with_src(&bell_path).unwrap().play().unwrap();
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
        };

        let minutes_left = time_left.as_secs() / 60;
        let secs_left = time_left.as_secs() % 60;
        write!(f, "{}:{:0>2}", minutes_left, secs_left)
    }
}

fn App(cx: Scope) -> Element {
    use_context_provider::<PomoTimer>(&cx, || { 
        PomoTimer::new(Duration::from_secs(25 * 60), Duration::from_secs(5 * 60)) 
    });
    let shared_timer = use_context::<PomoTimer>(&cx)?;

    shared_timer.write().update();

    cx.render(rsx! (
        body {
            class: "text-center flex justify-center items-center h-screen 
                    bg-gradient-to-bl from-pink-300 via-purple-300 to-indigo-400",
            div { 
                class: "w-96 items-center",
                Timer { }
                TimerControls { }
            }
        }
    ))
}

fn TimerControls(cx: Scope) -> Element {
    let shared_timer = use_context::<PomoTimer>(&cx)?;

    cx.render(rsx! (
        div { 
            class: "p-2",
            button { 
                class: "w-1/3 text-gray-500 hover:text-gray-700 border border-gray-800 focus:outline-none 
                        font-medium rounded-lg text-sm px-5 py-2.5 text-center 
                        mr-2 mb-2 dark:border-gray-600 dark:text-gray-400 
                        dark:hover:text-white dark:hover:bg-gray-600 dark:focus:ring-gray-800",
                onclick: move |_| shared_timer.write().pause(), 
                "Pause" 
            }
            button { 
                class: "w-1/3 text-purple-500 hover:text-purple-700 border 
                        border-purple-500 focus:outline-none 
                        font-medium rounded-lg text-sm px-5 py-2.5 text-center 
                        mr-2 mb-2 dark:border-purple-400 dark:text-purple-400 
                        dark:hover:text-white dark:hover:bg-purple-500 dark:focus:ring-purple-900",
                onclick: move |_| shared_timer.write().resume(), 
                "Resume" 
            }
        }
    ))
}

fn Timer(cx: Scope) -> Element {
    let shared_timer = use_context::<PomoTimer>(&cx)?;
    let mut timer = shared_timer.write();

    cx.render(rsx! (
        h1 { 
            class: "font-extrabold font-sans text-transparent text-8xl 
                    bg-clip-text bg-gradient-to-r from-purple-400 to-pink-600",
            "{timer}" 
        }
    ))
}

fn main() {
    dioxus::web::launch(App);
}
