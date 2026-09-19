use std::time::Duration;

use sublens::frame_pacing::{repaint_interval, DEFAULT_REFRESH_RATE_MILLIHERTZ};

#[test]
fn repaint_interval_tracks_monitor_refresh_rate() {
    assert_eq!(
        repaint_interval(Some(60_000)),
        Duration::from_nanos(16_666_666)
    );
    assert_eq!(
        repaint_interval(Some(144_000)),
        Duration::from_nanos(6_944_444)
    );
}

#[test]
fn repaint_interval_falls_back_to_60_hz_for_unknown_rates() {
    assert_eq!(
        repaint_interval(None),
        repaint_interval(Some(DEFAULT_REFRESH_RATE_MILLIHERTZ))
    );
    assert_eq!(
        repaint_interval(Some(0)),
        repaint_interval(Some(DEFAULT_REFRESH_RATE_MILLIHERTZ))
    );
}
