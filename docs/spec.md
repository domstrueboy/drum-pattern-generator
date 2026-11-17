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
the next one.

## Hi-hat submenu

- **Pulse:** column 1: 4, 8, 16. Column 2: 4т, 8т, 16т. Column 3: 4., 8.
  The chosen one is highlighted and the notation changes with it.
- **Random** (see above).

## Snare submenu

- **Time feel** - the snare's main hits:

  | Feel | Snare on |
  |---|---|
  | basic | beats 2 and 4 |
  | half time | beat 3 only |
  | double time | two main hits, as usual - see the open questions |

- **Accents** a bar: 2, 3 or 4 accented snare hits on the 8ths
  (`1 и 2 и 3 и 4 и`), added on top of the feel's main hits.

- **Ghost notes:** a count, and **Generate**. Mostly on weak 16ths.
  Each press of Generate gives a harder variant than the last - there's
  no separate difficulty setting. Simple variants avoid the positions
  right next to the accents; harder ones may take any 16th, but at first
  still avoid the kicks.
- **Random** (see above).

## Kick submenu

- **Base kicks:** a count the user sets, and **Generate**. On 8ths
  only, inside the bar, on the 8ths the snare doesn't take.
- **Extra kicks:** a count the user sets, and **Generate**. On 16ths,
  working like the ghost notes.

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

Two rounds were answered on 2026-11-17; this is what's left.

1. **Double time:** "two main hits, as usual" - on which 8ths? If they
   sit on beats 2 and 4, as in basic, what makes it double time - the
   hi-hat and kicks running twice as fast?
