# Spec

What the app does, organised from the original description
([spec-original-ru.md](spec-original-ru.md)). The Figma Make prototype
is one reading of it; where the two differ, this file wins. Unclear
points go to *Open questions* at the end, not into a guess in code.

## The idea

An app for drummers and anyone learning the kit, to invent and practise
rhythm patterns. Very simple to use: one screen, close to a metronome
app. Two layouts, portrait and landscape; the table below is portrait.
Every label starts with a capital letter.

## Screen, top to bottom

| Area | What's there |
|---|---|
| Bars | Length 1 / 2 / 3 / 4 bars. View: stacked (all bars visible) or scrolled left-right. Then **Bar skip**. |
| Metronome | **Metronome** toggle, BPM -/+ by 1, subdivision 4 / 8 / 8т / 16, sound button. |
| Transport | **Tap tempo** in the centre, **Start/Stop** under it. Left: **Color**, **Sound**, **Settings**. Right: **Drums** and **Metronome** faders, one above the other. |
| Notation | Full width. |
| Kit | Photo of a kit, half visible, bottom right. Left of it: **Hi-hat**, **Snare**, **Kick** buttons, each level with its drum, each with a volume fader and **S** (solo) / **M** (mute). |

The instrument buttons are pinned to their drums: they never move, even
when a submenu opens. A submenu opens under the notation and the
transport without covering them, and has **Back** at the top. In
portrait it may cover the kit photo.

**Orientation:** a very small arrow button in the top-left corner steps
through the four orientations - portrait, portrait upside down, and
landscape either way. The screen never rotates on its own.

## Notation

- Four instruments, placed by height as on a staff, with no staff lines
  drawn:

  | Instrument | Position | Notehead |
  |---|---|---|
  | Crash | top | filled diamond |
  | Hi-hat | top | x |
  | Snare | middle | normal; ghost note in parentheses |
  | Kick | bottom | normal |

- One voice, every stem up, as the prototype draws. Note values are
  written by the usual rules of notation (beams, triplets, dots) - like
  Guitar Pro without the lines. No grid in the background.
- Above the notes, a row of **note values** (4, 8, 16, triplets). Above
  that, the **8th count**: `1 и 2 и 3 и 4 и`.
- Below the notes, the **konnakol**, only under notes - nothing where
  there's no note. Syllables follow the usual konnakol groupings
  (та-ка for twos and fours, та-ки-та for threes), written by their
  first letter:

  | Note on | Syllable |
  |---|---|
  | a 16th on an 8th (`1`, `и`) | `т` |
  | a 16th between 8ths | `к` |
  | a triplet, any value | `т к т` along each group of three |

  Dotted values land on the 16th grid and take its syllable. A note
  starting a beat is `т` either way.
- Count, konnakol and notes are large, bold and bright: readable from a
  distance.
- A playhead moves in time while playing.

## Timing grid

12 ticks a quarter note - 48 a bar in 4/4 - the smallest grid that
holds every value the spec asks for. 4/4 first; other time signatures
(3/4, 6/8 and so on) are wanted later, so nothing assumes 4/4 for good.

| Value | Ticks |
|---|---|
| quarter (4) | 12 |
| dotted 8th (8.) | 9 |
| 8th (8) | 6 |
| quarter triplet (4т) | 8 |
| 8th triplet (8т) | 4 |
| 16th (16) | 3 |
| 16th triplet (16т) | 2 |
| dotted quarter (4.) | 18 |

## Metronome

- Off: grey, labelled **Metronome**. On: yellow, label still readable.
- It clicks only while playing, together with the pattern - never
  without Start.
- First launch: 120 BPM, metronome off, subdivision 4. Later launches
  start where the last one left off.
- BPM changes by 1.
- Subdivision: **4 / 8 / 8т / 16** = one, two, three or four clicks a beat.
- Sound button cycles through 5 sounds: click; Ableton-style click;
  cowbell; high clave, as on a CR-78-style drum machine; bongo.
- **Tap tempo** sets the BPM from taps, and blinks in time: beats
  brighter, the chosen subdivisions dimmer.

## Start/Stop

One button, larger than the others: Start becomes Stop while playing.

## Bar skip

A toggle like the metronome's. On: the pattern plays N bars, then is
silent for M bars, while the metronome and the time keep going; it loops
every N + M bars. Choosing opens a menu; after a choice it closes, and
the button shows the chosen variant and turns yellow.

| Column 1 | Column 2 |
|---|---|
| 1-1, 2-1, 3-1 | 2-2, 4-4, 8-8 |
| invert button (two arrows): 1-1, 1-2, 1-3 | |

`2-1` = 2 bars play, 1 is silent.

## Crash

A menu choosing where the crash plays, counted in bars of playback, not
of the pattern: **once at the start**, **every квадрат** (4 bars),
**every 2 квадрата** (8 bars), and so on. More options may come later.

## Random

Hi-hat and ghost notes each have **Random** with a count N from 1 to
24: a set of N variants, within the chosen settings; each press steps to
the next one, and after the last back to the first.

- **Hi-hat:** hi-hats at random places on the chosen pulse's grid. The
  beats (1, 2, 3, 4 in 4/4) always keep their hi-hat, where the pulse
  lands on them; each other hit stays with the chance the **Density**
  setting gives - by default a coin toss, about half. No bar is left
  empty.
- **Ghost notes:** the ghost notes for the chosen count and level,
  placed afresh for each variant.

## Hi-hat submenu

- **Pulse:** column 1: 4, 8, 16. Column 2: 4т, 8т, 16т. Column 3: 4., 8.
  The chosen one is highlighted and the notation changes with it. A
  pulse that doesn't divide the bar (4. and 8. in 4/4) runs on across
  the barline, landing somewhere new in each bar until it lines up
  again - after 3 bars in 4/4. The drift carries on across loops of
  the pattern too, counted in bars of playback like the crash, so a
  1-bar pattern still hears the full cycle. The notation redraws the
  hi-hat each loop to show what is played.
- **Random** (see above).
- **Density:** how many of Random's off-beat hits stay, sparser to
  denser; 50% by default.

## Snare submenu

- **Time feel** - the snare's main hits:

  | Feel | Snare on |
  |---|---|
  | basic | beats 2 and 4 |
  | half time | beat 3 only |
  | double time | every "и" - four a bar: 2 and 4 counted at double speed |

- **Accents** a bar: 2, 3 or 4 accented snare hits on the 8ths
  (`1 и 2 и 3 и 4 и`), added on top of the feel's main hits - never on
  a main hit or a kick. Each bar gets its own; a 1-bar pattern repeats
  them.

- **Ghost notes:** a count a bar, and **Generate**. Each press gives a
  harder variant than the last - there's no separate difficulty
  setting. Each level opens new positions, and a variant uses at least
  one of them:

  | Level | Ghost notes may go |
  |---|---|
  | 1 | on weak 16ths (`e`, `a`), not next to an accent, not on a kick |
  | 2 | on weak 16ths, next to accents too |
  | 3 | on any free 16th, 8ths too - still not on a kick |
  | 4 | on any free 16th, on a kick too |

  After the top level, Generate keeps giving new variants at the top.
- **Random** (see above).

## Kick submenu

- **Base kicks:** a count the user sets, and **Generate**. On 8ths
  only, inside the bar, on the 8ths the snare doesn't take.
- **Extra kicks:** a count a bar, and **Generate**, harder with each
  press like the ghost notes. Only on the 16ths between the 8ths, never
  on a snare hit:

  | Level | Extra kicks may go |
  |---|---|
  | 1 | on the `a` before a beat, not next to an accent |
  | 2 | on any weak 16th (`e` too), not next to an accent |
  | 3 | next to accents too |

## Sound menu

Five presets for each of hi-hat, snare and kick, chosen separately, and
switchable while the pattern plays:

| Preset | Character |
|---|---|
| Rock | open acoustic kit, like EZdrummer's default preset |
| Hip-hop | 909-style drum machine |
| Electro | 808-style drum machine |
| Jazz | soft hits, little attack - brushes or rods |
| Metal | aggressive triggered drums, hard attack |

Users can also pick their own samples. Recording happens outside the app
for now.

Accents and ghost notes: each instrument holds 1 or 2 samples, and the
user can assign each one.

| Samples | Accent | Normal | Ghost |
|---|---|---|---|
| 1 | louder | as is | quieter |
| 2 | normal, louder | normal | ghost |

More layers (hard, soft) may come later.

## Saved grooves

Wanted: a groove can be saved and opened again.

## Settings menu

Next to Color and Sound.

- **Play with the screen off:** on by default - the groove keeps
  playing when the phone locks or another app opens. Can be turned off.

## Color menu

Two faders:
- **Background colour:** from white through pastel shades and pleasant
  combinations to black.
- **Texture:** natural surfaces - stone, wood and the like - looking
  photo-real and three-dimensional, but calm: the background never pulls
  attention.

## Kit photo

A realistic photo of a real kit in a good-looking recording studio. While
playing, the hi-hat, snare and kick each light up with what the notation
plays, as in EZdrummer or Addictive Drums.

## Open questions

For the author. Answers replace the question with the rule.

1. **The crash in a silent bar.** With Bar skip on, when the crash's
   bar falls in the silent part, is the crash silent too? For now yes:
   Bar skip silences every drum, the crash included; only the metronome
   keeps going.
2. **The crash menu's first choice.** The menu lists "once at the start",
   "every квадрат", "every 2 квадрата" - is there also "never"? For now
   there isn't, and "once at the start" is the default.
3. **Back to easy.** Generate makes ghost notes and extra kicks harder
   with each press. How does the user get back to the easiest level?
   For now a new count starts the ladder again at level 1.
4. **Other accents.** The Accents menu picks 2, 3 or 4 - how does the
   user hear the same count placed differently? For now, choosing a
   count again places the accents afresh.
5. **Random's count to start with.** N runs from 1 to 24; for now it
   starts at 4.
6. **The note-value row.** Is it a value over each note (`16`, `8.`,
   `8t`), or one over each group of notes? For now one over each note,
   only where there's a note - like the konnakol.
