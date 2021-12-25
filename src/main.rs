#[macro_use]
extern crate lazy_static;

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

lazy_static! {
    pub(crate) static ref DEFAULT_CATEGORIES: Vec<PracticeCategory> = vec![
        PracticeCategory {
            category_name: "Ear Training".to_string(),
        },
        PracticeCategory {
            category_name: "Exercises".to_string(),
        },
        PracticeCategory {
            category_name: "Chords".to_string(),
        },
        PracticeCategory {
            category_name: "Scales".to_string(),
        },
        PracticeCategory {
            category_name: "Sight Reading".to_string(),
        },
        PracticeCategory {
            category_name: "Music Theory".to_string(),
        },
        PracticeCategory {
            category_name: "Improvisation".to_string(),
        },
        PracticeCategory {
            category_name: "Songwriting".to_string(),
        },
    ];
}

const KEY: &str = "yew.practiceplanner.self";

#[derive(Error, Debug)]
pub enum SchedulerError {
    // #[error("Invalid header (expected {expected:?}, got {found:?})")]
    // InvalidHeader {
    //     expected: String,
    //     found: String,
    // },
    #[error("You must have at least 4 categories to practice")]
    MissingCategories(),

    #[error(transparent)]
    Other(#[from] anyhow::Error), // source and Display delegate to anyhow::Error
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct PracticeSkill {
    pub skill_name: String,
}

#[derive(Serialize, Deserialize, Clone, Hash, PartialOrd, Ord, PartialEq, Eq, Debug)]
pub struct PracticeCategory {
    pub category_name: String,
}

#[serde_with::serde_as]
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct SchedulePlanner {
    /// The duration each category is to be practiced.
    #[serde_as(as = "serde_with::DurationSeconds<i64>")]
    category_practice_time: Duration,
    /// The max number of days allowed to elapse without practicing a category.
    category_repeat_days: usize,
    /// The number of categories to practice per day.
    categories_per_day: usize,
    categories: Vec<PracticeCategory>,
    /// BTreeMap containing historical practice sessions.
    history: BTreeMap<DateTime<Utc>, Vec<PracticeCategory>>,
    todays_schedule: Option<Vec<PracticeCategory>>,
}

impl SchedulePlanner {
    pub fn new() -> Self {
        // Call this here so it's cached and faster later
        // let (_stream, _stream_handle) = OutputStream::try_default().unwrap();
        SchedulePlanner {
            //category_practice_time: Duration::minutes(15),
            category_practice_time: Duration::seconds(1),
            category_repeat_days: 2,
            categories: DEFAULT_CATEGORIES.to_vec(),
            history: BTreeMap::new(),
            todays_schedule: None,
            categories_per_day: 4,
        }
    }

    pub fn get_todays_schedule(&self) -> Option<&Vec<PracticeCategory>> {
        self.todays_schedule.as_ref()
    }

    /// Returns the categories seen in the last n days of history as a HashSet<&PracticeCategory>
    pub fn get_history_n_days_back(
        &self,
        n: usize,
    ) -> Result<BTreeMap<Date<Utc>, HashSet<&PracticeCategory>>> {
        let now = Utc::now();
        let n_days_back = now.checked_sub_signed(Duration::days(n.try_into().unwrap()));
        if n_days_back.is_none() {
            return Err(anyhow::anyhow!("Invalid historical search term"));
        }
        let mut historical_categories: BTreeMap<Date<Utc>, HashSet<&PracticeCategory>> =
            BTreeMap::new();

        // TODO keys are sorted so we could shorten iteration here
        for (key, value) in self.history.iter() {
            println!("key: {}", key);
            // if the history item is within the last n days...
            if key > &n_days_back.unwrap() {
                for v in value {
                    // insert into the HashSet for that day
                    let day_categories = match historical_categories.contains_key(&key.date()) {
                        true => historical_categories.get_mut(&key.date()).unwrap(),
                        false => {
                            let hs = HashSet::new();
                            historical_categories.insert(key.date(), hs);
                            historical_categories.get_mut(&key.date()).unwrap()
                        }
                    };

                    day_categories.insert(v);
                }
            }
        }

        Ok(historical_categories)
    }

    /// Iterate every history item to determine the total days of history
    /// that have been seen.
    /// TODO requires iteration and allocation to calculate, could just be a counter
    pub fn days_of_history(&self) -> usize {
        let mut seen_days = HashSet::new();
        for (key, _value) in self.history.iter() {
            seen_days.insert(key.date());
        }

        seen_days.len()
    }

    pub fn update_todays_schedule(&mut self, force_update: bool) -> Result<(), SchedulerError> {
        if self.todays_schedule.is_some() && !force_update {
            // schedule is already set and we didn't force an update
            return Ok(());
        }

        if self.categories.len() == 0 {
            return Err(SchedulerError::MissingCategories());
        }

        let past_history = self.get_history_n_days_back(self.category_repeat_days)?;
        let prob_bandwidth: f64 = 100.0 / self.category_repeat_days as f64;

        let mut probabilities: BTreeMap<&PracticeCategory, u64> = BTreeMap::new();

        for category in &self.categories {
            let mut seen = false;
            let mut d = 0;
            for (_day, day_categories) in past_history.iter() {
                println!("On day: {}", _day);
                if day_categories.contains(category) {
                    seen = true;
                }

                // if we have seen this before, weight the probability by the day seen
                if seen {
                    probabilities.insert(category, prob_bandwidth as u64 * d);
                }
                d = d + 1;
            }

            // if any categories do not appear in last n days history at all, set probability to 100%
            if !seen {
                probabilities.insert(category, 100);
                continue;
            }
        }

        self.todays_schedule = Some(
            probabilities
                .iter()
                .collect::<Vec<_>>()
                .choose_multiple_weighted(&mut thread_rng(), self.categories_per_day, |item| {
                    item.1.to_owned() as f64
                })
                .unwrap()
                .map(|item| item.0.to_owned().to_owned())
                .collect::<Vec<PracticeCategory>>(),
        );
        return Ok(());
    }

    pub fn start_category(&self, category: &PracticeCategory) -> Result<()> {
        println!(
            "Starting {} minute practice for category: {:#?}",
            self.category_practice_time.num_minutes(),
            category
        );

        thread::sleep(self.category_practice_time.to_std()?);
        println!("Done practicing category: {:#?}", category);
        // self.play_ding_sound();

        Ok(())
    }

    pub fn play_ding_sound(&self) -> Result<()> {
        // TODO these should be singletons or maybe the Sink type idk or a separate audio player task
        // Get a output stream handle to the default physical sound device
        // let (_stream, stream_handle) = OutputStream::try_default().unwrap();
        // // Load a sound from a file, using a path relative to Cargo.toml
        // let file = BufReader::new(File::open("audio/ding.wav").unwrap());
        // // Decode that sound file into a source
        // let source = Decoder::new(file).unwrap();

        // stream_handle.play_raw(source.convert_samples())?;

        // need to sleep for the duration of the ding sound
        // (todo should probably move to a second thread/task/whatever in tokio)
        thread::sleep(std::time::Duration::from_secs(2));
        Ok(())
    }

    pub fn start_daily_practice(&mut self) -> Result<()> {
        // ensure today's schedule has been set
        self.update_todays_schedule(false)?;
        for category in self.todays_schedule.as_ref().unwrap().iter() {
            println!("Starting practice for category: {:#?}", category);
            self.start_category(&category)?;
        }
        println!("Finished practicing for today!");

        // record this practice session in history, save to disk
        self.mark_todays_practice_completed()?;

        Ok(())
    }

    fn mark_todays_practice_completed(&mut self) -> Result<()> {
        if self.todays_schedule.is_none() {
            return Err(anyhow::anyhow!(
                "today's practice must be set to mark completed"
            ));
        }

        // append today's practice to the history
        let now = Utc::now();
        self.history
            .insert(now, self.todays_schedule.as_ref().unwrap().to_vec());

        // unset today's practice on Self
        self.todays_schedule = None;

        self.save_to_disk()?;
        Ok(())
    }

    pub fn save_to_disk(&self) -> Result<()> {
        let encoded: Vec<u8> = bincode::serialize(self).unwrap();
        let mut file = File::create("./saved_data/history.bin")?;
        file.write_all(&encoded)?;

        Ok(())
    }

    pub fn new_from_disk() -> Result<Self> {
        let mut f = File::open("./saved_data/history.bin")?;
        let mut buffer = Vec::new();
        f.read_to_end(&mut buffer)?;
        let decoded: Self = bincode::deserialize(&buffer[..])?;

        Ok(decoded)
    }

    fn view_input(&self, link: &Scope<Self>) -> Html {
        let onkeypress = link.batch_callback(|e: KeyboardEvent| {
            if e.key() == "Enter" {
                let input: InputElement = e.target_unchecked_into();
                let value = input.value();
                input.set_value("");
                Some(Msg::Add(value))
            } else {
                None
            }
        });
        html! {
            <input
                class="new-todo"
                placeholder="What needs to be done?"
                {onkeypress}
            />
        }
    }
}

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

pub struct Model {
    // state: State,
    focus_ref: NodeRef,
}

impl Component for SchedulePlanner {
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
        log::info!("Hello yew");
        scheduler
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
                        { self.view_input(ctx.link()) }
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

    yew::start_app::<SchedulePlanner>();

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
