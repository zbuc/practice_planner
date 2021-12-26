#![recursion_limit = "1024"]
use std;
use std::collections::BTreeMap;
use std::collections::HashSet;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::thread;

use anyhow::Result;
use chrono::{Date, DateTime, Duration, Utc};
use gloo::storage::{LocalStorage, Storage};
use log;
use rand::prelude::*;
use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};
use text_io::read;
use thiserror::Error;
use web_sys::HtmlInputElement as InputElement;
use yew::{
    classes,
    events::{FocusEvent, KeyboardEvent},
    html,
    html::Scope,
    Classes, Component, Context, Html, NodeRef, TargetCast,
};

use pplib::{PracticeCategory, PracticeSkill, SchedulePlanner};

// Use `wee_alloc` as the global allocator.
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

pub enum Msg {
    Add(String),
    Edit((usize, String)),
    Remove(usize),
    ToggleAll,
    ToggleEdit(usize),
    Toggle(usize),
    ClearCompleted,
    Focus,
    BeginPractice,
}

pub struct PracticePlannerApp {
    // state: State,
    // focus_ref: NodeRef,
    scheduler: SchedulePlanner,
}

impl PracticePlannerApp {
    fn view_category(
        &self,
        (idx, category): (usize, &PracticeCategory),
        link: &Scope<Self>,
    ) -> Html {
        let mut class = Classes::from("todo");
        // if entry.editing {
        //     class.push(" editing");
        // }
        // if entry.completed {
        //     class.push(" completed");
        // }
        log::info!("making the category item");
        html! {
            <li {class}>
                <div class="view">
                    <input
                        type="checkbox"
                        class="toggle"
                        // checked={entry.completed}
                        onclick={link.callback(move |_| Msg::Toggle(idx))}
                    />
                    <label ondblclick={link.callback(move |_| Msg::ToggleEdit(idx))}>{ &category.category_name}</label>
                    <button class="destroy" onclick={link.callback(move |_| Msg::Remove(idx))} />
                </div>
                // { self.view_entry_edit_input((idx, category), link) }
            </li>
        }
    }

    fn view_history_list(
        &self,
        history_list: BTreeMap<Date<Utc>, HashSet<&PracticeCategory>>,
        link: &Scope<Self>,
    ) -> Html {
        let mut class = Classes::from("todo");
        // if entry.editing {
        //     class.push(" editing");
        // }
        // if entry.completed {
        //     class.push(" completed");
        // }
        log::info!("making the history list");

        if history_list.is_empty() {
            return html! {
                <strong>{ "No history" }</strong>
            };
        }

        html! {
            <ul class="history-list">
            {
                for history_list.iter().map(|(day, day_categories)|{
                    html! { <li><strong>{ day }</strong>{ ": lol" }</li> }
                })
            }
            // html!{    for (day, day_categories) in history_list.iter() {
            //         <span></span>
            //     }
            // }
            </ul>
        }
    }

    fn view_category_list(
        &self,
        category_list: &Vec<PracticeCategory>,
        link: &Scope<Self>,
    ) -> Html {
        let mut class = Classes::from("todo");
        // if entry.editing {
        //     class.push(" editing");
        // }
        // if entry.completed {
        //     class.push(" completed");
        // }
        log::info!("making the category list");

        html! {
            <>
            <ul class="todo-list">
            {
                if self.scheduler.get_todays_schedule().is_some() {
                    log::info!("got a schedule for today");
                    html! { for self.scheduler.get_todays_schedule().unwrap().iter().enumerate().map(|e| self.view_category(e, link)) }
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
        // let entries = LocalStorage::get(KEY).unwrap_or_else(|_| Vec::new());
        // let state = State {
        //     entries,
        //     filter: Filter::All,
        //     edit_value: "".into(),
        // };
        let focus_ref = NodeRef::default();
        // Self { state, focus_ref }

        // TODO webapp should use LocalStorage for storage
        let mut scheduler = match Path::new("./saved_data/history.bin").exists() {
            true => {
                log::info!("Saved data found, loading...");
                match SchedulePlanner::new_from_disk() {
                    Ok(sp) => sp,
                    Err(_e) => {
                        // TODO the import/export mechanism is extremely fragile
                        // if the data structure is changed
                        log::info!("Error loading history file");
                        SchedulePlanner::new()
                    }
                }
            }
            false => SchedulePlanner::new(),
        };

        scheduler
            .update_todays_schedule(false)
            .expect("Unable to update today's schedule");
        // let todays_schedule = scheduler.get_todays_schedule();
        Self { scheduler }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Add(description) => {
                if !description.is_empty() {
                    // let entry = Entry {
                    //     description: description.trim().to_string(),
                    //     completed: false,
                    //     editing: false,
                    // };
                    // self.state.entries.push(entry);
                }
            }
            Msg::Edit((idx, edit_value)) => {
                // self.state.complete_edit(idx, edit_value.trim().to_string());
                // self.state.edit_value = "".to_string();
            }
            Msg::Remove(idx) => {
                // self.state.remove(idx);
            }
            // Msg::SetFilter(filter) => {
            //     self.state.filter = filter;
            // }
            Msg::ToggleEdit(idx) => {
                // self.state.edit_value = self.state.entries[idx].description.clone();
                // self.state.clear_all_edit();
                // self.state.toggle_edit(idx);
            }
            Msg::BeginPractice => {
                self.scheduler
                    .start_daily_practice()
                    .expect("failed daily practice");
                self.scheduler
                    .update_todays_schedule(false)
                    .expect("able to update schedule");
                // let status = !self.state.is_all_completed();
                // self.state.toggle_all(status);
            }
            Msg::ToggleAll => {
                // let status = !self.state.is_all_completed();
                // self.state.toggle_all(status);
            }
            Msg::Toggle(idx) => {
                // self.state.toggle(idx);
            }
            Msg::ClearCompleted => {
                // self.state.clear_completed();
            }
            Msg::Focus => {
                // if let Some(input) = self.focus_ref.cast::<InputElement>() {
                //     input.focus().unwrap();
                // }
            }
        }
        // LocalStorage::set(KEY, &self.state.entries).expect("failed to set");
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let hidden_class = "hidden";
        // let hidden_class = if self.state.entries.is_empty() {
        //     "hidden"
        // } else {
        //     ""
        // };
        let category_list = self
            .scheduler
            .get_todays_schedule()
            .expect("unable to retrieve today's schedule");
        // TODO use a constant here
        let history_list = self
            .scheduler
            .get_history_n_days_back(3)
            .expect("unable to retrieve history");
        html! {
            <div class="todomvc-wrapper">
                <section class="todoapp">
                    <header class="header">
                        <h1>{ "Guitar Metis" }</h1>
                        // { self.view_input(ctx.link()) }
                    </header>
                    <section class={classes!("main")}>
                        <h2 class="history-list">{ "Practice History" }</h2>
                        {self.view_history_list(history_list, ctx.link())}
                        <input
                            type="checkbox"
                            class="toggle-all"
                            id="toggle-all"
                            // checked={self.state.is_all_completed()}
                            onclick={ctx.link().callback(|_| Msg::ToggleAll)}
                        />
                        <label for="toggle-all" />
                        <h2 class="category-list">{ "Today's Schedule" }</h2>
                        {self.view_category_list(category_list, ctx.link())}
                            // { for self.state.entries.iter().filter(|e| self.state.filter.fits(e)).enumerate().map(|e| self.view_entry(e, ctx.link())) }
                        <button class="favorite styled"
                                type="button"
                                onclick={ctx.link().callback(|_| Msg::BeginPractice)}
                                >
                            { "Begin Practice" }
                        </button>
                    </section>
                    <footer class={classes!("footer", hidden_class)}>
                        <span class="todo-count">
                            // <strong>{ self.state.total() }</strong>
                            { " item(s) left" }
                        </span>
                        <ul class="filters">
                            // { for Filter::iter().map(|flt| self.view_filter(flt, ctx.link())) }
                        </ul>
                        <button class="clear-completed" onclick={ctx.link().callback(|_| Msg::ClearCompleted)}>
                            // { format!("Clear completed ({})", self.state.total_completed()) }
                        </button>
                    </footer>
                </section>
                <footer class="info">
                    <p>{ "Some text goes here. Lorem ipsum dolor sit amet and so on." }</p>
                </footer>
            </div>
        }
    }
}

fn main() {
    wasm_logger::init(wasm_logger::Config::default());

    yew::start_app::<PracticePlannerApp>();

    // println!("Today's schedule: {:#?}", todays_schedule);
    // println!("Want to practice? ");
    // let line: String = read!("{}\n");
    // match line.to_lowercase().as_str() {
    //     "y\r" | "y\n" | "y" => {
    //         println!("Yeehaw");
    //         scheduler
    //             .start_daily_practice()
    //             .expect("Unable to run daily practice");
    //     }
    //     _ => {
    //         println!("Well, okay then.");
    //         println!("{}", line);
    //     }
    // };
}

pub fn get_todays_schedule() -> Result<Vec<PracticeCategory>> {
    Ok(vec![])
}
