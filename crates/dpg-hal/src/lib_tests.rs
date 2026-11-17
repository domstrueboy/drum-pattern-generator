use super::*;

#[test]
fn clip_leaves_full_scale_alone_and_cuts_beyond() {
    let mut samples = [0.0, 0.99, 1.0, 1.01, -1.0, -7.0];
    clip(&mut samples);
    assert_eq!(samples, [0.0, 0.99, 1.0, 1.0, -1.0, -1.0]);
}

#[test]
fn errors_read_as_sentences() {
    assert_eq!(HalError::NoDevice.to_string(), "no audio output");
    assert!(HalError::Stream("busy".into()).to_string().contains("busy"));
}
