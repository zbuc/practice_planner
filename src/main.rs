#[macro_use]
extern crate lazy_static;

use std;
use std::collections::BTreeMap;
use std::collections::HashSet;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::path::Path;

use anyhow::Result;
use chrono::serde::ts_seconds;
use chrono::{DateTime, Duration, Utc};
use rand::prelude::*;
use rand::seq::SliceRandom;
use rodio::{Decoder, OutputStream, Sink};
use rodio::{OutputStreamHandle, Source};
use serde::{Deserialize, Serialize};
use text_io::read;
use thiserror::Error;
use tokio::time;

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

#[derive(Serialize, Deserialize, Clone, Hash, PartialEq, Eq, Debug)]
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
    categories: Vec<PracticeCategory>,
    /// BTreeMap containing historical practice sessions.
    history: BTreeMap<DateTime<Utc>, Vec<PracticeCategory>>,
    todays_schedule: Option<Vec<PracticeCategory>>,
}

impl SchedulePlanner {
    pub fn new() -> Self {
        // Call this here so it's cached and faster later
        let (_stream, _stream_handle) = OutputStream::try_default().unwrap();
        SchedulePlanner {
            //category_practice_time: Duration::minutes(15),
            category_practice_time: Duration::seconds(1),
            category_repeat_days: 2,
            categories: DEFAULT_CATEGORIES.to_vec(),
            history: BTreeMap::new(),
            todays_schedule: None,
        }
    }

    pub fn get_todays_schedule(&self) -> Option<&Vec<PracticeCategory>> {
        self.todays_schedule.as_ref()
    }

    /// Returns the categories seen in the last n days of history as a HashSet<&PracticeCategory>
    /// XXX TODO really we want a Vec<HashSet<&PracticeCategory>> n-items large
    pub fn get_history_n_days_back(&self, n: usize) -> Result<HashSet<&PracticeCategory>> {
        let now = Utc::now();
        let n_days_back = now.checked_sub_signed(Duration::days(n.try_into().unwrap()));
        if n_days_back.is_none() {
            return Err(anyhow::anyhow!("Invalid historical search term"));
        }
        let mut historical_categories = HashSet::new();

        // iterate each history item and return if within last two days
        for (key, value) in self.history.iter() {
            if key > &n_days_back.unwrap() {
                for v in value {
                    historical_categories.insert(v);
                }
            } else {
                // keys are sorted so we can break early
                // XXX TODO i think they might be reverse ordered
                // so this could be a bug
                println!("breaking early");
                break;
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

        // no category may go more than 2 days without being practiced.
        // first construct a list of all categories sorted by date last practiced.

        // if there's not enough history, we can pick any four categories at random for today
        let past_history = self.get_history_n_days_back(self.category_repeat_days)?;
        println!(
            "Past {} days history: {:#?}",
            self.category_repeat_days, past_history
        );

        if self.days_of_history() <= self.category_repeat_days {
            self.todays_schedule = Some(
                self.categories
                    .choose_multiple(&mut thread_rng(), 4)
                    .cloned()
                    .collect(),
            );
            return Ok(());
        }

        let prob_bandwidth: f64 = 100.0 / self.category_repeat_days.into();

        // for each history element
        // check how many days ago it was from 0..category_repeat_days
        // set probability to prob_bandwidth * how many days ago it was
        // check if > 4 categories have maximum priority (100.0), if so,
        // the category count & category_repeat_days cannot create a satisfactory
        // solution

        for (key, value) in self.history.iter() {
            println!("{}: {:#?}", key, value);
        }

        // https://docs.rs/rand/latest/rand/seq/trait.SliceRandom.html#tymethod.choose_multiple_weighted is probably easiest
        // then find any categories which are going to go past 2 days if they aren't practiced today.
        // if there are more than four, raise an error because there are too many categories so something must be adjusted.
        // otherwise add them to the working set and randomly pick from the 1 day categories to fill the remaining slots.
        Err(SchedulerError::Other(anyhow::anyhow!("Not implemented")))
    }

    pub async fn start_category(&self, category: &PracticeCategory) -> Result<()> {
        println!(
            "Starting {} minute practice for category: {:#?}",
            self.category_practice_time.num_minutes(),
            category
        );
        time::sleep(self.category_practice_time.to_std()?).await;
        println!("Done practicing category: {:#?}", category);
        self.play_ding_sound().await?;

        Ok(())
    }

    pub async fn play_ding_sound(&self) -> Result<()> {
        // TODO these should be singletons or maybe the Sink type idk or a separate audio player task
        // Get a output stream handle to the default physical sound device
        let (_stream, stream_handle) = OutputStream::try_default().unwrap();
        // Load a sound from a file, using a path relative to Cargo.toml
        let file = BufReader::new(File::open("audio/ding.wav").unwrap());
        // Decode that sound file into a source
        let source = Decoder::new(file).unwrap();

        stream_handle.play_raw(source.convert_samples())?;

        // need to sleep for the duration of the ding sound
        // (todo should probably move to a second thread/task/whatever in tokio)
        time::sleep(std::time::Duration::from_secs(2)).await;
        Ok(())
    }

    pub async fn start_daily_practice(&mut self) -> Result<()> {
        // ensure today's schedule has been set
        self.update_todays_schedule(false)?;
        for category in self.todays_schedule.as_ref().unwrap().iter() {
            println!("Starting practice for category: {:#?}", category);
            self.start_category(&category).await?;
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
        let decoded: Self = bincode::deserialize(&buffer[..]).unwrap();

        Ok(decoded)
    }
}

#[tokio::main]
async fn main() {
    let mut scheduler = match Path::new("./saved_data/history.bin").exists() {
        true => {
            println!("Saved data found, loading...");
            SchedulePlanner::new_from_disk().unwrap()
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
        "y\r" | "y\n" => {
            println!("Yeehaw");
            scheduler
                .start_daily_practice()
                .await
                .expect("Unable to run daily practice");
        }
        _ => {
            println!("Well, okay then.");
        }
    };
}

pub fn get_todays_schedule() -> Result<Vec<PracticeCategory>> {
    Ok(vec![])
}
