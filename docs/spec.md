# Spec

What the app does, organised from the original description
([spec-original-ru.md](spec-original-ru.md)). The Figma Make prototype
is one reading of it; where the two differ, this file wins. Unclear
points go to *Open questions* at the end, not into a guess in code.

## The idea

An app for drummers and anyone learning the kit, to invent and practise
rhythm patterns. Very simple to use: one screen, close to a metronome
app. Portrait layout. Every label starts with a capital letter.

## Screen, top to bottom

| Area | What's there |
|---|---|
| Bars | Length 1 / 2 / 3 / 4 bars. View: stacked (all bars visible) or scrolled left-right. Then **Bar skip**. |
| Metronome | **Metronome** toggle, BPM -/+ by 1, subdivision 4 / 8 / 8т / 16, sound button. |
| Transport | **Tap tempo** in the centre, **Start/Stop** under it. Left: **Color**, **Sound**. Right: **Drums** and **Metronome** faders, one above the other. |
| Notation | Full width. |
| Kit | Photo of a kit, half visible, bottom right. Left of it: **Hi-hat**, **Snare**, **Kick** buttons, each level with its drum, each with a volume fader and **S** (solo) / **M** (mute). |

The instrument buttons are pinned to their drums: they never move, even
when a submenu opens. A submenu opens under the notation and the
transport without covering them, and has **Back** at the top.

## Notation

- Four instruments, placed by height as on a staff, with no staff lines
  drawn:

  | Instrument | Position | Notehead |
  |---|---|---|
  | Crash | top | filled diamond |
  | Hi-hat | top | x |
  | Snare | middle | normal; ghost note in parentheses |
  | Kick | bottom | normal |

- Every note has a stem; note values are written by the usual rules of
  notation (beams, triplets, dots) - like Guitar Pro without the lines.
  No grid in the background.
- Above the notes, a row of **note values** (4, 8, 16, triplets). Above
  that, the **8th count**: `1 и 2 и 3 и 4 и`.
- Below the notes, the **konnakol**: on every 16th that has a note, `т`
  on a strong 16th and `к` on a weak one. Nothing where there's no note.
- Count, konnakol and notes are large, bold and bright: readable from a
  distance.
- A playhead moves in time while playing.

## Timing grid

4/4, 48 ticks a bar (12 a beat) - the smallest grid that holds every
value the spec asks for:

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

## Hi-hat submenu

- **Pulse:** column 1: 4, 8, 16. Column 2: 4т, 8т, 16т. Column 3: 4., 8.
  The chosen one is highlighted and the notation changes with it.
- **Random** with a count from 1 to 24: each press places hi-hats at
  random positions in the bar, within the chosen settings. See Q1.

## Snare submenu

- **Accents** a bar: 2, 3 or 4.
- **Time feel:**

  | Feel | Snare on |
  |---|---|
  | basic | beats 2 and 4 |
  | half time | beat 3 only |
  | double time | every "и" - four a bar |

- **Ghost notes:** a count, and **Generate**. Mostly on weak 16ths.
  Simple variants come first and avoid the positions right next to the
  accents. Harder ones may take any 16th, but at first still avoid the
  kicks.
- **Random** with a count from 1 to 24: each press places the ghost
  notes at random, within the settings.

## Kick submenu

- **Base kicks:** on 8ths only, inside the bar, on every 8th the snare
  doesn't take. **Generate**.
- **Extra kicks:** on 16ths, working like the ghost notes. **Generate**.

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

1. **Random, count 1-24:** does the number mean the number of hits
   placed, or a set of N variants that each press steps through?
2. **Snare accents and time feel:** how do 2, 3 or 4 accents combine
   with the feel, when basic time already puts the snare on 2 and 4?
   Does double time really put a snare on every "и" (four a bar)?
3. **Time signature:** always 4/4, or others too (3/4, 6/8...)?
4. **Crash:** where does it go - beat 1 of the first bar, or wherever
   the user chooses? Does it get a button and a submenu?
5. **Konnakol for triplets and dotted values:** which syllables? `т`/`к`
   covers 16ths only.
6. **Accents and ghost notes:** louder and quieter versions of one
   sound, or separate samples (hard / normal / soft / ghost)?
7. **Saving grooves:** wanted? Not in the description yet.
8. **Notation voices:** one voice, all stems up (as the prototype
   draws), or two voices - hands' stems up, kick's stems down, as most
   drum books write it?
9. **Base kicks:** "base kick shifts within the 8ths" - is that one kick
   moving among the free 8ths, or a number of kicks placed on them? How
   many?
10. **Playing with the screen off:** should the groove keep playing when
    the phone locks or another app opens?
11. **Phone layout:** a phone screen has no room for the kit photo and
    an open submenu side by side. May the submenu cover the photo (the
    instrument buttons stay put)?
