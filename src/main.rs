#![allow(unused, dead_code, non_snake_case)]

use async_std::task::sleep;
use dioxus::{core::to_owned, prelude::*};
use dioxus_helmet::Helmet;
use instant::*;
use std::fmt::Display;
use wasm_bindgen::__rt::Start;
use web_sys::HtmlAudioElement;

const PUBLIC_URL: &str = "/";

#[derive(Clone, Copy)]
enum TimerState {
    Inactive,
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
        let deadline = Instant::now()
            .checked_add(work_duration)
            .unwrap_or_else(Instant::now);

        PomoTimer {
            work_duration,
            rest_duration,
            deadline,
            state: TimerState::Inactive,
        }
    }

    fn start(&mut self) {
        match self.state {
            TimerState::Inactive => {
                if let Duration::ZERO = self.work_duration {
                    return;
                }
                self.deadline = Instant::now()
                    .checked_add(self.work_duration)
                    .unwrap_or_else(Instant::now);
                self.state = TimerState::Working;
            }
            TimerState::Paused(paused_at) => {
                self.deadline += Instant::now()
                    .checked_duration_since(paused_at)
                    .unwrap_or(Duration::ZERO);
                // FIXME: Incorrect if paused during rest
                self.state = TimerState::Working;
            }
            _ => (),
        }
    }

    fn stop(&mut self) {
        match self.state {
            TimerState::Working | TimerState::Resting => {
                self.state = TimerState::Paused(Instant::now())
            }
            _ => (),
        }
    }

    fn reset(&mut self) {
        self.state = TimerState::Inactive;
    }

    fn update(&mut self) {
        match self.state {
            TimerState::Working | TimerState::Resting => {
                if self.time_left().is_zero() {
                    self.flip();
                }
            }
            _ => (),
        }
    }

    fn time_left(&self) -> Duration {
        self.deadline
            .checked_duration_since(Instant::now())
            .unwrap_or(Duration::ZERO)
    }

    /// Increases work duration of this [`PomoTimer`].
    ///
    /// Rest duration is defined as `1/5` of the work duration
    fn increase_duration(&mut self, increase: Duration) {
        let duration = self
            .work_duration
            .checked_add(increase)
            .unwrap_or(Duration::MAX);
        self.work_duration = duration;
        self.rest_duration = duration / 5;
        if let Some(deadline) = self.deadline.checked_add(increase) {
            self.deadline = deadline;
        }
    }

    /// Decreases work duration of this [`PomoTimer`].
    ///
    /// Rest duration is defined as `1/5` of the work duration
    fn decrease_duration(&mut self, decrease: Duration) {
        let mut duration = self
            .work_duration
            .checked_sub(decrease)
            .unwrap_or(Duration::from_secs(5 * 60));
        if duration < Duration::from_secs(5 * 60) {
            duration = Duration::from_secs(5 * 60);
        }
        self.work_duration = duration;
        self.rest_duration = duration / 5;
        if let Some(deadline) = self.deadline.checked_sub(decrease) {
            self.deadline = deadline;
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
            TimerState::Inactive => {
                self.state = TimerState::Working;
                Instant::now() + self.work_duration
            }
            TimerState::Paused(_) => {
                // FIXME: incorrect if paused at rest
                self.state = TimerState::Working;
                Instant::now() + self.work_duration
            }
        };
        self.ring();
    }

    fn toggle_pause(&mut self) {
        match self.state {
            TimerState::Working | TimerState::Resting => self.stop(),
            TimerState::Paused(_) | TimerState::Inactive => self.start(),
        }
    }

    fn ring(&self) {
        let bell_path = PUBLIC_URL.to_owned() + "assets/bell.mp3";
        HtmlAudioElement::new_with_src(&bell_path)
            .unwrap()
            .play()
            .unwrap();
    }
}

impl Display for PomoTimer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let time_left = match self.state {
            TimerState::Paused(paused_at) => self
                .deadline
                .checked_duration_since(paused_at)
                .unwrap_or(Duration::ZERO),
            TimerState::Inactive => self.work_duration,
            _ => self.time_left(),
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

    let state = shared_timer.write().state;
    let icon_path = match state {
        TimerState::Inactive | TimerState::Working => "assets/icon_work.png",
        TimerState::Resting | TimerState::Paused(_) => "assets/icon_rest.png",
    };

    cx.render(rsx! (
        body {
            class: "text-center flex justify-center items-center h-screen
                    bg-gradient-to-bl from-pink-300 via-purple-300 to-indigo-400",
            tabindex: "-1",
            // FIXME: add listener on main body instead 
            onkeypress: move |evt| {
                match &*evt.key {
                    "f" => shared_timer.write().flip(),
                    "i" => shared_timer.write().increase_duration(Duration::from_secs(5 * 60)),
                    "n" => shared_timer.write().reset(),
                    "d" => shared_timer.write().decrease_duration(Duration::from_secs(5 * 60)),
                    " " => shared_timer.write().toggle_pause(),
                    "p" => shared_timer.write().toggle_pause(),
                    _ => (),
                }
            },
            div {
                class: "w-96 p-1",
                PageIcon { path: icon_path.to_owned() }
                Timer { }
                TimerControls { }
                HelpText { }
                CreditsText { }
            }
        }
    ))
}

#[rustfmt::skip]
fn CreditsText(cx: Scope) -> Element {
    let emoji = use_state(&cx, || {
        if now() as i64 % 2 == 0 { "🦀" } else { "❤️️" }
    });

    cx.render(rsx!(
        div {
            class: "block absolute bottom-5 right-5 text-gray-600 font-base ",
            div {
                "made with {emoji} by ",
                span {
                    class: "underline decoration-blue-500",
                    a { href: "https://kualta.dev/", "kualta"}
                }
            }
        }
    ))
}

fn HelpText(cx: Scope) -> Element {
    let shared_timer = use_context::<PomoTimer>(&cx)?;
    let state = shared_timer.write().state;
    match state {
        TimerState::Inactive => (),
        _ => return None,
    }

    let kbd_css = "px-2 py-1 text-sm font-bold text-gray-800 
    bg-purple-200 border border-gray-500 rounded-lg dark:bg-gray-600 
    dark:text-gray-100 dark:border-gray-500";

    cx.render(rsx!(
        div {
            class: "flex flex-col text-center",
            div {
                h3 { class: "pt-5 text-gray-800 font-medium", "Pro Gamer Tips: " }
            }
            div {
                class: "flex-grow text-gray-700 dark:text-gray-500 text-lg text-left mx-auto",
                    kbd { class: "{kbd_css}", "f" } " lip the timer" br { }
                    kbd { class: "{kbd_css}", "i" } " ncrease duration" br { }
                    kbd { class: "{kbd_css}", "n" } " ew timer" br { }
                    kbd { class: "{kbd_css}", "d" } " ecrease duration" br { }
                    kbd { class: "{kbd_css}", "p" } " ause " br { }
                    kbd { class: "{kbd_css}", "Ctrl" } kbd { class: "{kbd_css}", "+" } " / "
                    kbd { class: "{kbd_css}", "-" } " change size " br { }
            }
        }
    ))
}

#[inline_props]
fn PageIcon(cx: Scope, path: String) -> Element {
    cx.render(rsx!(
        Helmet {
            link { rel: "icon", href: "{path}"}
        }
    ))
}

fn TimerControls(cx: Scope) -> Element {
    let shared_timer = use_context::<PomoTimer>(&cx)?;
    let state = shared_timer.write().state;
    let controls = match state {
        TimerState::Inactive => {
            rsx!(
                button {
                    class: "text-gray-500 hover:text-gray-700 border border-gray-800 focus:outline-none
                            font-medium rounded-lg text-sm px-4 py-2.5 text-center 
                            m-1 dark:border-gray-600 dark:text-gray-400 
                            dark:hover:text-white dark:hover:bg-gray-600 dark:focus:ring-gray-800",
                    onclick: move |_| shared_timer.write().decrease_duration(Duration::from_secs(5 * 60)),
                    "-"
                }
                button {
                    class: "w-1/2 text-purple-500 hover:text-purple-700 border border-purple-500 focus:outline-none
                            font-medium rounded-lg text-sm py-2.5 text-center 
                            m-1 dark:border-purple-400 dark:text-purple-400 
                            dark:hover:text-white dark:hover:bg-purple-500 dark:focus:ring-purple-900",
                    onclick: move |_| shared_timer.write().start(),
                    "Start"
                }
                button {
                    class: "text-gray-500 hover:text-gray-700 border border-gray-800 focus:outline-none
                            font-mono font-medium rounded-lg text-sm px-4 py-2.5 text-center 
                            m-1 dark:border-gray-600 dark:text-gray-400 
                            dark:hover:text-white dark:hover:bg-gray-600 dark:focus:ring-gray-800",
                    onclick: move |_| shared_timer.write().increase_duration(Duration::from_secs(5 * 60)),
                    "+"
                }
            )
        }
        TimerState::Working | TimerState::Resting => {
            rsx!(
                button {
                    class: "w-1/2 text-gray-500 hover:text-gray-700 border border-gray-800 focus:outline-none
                            font-medium rounded-lg text-sm px-5 py-2.5 text-center 
                            m-1 dark:border-gray-600 dark:text-gray-400 
                            dark:hover:text-white dark:hover:bg-gray-600 dark:focus:ring-gray-800",
                    onclick: move |_| shared_timer.write().stop(),
                    "Pause"
                }
            )
        }
        TimerState::Paused(_) => {
            rsx!(
                button {
                    class: "w-1/2 text-purple-500 hover:text-purple-700 border border-purple-500 focus:outline-none
                            font-medium rounded-lg text-sm px-5 py-2.5 text-center 
                            m-1 dark:border-purple-400 dark:text-purple-400 
                            dark:hover:text-white dark:hover:bg-purple-500 dark:focus:ring-purple-900",
                    onclick: move |_| shared_timer.write().start(),
                    "Resume"
                }
            )
        }
    };

    cx.render(rsx!(controls))
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
