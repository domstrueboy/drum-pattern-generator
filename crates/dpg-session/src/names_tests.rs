use super::*;

#[test]
fn every_pulse_goes_there_and_back() {
    for value in NoteValue::ALL {
        assert_eq!(NoteValue::from(PulseName::from(value)), value);
    }
}

#[test]
fn every_subdivision_goes_there_and_back() {
    for value in Subdivision::ALL {
        assert_eq!(Subdivision::from(SubdivisionName::from(value)), value);
    }
}

#[test]
fn names_read_like_the_menus() {
    let json = |v| serde_json::to_string(&v).unwrap();
    assert_eq!(json(PulseName::DottedEighth), "\"8.\"");
    assert_eq!(json(PulseName::EighthTriplet), "\"8t\"");
    assert_eq!(
        serde_json::to_string(&SubdivisionName::Triplet).unwrap(),
        "\"8t\""
    );
    assert_eq!(
        serde_json::to_string(&FeelName::HalfTime).unwrap(),
        "\"half\""
    );
    assert_eq!(
        serde_json::to_string(&InstrumentName::HiHat).unwrap(),
        "\"hiHat\""
    );
}
