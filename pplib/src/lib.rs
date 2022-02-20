#[macro_use]
extern crate lazy_static;

use std;
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::collections::HashSet;
use std::fmt;
use std::fs::File;
use std::io::prelude::*;
use std::ops::Sub;
use std::rc::Rc;
use std::sync::Arc;

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
    pub(crate) static ref DEFAULT_CATEGORIES: Vec<PracticeSkill> = vec![
        PracticeSkill {
            skill_name: "Ear Training".to_string(),
            exercises: vec![
                Arc::new(PracticeExercise {
                exercise_name: "Exercise 1".to_string(),
                exercise_markdown_contents:
                            "# Ear Training Exercises
## Exercise #1

Perform one of the exercises from [Justinguitar](https://www.justinguitar.com/guitar-lessons/justin-ear-training-exercises-s1-bc-118).

".to_string(),

            }),
            Arc::new(PracticeExercise {
                exercise_name: "Exercise 2".to_string(),
                exercise_markdown_contents:
                            "# Ear Training Exercises
## Exercise #2

Play random two-note dyads and try to identify the intervals by sound.

".to_string(),

            }),
            ]
        },
        PracticeSkill {
            skill_name: "Left Hand Exercises".to_string(),
            exercises: vec![Arc::new(PracticeExercise {
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

            })]
        },
        PracticeSkill {
            skill_name: "Alternate Picking Exercises".to_string(),
            exercises: vec![Arc::new(PracticeExercise {
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

            })]
        },
        PracticeSkill {
            skill_name: "Chords".to_string(),
            exercises: vec![Arc::new(PracticeExercise {
                exercise_name: "Exercise 1".to_string(),
                exercise_markdown_contents:
                            "# Chord Exercises
## Exercise #1

Play every major chord from A to G in root position, and then every minor chord.

Move up to the next position and repeat.

".to_string(),

            })]
        },
        PracticeSkill {
            skill_name: "Scales".to_string(),
            exercises: vec![Arc::new(PracticeExercise {
                exercise_name: "Exercise 1".to_string(),
                exercise_markdown_contents:
                            "# Scale Exercises
## Exercise #1

Play a scale to a metronome in different positions. Increase the tempo after you've played the scale perfectly four times.

".to_string(),

            })]
        },
        PracticeSkill {
            skill_name: "Sight Reading".to_string(),
            exercises: vec![Arc::new(PracticeExercise {
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

            })]
        },
        PracticeSkill {
            skill_name: "Music Theory".to_string(),
            exercises: vec![Arc::new(PracticeExercise {
                exercise_name: "Exercise 1".to_string(),
                exercise_markdown_contents:
                            "# Music Theory Exercises
## Exercise #1

For every note A to G, play the note and then the relative minor.

".to_string(),

            })]
        },
        PracticeSkill {
            skill_name: "Improvisation".to_string(),
            exercises: vec![Arc::new(PracticeExercise {
                exercise_name: "Exercise 1".to_string(),
                exercise_markdown_contents:
                            "# Improvisation Exercises
## Exercise #1

Play along to a backing track.

".to_string(),

            })]
        },
        PracticeSkill {
            skill_name: "Songwriting".to_string(),
            exercises: vec![Arc::new(PracticeExercise {
                exercise_name: "Exercise 1".to_string(),
                exercise_markdown_contents:
                            "# Songwriting Exercises
## Exercise #1

Work on a song.

Maybe you could write about your song here.

".to_string(),

            })]
        },
        PracticeSkill {
            skill_name: "Rhythm".to_string(),
            exercises: vec![Arc::new(PracticeExercise {
                exercise_name: "Exercise 1".to_string(),
                exercise_markdown_contents:
                            "# Rhythm Exercises
## Exercise #1

Play an open string along to a metronome at a slow tempo.

Alternate playing whole measures as quarter notes and eighth notes.

".to_string(),

            })]
        },
        PracticeSkill {
            skill_name: "Learn A Song".to_string(),
            exercises: vec![Arc::new(PracticeExercise {
                exercise_name: "Exercise 1".to_string(),
                exercise_markdown_contents:
                            "# Learn A Song
## Exercise #1

Work on learning that song you wanted to play.

You can embed videos here, for example:

<iframe width=\"560\" height=\"315\" src=\"https://www.youtube.com/embed/Z4z4hc5gg60\" title=\"YouTube video player\" frameborder=\"0\" allow=\"accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture\" allowfullscreen></iframe>

".to_string(),

            })]
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
    pub skills: Vec<PracticeSkill>,
}

#[derive(Debug)]
pub struct SchedulePlanner<'a> {
    pub config: PlannerConfiguration,
    /// BTreeMap containing historical practice sessions.
    pub history: BTreeMap<DateTime<Utc>, Vec<PracticeSkill>>,
    pub todays_schedule: Option<Vec<PracticeSkill>>,
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
    // duplicated data right now, we aren't using Rc right and it should
    // probably be Arc anyhow
    pub schedule: Vec<Rc<PracticeSkill>>,
    pub current_skill: Rc<PracticeSkill>,
    pub current_exercise: Option<Arc<PracticeExercise>>,
    // TODO: this isn't used at all
    pub completed: HashMap<&'a PracticeSkill, bool>,
    pub time_left: Duration,
    pub start_time: DateTime<Utc>,
    pub skill_start_time: DateTime<Utc>,
}

impl<'a> PracticeSession<'a> {
    pub fn new(schedule: Vec<PracticeSkill>, current_time: DateTime<Utc>) -> Self {
        let schedule: Vec<Rc<PracticeSkill>> =
            schedule.iter().map(|c| Rc::new(c.clone())).collect();
        let current_skill = &schedule[0];
        PracticeSession {
            schedule: schedule.to_owned(),
            current_skill: Rc::clone(current_skill),
            completed: HashMap::new(),
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

        if current_exercise_idx + 1 < self.current_skill.exercises.len() {
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

        // min(usize) == 0
        // so we don't need to worry about underflow here
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
                self.current_skill = Rc::clone(skill);
                break;
            }
            i = i + 1;
        }

        self.skill_start_time = current_time;

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

impl<'a> SchedulePlanner<'_> {
    pub fn new() -> Self {
        SchedulePlanner {
            config: PlannerConfiguration {
                skills_per_day: 4,
                skill_practice_time: Duration::minutes(15),
                skill_repeat_days: 2,
                skills: DEFAULT_CATEGORIES.to_vec(),
            },
            history: BTreeMap::new(),
            todays_schedule: None,
            practicing: false,
            practice_session: None,
        }
    }

    pub fn get_todays_schedule(&self) -> Option<&Vec<PracticeSkill>> {
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
    ) -> Result<BTreeMap<Date<Utc>, HashSet<&PracticeSkill>>> {
        let n_days_back = current_time.checked_sub_signed(Duration::days(n.try_into().unwrap()));
        if n_days_back.is_none() {
            return Err(anyhow::anyhow!("Invalid historical search term"));
        }
        let mut historical_skills: BTreeMap<Date<Utc>, HashSet<&PracticeSkill>> = BTreeMap::new();

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

                    day_skills.insert(v);
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

    pub fn delete_skill_string(&mut self, skill_string: String) -> Result<()> {
        let skill: Vec<&PracticeSkill> = self
            .config
            .skills
            .iter()
            .filter(|s| s.skill_name == skill_string)
            .collect();

        if skill.len() == 0 {
            return Err(anyhow::anyhow!("Expected to find skill being deleted"));
        }

        if let Some(pos) = self.config.skills.iter().position(|x| x == skill[0]) {
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

        let mut probabilities: BTreeMap<&PracticeSkill, u64> = BTreeMap::new();

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
                .collect::<Vec<PracticeSkill>>(),
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

    pub fn start_skill(&self, skill: &PracticeSkill) -> Result<()> {
        log::debug!(
            "Starting {} minute practice for skill: {:#?}",
            self.config.skill_practice_time.num_minutes(),
            skill
        );

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
        let schedule = self.todays_schedule.as_ref().unwrap().clone();
        self.practice_session = Some(PracticeSession::new(schedule, current_time));
        for skill in self.todays_schedule.as_ref().unwrap().iter() {
            log::debug!("Starting practice for skill: {:#?}", skill);
            self.start_skill(&skill)?;
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
