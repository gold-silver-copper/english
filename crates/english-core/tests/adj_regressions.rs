use english_core::{Degree, EnglishCore};

#[test]
fn adjective_degree_dispatch_matches_degree_requested() {
    assert_eq!(
        EnglishCore::adjective("careful", &Degree::Positive),
        "careful"
    );
    assert_eq!(
        EnglishCore::adjective("careful", &Degree::Comparative),
        "more careful"
    );
    assert_eq!(
        EnglishCore::adjective("careful", &Degree::Superlative),
        "most careful"
    );
}

#[test]
fn adjective_degree_dispatch_stays_consistent_with_specific_helpers() {
    assert_eq!(
        EnglishCore::adjective("quiet", &Degree::Comparative),
        EnglishCore::comparative("quiet")
    );
    assert_eq!(
        EnglishCore::adjective("quiet", &Degree::Superlative),
        EnglishCore::superlative("quiet")
    );
}
