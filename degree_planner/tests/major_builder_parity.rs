//! Regression snapshots for major-builder refactors. Golden JSON lives in
//! `tests/snapshots/`; regenerate with `cargo test generate_major_snapshots -- --ignored`.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use degree_planner::Major;
use degree_planner::Requirement;
use degree_planner::penn_data::college_data::{
    self, cas_concentration_names, create_anch_major, create_chem_major, create_cis_cas_major,
    create_dsgn_major, create_econ_major, create_mathecon_major, create_neur_major,
    create_psyc_major, create_ppe_major, create_phys_major, create_math_major,
};
use degree_planner::penn_data::nursing_data::{
    create_bsn_major, create_bsn_nofl_major, create_nutr_bsn_major, create_nutr_bsn_nofl_major,
};
use degree_planner::penn_data::seas_data::{
    concentration_names_for, create_ai_major, create_be_major, create_cis_major,
    create_cmpe_major, create_dmd_major, create_ee_major, create_meam_major, create_mse_major,
};
use degree_planner::penn_data::wharton_data::{
    create_wh_fl_major, create_wh_fl_mt_major, create_wh_nofl_major, create_wh_nofl_mt_major,
};
use degree_planner::requirement::PoolConstraint;
use degree_planner::schedule_template::ScheduleHint;
use serde::Serialize;

const SNAPSHOT_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/snapshots");

#[derive(Serialize)]
struct HintSnap {
    year: i32,
    semester: String,
    mode: String,
}

#[derive(Serialize)]
struct MajorSnap {
    short_name: String,
    name: String,
    requirements: Vec<Requirement>,
    concentrations: Option<BTreeMap<String, Vec<Requirement>>>,
    schedule_hints: BTreeMap<String, HintSnap>,
}

#[derive(Serialize)]
struct PoolSnap {
    category: Option<String>,
    fixed_slots_len: usize,
    flexible_slots: i32,
    constraints: Vec<PoolConstraint>,
}

fn hint_snap(h: &ScheduleHint) -> HintSnap {
    HintSnap {
        year: h.year,
        semester: h.semester.clone(),
        mode: format!("{:?}", h.mode),
    }
}

fn major_snap(major: &Major) -> MajorSnap {
    let schedule_hints = major
        .schedule_hints
        .iter()
        .map(|(k, v)| (k.clone(), hint_snap(v)))
        .collect();
    MajorSnap {
        short_name: major.short_name.clone(),
        name: major.name.clone(),
        requirements: major.requirements.clone(),
        concentrations: major.concentrations.clone(),
        schedule_hints,
    }
}

fn snap_json<T: Serialize>(value: &T) -> String {
    serde_json::to_string_pretty(value).expect("serialize snapshot")
}

fn write_snapshot(name: &str, json: &str) {
    let dir = Path::new(SNAPSHOT_DIR);
    fs::create_dir_all(dir).expect("create snapshot dir");
    fs::write(dir.join(format!("{name}.json")), json).expect("write snapshot");
}

fn assert_snapshot(name: &str, json: &str) {
    let path = Path::new(SNAPSHOT_DIR).join(format!("{name}.json"));
    let expected = fs::read_to_string(&path)
        .unwrap_or_else(|_| panic!("missing snapshot {path:?}; run generate_major_snapshots"));
    assert_eq!(expected, json, "snapshot mismatch for {name}");
}

fn assert_major_snapshot(name: &str, major: &Major) {
    assert_snapshot(name, &snap_json(&major_snap(major)));
}

fn find_pool<'a>(major: &'a Major, category: &str) -> &'a Requirement {
    major
        .requirements
        .iter()
        .find(|r| r.get_category() == category)
        .unwrap_or_else(|| panic!("pool {category:?} not found in {}", major.short_name))
}

fn pool_snap_from_req(req: &Requirement) -> PoolSnap {
    match req {
        Requirement::CoursePool {
            category,
            fixed_slots,
            flexible_slots,
            constraints,
        } => PoolSnap {
            category: category.clone(),
            fixed_slots_len: fixed_slots.len(),
            flexible_slots: *flexible_slots,
            constraints: constraints.clone(),
        },
        _ => panic!("expected CoursePool"),
    }
}

fn assert_pool_snapshot(name: &str, req: &Requirement) {
    assert_snapshot(name, &snap_json(&pool_snap_from_req(req)));
}

// ── Generator ────────────────────────────────────────────────────────────────

#[test]
#[ignore]
fn generate_major_snapshots() {
    // Nursing
    for (name, major) in [
        ("nursing_bsn", create_bsn_major()),
        ("nursing_bsn_nofl", create_bsn_nofl_major()),
        ("nursing_nutr_bsn", create_nutr_bsn_major()),
        ("nursing_nutr_bsn_nofl", create_nutr_bsn_nofl_major()),
    ] {
        write_snapshot(name, &snap_json(&major_snap(&major)));
    }

    // Wharton
    for (name, major) in [
        ("wh_fl_fnce", create_wh_fl_major(vec!["FNCE".into()])),
        ("wh_fl_mgmt_fnce", create_wh_fl_major(vec!["MGMT".into(), "FNCE".into()])),
        ("wh_nofl_fnce", create_wh_nofl_major(vec!["FNCE".into()])),
        ("wh_nofl_mgmt_fnce", create_wh_nofl_major(vec!["MGMT".into(), "FNCE".into()])),
        ("wh_fl_mt_fnce", create_wh_fl_mt_major(vec!["FNCE".into()])),
        ("wh_nofl_mt_stat", create_wh_nofl_mt_major(vec!["STAT".into()])),
    ] {
        write_snapshot(name, &snap_json(&major_snap(&major)));
    }

    // SEAS undergrad
    for (name, major) in [
        ("seas_ee", create_ee_major()),
        ("seas_mse", create_mse_major()),
        ("seas_cis", create_cis_major()),
        ("seas_ai", create_ai_major()),
        ("seas_cmpe", create_cmpe_major()),
        ("seas_be", create_be_major()),
        ("seas_dmd", create_dmd_major()),
    ] {
        write_snapshot(name, &snap_json(&major_snap(&major)));
    }
    write_snapshot(
        "seas_meam_general",
        &snap_json(&major_snap(&create_meam_major("General".into()))),
    );
    for conc in concentration_names_for("EE") {
        let key = format!("seas_ee_conc_{}", conc.replace(' ', "_"));
        // EE concentrations are embedded in create_ee_major; snapshot whole major once above.
        let _ = key;
    }
    for conc in concentration_names_for("MEAM") {
        let key = format!("seas_meam_conc_{}", conc.replace(' ', "_").replace(',', ""));
        write_snapshot(
            &key,
            &snap_json(&major_snap(&create_meam_major(conc))),
        );
    }

    // CAS
    for (name, major) in [
        ("cas_anch", create_anch_major()),
        ("cas_econ", create_econ_major()),
        ("cas_mathecon", create_mathecon_major()),
        ("cas_cis", create_cis_cas_major()),
        ("cas_chem", create_chem_major()),
        ("cas_neur", create_neur_major()),
        ("cas_psyc", create_psyc_major()),
        ("cas_dsgn", create_dsgn_major()),
    ] {
        write_snapshot(name, &snap_json(&major_snap(&major)));
    }
    for conc in cas_concentration_names("PPE") {
        let key = format!("cas_ppe_conc_{}", conc.replace(' ', "_").replace('&', "and"));
        write_snapshot(&key, &snap_json(&major_snap(&create_ppe_major(conc))));
    }
    for conc in cas_concentration_names("PHYS") {
        let key = format!("cas_phys_conc_{}", conc.replace(' ', "_"));
        write_snapshot(&key, &snap_json(&major_snap(&create_phys_major(conc))));
    }
    for conc in cas_concentration_names("MATH") {
        let key = format!("cas_math_conc_{}", conc.replace(' ', "_"));
        write_snapshot(&key, &snap_json(&major_snap(&create_math_major(conc))));
    }

    // Pool snapshots
    assert_pool_write("pool_cas_econ", find_pool(&create_econ_major(), "General Education"));
    assert_pool_write(
        "pool_cas_biol",
        find_pool(
            &college_data::create_cas_placeholder_major(
                college_data::cas_catalog_entry("BIOL").expect("BIOL"),
            ),
            "General Education",
        ),
    );
    assert_pool_write("pool_cas_neur", find_pool(&create_neur_major(), "General Education"));

    assert_pool_write(
        "pool_wh_fl_las",
        find_pool(&create_wh_fl_major(vec!["FNCE".into()]), "Liberal Arts and Sciences"),
    );
    assert_pool_write(
        "pool_wh_nofl_las",
        find_pool(&create_wh_nofl_major(vec!["FNCE".into()]), "Liberal Arts and Sciences"),
    );
    assert_pool_write(
        "pool_wh_fl_mt_las",
        find_pool(&create_wh_fl_mt_major(vec!["FNCE".into()]), "Liberal Arts and Sciences"),
    );

    let be = create_be_major();
    let be_pool = be
        .requirements
        .iter()
        .find(|r| matches!(r, Requirement::CoursePool { .. }))
        .expect("BE general electives pool");
    write_snapshot("pool_be_general_electives", &snap_json(&pool_snap_from_req(be_pool)));
}

fn assert_pool_write(name: &str, req: &Requirement) {
    write_snapshot(name, &snap_json(&pool_snap_from_req(req)));
}

// ── Parity assertions ────────────────────────────────────────────────────────

#[test]
fn nursing_majors_match_snapshots() {
    assert_major_snapshot("nursing_bsn", &create_bsn_major());
    assert_major_snapshot("nursing_bsn_nofl", &create_bsn_nofl_major());
    assert_major_snapshot("nursing_nutr_bsn", &create_nutr_bsn_major());
    assert_major_snapshot("nursing_nutr_bsn_nofl", &create_nutr_bsn_nofl_major());
}

#[test]
fn wharton_majors_match_snapshots() {
    assert_major_snapshot("wh_fl_fnce", &create_wh_fl_major(vec!["FNCE".into()]));
    assert_major_snapshot("wh_fl_mgmt_fnce", &create_wh_fl_major(vec!["MGMT".into(), "FNCE".into()]));
    assert_major_snapshot("wh_nofl_fnce", &create_wh_nofl_major(vec!["FNCE".into()]));
    assert_major_snapshot("wh_nofl_mgmt_fnce", &create_wh_nofl_major(vec!["MGMT".into(), "FNCE".into()]));
    assert_major_snapshot("wh_fl_mt_fnce", &create_wh_fl_mt_major(vec!["FNCE".into()]));
    assert_major_snapshot("wh_nofl_mt_stat", &create_wh_nofl_mt_major(vec!["STAT".into()]));
}

#[test]
fn seas_majors_match_snapshots() {
    assert_major_snapshot("seas_ee", &create_ee_major());
    assert_major_snapshot("seas_mse", &create_mse_major());
    assert_major_snapshot("seas_cis", &create_cis_major());
    assert_major_snapshot("seas_ai", &create_ai_major());
    assert_major_snapshot("seas_cmpe", &create_cmpe_major());
    assert_major_snapshot("seas_be", &create_be_major());
    assert_major_snapshot("seas_dmd", &create_dmd_major());
    assert_major_snapshot("seas_meam_general", &create_meam_major("General".into()));
    for conc in concentration_names_for("MEAM") {
        let key = format!("seas_meam_conc_{}", conc.replace(' ', "_").replace(',', ""));
        assert_major_snapshot(&key, &create_meam_major(conc));
    }
}

#[test]
fn cas_majors_match_snapshots() {
    assert_major_snapshot("cas_anch", &create_anch_major());
    assert_major_snapshot("cas_econ", &create_econ_major());
    assert_major_snapshot("cas_mathecon", &create_mathecon_major());
    assert_major_snapshot("cas_cis", &create_cis_cas_major());
    assert_major_snapshot("cas_chem", &create_chem_major());
    assert_major_snapshot("cas_neur", &create_neur_major());
    assert_major_snapshot("cas_psyc", &create_psyc_major());
    assert_major_snapshot("cas_dsgn", &create_dsgn_major());
    for conc in cas_concentration_names("PPE") {
        let key = format!("cas_ppe_conc_{}", conc.replace(' ', "_").replace('&', "and"));
        assert_major_snapshot(&key, &create_ppe_major(conc));
    }
    for conc in cas_concentration_names("PHYS") {
        let key = format!("cas_phys_conc_{}", conc.replace(' ', "_"));
        assert_major_snapshot(&key, &create_phys_major(conc));
    }
    for conc in cas_concentration_names("MATH") {
        let key = format!("cas_math_conc_{}", conc.replace(' ', "_"));
        assert_major_snapshot(&key, &create_math_major(conc));
    }
}

#[test]
fn course_pools_match_snapshots() {
    assert_pool_snapshot(
        "pool_cas_econ",
        find_pool(&create_econ_major(), "General Education"),
    );
    assert_pool_snapshot(
        "pool_cas_biol",
        find_pool(
            &college_data::create_cas_placeholder_major(
                college_data::cas_catalog_entry("BIOL").expect("BIOL"),
            ),
            "General Education",
        ),
    );
    assert_pool_snapshot(
        "pool_cas_neur",
        find_pool(&create_neur_major(), "General Education"),
    );
    assert_pool_snapshot(
        "pool_wh_fl_las",
        find_pool(&create_wh_fl_major(vec!["FNCE".into()]), "Liberal Arts and Sciences"),
    );
    assert_pool_snapshot(
        "pool_wh_nofl_las",
        find_pool(&create_wh_nofl_major(vec!["FNCE".into()]), "Liberal Arts and Sciences"),
    );
    assert_pool_snapshot(
        "pool_wh_fl_mt_las",
        find_pool(&create_wh_fl_mt_major(vec!["FNCE".into()]), "Liberal Arts and Sciences"),
    );
    let be = create_be_major();
    let be_pool = be
        .requirements
        .iter()
        .find(|r| matches!(r, Requirement::CoursePool { .. }))
        .expect("BE pool");
    assert_pool_snapshot("pool_be_general_electives", be_pool);
}

#[test]
fn wh_nofl_mt_has_no_course_pool() {
    let major = create_wh_nofl_mt_major(vec!["STAT".into()]);
    assert!(
        !major
            .requirements
            .iter()
            .any(|r| matches!(r, Requirement::CoursePool { .. }))
    );
}
