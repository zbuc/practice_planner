#[macro_use]
extern crate lazy_static;

use std;
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::collections::HashSet;
use std::fmt;
use std::fs::File;
use std::io::prelude::*;

use anyhow::Result;
use chrono::{Date, DateTime, Duration, Utc};
use log;
use rand::prelude::*;
use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};
use thiserror::Error;

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

#[derive(Serialize, Deserialize, Clone, Hash, PartialOrd, Ord, PartialEq, Eq, Debug)]
pub struct PracticeCategory {
    pub category_name: String,
}

impl fmt::Display for PracticeCategory {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.category_name)
    }
}

#[serde_with::serde_as]
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct PlannerConfiguration {
    /// The duration each category is to be practiced.
    #[serde_as(as = "serde_with::DurationSeconds<i64>")]
    pub category_practice_time: Duration,
    /// The max number of days allowed to elapse without practicing a category.
    pub category_repeat_days: usize,
    /// The number of categories to practice per day.
    pub categories_per_day: usize,
    pub categories: Vec<PracticeCategory>,
}

#[derive(Debug)]
pub struct SchedulePlanner<'a> {
    pub config: PlannerConfiguration,
    /// BTreeMap containing historical practice sessions.
    pub history: BTreeMap<DateTime<Utc>, Vec<PracticeCategory>>,
    pub todays_schedule: Option<Vec<PracticeCategory>>,
    /// Whether a practice session is currently underway
    pub practicing: bool,
    /// The in-progress practice session
    pub practice_session: Option<PracticeSession<'a>>,
}

#[derive(Debug)]
pub struct PracticeSession<'a> {
    // TODO these could be references to the state on SchedulePlanner
    // but the lifetimes got annoying and I gave up and there is some
    // duplicated data
    pub schedule: Vec<PracticeCategory>,
    pub current_category: u64,
    pub completed: HashMap<&'a PracticeCategory, bool>,
    pub time_left: Duration,
    pub start_time: DateTime<Utc>,
    pub category_start_time: DateTime<Utc>,
}

impl<'a> PracticeSession<'a> {
    pub fn set_time_left(&mut self, time_left: Duration) {
        self.time_left = time_left;
    }
}

impl SchedulePlanner<'_> {
    pub fn new() -> Self {
        // Call this here so it's cached and faster later
        // let (_stream, _stream_handle) = OutputStream::try_default().unwrap();
        SchedulePlanner {
            //category_practice_time: Duration::minutes(15),
            config: PlannerConfiguration {
                categories_per_day: 4,
                // category_practice_time: Duration::minutes(15),
                category_practice_time: Duration::seconds(2),
                category_repeat_days: 2,
                categories: DEFAULT_CATEGORIES.to_vec(),
            },
            history: BTreeMap::new(),
            todays_schedule: None,
            practicing: false,
            practice_session: None,
        }
    }

    pub fn get_todays_schedule(&self) -> Option<&Vec<PracticeCategory>> {
        log::info!("get_todays_schedule");
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
            log::info!("key: {}", key);
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

        if self.config.categories.len() == 0 {
            return Err(SchedulerError::MissingCategories());
        }

        let past_history = self.get_history_n_days_back(self.config.category_repeat_days)?;
        let prob_bandwidth: f64 = 100.0 / self.config.category_repeat_days as f64;

        let mut probabilities: BTreeMap<&PracticeCategory, u64> = BTreeMap::new();

        for category in &self.config.categories {
            let mut seen = false;
            let mut d = 0;
            for (_day, day_categories) in past_history.iter() {
                log::info!("On day: {}", _day);
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
                .choose_multiple_weighted(
                    &mut thread_rng(),
                    self.config.categories_per_day,
                    |item| item.1.to_owned() as f64,
                )
                .unwrap()
                .map(|item| item.0.to_owned().to_owned())
                .collect::<Vec<PracticeCategory>>(),
        );
        return Ok(());
    }

    pub fn advance_practice_session(&mut self) -> Result<()> {
        log::info!("Advancing to next category...");
        if self.practice_session.is_none() {
            return Err(anyhow::anyhow!("Expected practice session"));
        }
        if self.practice_session.as_ref().unwrap().current_category as usize
            == self.practice_session.as_ref().unwrap().schedule.len() - 1
        {
            // practice session is complete
            return self.mark_todays_practice_completed();
        }

        // advance to the next category
        self.practice_session.as_mut().unwrap().current_category =
            self.practice_session.as_mut().unwrap().current_category + 1;
        self.practice_session.as_mut().unwrap().category_start_time = chrono::Utc::now();
        // self.practice_session.unwrap().category_start_time = now;
        Ok(())
    }

    pub fn start_category(&self, category: &PracticeCategory) -> Result<()> {
        log::info!(
            "Starting {} minute practice for category: {:#?}",
            self.config.category_practice_time.num_minutes(),
            category
        );

        // TODO can't sleep in yew context. need to handle differently for CLI vs
        // webapp
        // cli:
        // thread::sleep(self.category_practice_time.to_std()?);
        //
        // webapp:
        // let handle = TimeoutService::spawn(
        //     Duration::from_secs(3),
        //     self.link.callback(|_| Msg::Done),
        // );
        // // Keep the task or timer will be cancelled
        // self.timeout_job = Some(handle);

        log::info!("Done practicing category: {:#?}", category);
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
        // TODO can't sleep in yew app
        // thread::sleep(std::time::Duration::from_secs(2));
        Ok(())
    }

    pub fn stop_practicing(&mut self) -> Result<()> {
        if !self.practicing {
            return Err(anyhow::anyhow!(
                "tried to stop practicing while not practicing"
            ));
        }
        self.practicing = false;
        Ok(())
    }

    pub fn start_daily_practice<'a>(&'a mut self) -> Result<()> {
        self.practicing = true;
        // ensure today's schedule has been set
        self.update_todays_schedule(false)?;
        let schedule = self.todays_schedule.as_ref().unwrap().clone();
        self.practice_session = Some(PracticeSession {
            schedule: schedule,
            current_category: 0,
            completed: HashMap::new(),
            time_left: Duration::seconds(0),
            // TODO maybe make an Option type
            start_time: chrono::Utc::now(),
            category_start_time: chrono::Utc::now(),
        });
        for category in self.todays_schedule.as_ref().unwrap().iter() {
            log::info!("Starting practice for category: {:#?}", category);
            self.start_category(&category)?;
        }
        log::info!("Finished practicing for today!");

        // record this practice session in history, save to disk
        // self.mark_todays_practice_completed()?;

        Ok(())
    }

    fn mark_todays_practice_completed(&mut self) -> Result<()> {
        if self.todays_schedule.is_none() {
            return Err(anyhow::anyhow!(
                "today's practice must be set to mark completed"
            ));
        }

        self.practicing = false;

        // append today's practice to the history
        let now = Utc::now();
        self.history
            .insert(now, self.todays_schedule.as_ref().unwrap().to_vec());

        // unset today's practice on Self
        self.todays_schedule = None;

        // TODO can't save to disk on wasm
        // self.save_to_disk()?;
        Ok(())
    }

    pub fn save_to_disk(&self) -> Result<()> {
        // TODO need to save history and config
        let encoded: Vec<u8> = bincode::serialize(&self.config).unwrap();
        let mut file = File::create("./saved_data/history.bin")?;
        file.write_all(&encoded)?;

        Ok(())
    }

    pub fn new_from_disk() -> Result<Self> {
        // TODO need to load history and config
        let mut f = File::open("./saved_data/history.bin")?;
        let mut buffer = Vec::new();
        f.read_to_end(&mut buffer)?;
        let decoded: PlannerConfiguration = bincode::deserialize(&buffer[..])?;

        Ok(Self {
            config: decoded,
            history: BTreeMap::new(),
            todays_schedule: None,
            practicing: false,
            practice_session: None,
        })
    }
}
