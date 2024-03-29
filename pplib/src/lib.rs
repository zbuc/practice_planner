#[macro_use]
extern crate lazy_static;

use std;
use std::collections::BTreeMap;
use std::collections::HashSet;
use std::fmt;
use std::fs::File;
use std::io::prelude::*;
use std::ops::Sub;
use std::sync::Arc;

use anyhow::Result;
use chrono::{Date, DateTime, Duration, Utc};
use log;
use rand::prelude::*;
use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};
use thiserror::Error;

mod constants;
use crate::constants::*;

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

#[derive(Serialize, Deserialize, Clone, Hash, PartialOrd, Ord, PartialEq, Eq, Debug)]
pub struct PracticeExercise {
    pub exercise_name: String,
    pub exercise_markdown_contents: String,
}

#[derive(Serialize, Deserialize, Clone, Hash, PartialOrd, Ord, PartialEq, Eq, Debug)]
pub struct PracticeSkill {
    pub skill_name: String,
    pub exercises: Vec<Arc<PracticeExercise>>,
}

impl fmt::Display for PracticeSkill {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.skill_name)
    }
}

#[serde_with::serde_as]
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct PlannerConfiguration {
    /// The duration each skill is to be practiced.
    #[serde_as(as = "serde_with::DurationSeconds<i64>")]
    pub skill_practice_time: Duration,
    /// The max number of days allowed to elapse without practicing a skill.
    pub skill_repeat_days: usize,
    /// The number of skills to practice per day.
    pub skills_per_day: usize,
    pub skills: Vec<Arc<PracticeSkill>>,
}

#[derive(Debug)]
pub struct SchedulePlanner {
    pub config: PlannerConfiguration,
    /// BTreeMap containing historical practice sessions.
    pub history: BTreeMap<DateTime<Utc>, Vec<Arc<PracticeSkill>>>,
    pub todays_schedule: Option<Vec<Arc<PracticeSkill>>>,
    /// Whether a practice session is currently underway
    pub practicing: bool,
    /// The in-progress practice session
    pub practice_session: Option<PracticeSession>,
}

/// Practice sessions. If one exists, it is active.
#[derive(Debug)]
pub struct PracticeSession {
    // TODO these could be references to the state on SchedulePlanner
    // but the lifetimes got annoying and I gave up and there is some
    // duplicated data right now, we aren't using Rc right and it should
    // probably be Arc anyhow
    pub schedule: Vec<Arc<PracticeSkill>>,
    pub current_skill: Arc<PracticeSkill>,
    pub current_exercise: Option<Arc<PracticeExercise>>,
    pub time_left: Duration,
    pub start_time: DateTime<Utc>,
    pub skill_start_time: DateTime<Utc>,
}

impl PracticeSession {
    pub fn new(schedule: Vec<Arc<PracticeSkill>>, current_time: DateTime<Utc>) -> Self {
        let schedule: Vec<Arc<PracticeSkill>> = schedule.iter().map(|c| c.clone()).collect();
        let current_skill = &schedule[0];
        PracticeSession {
            schedule: schedule.to_owned(),
            current_skill: Arc::clone(current_skill),
            time_left: Duration::seconds(0),
            // TODO maybe make an Option type
            start_time: current_time,
            skill_start_time: current_time,
            current_exercise: None,
        }
    }

    pub fn next_exercise(&mut self) {
        // if current exercise is none, use the first exercise
        if self.current_exercise.is_none() {
            // TODO: what if there are no exercises?
            self.current_exercise = Some(self.current_skill.exercises[0].clone());
            return;
        }

        let current_exercise_idx = self
            .current_skill
            .exercises
            .iter()
            .position(|e| *e == self.current_exercise.clone().unwrap())
            .unwrap();

        if current_exercise_idx + 1 >= self.current_skill.exercises.len() {
            // can't go past the last exercise
            return;
        }

        self.current_exercise =
            Some(self.current_skill.exercises[current_exercise_idx + 1].clone());
    }

    pub fn previous_exercise(&mut self) {
        // if current exercise is none, use the first exercise
        if self.current_exercise.is_none() {
            // TODO: what if there are no exercises?
            self.current_exercise = Some(self.current_skill.exercises[0].clone());
            return;
        }

        let current_exercise_idx = self
            .current_skill
            .exercises
            .iter()
            .position(|e| *e == self.current_exercise.clone().unwrap())
            .unwrap();

        if current_exercise_idx == 0 {
            return;
        }

        self.current_exercise =
            Some(self.current_skill.exercises[current_exercise_idx - 1].clone());
    }

    pub fn set_time_left(&mut self, time_left: Duration) {
        self.time_left = time_left;
    }

    pub fn get_current_skill_idx(&self) -> usize {
        self.schedule
            .iter()
            .position(|r| *r == self.current_skill)
            .expect("expected current skill to always be present in schedule")
    }

    pub fn set_current_skill_idx(&mut self, idx: usize, current_time: DateTime<Utc>) -> Result<()> {
        if idx > self.schedule.len() {
            return Err(anyhow::anyhow!("Invalid skill index"));
        }

        let mut i = 0;
        for skill in &self.schedule {
            if i == idx {
                self.current_skill = Arc::clone(skill);
                break;
            }
            i = i + 1;
        }

        self.skill_start_time = current_time;

        // select the correct exercise
        self.current_exercise = None;
        self.next_exercise();

        // can't get this working due to lifetimes
        // self.current_skill = self
        //     .schedule
        //     .iter()
        //     .enumerate()
        //     .find(|(i, &val)| *i == idx)
        //     .expect("invalid skill index")
        //     .1;
        Ok(())
    }

    // pub fn get_active_skill(&self) -> &PracticeSkill {
    //     &self.scheduler.config.skills[self
    //         .scheduler
    //         .practice_session
    //         .as_ref()
    //         .expect()
    //         .current_skill as usize];
    // }
}

impl<'a> SchedulePlanner {
    pub fn new() -> Self {
        SchedulePlanner {
            config: PlannerConfiguration {
                skills_per_day: 4,
                skill_practice_time: Duration::minutes(15),
                skill_repeat_days: 2,
                skills: DEFAULT_CATEGORIES
                    .to_vec()
                    .iter()
                    .map(|c| Arc::new(c.clone()))
                    .collect(),
            },
            history: BTreeMap::new(),
            todays_schedule: None,
            practicing: false,
            practice_session: None,
        }
    }

    pub fn get_todays_schedule(&self) -> Option<&Vec<Arc<PracticeSkill>>> {
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

    /// Returns the skills seen in the last n days of history as a HashSet<&PracticeSkill>
    pub fn get_history_n_days_back(
        &self,
        n: usize,
        current_time: DateTime<Utc>,
    ) -> Result<BTreeMap<Date<Utc>, HashSet<Arc<PracticeSkill>>>> {
        let n_days_back = current_time.checked_sub_signed(Duration::days(n.try_into().unwrap()));
        if n_days_back.is_none() {
            return Err(anyhow::anyhow!("Invalid historical search term"));
        }
        let mut historical_skills: BTreeMap<Date<Utc>, HashSet<Arc<PracticeSkill>>> =
            BTreeMap::new();

        for (key, value) in self.history.iter().rev() {
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

                    day_skills.insert(v.clone());
                }
            } else {
                // since keys are sorted we can break early
                break;
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

    pub fn reset_history(&mut self) {
        self.history = BTreeMap::new();
    }

    pub fn delete_skill(&mut self, skill: Arc<PracticeSkill>) -> Result<()> {
        if let Some(pos) = self.config.skills.iter().position(|x| *x == skill) {
            self.config.skills.remove(pos);
        }

        Ok(())
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

        if self.config.skills.len() == 0 {
            return Err(SchedulerError::MissingSkills());
        }

        let past_history =
            self.get_history_n_days_back(self.config.skill_repeat_days, current_time)?;
        let prob_bandwidth: f64 = 100.0 / self.config.skill_repeat_days as f64;

        let mut probabilities: BTreeMap<Arc<PracticeSkill>, u64> = BTreeMap::new();

        for skill in &self.config.skills {
            let mut seen = false;
            let mut d = 0;
            for (_day, day_skills) in past_history.iter() {
                log::debug!("On day: {}", _day);
                if day_skills.contains(skill) {
                    seen = true;
                }

                // if we have seen this before, weight the probability by the day seen
                if seen {
                    probabilities.insert(skill.clone(), prob_bandwidth as u64 * d);
                }
                d = d + 1;
            }

            // if any skills do not appear in last n days history at all, set probability to 100%
            if !seen {
                probabilities.insert(skill.clone(), 100);
                continue;
            }
        }

        log::debug!("probabilities: {:#?}", probabilities);
        self.todays_schedule = Some(
            probabilities
                .iter()
                .collect::<Vec<_>>()
                .choose_multiple_weighted(&mut thread_rng(), self.config.skills_per_day, |item| {
                    item.1.to_owned() as f64
                })
                .unwrap()
                .map(|item| item.0.to_owned().to_owned())
                .collect::<Vec<Arc<PracticeSkill>>>(),
        );
        return Ok(());
    }

    pub fn advance_practice_session(&mut self, current_time: DateTime<Utc>) -> Result<()> {
        log::debug!("Advancing to next skill...");
        if self.practice_session.is_none() {
            return Err(anyhow::anyhow!("Expected practice session"));
        }

        let current_skill_idx = self
            .practice_session
            .as_ref()
            .unwrap()
            .get_current_skill_idx();
        log::debug!(
            "current_skill_idx: {} schedule len: {}",
            current_skill_idx,
            self.practice_session.as_ref().unwrap().schedule.len() - 1
        );
        if current_skill_idx == self.practice_session.as_ref().unwrap().schedule.len() - 1 {
            // practice session is complete
            return self.mark_todays_practice_completed(current_time);
        }

        // advance to the next skill
        // let mut_practice = self.practice_session.as_mut().unwrap();
        // mut_practice.set_current_skill_idx(current_skill_idx, current_time)?;
        self.practice_session
            .as_mut()
            .unwrap()
            .set_current_skill_idx(current_skill_idx + 1, current_time)?;
        // self.practice_session = mut_practice;
        // self.practice_session.unwrap().skill_start_time = now;
        Ok(())
    }

    pub fn start_skill(&mut self, skill: &PracticeSkill) -> Result<()> {
        log::debug!(
            "Starting {} minute practice for skill: {:#?}",
            self.config.skill_practice_time.num_minutes(),
            skill
        );

        // Select the initial exercise to display
        // TODO: maybe remember the exercise that was left off on last practice?
        self.practice_session.as_mut().unwrap().current_exercise = None;
        self.practice_session.as_mut().unwrap().next_exercise();

        // TODO can't sleep in yew context. need to handle differently for CLI vs
        // webapp
        // cli:
        // thread::sleep(self.skill_practice_time.to_std()?);
        //
        // webapp:
        // let handle = TimeoutService::spawn(
        //     Duration::from_secs(3),
        //     self.link.callback(|_| Msg::Done),
        // );
        // // Keep the task or timer will be cancelled
        // self.timeout_job = Some(handle);

        log::debug!("Done practicing skill: {:#?}", skill);
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
        let schedule = self.todays_schedule.as_mut().unwrap().clone();
        self.practice_session = Some(PracticeSession::new(schedule, current_time));
        // TODO: this clone could lead to bugs if the UI let you edit exercises during
        // a practice session, but the mutability was giving me issues
        // so this was easiest for now
        for skill in self.todays_schedule.clone().unwrap().iter() {
            log::debug!("Starting practice for skill: {:#?}", skill);
            self.start_skill(skill)?;
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
