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
use rand::prelude::*;
use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};
use text_io::read;
use thiserror::Error;

lazy_static! {
    pub(crate) static ref DEFAULT_CATEGORIES: Vec<PracticeSkill> = vec![
        PracticeSkill {
            skill_name: "Ear Training".to_string(),
        },
        PracticeSkill {
            skill_name: "Exercises".to_string(),
        },
        PracticeSkill {
            skill_name: "Chords".to_string(),
        },
        PracticeSkill {
            skill_name: "Scales".to_string(),
        },
        PracticeSkill {
            skill_name: "Sight Reading".to_string(),
        },
        PracticeSkill {
            skill_name: "Music Theory".to_string(),
        },
        PracticeSkill {
            skill_name: "Improvisation".to_string(),
        },
        PracticeSkill {
            skill_name: "Songwriting".to_string(),
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
    #[error("You must have at least 4 skills to practice")]
    MissingSkills(),

    #[error(transparent)]
    Other(#[from] anyhow::Error), // source and Display delegate to anyhow::Error
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub struct PracticeActivity {
    pub activity_name: String,
}

#[derive(Serialize, Deserialize, Clone, Hash, PartialOrd, Ord, PartialEq, Eq, Debug)]
pub struct PracticeSkill {
    pub skill_name: String,
}

#[serde_with::serde_as]
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct SchedulePlanner {
    /// The duration each skill is to be practiced.
    #[serde_as(as = "serde_with::DurationSeconds<i64>")]
    skill_practice_time: Duration,
    /// The max number of days allowed to elapse without practicing a skill.
    skill_repeat_days: usize,
    /// The number of skills to practice per day.
    skills_per_day: usize,
    skills: Vec<PracticeSkill>,
    /// BTreeMap containing historical practice sessions.
    history: BTreeMap<DateTime<Utc>, Vec<PracticeSkill>>,
    todays_schedule: Option<Vec<PracticeSkill>>,
}

impl SchedulePlanner {
    pub fn new() -> Self {
        // Call this here so it's cached and faster later
        // let (_stream, _stream_handle) = OutputStream::try_default().unwrap();
        SchedulePlanner {
            //skill_practice_time: Duration::minutes(15),
            skill_practice_time: Duration::minutes(1),
            skill_repeat_days: 2,
            skills: DEFAULT_CATEGORIES.to_vec(),
            history: BTreeMap::new(),
            todays_schedule: None,
            skills_per_day: 4,
        }
    }

    pub fn get_todays_schedule(&self) -> Option<&Vec<PracticeSkill>> {
        self.todays_schedule.as_ref()
    }

    /// Returns the skills seen in the last n days of history as a HashSet<&PracticeSkill>
    pub fn get_history_n_days_back(
        &self,
        n: usize,
    ) -> Result<BTreeMap<Date<Utc>, HashSet<&PracticeSkill>>> {
        let now = Utc::now();
        let n_days_back = now.checked_sub_signed(Duration::days(n.try_into().unwrap()));
        if n_days_back.is_none() {
            return Err(anyhow::anyhow!("Invalid historical search term"));
        }
        let mut historical_skills: BTreeMap<Date<Utc>, HashSet<&PracticeSkill>> = BTreeMap::new();

        // TODO keys are sorted so we could shorten iteration here
        for (key, value) in self.history.iter() {
            println!("key: {}", key);
            // if the history item is within the last n days...
            if key > &n_days_back.unwrap() {
                for v in value {
                    // insert into the HashSet for that day
                    let day_skills = match historical_skills.contains_key(&key.date()) {
                        true => historical_skills.get_mut(&key.date()).unwrap(),
                        false => {
                            let hs = HashSet::new();
                            historical_skills.insert(key.date(), hs);
                            historical_skills.get_mut(&key.date()).unwrap()
                        }
                    };

                    day_skills.insert(v);
                }
            }
        }

        Ok(historical_skills)
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

        if self.skills.len() == 0 {
            return Err(SchedulerError::MissingSkills());
        }

        let past_history = self.get_history_n_days_back(self.skill_repeat_days)?;
        let prob_bandwidth: f64 = 100.0 / self.skill_repeat_days as f64;

        let mut probabilities: BTreeMap<&PracticeSkill, u64> = BTreeMap::new();

        for skill in &self.skills {
            let mut seen = false;
            let mut d = 0;
            for (_day, day_skills) in past_history.iter() {
                println!("On day: {}", _day);
                if day_skills.contains(skill) {
                    seen = true;
                }

                // if we have seen this before, weight the probability by the day seen
                if seen {
                    probabilities.insert(skill, prob_bandwidth as u64 * d);
                }
                d = d + 1;
            }

            // if any skills do not appear in last n days history at all, set probability to 100%
            if !seen {
                probabilities.insert(skill, 100);
                continue;
            }
        }

        self.todays_schedule = Some(
            probabilities
                .iter()
                .collect::<Vec<_>>()
                .choose_multiple_weighted(&mut thread_rng(), self.skills_per_day, |item| {
                    item.1.to_owned() as f64
                })
                .unwrap()
                .map(|item| item.0.to_owned().to_owned())
                .collect::<Vec<PracticeSkill>>(),
        );
        return Ok(());
    }

    pub fn start_skill(&self, skill: &PracticeSkill) -> Result<()> {
        println!(
            "Starting {} minute practice for skill: {:#?}",
            self.skill_practice_time.num_minutes(),
            skill
        );

        thread::sleep(self.skill_practice_time.to_std()?);
        println!("Done practicing skill: {:#?}", skill);
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
        for skill in self.todays_schedule.as_ref().unwrap().iter() {
            println!("Starting practice for skill: {:#?}", skill);
            self.start_skill(&skill)?;
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
}

fn main() {
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

pub fn get_todays_schedule() -> Result<Vec<PracticeSkill>> {
    Ok(vec![])
}
