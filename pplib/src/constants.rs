use std::sync::Arc;

use crate::{PracticeExercise, PracticeSkill};

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
