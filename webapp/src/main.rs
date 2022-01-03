#![recursion_limit = "1024"]
#[macro_use]
extern crate lazy_static;

// use std;
use std::collections::BTreeMap;
use std::collections::HashSet;
use std::ops::Sub;

use anyhow::Result;
use chrono::{Date, DateTime, Utc};
use gloo::storage::{LocalStorage, Storage};
use gloo::timers::callback::Interval;
use hhmmss::Hhmmss;
use log;
use pplib::PracticeSession;
use pulldown_cmark::{html::push_html, Options, Parser};
#[allow(unused_imports)]
use yew::prelude::*;
use yew::virtual_dom::VNode;
use yew::{html, html::Scope, Classes, Component, Context, Html};
use yew_agent::{Dispatched, Dispatcher};

use crate::bindings::tablature::*;
use crate::components::audio_player::*;
use crate::components::event_bus::{EventBus, Request};
use crate::components::modal::*;
use crate::components::tabs::*;
use pplib::{PracticeCategory, SchedulePlanner};

mod bindings;
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
    ResetHistoryPrompt,
    ResetHistory,
    ShuffleToday,
    ChangeTab(usize),
    CloseModal,
    OpenModal,
}

// Splitting this out makes local debugging easier
pub fn get_current_time() -> DateTime<Utc> {
    // Utc::now().sub(Duration::days(3))
    Utc::now()
}

pub struct PracticePlannerApp {
    scheduler: SchedulePlanner<'static>,
    interval: Option<Interval>,
    event_bus: Dispatcher<EventBus>,
    // TODO consider using yewdux for all this
    active_tab: usize,
    modal_closed: bool,
    displaying_modal: bool,
    modal_content: Html,
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

        // XXX TODO convert to a table view https://bulma.io/documentation/elements/table/
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

    fn view_practice_tab(
        &self,
        _practice_session: &Option<PracticeSession>,
        link: &Scope<Self>,
    ) -> Html {
        let practicing = self.scheduler.practicing;

        html! {
            <>
            {self.view_category_list(&self.scheduler.practice_session, link)}
            <nav class="level">
                // Left side
                <div class="level-left">
                    { if practicing {
                        html!{
                            <>
                            <div class="level-item">
                                <h3>{ "Time left: " }{ self.scheduler.practice_session.as_ref().unwrap().time_left.hhmmss() }</h3>
                                <button class="favorite styled"
                                        type="button"
                                        onclick={link.callback(|_| Msg::StopPracticing)}
                                        >
                                        {"Stop Practicing"}
                                </button>
                            </div>
                            </>
                        }
                    } else {
                        html!{
                            <div class="level-item">
                                <div class="icon-text">
                                    <a title="Start Practicing" onclick={link.callback(|_| Msg::StartPracticing)}>
                                        <span class="icon is-medium has-text-success">
                                            <i class="fas fa-play fa-lg"></i>
                                        </span>
                                    </a>
                                </div>
                            </div>
                        }
                    }}
                </div>
                // Right side
                <div class="level-right">
                    <div class="level-item">
                        <div class="icon-text">
                            <a title="Shuffle Today's Categories" onclick={link.callback(|_| Msg::ShuffleToday)}>
                                <span class="icon is-medium has-text-success">
                                    <i class="fas fa-random fa-lg"></i>
                                </span>
                            </a>
                        </div>
                    </div>
                </div>
            </nav>
            </>
        }
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
    type Properties = TabDisplayProps;

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
            active_tab: 0,
            modal_closed: false,
            displaying_modal: false,
            modal_content: html! {},
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::ResetHistoryPrompt => {
                self.displaying_modal = true;
                self.modal_closed = false;
                self.modal_content = html! {
                    <div>
                    <h1>{"Are you sure?"}</h1>
                    <p>{"Are you sure you'd like to reset your history?"}</p>
                    <div>
                        <div class="icon-text">
                            <a title="Reset History" onclick={ctx.link().callback(|_| Msg::ResetHistory)}>
                                <span class="icon is-medium">
                                    <i class="fas fa-trash fa-lg"></i>
                                </span>
                            </a>
                        </div>
                        <a title="Reset History" onclick={ctx.link().callback(|_| Msg::ResetHistory)}>
                        {"Reset History"}
                        </a>
                        </div>
                    </div>
                };
                // XXX TODO implement again
                // log::info!("Prompt not implemented... resetting data anyways");
                // self.scheduler = SchedulePlanner::new();
                // let current_time = get_current_time();
                // self.scheduler
                //     .update_todays_schedule(false, current_time)
                //     .expect("able to update schedule");
                // self.save().expect("umable to save");
                // TODO this would be better as a modal probably but there's
                // no easy way to trigger those in patternfly-yew
                // let fix = ctx
                //     .link()
                //     .callback(|_| Msg::ResetData)
                //     .into_action("Reset Data");
                // let toast = Toast {
                //     title: "Are you sure?".into(),
                //     r#type: Type::Danger,
                //     body: html! {
                //         <p>{"Are you sure you'd like to reset all your configuration and history?"}</p>
                //     },
                //     actions: vec![fix.clone()],
                //     ..Default::default()
                // };
                // ToastDispatcher::new().toast(toast);
            }
            Msg::ResetHistory => {
                // self.scheduler = SchedulePlanner::new();
                self.scheduler.reset_history();
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
                    // TODO this seems overcomplicated maybe just use a callback
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
            Msg::ChangeTab(i) => {
                self.active_tab = i;
            }
            Msg::CloseModal => {
                log::info!("Close modal");
                self.modal_closed = true;
            }
            Msg::OpenModal => {
                self.modal_closed = false;
                self.displaying_modal = true;
            }
        }

        true
    }

    fn rendered(&mut self, _ctx: &Context<Self>, first_render: bool) {
        // render the tabs
        log::info!("Create_Tab");
        create_tab();
        log::info!("Created???");
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        // XXX TODO there is too much recalculation going on in here
        // we can probably do better
        // i.e. have today's schedule stored in state and updated in `update` rather than
        // recalculated every re-render
        let _hidden_class = "hidden";
        if self.scheduler.practicing {
            log::info!("currently practicing");
        }

        let props = TabDisplayProps {
            on_tab_change: ctx.link().callback(|i: usize| Msg::ChangeTab(i)),
        };
        let modal_props = ModalDisplayProps {
            modal_type: "warning".to_string(),
            content: self.modal_content.clone(),
            active: self.displaying_modal && !self.modal_closed,
            on_close_modal: ctx.link().callback(|_: usize| Msg::CloseModal),
        };

        // TODO this is bad lol
        let active_category: &PracticeCategory = match self.scheduler.practicing {
            true => {
                &self.scheduler.config.categories[self
                    .scheduler
                    .practice_session
                    .as_ref()
                    .unwrap()
                    .current_category as usize]
            }
            false => &self.scheduler.config.categories[0],
        };

        let visible_exercise_md = &active_category.exercises[0].exercise_markdown_contents;
        // TODO split the individual tab contents into their own components
        let parse_html = parse_markdown_text(&visible_exercise_md);
        let html_text = format!("<div class='preview'>{}</div>", &parse_html);
        let window = web_sys::window().expect("no global `window` exists");
        let document = window.document().expect("should have a document on window");
        let val = document.create_element("div").unwrap();
        val.set_inner_html(&html_text);
        let preview = VNode::VRef(val.into());

        let current_time = get_current_time();
        let streak = self.scheduler.get_streak(current_time);
        let _category_list = self.scheduler.get_todays_schedule();
        // TODO use a constant here
        let history_list = self
            .scheduler
            .get_history_n_days_back(3, current_time)
            .expect("unable to retrieve history");
        html! {
            <>
            <ModalDisplay ..modal_props/>

            <section class="hero is-primary">
                <div class="hero-body">
                    <p class="title">
                    {"Guitar Practice Planner"}
                    </p>
                    <p class="subtitle">
                      {"You know, for learning to play guitar"}
                    </p>
                </div>
            </section>

            <div class="tile is-ancestor">
                <div class="tile is-3">
                </div>
                <div class="main-content tile is-6 is-vertical box">
                    <TabDisplay ..props/>
                    <div class="tile is-parent">
                    <div class="tile is-child content is-large is-vertical app-panel">
                    if self.active_tab == 0 {
                        {self.view_practice_tab(&self.scheduler.practice_session, ctx.link())}
                    } else if self.active_tab == 1 {
                        // <p class="title">{ "Practice History" }</p>
                        {self.view_history_list(history_list, ctx.link())}
                        <p>{ "Streak: " }<strong>{ streak }{ " days" }</strong></p>
                        <button class="favorite styled"
                                type="button"
                                onclick={ctx.link().callback(|_| Msg::ResetHistoryPrompt)}
                                >
                            { "Reset History" }
                        </button>
                    } else if self.active_tab == 2 {
                    }
                    </div>
                    // this should only show on the practice tab
                    if self.active_tab == 0 {

                    <div class="tile is-child content app-panel">
                    // <div class="vextab-auto" width="680" scale="1.0" editor="false" show-errors="true">
                    //     {"options space=20
                    //     tabstave notation=true key=A time=4/4

                    //     notes :q =|: (5/2.5/3.7/4) :8 7-5h6/3 ^3^ 5h6-7/5 ^3^ :q 7V/4 |
                    //     notes :8 t12p7/4 s5s3/4 :8 3s:16:5-7/5 :h p5/4
                    //     text :w, |#segno, ,|, :hd, , #tr

                    //     options space=25"}
                    // </div>
                    // <div class="vextab-auto" width="680" scale="1.0" editor="false" show-errors="true">
                    //     {"options space=20
                    //     tabstave notation=true key=A time=4/4

                    //     notes :q =|: (5/2.5/3.7/4) :8 7-5h6/3 ^3^ 5h6-7/5 ^3^ :q 7V/4 |
                    //     notes :8 t12p7/4 s5s3/4 :8 3s:16:5-7/5 :h p5/4
                    //     text :w, |#segno, ,|, :hd, , #tr

                    //     options space=25"}
                    // </div>
                    // {create_tab();}

                        // <div id="boo"></div>
                        {preview}

                        <nav class="level content is-large">
                            // Left side
                            <div class="level-left">
                                <div class="level-item">
                                    <div class="icon-text">
                                        <a title="Previous Exercise" onclick={ctx.link().callback(|_| Msg::ShuffleToday)}>
                                            <span class="icon is-medium has-text-success">
                                                <i class="fas fa-long-arrow-alt-left fa-lg"></i>
                                            </span>
                                        </a>
                                    </div>
                                </div>
                            </div>
                            <div class="level-right">
                                <div class="level-item">
                                    <div class="icon-text">
                                        <a title="Next Exercise" onclick={ctx.link().callback(|_| Msg::ShuffleToday)}>
                                            <span class="icon is-medium has-text-success">
                                                <i class="fas fa-long-arrow-alt-right fa-lg"></i>
                                            </span>
                                        </a>
                                    </div>
                                </div>
                            </div>
                        </nav>

                    </div>
                    }
                    </div>
                </div>
                <div class="tile is-3">
                </div>
            </div>
            <AudioPlayer />
            </>
        }
    }
}

// https://github.com/AkifumiSato/yew-markdown-demo
fn parse_markdown_text(value: &str) -> String {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_FOOTNOTES);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TASKLISTS);

    let parser = Parser::new_ext(&value, options);
    let mut parsed_text = String::new();
    push_html(&mut parsed_text, parser);

    parsed_text
}

fn main() {
    wasm_logger::init(wasm_logger::Config::default());

    yew::start_app::<PracticePlannerApp>();
}
