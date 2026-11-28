use super::*;

/// 120 BPM at 12 ticks a beat.
const SPEED: f64 = 24.0;

#[test]
fn stopped_shows_nothing() {
    let mut glide = Glide::default();
    assert_eq!(glide.at(None, 0.0, SPEED), None);
}

#[test]
fn a_new_report_is_shown_as_it_is() {
    let mut glide = Glide::default();
    assert_eq!(glide.at(Some(5.0), 1.0, SPEED), Some(5.0));
    assert_eq!(glide.at(Some(6.0), 1.04, SPEED), Some(6.0));
}

#[test]
fn between_reports_it_runs_on_at_the_tempo() {
    let mut glide = Glide::default();
    glide.at(Some(5.0), 1.0, SPEED);
    assert_eq!(
        glide.at(Some(5.0), 1.0625, SPEED),
        Some(5.0 + 0.0625 * SPEED)
    );
}

#[test]
fn a_stalled_engine_stops_the_line() {
    let mut glide = Glide::default();
    glide.at(Some(5.0), 1.0, SPEED);
    let far = 5.0 + MAX_GLIDE * SPEED;
    assert_eq!(glide.at(Some(5.0), 1.5, SPEED), Some(far));
    assert_eq!(glide.at(Some(5.0), 9.0, SPEED), Some(far));
}

#[test]
fn a_late_report_never_pulls_the_line_back() {
    let mut glide = Glide::default();
    glide.at(Some(5.0), 1.0, SPEED);
    let ahead = glide.at(Some(5.0), 1.08, SPEED).unwrap();
    assert_eq!(glide.at(Some(6.0), 1.09, SPEED), Some(ahead));
}

#[test]
fn start_again_begins_from_the_new_report() {
    let mut glide = Glide::default();
    glide.at(Some(90.0), 1.0, SPEED);
    glide.at(None, 2.0, SPEED);
    assert_eq!(glide.at(Some(0.0), 3.0, SPEED), Some(0.0));
}

#[test]
fn a_start_after_an_unseen_stop_begins_again() {
    let mut glide = Glide::default();
    glide.at(Some(90.0), 1.0, SPEED);
    glide.at(Some(90.0), 1.05, SPEED);
    assert_eq!(glide.at(Some(2.0), 3.0, SPEED), Some(2.0));
    assert_eq!(glide.at(Some(3.0), 3.04, SPEED), Some(3.0));
}

#[test]
fn laps_take_turns_one_a_pass() {
    assert_eq!(locate(10.5, 48, 3), Playhead { lap: 0, tick: 10.5 });
    assert_eq!(locate(48.0, 48, 3), Playhead { lap: 1, tick: 0.0 });
    assert_eq!(
        locate(3.0 * 48.0 + 1.0, 48, 3),
        Playhead { lap: 0, tick: 1.0 }
    );
}
