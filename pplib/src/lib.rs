#[macro_use]
extern crate lazy_static;

use std;
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::collections::HashSet;
use std::fmt;
use std::fs::File;
use std::io::prelude::*;
use std::ops::Index;
use std::ops::Sub;
use std::rc::Rc;

use anyhow::Result;
use chrono::{Date, DateTime, Duration, Utc};
use log;
use rand::prelude::*;
use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};
use thiserror::Error;

// TODO look at https://www.vexflow.com/ for notation
// https://rustwasm.github.io/wasm-bindgen/examples/import-js.html
lazy_static! {
    pub(crate) static ref DEFAULT_CATEGORIES: Vec<PracticeCategory> = vec![
        PracticeCategory {
            category_name: "Ear Training".to_string(),
            exercises: vec![
                PracticeExercise {
                exercise_name: "Exercise 1".to_string(),
                exercise_markdown_contents:
                            "# Ear Training Exercises
## Exercise #1

Perform one of the exercises from [Justinguitar](https://www.justinguitar.com/guitar-lessons/justin-ear-training-exercises-s1-bc-118).

".to_string(),

            },
            PracticeExercise {
                exercise_name: "Exercise 2".to_string(),
                exercise_markdown_contents:
                            "# Ear Training Exercises
## Exercise #2

Play random two-note dyads and try to identify the intervals by sound.

".to_string(),

            },
            ]
        },
        PracticeCategory {
            category_name: "Left Hand Exercises".to_string(),
            exercises: vec![PracticeExercise {
                exercise_name: "Exercise 1".to_string(),
                exercise_markdown_contents:
            "# Left Hand Exercises
## Exercise #1

Practice the following pattern starting at every fret from 1 to 12, starting at a lower tempo with equal note durations.

Either alternate pick or use all downstrokes.

<div class=\"vextab-auto\" width=\"680\" scale=\"1.0\" show_errors=\"true\" editor=\"false\">options space=20
tab-stems=true tab-stem-direction=up
tabstave notation=false time=4/4

notes :8 1-2-3-4/6 1-2-3-4/5 | 1-2-3-4/4 1-2-3-4/3 | 1-2-3-4/2 1-2-3-4/1 |
tabstave notation=false time=4/4
notes :8 1-2-3-4/2 1-2-3-4/3 | 1-2-3-4/4 1-2-3-4/5 | 1-2-3-4/6 :h ## =|=

options space=25
</div>
".to_string(),

            }]
        },
        PracticeCategory {
            category_name: "Alternate Picking Exercises".to_string(),
            exercises: vec![PracticeExercise {
                exercise_name: "Exercise 1".to_string(),
                exercise_markdown_contents:
                            "# Alternate Picking Exercises
## Exercise #1

Practice the following pattern starting at every fret from 1 to 12, starting at a lower tempo with equal note durations.

Use alternate picking. Try starting with either an upstroke or downstroke.

<div class=\"vextab-auto\" width=\"680\" scale=\"1.0\" show_errors=\"true\" editor=\"false\">options space=20
tab-stems=true tab-stem-direction=up
tabstave notation=false time=4/4

notes :8 1/6 2/5 3/6 4/5 1/5 2/4 3/5 4/4 | 1/4 2/3 3/4 4/3 1/3 2/2 3/3 4/2 | 1/2 2/1 3/2 4/1 1/1 2/2 3/1 4/2 |
tabstave notation=false time=4/4
notes :8 1/2 2/3 3/2 4/3 1/3 2/4 3/3 4/4 | 1/4 2/5 3/4 4/5 1/5 2/6 3/5 4/6 =|=

options space=25
</div>
```
```

".to_string(),

            }]
        },
        PracticeCategory {
            category_name: "Chords".to_string(),
            exercises: vec![PracticeExercise {
                exercise_name: "Exercise 1".to_string(),
                exercise_markdown_contents:
                            "# Chord Exercises
## Exercise #1

Play every major chord from A to G in root position, and then every minor chord.

Move up to the next position and repeat.

".to_string(),

            }]
        },
        PracticeCategory {
            category_name: "Scales".to_string(),
            exercises: vec![PracticeExercise {
                exercise_name: "Exercise 1".to_string(),
                exercise_markdown_contents:
                            "# Scale Exercises
## Exercise #1

Play a scale to a metronome in different positions. Increase the tempo after you've played the scale perfectly four times.

".to_string(),

            }]
        },
        PracticeCategory {
            category_name: "Sight Reading".to_string(),
            exercises: vec![PracticeExercise {
                exercise_name: "Exercise 1".to_string(),
                exercise_markdown_contents:
                            "# Sight Reading Exercises
## Exercise #1

Play the following passage:


<div class=\"vextab-auto\" width=\"680\" scale=\"1.0\" show_errors=\"true\" editor=\"false\">options space=20
tabstave notation=true time=4/4 tablature=false

notes :8 1-2-3-4/6 1-2-3-4/5 | 1-2-3-4/4 1-2-3-4/3 | 1-2-3-4/2 1-2-3-4/1 |
tabstave notation=true time=4/4 tablature=false
notes :8 1-2-3-4/2 1-2-3-4/3 | 1-2-3-4/4 1-2-3-4/5 | 1-2-3-4/6 :h ## =|=

options space=25
</div>
".to_string(),

            }]
        },
        PracticeCategory {
            category_name: "Music Theory".to_string(),
            exercises: vec![PracticeExercise {
                exercise_name: "Exercise 1".to_string(),
                exercise_markdown_contents:
                            "# Music Theory Exercises
## Exercise #1

For every note A to G, play the note and then the relative minor.

".to_string(),

            }]
        },
        PracticeCategory {
            category_name: "Improvisation".to_string(),
            exercises: vec![PracticeExercise {
                exercise_name: "Exercise 1".to_string(),
                exercise_markdown_contents:
                            "# Improvisation Exercises
## Exercise #1

Play along to a backing track.

".to_string(),

            }]
        },
        PracticeCategory {
            category_name: "Songwriting".to_string(),
            exercises: vec![PracticeExercise {
                exercise_name: "Exercise 1".to_string(),
                exercise_markdown_contents:
                            "# Songwriting Exercises
## Exercise #1

Work on a song.

Maybe you could write about your song here.

".to_string(),

            }]
        },
        PracticeCategory {
            category_name: "Rhythm".to_string(),
            exercises: vec![PracticeExercise {
                exercise_name: "Exercise 1".to_string(),
                exercise_markdown_contents:
                            "# Rhythm Exercises
## Exercise #1

Play an open string along to a metronome at a slow tempo.

Alternate playing whole measures as quarter notes and eighth notes.

".to_string(),

            }]
        },
        PracticeCategory {
            category_name: "Learn A Song".to_string(),
            exercises: vec![PracticeExercise {
                exercise_name: "Exercise 1".to_string(),
                exercise_markdown_contents:
                            "# Learn A Song
## Exercise #1

Work on learning that song you wanted to play.

You can embed videos here, for example:

<iframe width=\"560\" height=\"315\" src=\"https://www.youtube.com/embed/Z4z4hc5gg60\" title=\"YouTube video player\" frameborder=\"0\" allow=\"accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture\" allowfullscreen></iframe>

".to_string(),

            }]
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

#[derive(Serialize, Deserialize, Clone, Hash, PartialOrd, Ord, PartialEq, Eq, Debug)]
pub struct PracticeExercise {
    pub exercise_name: String,
    pub exercise_markdown_contents: String,
}

#[derive(Serialize, Deserialize, Clone, Hash, PartialOrd, Ord, PartialEq, Eq, Debug)]
pub struct PracticeCategory {
    pub category_name: String,
    pub exercises: Vec<PracticeExercise>,
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

/// Practice sessions. If one exists, it is active.
#[derive(Debug)]
pub struct PracticeSession<'a> {
    // TODO these could be references to the state on SchedulePlanner
    // but the lifetimes got annoying and I gave up and there is some
    // duplicated data
    pub schedule: Vec<Rc<PracticeCategory>>,
    pub current_category: Rc<PracticeCategory>,
    pub completed: HashMap<&'a PracticeCategory, bool>,
    pub time_left: Duration,
    pub start_time: DateTime<Utc>,
    pub category_start_time: DateTime<Utc>,
}

impl<'a> PracticeSession<'a> {
    pub fn new(schedule: Vec<PracticeCategory>, current_time: DateTime<Utc>) -> Self {
        let schedule: Vec<Rc<PracticeCategory>> =
            schedule.iter().map(|c| Rc::new(c.clone())).collect();
        let current_category = &schedule[0];
        PracticeSession {
            schedule: schedule.to_owned(),
            current_category: Rc::clone(current_category),
            completed: HashMap::new(),
            time_left: Duration::seconds(0),
            // TODO maybe make an Option type
            start_time: current_time,
            category_start_time: current_time,
        }
    }
    pub fn set_time_left(&mut self, time_left: Duration) {
        self.time_left = time_left;
    }

    pub fn get_current_category_idx(&self) -> usize {
        self.schedule
            .iter()
            .position(|r| *r == self.current_category)
            .expect("expected current category to always be present in schedule")
    }

    pub fn set_current_category_idx(
        &mut self,
        idx: usize,
        current_time: DateTime<Utc>,
    ) -> Result<()> {
        if idx > self.schedule.len() {
            return Err(anyhow::anyhow!("Invalid category index"));
        }

        let mut i = 0;
        for category in &self.schedule {
            if i == idx {
                self.current_category = Rc::clone(category);
                break;
            }
            i = i + 1;
        }

        self.category_start_time = current_time;

        // can't get this working due to lifetimes
        // self.current_category = self
        //     .schedule
        //     .iter()
        //     .enumerate()
        //     .find(|(i, &val)| *i == idx)
        //     .expect("invalid category index")
        //     .1;
        Ok(())
    }

    // pub fn get_active_category(&self) -> &PracticeCategory {
    //     &self.scheduler.config.categories[self
    //         .scheduler
    //         .practice_session
    //         .as_ref()
    //         .expect()
    //         .current_category as usize];
    // }
}

impl<'a> SchedulePlanner<'_> {
    pub fn new() -> Self {
        SchedulePlanner {
            config: PlannerConfiguration {
                categories_per_day: 4,
                // category_practice_time: Duration::minutes(15),
                category_practice_time: Duration::seconds(1),
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
        log::debug!("get_todays_schedule");
        self.todays_schedule.as_ref()
    }

    /// Returns the number of consecutive days of practice prior to today.
    pub fn get_streak(&self, current_time: DateTime<Utc>) -> usize {
        let mut streak_count = 0;
        let mut next_expected_day = current_time.date().sub(Duration::days(1));
        let mut counted_today = false;
        for (key, _value) in self.history.iter().rev() {
            // today counts but is not required to be present
            if key.date() == current_time.date() {
                if !counted_today {
                    streak_count = streak_count + 1;
                    counted_today = true;
                }
                continue;
            }

            if key.date() != next_expected_day {
                log::debug!("Streak broken on day: {}", key.date());
                break;
            }

            streak_count = streak_count + 1;
            next_expected_day = next_expected_day.sub(Duration::days(1));
        }

        streak_count
    }

    /// Returns the categories seen in the last n days of history as a HashSet<&PracticeCategory>
    pub fn get_history_n_days_back(
        &self,
        n: usize,
        current_time: DateTime<Utc>,
    ) -> Result<BTreeMap<Date<Utc>, HashSet<&PracticeCategory>>> {
        let n_days_back = current_time.checked_sub_signed(Duration::days(n.try_into().unwrap()));
        if n_days_back.is_none() {
            return Err(anyhow::anyhow!("Invalid historical search term"));
        }
        let mut historical_categories: BTreeMap<Date<Utc>, HashSet<&PracticeCategory>> =
            BTreeMap::new();

        for (key, value) in self.history.iter().rev() {
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
            } else {
                // since keys are sorted we can break early
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

    pub fn reset_history(&mut self) {
        self.history = BTreeMap::new();
    }

    pub fn update_todays_schedule(
        &mut self,
        force_update: bool,
        current_time: DateTime<Utc>,
    ) -> Result<(), SchedulerError> {
        if self.todays_schedule.is_some() && !force_update {
            // schedule is already set and we didn't force an update
            return Ok(());
        }

        if self.config.categories.len() == 0 {
            return Err(SchedulerError::MissingCategories());
        }

        let past_history =
            self.get_history_n_days_back(self.config.category_repeat_days, current_time)?;
        let prob_bandwidth: f64 = 100.0 / self.config.category_repeat_days as f64;

        let mut probabilities: BTreeMap<&PracticeCategory, u64> = BTreeMap::new();

        for category in &self.config.categories {
            let mut seen = false;
            let mut d = 0;
            for (_day, day_categories) in past_history.iter() {
                log::debug!("On day: {}", _day);
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

        log::debug!("probabilities: {:#?}", probabilities);
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

    pub fn advance_practice_session(&mut self, current_time: DateTime<Utc>) -> Result<()> {
        log::debug!("Advancing to next category...");
        if self.practice_session.is_none() {
            return Err(anyhow::anyhow!("Expected practice session"));
        }

        let current_category_idx = self
            .practice_session
            .as_ref()
            .unwrap()
            .get_current_category_idx();
        log::debug!(
            "current_category_idx: {} schedule len: {}",
            current_category_idx,
            self.practice_session.as_ref().unwrap().schedule.len() - 1
        );
        if current_category_idx == self.practice_session.as_ref().unwrap().schedule.len() - 1 {
            // practice session is complete
            return self.mark_todays_practice_completed(current_time);
        }

        // advance to the next category
        // let mut_practice = self.practice_session.as_mut().unwrap();
        // mut_practice.set_current_category_idx(current_category_idx, current_time)?;
        self.practice_session
            .as_mut()
            .unwrap()
            .set_current_category_idx(current_category_idx + 1, current_time)?;
        // self.practice_session = mut_practice;
        // self.practice_session.unwrap().category_start_time = now;
        Ok(())
    }

    pub fn start_category(&self, category: &PracticeCategory) -> Result<()> {
        log::debug!(
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

        log::debug!("Done practicing category: {:#?}", category);
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

    pub fn start_daily_practice(&mut self, current_time: DateTime<Utc>) -> Result<()> {
        self.practicing = true;
        // ensure today's schedule has been set
        self.update_todays_schedule(false, current_time)?;
        let schedule = self.todays_schedule.as_ref().unwrap().clone();
        self.practice_session = Some(PracticeSession::new(schedule, current_time));
        for category in self.todays_schedule.as_ref().unwrap().iter() {
            log::debug!("Starting practice for category: {:#?}", category);
            self.start_category(&category)?;
        }
        log::debug!("Finished practicing for today!");

        // record this practice session in history, save to disk
        // self.mark_todays_practice_completed()?;

        Ok(())
    }

    fn mark_todays_practice_completed(&mut self, current_time: DateTime<Utc>) -> Result<()> {
        if self.todays_schedule.is_none() {
            return Err(anyhow::anyhow!(
                "today's practice must be set to mark completed"
            ));
        }

        self.practicing = false;

        // append today's practice to the history
        self.history.insert(
            current_time,
            self.todays_schedule.as_ref().unwrap().to_vec(),
        );

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
