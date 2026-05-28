// Tests for preservation of TVING vendor-prefixed custom attributes during a parse/serialize
// round-trip.
//
// To run this test while enabling printing to stdout/stderr
//
//    cargo test --test tving_extension -- --show-output

pub mod common;
use dash_mpd::parse;
use common::setup_logging;

// A tving:label attribute on AdaptationSet should survive parse -> serialize -> parse, and the
// xmlns:tving namespace declaration on the root MPD should be preserved.
#[test]
fn test_tving_label_roundtrip() {
    setup_logging();
    let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<MPD xmlns="urn:mpeg:dash:schema:mpd:2011" xmlns:tving="urn:tving:dash:ext:2025" type="static">
  <Period>
    <AdaptationSet contentType="video" tving:label="premium">
      <Representation id="1" bandwidth="1000000"/>
    </AdaptationSet>
  </Period>
</MPD>"#;
    let mpd = parse(xml).unwrap();
    assert_eq!(mpd.tving.as_deref(), Some("urn:tving:dash:ext:2025"));
    let a = &mpd.periods[0].adaptations[0];
    assert_eq!(a.tving_label.as_deref(), Some("premium"));

    let serialized = mpd.to_string();
    assert!(serialized.contains(r#"tving:label="premium""#),
            "tving:label attribute lost on serialization: {serialized}");
    assert!(serialized.contains("xmlns:tving=\"urn:tving:dash:ext:2025\""),
            "xmlns:tving declaration lost on serialization: {serialized}");

    let reparsed = parse(&serialized).unwrap();
    let a2 = &reparsed.periods[0].adaptations[0];
    assert_eq!(a2.tving_label.as_deref(), Some("premium"));
    assert_eq!(reparsed.tving.as_deref(), Some("urn:tving:dash:ext:2025"));
}

// An MPD without any tving content should not gain a spurious xmlns:tving declaration.
#[test]
fn test_no_tving_namespace_when_absent() {
    setup_logging();
    let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<MPD xmlns="urn:mpeg:dash:schema:mpd:2011" type="static">
  <Period><AdaptationSet contentType="video"/></Period>
</MPD>"#;
    let mpd = parse(xml).unwrap();
    assert!(mpd.tving.is_none());
    let a = &mpd.periods[0].adaptations[0];
    assert!(a.tving_label.is_none());

    let serialized = mpd.to_string();
    assert!(!serialized.contains("xmlns:tving"),
            "unexpected xmlns:tving declaration: {serialized}");
    assert!(!serialized.contains("tving:label"),
            "unexpected tving:label attribute: {serialized}");
}

// The unprefixed `label` alias is still accepted on input, but is normalized to the prefixed form
// on output.
#[test]
fn test_tving_label_unprefixed_alias() {
    setup_logging();
    let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<MPD xmlns="urn:mpeg:dash:schema:mpd:2011" xmlns:tving="urn:tving:dash:ext:2025" type="static">
  <Period><AdaptationSet contentType="video" label="premium"/></Period>
</MPD>"#;
    let mpd = parse(xml).unwrap();
    let a = &mpd.periods[0].adaptations[0];
    assert_eq!(a.tving_label.as_deref(), Some("premium"));

    let serialized = mpd.to_string();
    assert!(serialized.contains(r#"tving:label="premium""#),
            "tving:label not emitted in prefixed form: {serialized}");
}
