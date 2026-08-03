//! Timing harness for course_relations overhead.
//! Run: `cargo bench --bench course_relations_overhead`
//!
//! Uses a partially-completed CIS plan so `generate_schedule` stays tractable.
//! Full empty dual-degree auto-placement is intentionally out of scope here
//! (tens of seconds per call on this machine).

use std::collections::HashSet;
use std::time::Instant;

use degree_planner::course_relations;
use degree_planner::cross_degree::{self, CrossDegreeState};
use degree_planner::penn_data::courses_data;
use degree_planner::scheduler::{generate_schedule, DegreeInput, FrozenCourse, ScheduleInput};

fn percentile(sorted_ms: &[f64], p: f64) -> f64 {
    if sorted_ms.is_empty() {
        return 0.0;
    }
    let idx = ((sorted_ms.len() as f64 - 1.0) * p).round() as usize;
    sorted_ms[idx.min(sorted_ms.len() - 1)]
}

fn summarize(label: &str, samples_ms: &mut [f64]) {
    samples_ms.sort_by(|a, b| a.partial_cmp(b).unwrap());
    println!(
        "{:<44} n={:<3} p50={:>8.3} ms  p95={:>8.3} ms  max={:>8.3} ms",
        label,
        samples_ms.len(),
        percentile(samples_ms, 0.50),
        percentile(samples_ms, 0.95),
        samples_ms.last().copied().unwrap_or(0.0)
    );
}

fn major(school: &str, major: &str) -> DegreeInput {
    DegreeInput {
        major: major.to_string(),
        school: school.to_string(),
        kind: "major".to_string(),
        concentrations: vec![],
        concentration: None,
    }
}

fn time_calls<F: FnMut()>(n: usize, warmup: usize, mut f: F) -> Vec<f64> {
    for _ in 0..warmup {
        f();
    }
    let mut samples = Vec::with_capacity(n);
    for _ in 0..n {
        let t0 = Instant::now();
        f();
        samples.push(t0.elapsed().as_secs_f64() * 1000.0);
    }
    samples
}

fn cis_partial_plan(extra_taken: Vec<String>) -> ScheduleInput {
    let mut taken = vec![
        "CIS 1100".into(),
        "CIS 1200".into(),
        "CIS 1600".into(),
        "CIS 1210".into(),
        "MATH 1400".into(),
        "MATH 1410".into(),
        "PHYS 0150".into(),
        "CIS 2400".into(),
        "CIS 2620".into(),
        "CIS 3200".into(),
        "WRIT 0020".into(),
        "ECON 0100".into(),
    ];
    taken.extend(extra_taken);
    ScheduleInput {
        taken,
        degrees: vec![major("SEAS", "CIS")],
        frozen: vec![
            FrozenCourse {
                course_id: "CIS 4190".into(),
                year: 3,
                semester: "Fall".into(),
            },
        ],
        allow_summer: Some(false),
        semester_cu_limits: None,
        anon_session_id: None,
    }
}

fn main() {
    println!("course_relations overhead bench");
    println!("===============================");

    let t0 = Instant::now();
    let _ = course_relations::relations();
    let cold_ms = t0.elapsed().as_secs_f64() * 1000.0;
    println!(
        "{:<44} p50={:>8.3} ms  (single cold call)",
        "Cold graph init", cold_ms
    );

    let mut warm_lookups = time_calls(500, 50, || {
        let _ = course_relations::canonical("ACCT 2110");
        let _ = course_relations::equivalent("ACCT 2110", "BEPP 2110");
        let _ = course_relations::codes_conflict("CIS 4190", "CIS 5190");
        let _ = course_relations::aliases("BEPP 2110");
        for p in course_relations::mutex_partners("CIS 5190") {
            let _ = course_relations::equivalent(p, "CIS 4190");
        }
    });
    summarize("Warm relations lookups (batch)", &mut warm_lookups);

    let cu = courses_data::cu_map();
    let mut claim_samples = time_calls(200, 20, || {
        let mut state = CrossDegreeState::new(
            vec!["SEAS".into(), "SEAS_MS".into()],
            vec!["CIS".into(), "MS_ROBO".into()],
        );
        state.register_claim("ACCT 2110", 0, cu);
        state.register_claim("BEPP 2110", 1, cu);
        let _ = state.can_claim("CIS 5200", 0, cu);
        let summary = state.to_summary_with_plan_codes(Some(&{
            let mut s = HashSet::new();
            s.insert("ACCT 2110".into());
            s.insert("BEPP 2110".into());
            s
        }));
        let _ = summary.course_allocations.len();
    });
    summarize("Claims + alias allocation mirror", &mut claim_samples);

    let baseline = cis_partial_plan(vec!["ACCT 2110".into()]);
    let mut warm_base = time_calls(20, 3, || {
        let out = generate_schedule(baseline.clone());
        assert!(out.error.is_none(), "{:?}", out.error);
    });
    summarize("Warm generate_schedule (CIS partial)", &mut warm_base);

    // Same plan + alias spelling in taken + mutex mate frozen on grid.
    let mut stress = cis_partial_plan(vec!["ACCT 2110".into(), "BEPP 2110".into()]);
    stress.frozen.push(FrozenCourse {
        course_id: "CIS 5190".into(),
        year: 3,
        semester: "Spring".into(),
    });
    let mut warm_stress = time_calls(20, 3, || {
        let out = generate_schedule(stress.clone());
        assert!(out.error.is_none(), "{:?}", out.error);
    });
    summarize("Warm generate_schedule (alias+mutex)", &mut warm_stress);

    let mut schedule_codes = HashSet::new();
    for i in 0..40 {
        schedule_codes.insert(format!("CIS {:04}", 1000 + i));
    }
    schedule_codes.insert("CIS 4190".into());
    schedule_codes.insert("CIS 5190".into());
    schedule_codes.insert("ACCT 2110".into());
    schedule_codes.insert("BEPP 2110".into());
    let mut scan = time_calls(1000, 100, || {
        let _ = cross_degree::detect_mutex_violations(&schedule_codes);
    });
    summarize("Isolated mutex scan (~40 codes)", &mut scan);

    let base_p50 = {
        let mut v = warm_base.clone();
        v.sort_by(|a, b| a.partial_cmp(b).unwrap());
        percentile(&v, 0.50)
    };
    let stress_p50 = {
        let mut v = warm_stress.clone();
        v.sort_by(|a, b| a.partial_cmp(b).unwrap());
        percentile(&v, 0.50)
    };
    let delta = stress_p50 - base_p50;

    println!();
    println!("Benchmark results (this machine)");
    println!("  Cold graph init:                 {cold_ms:.3} ms");
    println!("  generate_schedule baseline p50:  {base_p50:.3} ms");
    println!("  generate_schedule stress p50:    {stress_p50:.3} ms");
    println!("  Delta (stress - baseline) p50:   {delta:.3} ms");
    println!("  mutex scan p50:                  {:.3} ms", {
        let mut v = scan.clone();
        v.sort_by(|a, b| a.partial_cmp(b).unwrap());
        percentile(&v, 0.50)
    });
    println!();
    println!("Guidance: cold init << 100ms; relations delta ideally < 5ms p50.");
}
