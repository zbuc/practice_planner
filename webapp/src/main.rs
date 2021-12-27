#![recursion_limit = "1024"]
use chrono::DateTime;
use std;
use std::collections::BTreeMap;
use std::collections::HashSet;
use std::ops::Sub;
#[allow(unused_imports)]
use yew::prelude::*;
use yew_agent::{Dispatched, Dispatcher};

use anyhow::Result;
use chrono::{Date, Utc};
use gloo::storage::{LocalStorage, Storage};
use gloo::timers::callback::Interval;
use hhmmss::Hhmmss;
use log;
use patternfly_yew::*;
use pplib::PracticeSession;
use yew::{html, html::Scope, Classes, Component, Context, Html};

use crate::components::audio_player::*;
use crate::components::event_bus::{EventBus, Request};
use pplib::{PracticeCategory, SchedulePlanner};

mod components;

// Use `wee_alloc` as the global allocator.
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

const CONFIG_KEY: &str = "yew.practiceplanner.config";
const HISTORY_KEY: &str = "yew.practiceplanner.history";

pub enum Msg {
    StartPracticing,
    StopPracticing,
    PracticeTick,
    ResetDataPrompt,
    ResetData,
    ShuffleToday,
}

// Splitting this out makes local debugging easier
pub fn get_current_time() -> DateTime<Utc> {
    // Utc::now().sub(Duration::days(3))
    Utc::now()
}

pub struct PracticePlannerApp {
    // state: State,
    // focus_ref: NodeRef,
    scheduler: SchedulePlanner<'static>,
    interval: Option<Interval>,
    event_bus: Dispatcher<EventBus>,
}

impl PracticePlannerApp {
    fn view_category(
        &self,
        (idx, category): (usize, &PracticeCategory),
        active: u64,
        practicing: bool,
        _link: &Scope<Self>,
    ) -> Html {
        let mut class = Classes::from("todo");
        if practicing && active as usize == idx {
            class.push("active-category");
        } else if practicing && active as usize > idx {
            class.push("completed-category")
        }
        log::info!("making the category item");
        html! {
            <li {class}>
                <div class="view">
                    <input
                        type="checkbox"
                        class="toggle"
                        checked={practicing && active as usize > idx}
                        disabled=true
                    />
                    <label>{ &category.category_name}</label>
                </div>
                // { self.view_entry_edit_input((idx, category), link) }
            </li>
        }
    }

    fn view_history_list(
        &self,
        history_list: BTreeMap<Date<Utc>, HashSet<&PracticeCategory>>,
        _link: &Scope<Self>,
    ) -> Html {
        let _class = Classes::from("todo");
        log::info!("making the history list");

        if history_list.is_empty() {
            return html! {
                <strong>{ "No history" }</strong>
            };
        }

        let hl = history_list
            .iter()
            .map(|(day, day_categories)| {
                let mut dc = day_categories
                    .iter()
                    .map(|cat| format!("{}", cat))
                    .collect::<Vec<_>>();
                dc.sort();
                html! { <li><strong>{ day }</strong>{ dc.join(", ") }</li> }
            })
            .collect::<Vec<_>>();
        html! {
            <ul class="history-list">
            { hl }
            </ul>
        }
    }

    fn save(&self) -> Result<()> {
        // TODO need to bubble this error up actually
        log::info!("Saving...");
        LocalStorage::set(CONFIG_KEY, &self.scheduler.config).expect("able to save");
        LocalStorage::set(HISTORY_KEY, &self.scheduler.history).expect("able to save");
        Ok(())
    }

    fn view_category_list(
        &self,
        practice_session: &Option<PracticeSession>,
        link: &Scope<Self>,
    ) -> Html {
        log::info!("making the category list");
        let active = match practice_session {
            Some(ps) => ps.current_category,
            None => 0,
        };
        let practicing = self.scheduler.practicing;

        html! {
            <>
            <ul class="category-list">
            {
                if self.scheduler.get_todays_schedule().is_some() {
                    log::info!("got a schedule for today");
                    html! { for self.scheduler.get_todays_schedule().unwrap().iter().enumerate().map(|e| self.view_category(e, active, practicing, link)) }
                } else {
                    log::info!("no schedule available :(");
                    html! {}
                }
            }
            </ul>
            </>
        }
    }
}

impl Component for PracticePlannerApp {
    type Message = Msg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        let config = LocalStorage::get(CONFIG_KEY);
        let history = LocalStorage::get(HISTORY_KEY);
        let mut scheduler = match config {
            Ok(conf) => {
                log::info!("Found saved data: {:#?}", history);
                SchedulePlanner {
                    config: conf,
                    history: history.unwrap_or_default(),
                    todays_schedule: None,
                    practicing: false,
                    practice_session: None,
                }
            }
            Err(_e) => {
                log::info!("Did not find saved data");
                SchedulePlanner::new()
            }
        };

        let current_time = get_current_time();
        scheduler
            .update_todays_schedule(false, current_time)
            .expect("Unable to update today's schedule");
        Self {
            scheduler,
            interval: None,
            event_bus: EventBus::dispatcher(),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::ResetDataPrompt => {
                // TODO this would be better as a modal probably but there's
                // no easy way to trigger those in patternfly-yew
                let fix = ctx
                    .link()
                    .callback(|_| Msg::ResetData)
                    .into_action("Reset Data");
                let toast = Toast {
                    title: "Are you sure?".into(),
                    r#type: Type::Danger,
                    body: html! {
                        <p>{"Are you sure you'd like to reset all your configuration and history?"}</p>
                    },
                    actions: vec![fix.clone()],
                    ..Default::default()
                };
                ToastDispatcher::new().toast(toast);
            }
            Msg::ResetData => {
                self.scheduler = SchedulePlanner::new();
                let current_time = get_current_time();
                self.scheduler
                    .update_todays_schedule(false, current_time)
                    .expect("able to update schedule");
                self.save().expect("umable to save");
            }
            Msg::ShuffleToday => {
                if !self.scheduler.practicing {
                    let current_time = get_current_time();
                    self.scheduler
                        .update_todays_schedule(true, current_time)
                        .expect("able to update schedule");
                }
            }
            Msg::StopPracticing => {
                let current_time = get_current_time();
                self.scheduler
                    .update_todays_schedule(false, current_time)
                    .expect("able to update schedule");
                // save state
                self.save().expect("unable to save");
                self.scheduler
                    .stop_practicing()
                    .expect("failed to stop practicing");
                if let Some(timer) = self.interval.take() {
                    drop(timer);
                }
            }
            Msg::PracticeTick => {
                log::info!("Tick");

                let now = get_current_time();
                let time_elapsed = now.sub(
                    self.scheduler
                        .practice_session
                        .as_ref()
                        .unwrap()
                        .category_start_time,
                );
                let total_time = self.scheduler.config.category_practice_time;

                if time_elapsed > total_time {
                    // move to next category
                    self.scheduler
                        .advance_practice_session(now)
                        .expect("unable to advance");

                    // play a ding sound
                    self.event_bus
                        .send(Request::EventBusMsg("ding.wav".to_owned()));

                    if !self.scheduler.practicing {
                        if let Some(timer) = self.interval.take() {
                            drop(timer);
                        }
                        self.save().expect("unable to save");
                        self.scheduler
                            .update_todays_schedule(false, now)
                            .expect("unable to update schedule");
                    }
                }
                let time_left = total_time - time_elapsed;
                self.scheduler
                    .practice_session
                    .as_mut()
                    .unwrap()
                    .set_time_left(time_left);
            }
            Msg::StartPracticing => {
                let current_time = get_current_time();
                self.scheduler
                    .start_daily_practice(current_time)
                    .expect("failed daily practice");
                self.scheduler.practice_session.as_mut().unwrap().start_time = current_time;
                self.scheduler
                    .practice_session
                    .as_mut()
                    .unwrap()
                    .category_start_time = current_time;
                self.scheduler.practice_session.as_mut().unwrap().time_left =
                    self.scheduler.config.category_practice_time;
                let handle = {
                    let link = ctx.link().clone();
                    Interval::new(100, move || link.send_message(Msg::PracticeTick))
                };
                self.interval = Some(handle);
            }
        }

        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let _hidden_class = "hidden";
        if self.scheduler.practicing {
            log::info!("currently practicing");
        }

        let current_time = get_current_time();
        let streak = self.scheduler.get_streak(current_time);
        let practicing = self.scheduler.practicing;
        let _category_list = self.scheduler.get_todays_schedule();
        // TODO use a constant here
        let history_list = self
            .scheduler
            .get_history_n_days_back(3, current_time)
            .expect("unable to retrieve history");
        html! {
            <>
            <ToastViewer/>
            <Bullseye>
                <Grid gutter=true>
                    <GridItem cols={[12]}>
                        <h1 class="pf-u-text-align-center">{ "Planner" }</h1>
                    </GridItem>
                    <GridItem cols={[6]} rows={[4]}>
                        <h2 class="history-list">{ "Practice History" }</h2>
                        {self.view_history_list(history_list, ctx.link())}
                        <p>{ "Streak: " }<strong>{ streak }{ " days" }</strong></p>
                        <button class="favorite styled"
                                type="button"
                                onclick={ctx.link().callback(|_| Msg::ResetDataPrompt)}
                                >
                            { "Reset History" }
                        </button>
                    </GridItem>
                    <GridItem cols={[6]}>
                        <h2 class="category-list">{ "Today's Schedule" }</h2>
                        {self.view_category_list(&self.scheduler.practice_session, ctx.link())}
                            // { for self.state.entries.iter().filter(|e| self.state.filter.fits(e)).enumerate().map(|e| self.view_entry(e, ctx.link())) }
                        { if practicing {
                            html!{
                                <>
                                <h3>{ "Time left: " }{ self.scheduler.practice_session.as_ref().unwrap().time_left.hhmmss() }</h3>
                                <button class="favorite styled"
                                        type="button"
                                        onclick={ctx.link().callback(|_| Msg::StopPracticing)}
                                        >
                                        {"Stop Practicing"}
                                </button>
                                </>
                            }
                        } else {
                            html!{
                                <button class="favorite styled"
                                        type="button"
                                        onclick={ctx.link().callback(|_| Msg::StartPracticing)}
                                        >
                                        {"Start Practicing"}
                                </button>
                            }
                        }}
                        <button class="favorite styled"
                                type="button"
                                onclick={ctx.link().callback(|_| Msg::ShuffleToday)}
                                >
                            { "Shuffle Today's Categories" }
                        </button>
                    </GridItem>
                </Grid>
            </Bullseye>
            <AudioPlayer />
            </>
        }
    }
}

fn main() {
    wasm_logger::init(wasm_logger::Config::default());

    yew::start_app::<PracticePlannerApp>();
}
