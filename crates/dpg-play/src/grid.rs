//! The groove as text: a row an instrument, a column for each step of the
//! finest grid it uses. `X` an accent, `x` a hit, `g` a ghost.

use drum_patterns::{HitKind, Instrument, Pattern};

#[cfg(test)]
#[path = "grid_tests.rs"]
mod tests;

/// The laps of a groove as rows of text. Laps differ only in the hi-hat,
/// so one hi-hat row is shown for each.
pub fn grid(laps: &[Pattern]) -> String {
    let first = &laps[0];
    let time = first.time();
    let (per_beat, per_bar) = (time.ticks_per_beat(), time.ticks_per_bar());
    let step = laps
        .iter()
        .flat_map(Pattern::hits)
        .fold(per_beat, |g, h| gcd(g, h.tick));
    let ticks: Vec<u32> = (0..first.len_ticks()).step_by(step as usize).collect();
    let row = |label: &str, bar: char, cell: &dyn Fn(u32) -> char| {
        let mut line = format!("{label:<4}");
        for &t in &ticks {
            if t % per_bar == 0 {
                line.push(bar);
            }
            line.push(cell(t));
        }
        line.push(bar);
        line.trim_end().to_string()
    };
    let mut lines = vec![row("", ' ', &|t| {
        if t % per_beat == 0 {
            char::from_digit(t % per_bar / per_beat + 1, 10).unwrap_or('+')
        } else {
            ' '
        }
    })];
    if first.hits_of(Instrument::Crash).next().is_some() {
        lines.push(row("CR", '|', &|t| mark(first, Instrument::Crash, t)));
    }
    for (n, lap) in laps.iter().enumerate() {
        let label = if laps.len() == 1 {
            "HH".to_string()
        } else {
            format!("HH{}", n + 1)
        };
        lines.push(row(&label, '|', &|t| mark(lap, Instrument::HiHat, t)));
    }
    lines.push(row("SN", '|', &|t| mark(first, Instrument::Snare, t)));
    lines.push(row("BD", '|', &|t| mark(first, Instrument::Kick, t)));
    lines.join("\n")
}

fn mark(pattern: &Pattern, instrument: Instrument, tick: u32) -> char {
    match pattern.get(tick, instrument).map(|h| h.kind) {
        Some(HitKind::Accent) => 'X',
        Some(HitKind::Normal) => 'x',
        Some(HitKind::Ghost) => 'g',
        None => '-',
    }
}

fn gcd(a: u32, b: u32) -> u32 {
    if b == 0 { a } else { gcd(b, a % b) }
}
