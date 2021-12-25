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

pub enum Msg {
    Add(String),
    Edit((usize, String)),
    Remove(usize),
    ToggleAll,
    ToggleEdit(usize),
    Toggle(usize),
    ClearCompleted,
    Focus,
}

pub struct PracticePlannerApp {
    // state: State,
    // focus_ref: NodeRef,
    scheduler: SchedulePlanner,
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
        html! {
            <div class="todomvc-wrapper">
                <section class="todoapp">
                    <header class="header">
                        <h1>{ "todos" }</h1>
                        // { self.view_input(ctx.link()) }
                    </header>
                    <section class={classes!("main", hidden_class)}>
                        <input
                            type="checkbox"
                            class="toggle-all"
                            id="toggle-all"
                            // checked={self.state.is_all_completed()}
                            onclick={ctx.link().callback(|_| Msg::ToggleAll)}
                        />
                        <label for="toggle-all" />
                        <ul class="todo-list">
                            // { for self.state.entries.iter().filter(|e| self.state.filter.fits(e)).enumerate().map(|e| self.view_entry(e, ctx.link())) }
                        </ul>
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
                    <p>{ "Double-click to edit a todo" }</p>
                    <p>{ "Written by " }<a href="https://github.com/DenisKolodin/" target="_blank">{ "Denis Kolodin" }</a></p>
                    <p>{ "Part of " }<a href="http://todomvc.com/" target="_blank">{ "TodoMVC" }</a></p>
                </footer>
            </div>
        }
    }
}

fn main() {
    wasm_logger::init(wasm_logger::Config::default());

    yew::start_app::<PracticePlannerApp>();

    let mut scheduler = match Path::new("./saved_data/history.bin").exists() {
        true => {
            println!("Saved data found, loading...");
            match SchedulePlanner::new_from_disk() {
                Ok(sp) => sp,
                Err(_e) => {
                    // TODO the import/export mechanism is extremely fragile
                    // if the data structure is changed
                    println!("Error loading history file");
                    SchedulePlanner::new()
                }
            }
        }
        false => SchedulePlanner::new(),
    };

    scheduler
        .update_todays_schedule(false)
        .expect("Unable to update today's schedule");
    let todays_schedule = scheduler.get_todays_schedule();
    println!("Today's schedule: {:#?}", todays_schedule);
    println!("Want to practice? ");
    let line: String = read!("{}\n");
    match line.to_lowercase().as_str() {
        "y\r" | "y\n" | "y" => {
            println!("Yeehaw");
            scheduler
                .start_daily_practice()
                .expect("Unable to run daily practice");
        }
        _ => {
            println!("Well, okay then.");
            println!("{}", line);
        }
    };
}

pub fn get_todays_schedule() -> Result<Vec<PracticeCategory>> {
    Ok(vec![])
}
