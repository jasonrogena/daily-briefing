use super::*;
use chrono::TimeZone;

fn change(state: &str, ts: DateTime<Utc>) -> StateChange {
    StateChange {
        state: state.to_string(),
        last_changed: ts,
    }
}

fn ts(y: i32, mo: u32, d: u32, h: u32, m: u32) -> DateTime<Utc> {
    Utc.with_ymd_and_hms(y, mo, d, h, m, 0).unwrap()
}

// ── compute_period_total ──────────────────────────────────────────────────────

#[test]
fn test_total_empty_is_none() {
    assert!(compute_period_total(&[]).is_none());
}

#[test]
fn test_total_non_resetting_last_minus_first() {
    // Ever-increasing: 1000 → 1500 → 2000. Total = 2000 - 1000 = 1000.
    let changes = vec![
        change("1000", ts(2026, 3, 19, 5, 0)),
        change("1500", ts(2026, 3, 19, 10, 0)),
        change("2000", ts(2026, 3, 19, 20, 0)),
    ];
    let total = compute_period_total(&changes).unwrap();
    assert!((total - 1000.0).abs() < 1.0);
}

#[test]
fn test_total_resetting_sums_segment_peaks() {
    // Hourly-resetting: hour1 peaks at 500, hour2 peaks at 700.
    let changes = vec![
        change("100", ts(2026, 3, 19, 5, 10)),
        change("300", ts(2026, 3, 19, 5, 30)),
        change("500", ts(2026, 3, 19, 5, 55)),
        change("0", ts(2026, 3, 19, 6, 0)),   // reset
        change("200", ts(2026, 3, 19, 6, 20)),
        change("700", ts(2026, 3, 19, 6, 55)),
    ];
    let total = compute_period_total(&changes).unwrap();
    assert!((total - 1200.0).abs() < 1.0); // 500 + 700
}

#[test]
fn test_total_resetting_multiple_cycles() {
    // Three hourly cycles: peaks 400, 600, 300.
    let changes = vec![
        change("400", ts(2026, 3, 19, 5, 59)),
        change("0", ts(2026, 3, 19, 6, 0)),
        change("600", ts(2026, 3, 19, 6, 59)),
        change("0", ts(2026, 3, 19, 7, 0)),
        change("300", ts(2026, 3, 19, 7, 59)),
    ];
    let total = compute_period_total(&changes).unwrap();
    assert!((total - 1300.0).abs() < 1.0); // 400 + 600 + 300
}

#[test]
fn test_total_non_resetting_floors_at_zero() {
    // Pathological case: slight decrease (sensor glitch). Should not go negative.
    let changes = vec![
        change("1000", ts(2026, 3, 19, 5, 0)),
        change("990", ts(2026, 3, 19, 5, 1)), // tiny drop, not a reset (> 50%)
        change("1200", ts(2026, 3, 19, 6, 0)),
    ];
    let total = compute_period_total(&changes).unwrap();
    assert!(total >= 0.0);
}

#[test]
fn test_total_unparseable_states_ignored() {
    let changes = vec![
        change("unavailable", ts(2026, 3, 19, 5, 0)),
        change("100", ts(2026, 3, 19, 5, 30)),
        change("200", ts(2026, 3, 19, 6, 0)),
    ];
    // Only numeric values: 100 → 200, non-resetting, total = 100.
    let total = compute_period_total(&changes).unwrap();
    assert!((total - 100.0).abs() < 1.0);
}

#[test]
fn test_total_all_unparseable_is_none() {
    let changes = vec![
        change("unavailable", ts(2026, 3, 19, 5, 0)),
        change("unknown", ts(2026, 3, 19, 6, 0)),
    ];
    assert!(compute_period_total(&changes).is_none());
}

// ── format_comparison ─────────────────────────────────────────────────────────

fn make_entity(id: &str, label: Option<&str>) -> EntityConfig {
    EntityConfig {
        id: id.to_string(),
        label: label.map(str::to_string),
        kind: crate::EntityKind::Cumulative,
    }
}

fn hourly_cycle(peak: f64) -> Vec<StateChange> {
    // Simulate a sensor that starts at 0 and rises to peak.
    vec![
        change("0", ts(2026, 3, 18, 8, 0)),
        change(&(peak / 2.0).to_string(), ts(2026, 3, 18, 8, 30)),
        change(&peak.to_string(), ts(2026, 3, 18, 8, 59)),
    ]
}

#[test]
fn test_format_comparison_both_periods() {
    let entity = make_entity("sensor.grid", Some("Grid"));
    let mut ha = HashMap::new();
    let mut hb = HashMap::new();
    ha.insert("sensor.grid".into(), hourly_cycle(12000.0));
    hb.insert("sensor.grid".into(), hourly_cycle(10000.0));

    let text = format_comparison(
        &[entity],
        &ha,
        &hb,
        ts(2026, 3, 18, 8, 0),
        ts(2026, 3, 19, 8, 0),
        ts(2026, 3, 17, 8, 0),
        ts(2026, 3, 18, 8, 0),
    );

    assert!(text.contains("Grid (sensor.grid)"));
    assert!(text.contains("12000 Wh"));
    assert!(text.contains("10000 Wh"));
    assert!(text.contains("+2000 Wh"));
    assert!(text.contains("+20.0%"));
}

#[test]
fn test_format_comparison_no_previous_data() {
    let entity = make_entity("sensor.solar", None);
    let mut ha = HashMap::new();
    ha.insert("sensor.solar".into(), hourly_cycle(5000.0));
    let hb = HashMap::new();

    let text = format_comparison(
        &[entity],
        &ha,
        &hb,
        ts(2026, 3, 18, 8, 0),
        ts(2026, 3, 19, 8, 0),
        ts(2026, 3, 17, 8, 0),
        ts(2026, 3, 18, 8, 0),
    );

    assert!(text.contains("sensor.solar")); // falls back to entity id
    assert!(text.contains("5000 Wh"));
    assert!(text.contains("no data"));
}

#[test]
fn test_format_comparison_negative_delta() {
    let entity = make_entity("sensor.grid", Some("Grid"));
    let mut ha = HashMap::new();
    let mut hb = HashMap::new();
    ha.insert("sensor.grid".into(), hourly_cycle(8000.0));
    hb.insert("sensor.grid".into(), hourly_cycle(10000.0));

    let text = format_comparison(
        &[entity],
        &ha,
        &hb,
        ts(2026, 3, 18, 8, 0),
        ts(2026, 3, 19, 8, 0),
        ts(2026, 3, 17, 8, 0),
        ts(2026, 3, 18, 8, 0),
    );

    assert!(text.contains("-2000 Wh"));
    assert!(text.contains("-20.0%"));
}
