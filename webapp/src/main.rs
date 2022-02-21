#![recursion_limit = "1024"]
#[macro_use]
extern crate lazy_static;

// use std;
use std::collections::BTreeMap;
use std::collections::HashSet;
use std::ops::Add;
use std::ops::Sub;
use std::rc::Rc;

use anyhow::Result;
use chrono::Duration;
use chrono::{Date, DateTime, Utc};
use gloo::storage::{LocalStorage, Storage};
use gloo::timers::callback::Interval;
use hhmmss::Hhmmss;
use pplib::PracticeSession;
use pulldown_cmark::{html::push_html, Options, Parser};
use wasm_bindgen::JsCast;
use web_sys::HtmlInputElement;
use web_sys::{HtmlOptionElement, MouseEvent};
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
use pplib::{PracticeSkill, SchedulePlanner};

mod bindings;
mod components;

// Use `wee_alloc` as the global allocator.
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

const CONFIG_KEY: &str = "yew.practiceplanner.config";
const HISTORY_KEY: &str = "yew.practiceplanner.history";
const FIRST_PAGE_VIEW: &str = "yew.practiceplanner.first_page_view";

pub enum Msg {
    StartPracticing,
    StopPracticing,
    PracticeTick,
    ShowResetHistoryPrompt,
    ResetHistory,
    ShuffleToday,
    ChangeTab(usize),
    CloseModal,
    OpenModal,
    ShowHelp,
    SetHelp,
    SelectSkill(HtmlOptionElement),
    ShowDeleteSkillPrompt,
    ShowResetSettingsPrompt,
    ResetSettings,
    SaveSettings,
    DeleteSkill,
    PausePracticing,
    NextExercise,
    PreviousExercise,
}

// Splitting this out makes local debugging easier
pub fn get_current_time() -> DateTime<Utc> {
    // Utc::now().sub(Duration::days(3))
    Utc::now()
}

pub struct PracticePlannerApp {
    scheduler: SchedulePlanner,
    interval: Option<Interval>,
    event_bus: Dispatcher<EventBus>,
    // TODO consider using yewdux for all this
    active_tab: usize,
    modal_closed: bool,
    displaying_modal: bool,
    modal_content: Html,
    modal_title: String,
    modal_type: String,
    first_page_view: bool,
    // the web app allows users to set practice time in terms of minutes
    practice_minutes: usize,
    // TODO this should really be a prop in a Settings (sub)component
    selected_skill: Option<String>,
    paused: bool,
    pause_time_elapsed: Duration,
    pause_time_started: Option<DateTime<Utc>>,
}

impl PracticePlannerApp {
    fn view_skill(
        &self,
        (idx, skill): (usize, &PracticeSkill),
        active_idx: Option<usize>,
        practicing: bool,
        _link: &Scope<Self>,
    ) -> Html {
        // XXX TODO handle this better?
        let active = active_idx.unwrap_or_default();
        let mut class = Classes::from("todo");
        if practicing && active as usize == idx {
            class.push("active-skill");
        } else if practicing && active as usize > idx {
            class.push("completed-skill")
        }
        html! {
            <li {class}>
                <div class="view">
                    <input
                        type="checkbox"
                        class="toggle"
                        checked={practicing && active as usize > idx}
                        disabled=true
                    />
                    <label>{ &skill.skill_name}</label>
                </div>
                // { self.view_entry_edit_input((idx, skill), link) }
            </li>
        }
    }

    fn view_history_list(
        &self,
        history_list: BTreeMap<Date<Utc>, HashSet<&PracticeSkill>>,
        _link: &Scope<Self>,
    ) -> Html {
        let _class = Classes::from("todo");

        if history_list.is_empty() {
            return html! {
                <strong>{ "No history" }</strong>
            };
        }

        // XXX TODO convert to a table view https://bulma.io/documentation/elements/table/
        let hl = history_list
            .iter()
            .map(|(day, day_skills)| {
                let mut dc = day_skills
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
        log::debug!("Saving...");
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
            {self.view_skill_list(&self.scheduler.practice_session, link)}
            <nav id="practice-buttons" class="level is-mobile">
                // Left side
                <div class="level-left">
                    { if practicing {
                        html!{
                            <>
                            <div class="level-item">
                                <div class="column">
                                    <div><strong>{ "Time left: " }{ self.scheduler.practice_session.as_ref().unwrap().time_left.hhmmss() }</strong></div>
                                    <div>
                                        <button class="favorite styled"
                                                type="button"
                                                onclick={link.callback(|_| Msg::PausePracticing)}
                                                >
                                                {"Pause Practicing"}
                                        </button>
                                        <button class="favorite styled"
                                                type="button"
                                                onclick={link.callback(|_| Msg::StopPracticing)}
                                                >
                                                {"Stop Practicing"}
                                        </button>
                                    </div>
                                </div>
                            </div>
                            </>
                        }
                    } else {
                        html!{
                            <>
                            <div class="level-item">
                                <div class="icon-text">
                                    <a title="Start Practicing" onclick={link.callback(|_| Msg::StartPracticing)}>
                                        <span class="icon is-medium has-text-success">
                                            <i class="fas fa-play fa-lg"></i>
                                        </span>
                                    </a>
                                </div>
                            </div>
                            <div class="level-item">
                                <div class="icon-text">
                                    <a title="Shuffle Today's Skills" onclick={link.callback(|_| Msg::ShuffleToday)}>
                                        <span class="icon is-medium has-text-success">
                                            <i class="fas fa-random fa-lg"></i>
                                        </span>
                                    </a>
                                </div>
                            </div>
                            </>
                        }
                    }}
                </div>
                // Right side
                <div class="level-right">
                </div>
            </nav>
            </>
        }
    }

    fn view_skill_list(
        &self,
        practice_session: &Option<PracticeSession>,
        link: &Scope<Self>,
    ) -> Html {
        let active = match practice_session {
            Some(ps) => Some(ps.get_current_skill_idx()),
            None => None,
        };
        let practicing = self.scheduler.practicing;

        html! {
            <>
            <ul class="skill-list">
            {
                if self.scheduler.get_todays_schedule().is_some() {
                    html! { for self.scheduler.get_todays_schedule().unwrap().iter().enumerate().map(|e| self.view_skill(e, active, practicing, link)) }
                } else {
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
                log::debug!("Found saved data: {:#?}", history);
                SchedulePlanner {
                    config: conf,
                    history: history.unwrap_or_default(),
                    todays_schedule: None,
                    practicing: false,
                    practice_session: None,
                }
            }
            Err(_e) => {
                log::debug!("Did not find saved data");
                SchedulePlanner::new()
            }
        };

        let practice_minutes = scheduler.config.skill_practice_time.num_minutes() as usize;

        let current_time = get_current_time();
        scheduler
            .update_todays_schedule(false, current_time)
            .expect("Unable to update today's schedule");

        let first_page_view = LocalStorage::get(FIRST_PAGE_VIEW).unwrap_or_else(|_| {
            LocalStorage::set(FIRST_PAGE_VIEW, false).unwrap();
            true
        });
        // Display the info modal on first page load
        if first_page_view {
            Self {
                scheduler,
                interval: None,
                event_bus: EventBus::dispatcher(),
                active_tab: 0,
                modal_closed: false,
                paused: false,
                displaying_modal: true,
                modal_content: html! {
                    <div>
                    <p>{"Welcome to Practice Planner. It's easy to get started."}</p>
                    <p>{"The entire app is built around the idea of making it easy for a musician to sit and have an effective practice session that works towards their individual goals."}</p>
                    <p>{"Once you've set the app up to your liking, you'll be able to jump into a practice session as soon as you open the app."}</p>
                    <p>{"The way this works is through "}<strong>{"Skills"}</strong>{" and "}<strong>{"Activities"}</strong>{"."}</p>
                    <p>{"First, use the "}<strong>{"Settings"}</strong>{" tab to define the different skills you want to practice. These would be things like rhythm, songwriting, left-hand technique, scales, etc."}</p>
                    <p>{"Then, decide which activities you want to perform. For example, you might write different patterns to play with your hand, embed a YouTube song you wanted to practice, or display chord fingerings to practice."}</p>
                    <p>{"We've also provided you with some initial skills and activities to get you started and show some of the possibilities."}</p>
                    <p>{"We support Markdown syntax for setting up activities -- this lets you embed tablature or score, YouTube videos, insert links, and have full control over the activity to suit your needs."}</p>
                    <p>{"Our integrated metronome and note synthesizer give you more tools at your disposal to have a smooth practice session."}</p>
                    <p>{"Each day, you will have a "}<strong>{"Practice Session"}</strong>{" created for you based on the skills you want to practice."}</p>
                    <p>{"Just click the "}<strong>{"Start"}</strong>{" button and you'll practice each skill scheduled for the day's session for an equal amount of time."}</p>
                    <p>{"Skills will be selected so that you never go too long without practicing a given skill."}</p>
                    </div>
                },
                modal_title: "Info".to_string(),
                modal_type: "info".to_string(),
                first_page_view,
                practice_minutes,
                selected_skill: None,
                pause_time_elapsed: Duration::seconds(0),
                pause_time_started: None,
            }
        } else {
            Self {
                scheduler,
                interval: None,
                event_bus: EventBus::dispatcher(),
                active_tab: 0,
                pause_time_elapsed: Duration::seconds(0),
                modal_closed: false,
                paused: false,
                displaying_modal: false,
                modal_content: html! {},
                modal_title: "Danger".to_string(),
                modal_type: "danger".to_string(),
                first_page_view,
                practice_minutes,
                selected_skill: None,
                pause_time_started: None,
            }
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::NextExercise => {
                self.scheduler
                    .practice_session
                    .as_mut()
                    .unwrap()
                    .next_exercise();
            }
            Msg::PreviousExercise => {
                self.scheduler
                    .practice_session
                    .as_mut()
                    .unwrap()
                    .previous_exercise();
            }
            Msg::SaveSettings => {
                // TODO: there has to be a better way to handle forms than this, but it was easy for now
                // fetch form values
                let window = web_sys::window().expect("no global `window` exists");
                let document = window.document().expect("should have a document on window");
                let skill_minutes_el = document
                    .get_element_by_id("skill_minutes")
                    .expect("should have a skill minutes element")
                    .unchecked_into::<HtmlInputElement>();
                let skill_count_el = document
                    .get_element_by_id("skill_count")
                    .expect("should have a skill count element")
                    .unchecked_into::<HtmlInputElement>();
                let skill_minutes = skill_minutes_el.value();
                let skill_count = skill_count_el.value();

                // validate
                let skill_minutes = skill_minutes.parse::<i64>().unwrap();
                let skill_count = skill_count.parse::<usize>().unwrap();

                // set on self
                self.scheduler.config.skill_practice_time = Duration::minutes(skill_minutes);
                self.scheduler.config.skills_per_day = skill_count;

                // persist to localstorage
                self.save().expect("able to save");
                return false;
            }
            Msg::ShowResetSettingsPrompt => {
                self.displaying_modal = true;
                self.modal_closed = false;
                self.modal_title = "Danger".to_string();
                self.modal_type = "danger".to_string();
                self.modal_content = html! {
                    <div>
                    <h1>{"Are you sure?"}</h1>
                    <p>{"Are you sure you'd like to reset your settings?"}</p>
                    <p>{"This is "}<strong>{"irreversible"}</strong>{", and will reset all of your history and settings to default."}</p>
                    <div>
                        <div class="icon-text">
                            <a title="Reset Settings" onclick={ctx.link().callback(|_| Msg::ResetSettings)}>
                                <span class="icon is-medium">
                                    <i class="fas fa-trash fa-lg"></i>
                                </span>
                            </a>
                        </div>
                        <a title="Reset Settings" onclick={ctx.link().callback(|_| Msg::ResetSettings)}>
                        {"Reset Settings"}
                        </a>
                        </div>
                    </div>
                };
            }
            Msg::ResetSettings => {
                // vv full reset
                self.scheduler = SchedulePlanner::new();
                let current_time = get_current_time();
                self.scheduler
                    .update_todays_schedule(false, current_time)
                    .expect("able to update schedule");
                self.save().expect("umable to save");

                self.modal_closed = true;
                self.displaying_modal = false;
                return true;
            }
            Msg::ShowResetHistoryPrompt => {
                self.displaying_modal = true;
                self.modal_closed = false;
                self.modal_title = "Danger".to_string();
                self.modal_type = "danger".to_string();
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
            }
            Msg::ResetHistory => {
                self.scheduler.reset_history();
                let current_time = get_current_time();
                self.scheduler
                    .update_todays_schedule(false, current_time)
                    .expect("able to update schedule");
                self.save().expect("umable to save");

                self.modal_closed = true;
                self.displaying_modal = false;
                return true;
            }
            Msg::ShuffleToday => {
                if !self.scheduler.practicing {
                    let current_time = get_current_time();
                    self.scheduler
                        .update_todays_schedule(true, current_time)
                        .expect("able to update schedule");
                }
            }
            Msg::PausePracticing => {
                self.paused = !self.paused;
                if !self.paused {
                    let now = get_current_time();
                    // pause time elapsed is a cumulative amount of time spent paused that has elapsed since the category started
                    self.pause_time_elapsed = self
                        .pause_time_elapsed
                        .add(now - self.pause_time_started.unwrap());
                    self.pause_time_started = None;
                    let handle = {
                        let link = ctx.link().clone();
                        Interval::new(100, move || link.send_message(Msg::PracticeTick))
                    };
                    self.interval = Some(handle);
                    return true;
                }
                let now = get_current_time();
                self.pause_time_started = Some(now);
                if let Some(timer) = self.interval.take() {
                    drop(timer);
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

                // Reset pause
                self.pause_time_elapsed = Duration::seconds(0);
                self.pause_time_started = None;
                self.paused = false;
            }
            Msg::PracticeTick => {
                let now = get_current_time();
                let time_elapsed = now
                    .sub(
                        self.scheduler
                            .practice_session
                            .as_ref()
                            .unwrap()
                            .skill_start_time,
                    )
                    .sub(self.pause_time_elapsed);
                let total_time = self.scheduler.config.skill_practice_time;

                if time_elapsed > total_time {
                    // move to next skill
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

                    self.pause_time_elapsed = Duration::seconds(0);
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
                    .skill_start_time = current_time;
                self.scheduler.practice_session.as_mut().unwrap().time_left =
                    self.scheduler.config.skill_practice_time;
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
                self.modal_closed = true;
                // vv that's bad TODO fix
                self.first_page_view = false;
            }
            Msg::OpenModal => {
                self.modal_closed = false;
                self.displaying_modal = true;
            }
            Msg::ShowHelp => {
                LocalStorage::set(FIRST_PAGE_VIEW, false).expect("able to save");
                self.displaying_modal = true;
                self.modal_closed = false;
                self.modal_content = html! {
                    <div>
                    <p>{"Welcome to Practice Planner. It's easy to get started."}</p>
                    <p>{"The entire app is built around the idea of making it easy for a musician to sit and have an effective practice session that works towards their individual goals."}</p>
                    <p>{"Once you've set the app up to your liking, you'll be able to jump into a practice session as soon as you open the app."}</p>
                    <p>{"The way this works is through "}<strong>{"Skills"}</strong>{" and "}<strong>{"Activities"}</strong>{"."}</p>
                    <p>{"First, use the "}<strong>{"Settings"}</strong>{" tab to define the different skills you want to practice. These would be things like rhythm, songwriting, left-hand technique, scales, etc."}</p>
                    <p>{"Then, decide which activities you want to perform. For example, you might write different patterns to play with your hand, embed a YouTube song you wanted to practice, or display chord fingerings to practice."}</p>
                    <p>{"We've also provided you with some initial skills and activities to get you started and show some of the possibilities."}</p>
                    <p>{"We support Markdown syntax for setting up activities -- this lets you embed tablature or score, YouTube videos, insert links, and have full control over the activity to suit your needs."}</p>
                    <p>{"Our integrated metronome and note synthesizer give you more tools at your disposal to have a smooth practice session."}</p>
                    <p>{"Each day, you will have a "}<strong>{"Practice Session"}</strong>{" created for you based on the skills you want to practice."}</p>
                    <p>{"Just click the "}<strong>{"Start"}</strong>{" button and you'll practice each skill scheduled for the day's session for an equal amount of time."}</p>
                    <p>{"Skills will be selected so that you never go too long without practicing a given skill."}</p>
                    </div>
                };
                self.modal_title = "Info".to_string();
                self.modal_type = "info".to_string();
            }
            Msg::SetHelp => {
                // TODO duplication here, should share
                LocalStorage::set(FIRST_PAGE_VIEW, false).expect("able to save");
                self.displaying_modal = true;
                self.modal_closed = false;
                self.modal_content = html! {
                    <div>
                    <p>{"Welcome to Practice Planner. It's easy to get started."}</p>
                    <p>{"The entire app is built around the idea of making it easy for a musician to sit and have an effective practice session that works towards their individual goals."}</p>
                    <p>{"Once you've set the app up to your liking, you'll be able to jump into a practice session as soon as you open the app."}</p>
                    <p>{"The way this works is through "}<strong>{"Skills"}</strong>{" and "}<strong>{"Activities"}</strong>{"."}</p>
                    <p>{"First, use the "}<strong>{"Settings"}</strong>{" tab to define the different skills you want to practice. These would be things like rhythm, songwriting, left-hand technique, scales, etc."}</p>
                    <p>{"Then, decide which activities you want to perform. For example, you might write different patterns to play with your hand, embed a YouTube song you wanted to practice, or display chord fingerings to practice."}</p>
                    <p>{"We've also provided you with some initial skills and activities to get you started and show some of the possibilities."}</p>
                    <p>{"We support Markdown syntax for setting up activities -- this lets you embed tablature or score, YouTube videos, insert links, and have full control over the activity to suit your needs."}</p>
                    <p>{"Our integrated metronome and note synthesizer give you more tools at your disposal to have a smooth practice session."}</p>
                    <p>{"Each day, you will have a "}<strong>{"Practice Session"}</strong>{" created for you based on the skills you want to practice."}</p>
                    <p>{"Just click the "}<strong>{"Start"}</strong>{" button and you'll practice each skill scheduled for the day's session for an equal amount of time."}</p>
                    <p>{"Skills will be selected so that you never go too long without practicing a given skill."}</p>
                    </div>
                };
                self.modal_title = "Info".to_string();
                self.modal_type = "info".to_string();
                return false;
            }
            Msg::SelectSkill(opt) => {
                // display the delete icon next to it
                self.selected_skill = Some(opt.value());
                return true;
            }
            Msg::ShowDeleteSkillPrompt => {
                // display a prompt because this is a pretty srs irreversible move

                self.displaying_modal = true;
                self.modal_closed = false;
                self.modal_title = "Danger".to_string();
                self.modal_type = "danger".to_string();
                self.modal_content = html! {
                    <div>
                    <h1>{"Are you sure?"}</h1>
                    <p>{"Are you sure you'd like to delete this skill?"}</p>
                    <p>{"This is "}<strong>{"irreversible"}</strong>{", and you will also delete any activities associated with this skill."}</p>
                    <div>
                        <div class="icon-text">
                            <a title="Delete Skill" onclick={ctx.link().callback(|_| Msg::DeleteSkill)}>
                                <span class="icon is-medium">
                                    <i class="fas fa-trash fa-lg"></i>
                                </span>
                            </a>
                        </div>
                        <a title="Delete Skill" onclick={ctx.link().callback(|_| Msg::DeleteSkill)}>
                        {"Delete Skill"}
                        </a>
                        </div>
                    </div>
                };
            }
            Msg::DeleteSkill => {
                if self.selected_skill.is_none() {
                    log::warn!("Tried deleting with nonexistent selected skill");
                    return false;
                }

                // TODO should find a better identifier to pass between client/server
                let skill = self.selected_skill.as_ref().unwrap().clone();
                self.scheduler
                    .delete_skill_string(skill)
                    .expect("delete skill failure");

                self.save().expect("unable to save");
                self.modal_closed = true;
                self.displaying_modal = false;
                return true;
            }
        }

        true
    }

    fn rendered(&mut self, ctx: &Context<Self>, first_render: bool) {
        // render the tabs
        create_tab();
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        // XXX TODO there is too much recalculation going on in here
        // we can probably do better
        // i.e. have today's schedule stored in state and updated in `update` rather than
        // recalculated every re-render
        let _hidden_class = "hidden";

        let modal_props = ModalDisplayProps {
            modal_type: self.modal_type.to_string(),
            content: self.modal_content.clone(),
            modal_title: self.modal_title.clone(),
            active: self.first_page_view || (self.displaying_modal && !self.modal_closed),
            on_close_modal: ctx.link().callback(|_: usize| Msg::CloseModal),
        };

        let props = TabDisplayProps {
            on_tab_change: ctx.link().callback(|i: usize| Msg::ChangeTab(i)),
        };

        let active_skill: Option<Rc<PracticeSkill>> =
            match &self.scheduler.practice_session.as_ref() {
                Some(ps) => Some(Rc::clone(&ps.current_skill)),
                None => None,
            };
        let visible_exercise_md = match &self.scheduler.practice_session {
            Some(ps) => match &ps.current_exercise {
                Some(ce) => ce.exercise_markdown_contents.clone(),
                None => "".to_string(),
            },
            None => "".to_string(),
        };

        // TODO split the individual tab contents into their own components
        let parse_html = parse_markdown_text(&visible_exercise_md);
        let html_text = format!("<div class='preview'>{}</div>", &parse_html);
        let window = web_sys::window().expect("no global `window` exists");
        let document = window.document().expect("should have a document on window");
        let val = document.create_element("div").unwrap();
        val.set_inner_html(&html_text);
        let rendered_exercise = VNode::VRef(val.into());

        let current_time = get_current_time();
        let streak = self.scheduler.get_streak(current_time);
        let _skill_list = self.scheduler.get_todays_schedule();
        // TODO use a constant here
        let history_list = self
            .scheduler
            .get_history_n_days_back(3, current_time)
            .expect("unable to retrieve history");

        let cl = self.scheduler.config.skills
                .iter()
                .map(|skill| {
                    html! { <option value={skill.skill_name.clone()} onclick={ctx.link().callback(|e: MouseEvent| Msg::SelectSkill(e.target_unchecked_into::<HtmlOptionElement>()))}>{skill.skill_name.clone()}</option> }
                })
                .collect::<Vec<_>>();
        html! {
            <>
            <ModalDisplay ..modal_props/>

            <section class="hero is-primary">
                <div class="hero-body">
                    <p class="title">
                    {"Practice Planner"}
                    </p>
                </div>
            </section>

            <div class="tile is-ancestor">
                <div class="tile is-3">
                </div>
                <div class="main-content tile is-6 is-vertical box">
                    <TabDisplay ..props/>
                    <a title="Help" onclick={ctx.link().callback(|_| Msg::ShowHelp)}>
                        <span class="icon is-medium has-text-success icon-help">
                            <i class="far fa-question-circle fa-lg"></i>
                        </span>
                    </a>
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
                                onclick={ctx.link().callback(|_| Msg::ShowResetHistoryPrompt)}
                                >
                            { "Reset History" }
                        </button>
                    } else if self.active_tab == 2 {
                        <p>
                        <label for="skill_list">{"Skills"}</label>
                        </p>
                        <div class="select is-multiple">
                        <select id="skill_list" multiple=true>
                            { cl }
                        </select>
                        </div>

                        {
                            if self.selected_skill.is_some() {
                                html! {
                                    <div class="icon-text">
                                        <a title="Delete Skill" onclick={ctx.link().callback(|_| Msg::ShowDeleteSkillPrompt)}>
                                            <span class="icon is-medium has-text-success">
                                                <i class="fas fa-trash fa-lg"></i>
                                            </span>
                                        </a>
                                    </div>
                                }
                            } else {
                                html! {<></>}
                            }
                        }

                        <p><label for="skill_minutes">{"Minutes to Practice Each Skill"}</label></p>
                        <input id="skill_minutes" class="input is-primary" type="text" placeholder="15" value={format!("{}", self.practice_minutes)} />

                        <p><label for="skill_count">{"Number of Skills to Practice Per Day"}</label></p>
                        <input id="skill_count" class="input is-primary" type="text" placeholder="4" value={format!("{}", self.scheduler.config.skills_per_day)} />

                        <button class="favorite styled"
                                type="button"
                                onclick={ctx.link().callback(|_| Msg::ShowResetSettingsPrompt)}
                                >
                                {"Reset Settings to Default"}
                        </button>

                        <button class="favorite styled"
                                type="button"
                                onclick={ctx.link().callback(|_| Msg::SaveSettings)}
                                >
                                {"Save Changes"}
                        </button>
                    }
                    </div>
                    // this should only show on the practice tab
                    if self.active_tab == 0 {

                    <div class="tile is-child content app-panel">
                        {if self.scheduler.practicing {

                        rendered_exercise
                        } else {
                            html! {<></>}
                        }
                        }

                        <nav class="level content is-large is-mobile">
                            // Left side
                            { if self.scheduler.practicing {

                            html!(<><div class="level-left">
                                <div class="level-item">
                                    <div class="icon-text">
                                        <a title="Previous Exercise" onclick={ctx.link().callback(|_| Msg::PreviousExercise)}>
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
                                        <a title="Next Exercise" onclick={ctx.link().callback(|_| Msg::NextExercise)}>
                                            <span class="icon is-medium has-text-success">
                                                <i class="fas fa-long-arrow-alt-right fa-lg"></i>
                                            </span>
                                        </a>
                                    </div>
                                </div>
                            </div></>)
                            } else {
                                html!(<></>)
                            }
                        }
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
