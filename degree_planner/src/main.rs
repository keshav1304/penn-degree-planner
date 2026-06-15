use std::vec;

pub mod course;
pub mod cross_degree;
pub mod major;
pub mod overlap_planner;
pub mod requirement;
pub mod schedule_template;
pub mod penn_data;

use std::collections::{BTreeMap, HashMap, HashSet};

use course::Course;
use requirement::Requirement;
use requirement::MappedRequirement;
use requirement::PoolCoverageInfo;
use requirement::ConcentrationInfo;
use penn_data::college_data::CasGenEdInfo;
use cross_degree::{is_graduate_degree, CrossDegreeSummary};
use overlap_planner::{OverlapPlan, OverlapScheduleGroup};
use major::Major;
use schedule_template::{
    later_semesters, ms_default_semester_target, ms_default_semester_target_for_requirement,
    ms_grad_placement_candidates, resolve_semester_hint, semester_order,
};

use axum:: {
    http::{header, Method, HeaderValue},
    Json, Router, routing::{delete, get, patch, post, put},
    debug_handler
};
use serde::{Serialize, Deserialize};
use tower_http::cors::{Any, CorsLayer};

const DEFAULT_SEMESTER_CU_LIMIT: f64 = 5.5;
const DUAL_UG_SEMESTER_CU_LIMIT: f64 = 6.5;
const DEFAULT_SUMMER_CU_LIMIT: f64 = 2.0;
const CU_EPS: f64 = 0.001;

fn dual_undergrad_only(schools: &[String]) -> bool {
    schools.len() >= 2 && schools.iter().all(|s| !is_graduate_degree(s))
}

fn all_cas_college(schools: &[String]) -> bool {
    !schools.is_empty() && schools.iter().all(|s| s == "CAS")
}

/// Default max CU for a semester before user overrides.
fn default_semester_cu_limit(schools: &[String], year: i32, semester: &str) -> f64 {
    if semester == "Summer" {
        return DEFAULT_SUMMER_CU_LIMIT;
    }
    if year == 1 && semester == "Fall" {
        return DEFAULT_SEMESTER_CU_LIMIT;
    }
    if dual_undergrad_only(schools) && !all_cas_college(schools) {
        return DUAL_UG_SEMESTER_CU_LIMIT;
    }
    DEFAULT_SEMESTER_CU_LIMIT
}

fn undergrad_schedule_years(schools: &[String]) -> i32 {
    if schools.len() < 2 {
        return 4;
    }
    if all_cas_college(schools) {
        return 4;
    }
    if dual_undergrad_only(schools) {
        return 5;
    }
    4
}

#[tokio::main]
async fn main() {
    let cors = CorsLayer::new()
        // Allow requests from any origin (use a specific origin in production)
        .allow_origin(Any)
        // Allow specific methods
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        // Allow specific headers
        .allow_headers([header::CONTENT_TYPE, header::AUTHORIZATION]);


    let app = Router::new()
        .route("/", get(root_get))
        .route("/", post(root_post))
        .route("/all_courses", get(all_courses_get))
        .route("/course", get(course_get))
        .route("/all_majors", get(all_majors_get))
        .route("/degree_catalog", get(degree_catalog_get))
        .route("/concentrations", get(concentrations_get))
        .route("/all_concentrations", get(all_concentrations_get))
        .route("/generate_schedule", post(generate_schedule_post))
        .layer(cors);

    let address = "0.0.0.0:8080";
    let listener = tokio::net::TcpListener::bind(address).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

#[derive(Serialize)]
struct SimpleResponse {
    response_str: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct RootPostInput {
    taken: Vec<String>,
    major: String,
    school: String,
    concentration: Option<String>,
}

#[derive(Serialize)]
struct RootPostOutput {
    fulfilled_requirements: Vec<MappedRequirement>,
    unfulfilled_requirements: Vec<MappedRequirement>,
    suggested_for_unfulfilled: Vec<MappedRequirement>,
    unapplicable_courses: Vec<String>,
    pool_coverage_info: Vec<PoolCoverageInfo>,
    error: Option<String>
}

#[debug_handler]
async fn root_get() -> Json<SimpleResponse> {
    println!("GET / request made");

    Json(SimpleResponse {
        response_str: "GET does not exist for /".to_string(),
    })
}

use major::resolve_major;

#[debug_handler]
async fn root_post(Json(payload): Json<RootPostInput>) -> Json<RootPostOutput> {
    println!("POST / request made");

    let taken = payload.taken;
    let major = payload.major;
    let school = payload.school;
    let concentrations: Vec<String> = payload
        .concentration
        .clone()
        .into_iter()
        .collect();

    let major_req: Option<Major> = resolve_major(&school, &major, &concentrations);

    let response: RootPostOutput;

    if let Some(major_req_unwrapped) = major_req {
        let all_courses = penn_data::courses_data::all_courses();
        let cu_map: HashMap<String, f64> = all_courses.iter()
            .map(|c| (c.course_code.clone(), c.cu))
            .collect();
        let validation = requirement::validate_courses_for_degree(
                major_req_unwrapped.requirements.clone(),
                &taken,
                &cu_map,
            );
        let mut fulfilled_requirements = validation.fulfilled;
        let unfulfilled_requirements = validation.unfulfilled;
        let pool_coverage_info = validation.pool_coverage_info;

        fulfilled_requirements.sort_by_key(|r| r.requirement.get_category());
        let suggested_for_unfulfilled = requirement::suggest_courses_for_requirements(
            &unfulfilled_requirements,
            &taken,
            &cu_map,
            None,
            None,
        );

        let mut unapplicable_courses = taken.clone();
        for req in &fulfilled_requirements {
            for course in &req.course_ids {
                if unapplicable_courses.contains(course) {
                    unapplicable_courses.retain(|x| x != course);
                }
            }
        }
        response = RootPostOutput {
            fulfilled_requirements,
            unfulfilled_requirements,
            suggested_for_unfulfilled,
            unapplicable_courses,
            pool_coverage_info,
            error: None,
        };
    } else {
        response = RootPostOutput { 
            fulfilled_requirements: vec![], unfulfilled_requirements: vec![], 
            suggested_for_unfulfilled: vec![], unapplicable_courses: vec![],
            pool_coverage_info: vec![],
            error: Some("Major provided is not valid or has no data associated with it yet!".to_string()),
        }
    }

    Json(response)
}

#[debug_handler]
async fn all_majors_get() -> Json<BTreeMap<String, Vec<String>>> {
    println!("GET /all_majors request made");

    Json(all_majors())

}

#[debug_handler]
async fn degree_catalog_get() -> Json<Vec<major::SchoolCatalogEntry>> {
    println!("GET /degree_catalog request made");

    Json(degree_catalog())
}

#[derive(Debug, Deserialize)]
struct ConcentrationsGetParams {
    school: String,
    major: String,
}

#[derive(Serialize)]
struct ConcentrationsResponse {
    concentrations: Vec<String>,
}

#[debug_handler]
async fn concentrations_get(
    Query(params): Query<ConcentrationsGetParams>,
) -> Json<ConcentrationsResponse> {
    println!(
        "GET /concentrations request made for {} / {}",
        params.school, params.major
    );

    Json(ConcentrationsResponse {
        concentrations: concentrations_for(&params.school, &params.major),
    })
}

#[debug_handler]
async fn all_concentrations_get() -> Json<BTreeMap<String, Vec<String>>> {
    println!("GET /all_concentrations request made");

    Json(all_concentrations())
}

#[debug_handler]
async fn all_courses_get() -> Json<Vec<Course>> {
    println!("GET /all_courses request made");

    let all_courses = penn_data::courses_data::all_courses();

    return Json(all_courses);
}

use axum::{extract::Query};

use crate::course::find_course;
use crate::major::{all_majors, all_concentrations, concentrations_for, degree_catalog};
#[derive(Debug, Deserialize)]
struct CourseGetParams {
    course_id: String,
}

#[debug_handler]
async fn course_get(Query(params): Query<CourseGetParams>) -> Json<Course> {
    println!("GET /course request made with {:?}", params);

    let course_search_result = find_course("all_courses.csv", &params.course_id);

    match course_search_result {
        Ok(res) => {
            match res {
                Some(val) => return Json(val),
                None => {
                    return Json( Course {
                        dept_code: "".to_string(),
                        course_code: "".to_string(),
                        title: "".to_string(),
                        description: None,
                        semester: None,
                        prereq: None,
                        cu: 0.0,
                        also_offered_as: None,
                        mutually_exclusive: None,
                        coreq: None,
                    } )
                }
            }
        }
        Err(e) => {
            println!("Error: {}", e);
            return Json( Course {
                dept_code: "".to_string(),
                course_code: "".to_string(),
                title: "".to_string(),
                description: None,
                semester: None,
                prereq: None,
                cu: 0.0,
                also_offered_as: None,
                mutually_exclusive: None,
                coreq: None,
            } )
        }
    }

    
}


#[derive(Debug, Clone, Deserialize)]
struct DegreeInput {
    major: String,
    school: String,
    #[serde(default)]
    concentrations: Vec<String>,
    concentration: Option<String>,
}

impl DegreeInput {
    fn effective_concentrations(&self) -> Vec<String> {
        if !self.concentrations.is_empty() {
            return major::normalize_degree_concentrations(&self.school, &self.concentrations);
        }
        self.concentration.clone().into_iter().collect()
    }
}

#[derive(Debug, Clone, Deserialize)]
struct FrozenCourse {
    course_id: String,
    year: i32,
    semester: String,
}

#[derive(Debug, Deserialize)]
struct ScheduleInput {
    taken: Vec<String>,
    degrees: Vec<DegreeInput>,
    frozen: Vec<FrozenCourse>,
    allow_summer: Option<bool>,
    semester_cu_limits: Option<HashMap<String, f64>>,
}

#[derive(Serialize)]
struct SemesterPlan {
    year: i32,
    semester: String,
    courses: Vec<String>,
    /// Open requirement placeholders (stable `req:` ids — not course codes).
    requirement_slots: Vec<String>,
    total_cu: f64,
}

#[derive(Serialize)]
struct DegreeResult {
    school: String,
    major: String,
    fulfilled_requirements: Vec<MappedRequirement>,
    unfulfilled_requirements: Vec<MappedRequirement>,
    suggested_for_unfulfilled: Vec<MappedRequirement>,
    unapplicable_courses: Vec<String>,
    pool_coverage_info: Vec<PoolCoverageInfo>,
    concentration_info: Vec<ConcentrationInfo>,
    available_concentrations: Vec<String>,
    has_core_concentration: bool,
    category_order: Vec<String>,
    cas_gen_ed: Option<CasGenEdInfo>,
    error: Option<String>,
}

#[derive(Serialize)]
struct ScheduleOutput {
    schedule: Vec<SemesterPlan>,
    degree_results: Vec<DegreeResult>,
    /// Maps requirement slot id → human-readable description for the schedule UI.
    slot_labels: HashMap<String, String>,
    cross_degree_summary: Option<CrossDegreeSummary>,
    overlap_plan: Option<OverlapPlan>,
    /// Paired requirement blocks (one CU) shown together on the schedule grid.
    overlap_schedule_groups: Vec<OverlapScheduleGroup>,
    error: Option<String>,
}

#[debug_handler]
async fn generate_schedule_post(Json(payload): Json<ScheduleInput>) -> Json<ScheduleOutput> {
    println!("POST /generate_schedule request made");
    Json(generate_schedule(payload))
}

fn schedule_target_for_dual_degrees(
    year: i32,
    semester: &str,
    schools: &[String],
) -> (i32, String) {
    if !dual_undergrad_only(schools) || all_cas_college(schools) {
        return (year, semester.to_string());
    }
    let adjusted_year = match year {
        1 | 2 => year,
        3 => 2,
        4 => 3,
        _ => 4,
    };
    (adjusted_year, semester.to_string())
}

fn generate_schedule(payload: ScheduleInput) -> ScheduleOutput {

    let mut taken: Vec<String> = payload
        .taken
        .iter()
        .filter(|c| course::is_valid_course_code(c))
        .cloned()
        .collect();
    let frozen: Vec<FrozenCourse> = payload
        .frozen
        .iter()
        .filter(|f| {
            course::is_valid_course_code(&f.course_id)
                || requirement::is_requirement_slot_id(&f.course_id)
        })
        .cloned()
        .collect();
    // Taken + frozen course codes count toward requirement fulfillment (frozen ≠ completed).
    let mut courses_for_validation: Vec<String> = taken.clone();
    for f in &frozen {
        if course::is_valid_course_code(&f.course_id) && !courses_for_validation.contains(&f.course_id) {
            courses_for_validation.push(f.course_id.clone());
        }
    }

    let mut degree_results: Vec<DegreeResult> = Vec::new();
    let mut all_suggested_courses: Vec<String> = Vec::new();
    let mut all_requirement_slots: Vec<String> = Vec::new();
    let mut slot_labels: HashMap<String, String> = HashMap::new();
    let mut item_targets: HashMap<String, (i32, String)> = HashMap::new();
    let mut ug_schedule_items: HashSet<String> = HashSet::new();
    let mut ms_schedule_items: HashSet<String> = HashSet::new();
    let mut ms_grad_schedule_items: HashSet<String> = HashSet::new();

    // Build a CU lookup map from all courses
    let all_courses = penn_data::courses_data::all_courses();
    let cu_map: HashMap<String, f64> = all_courses.iter()
        .map(|c| (c.course_code.clone(), c.cu))
        .collect();

    struct ResolvedDegree {
        input: DegreeInput,
        major_data: Major,
        concs: Vec<String>,
    }

    let mut resolved_degrees: Vec<ResolvedDegree> = Vec::new();
    let mut per_degree_validation: Vec<requirement::DegreeValidationResult> = Vec::new();
    let mut degree_schools: Vec<String> = Vec::new();
    let mut degree_majors: Vec<String> = Vec::new();

    for degree in &payload.degrees {
        let concs = degree.effective_concentrations();
        if let Some(major_data) = resolve_major(&degree.school, &degree.major, &concs) {
            let mut validation = requirement::validate_courses_for_degree(
                major_data.requirements.clone(),
                &courses_for_validation,
                &cu_map,
            );
            for mapped in &mut validation.fulfilled {
                mapped.course_ids = requirement::filter_valid_course_ids(mapped.course_ids.clone());
            }
            validation.fulfilled.retain(|m| !m.course_ids.is_empty());
            validation
                .fulfilled
                .sort_by_key(|r| r.requirement.get_category());

            per_degree_validation.push(validation);
            degree_schools.push(degree.school.clone());
            degree_majors.push(degree.major.clone());
            resolved_degrees.push(ResolvedDegree {
                input: degree.clone(),
                major_data,
                concs,
            });
        } else {
            degree_results.push(DegreeResult {
                school: degree.school.clone(),
                major: degree.major.clone(),
                fulfilled_requirements: vec![],
                unfulfilled_requirements: vec![],
                suggested_for_unfulfilled: vec![],
                unapplicable_courses: vec![],
                pool_coverage_info: vec![],
                concentration_info: vec![],
                available_concentrations: vec![],
                has_core_concentration: false,
                category_order: vec![],
                cas_gen_ed: None,
                error: Some(format!(
                    "Major '{}' in school '{}' is not implemented yet.",
                    degree.major, degree.school
                )),
            });
        }
    }

    let conc_contexts: Vec<requirement::DegreeConcentrationContext> = resolved_degrees
        .iter()
        .map(|resolved| {
            requirement::degree_concentration_context_from_major(
                &resolved.major_data.requirements,
                &resolved.major_data.concentrations,
                &resolved.concs,
            )
        })
        .collect();

    if !per_degree_validation.is_empty() {
        requirement::resolve_cross_degree_conflicts(
            &mut per_degree_validation,
            &degree_schools,
            &degree_majors,
            &cu_map,
            Some(&conc_contexts),
            Some(&courses_for_validation),
        );
    }

    let ug_conc_claims = requirement::build_ug_concentration_claims(
        &conc_contexts,
        &degree_schools,
        &per_degree_validation,
        &courses_for_validation,
        &cu_map,
    );

    let mut cross_state = cross_degree::CrossDegreeState::new(
        degree_schools.clone(),
        degree_majors.clone(),
    );
    if !per_degree_validation.is_empty() {
        let mut fulfilled_allocations =
            requirement::build_allocations_from_fulfilled(&per_degree_validation);
        requirement::merge_concentration_claims_into(&mut fulfilled_allocations, &ug_conc_claims);
        cross_state.rebuild_from_allocations(&fulfilled_allocations, &cu_map);
        cross_state.ug_concentration_courses = ug_conc_claims;
    }

    let major_refs: Vec<&Major> = resolved_degrees
        .iter()
        .map(|r| &r.major_data)
        .collect();
    for (degree_idx, resolved) in resolved_degrees.iter().enumerate() {
        per_degree_validation[degree_idx].refresh_pool_coverage_info(
            &resolved.major_data.requirements,
            &cu_map,
        );
    }
    let mut overlap_plan = if per_degree_validation.len() > 1 {
        Some(overlap_planner::compute_overlap_plan(
            &per_degree_validation,
            &major_refs,
            &degree_schools,
            &degree_majors,
            &courses_for_validation.iter().cloned().collect(),
            &cross_state,
            &cu_map,
        ))
    } else {
        None
    };

    for (degree_idx, resolved) in resolved_degrees.iter().enumerate() {
        let degree = &resolved.input;
        let major_data = &resolved.major_data;
        let concs = &resolved.concs;
        per_degree_validation[degree_idx]
            .refresh_pool_coverage_info(&major_data.requirements, &cu_map);
        let validation = &mut per_degree_validation[degree_idx];
        validation
            .fulfilled
            .sort_by_key(|r| r.requirement.get_category());

        let fulfilled = validation.fulfilled.clone();
        let unfulfilled = validation.unfulfilled.clone();
        let pool_coverage = validation.pool_coverage_info.clone();

        let suggested = requirement::suggest_courses_for_requirements(
            &unfulfilled,
            &courses_for_validation,
            &cu_map,
            Some(&cross_state),
            Some(degree_idx),
        );

        for mapped in &suggested {
            for course_id in &mapped.course_ids {
                if course::is_valid_course_code(course_id)
                    && cross_state
                        .can_claim(course_id, degree_idx, &cu_map)
                        .is_ok()
                {
                    cross_state.register_claim(course_id, degree_idx, &cu_map);
                }
            }
        }

        // Collect unique suggested courses and requirement slots for the schedule
        for mapped in &suggested {
                if let Some(instance_id) = mapped.instance_id.as_deref() {
                    if let Some(target) =
                        resolve_semester_hint(instance_id, &major_data.schedule_hints)
                    {
                        for course_id in &mapped.course_ids {
                            item_targets
                                .entry(course_id.clone())
                                .or_insert_with(|| {
                                    schedule_target_for_dual_degrees(
                                        target.0,
                                        &target.1,
                                        &degree_schools,
                                    )
                                });
                        }
                    }
                }
                if is_graduate_degree(&degree.school) {
                    for course_id in &mapped.course_ids {
                        let target = if course::is_valid_course_code(course_id) {
                            ms_default_semester_target(course_id)
                        } else if requirement::is_requirement_slot_id(course_id) {
                            ms_default_semester_target_for_requirement(&mapped.requirement)
                        } else {
                            continue;
                        };
                        item_targets
                            .entry(course_id.clone())
                            .or_insert_with(|| target.clone());
                        ms_schedule_items.insert(course_id.clone());
                        let is_grad = (course::is_valid_course_code(course_id)
                            && course::is_graduate_level(course_id))
                            || (requirement::is_requirement_slot_id(course_id) && target.0 >= 3);
                        if is_grad {
                            ms_grad_schedule_items.insert(course_id.clone());
                        }
                    }
                } else {
                    for course_id in &mapped.course_ids {
                        ug_schedule_items.insert(course_id.clone());
                    }
                }
                for course_id in &mapped.course_ids {
                    if course::is_valid_course_code(course_id)
                        && !all_suggested_courses.contains(course_id)
                        && !courses_for_validation.contains(course_id)
                        && cross_state
                            .claims
                            .get(course_id)
                            .map(|indices| indices.contains(&degree_idx))
                            .unwrap_or(false)
                    {
                        all_suggested_courses.push(course_id.clone());
                    } else if requirement::is_schedulable_requirement_slot_id(course_id)
                        && !all_requirement_slots.contains(course_id)
                    {
                        all_requirement_slots.push(course_id.clone());
                        let label = if mapped
                            .instance_id
                            .as_deref()
                            .is_some_and(|id| id.contains(":p"))
                        {
                            if let Some(pool_idx) = mapped
                                .instance_id
                                .as_deref()
                                .and_then(|id| id.split(':').next())
                                .and_then(|s| s.parse::<usize>().ok())
                            {
                                pool_coverage
                                    .iter()
                                    .find(|p| p.pool_index == pool_idx)
                                    .map(|p| format!("1 CU from {}", p.category))
                                    .unwrap_or_else(|| {
                                        mapped.requirement.slot_label_for_id(course_id)
                                    })
                            } else {
                                mapped.requirement.slot_label_for_id(course_id)
                            }
                        } else {
                            mapped.requirement.slot_label_for_id(course_id)
                        };
                        slot_labels.insert(course_id.clone(), label);
                    }
                }
            }

            for (key, target) in &major_data.schedule_hints {
                if course::is_valid_course_code(key) {
                    item_targets.insert(
                        key.clone(),
                        schedule_target_for_dual_degrees(
                            target.0,
                            &target.1,
                            &degree_schools,
                        ),
                    );
                }
            }

            let mut unapplicable = courses_for_validation.clone();
            for req in &fulfilled {
                for course in &req.course_ids {
                    unapplicable.retain(|x| x != course);
                }
            }

            let conc_info = requirement::extract_concentration_info(
                &major_data.requirements,
                &major_data.concentrations,
                &concs,
                &courses_for_validation,
                &cu_map,
                Some(&per_degree_validation[degree_idx]),
            );

            // Available concentration names
            let available_concs: Vec<String> = major_data.concentrations.as_ref()
                .map(|m| m.keys().cloned().collect())
                .unwrap_or_default();

            // Check if this major uses core concentrations
            let has_core = degree.major == "MEAM"
                || requirement::requirements_contain_concentration(&major_data.requirements);

            // Extract category order from requirement definition (includes nested CAS/DC children)
            let mut category_order: Vec<String> = Vec::new();
            for req in &major_data.requirements {
                req.collect_category_order(&mut category_order);
            }

            let cas_gen_ed = if degree.school == "CAS" {
                pool_coverage
                    .iter()
                    .find(|p| p.category == "General Education")
                    .map(|pool| {
                        penn_data::college_data::build_cas_gen_ed_info(
                            pool,
                            &penn_data::college_data::cas_auto_completed_sectors_for(&major_data.short_name),
                        )
                    })
            } else {
                None
            };

            degree_results.push(DegreeResult {
                school: degree.school.clone(),
                major: degree.major.clone(),
                fulfilled_requirements: fulfilled,
                unfulfilled_requirements: unfulfilled,
                suggested_for_unfulfilled: suggested,
                unapplicable_courses: unapplicable,
                pool_coverage_info: pool_coverage,
                concentration_info: conc_info,
                available_concentrations: available_concs,
                has_core_concentration: has_core,
                category_order,
                cas_gen_ed,
                error: None,
            });
    }

    let mut schedulable_slot_lookup: HashMap<(usize, String), String> = HashMap::new();
    for (degree_idx, result) in degree_results.iter().enumerate() {
        for mapped in &result.suggested_for_unfulfilled {
            let instance = mapped.instance_id.clone().unwrap_or_default();
            for cid in &mapped.course_ids {
                if requirement::is_schedulable_requirement_slot_id(cid) {
                    schedulable_slot_lookup.insert((degree_idx, instance.clone()), cid.clone());
                    if let Some(rest) = cid.strip_prefix("req:") {
                        let scope = rest.split(":R:").next().unwrap_or(rest);
                        schedulable_slot_lookup.insert((degree_idx, scope.to_string()), cid.clone());
                    }
                }
            }
        }
    }

    let mut overlap_schedule_groups: Vec<OverlapScheduleGroup> = Vec::new();
    let mut suppressed_overlap_slots: HashSet<String> = HashSet::new();

    if let Some(ref plan) = overlap_plan {
        for pair in &plan.pairs {
            let mut members: Vec<overlap_planner::OverlapScheduleGroupMember> = Vec::new();
            let mut resolved_schedulable: Vec<String> = Vec::new();

            for slot_ref in &pair.slots {
                let schedule_id = schedulable_slot_lookup
                    .get(&(slot_ref.degree_index, slot_ref.slot_key.clone()))
                    .cloned()
                    .or_else(|| {
                        all_requirement_slots.iter().find_map(|id| {
                            if !requirement::is_schedulable_requirement_slot_id(id) {
                                return None;
                            }
                            let rest = id.strip_prefix("req:")?;
                            if rest == slot_ref.slot_key
                                || rest.starts_with(&format!("{}:", slot_ref.slot_key))
                            {
                                Some(id.clone())
                            } else {
                                None
                            }
                        })
                    });

                if let Some(schedule_slot_id) = schedule_id {
                    resolved_schedulable.push(schedule_slot_id.clone());
                    members.push(overlap_planner::OverlapScheduleGroupMember {
                        schedule_slot_id,
                        label: slot_ref.label.clone(),
                        degree_index: slot_ref.degree_index,
                        school: slot_ref.school.clone(),
                        major: slot_ref.major.clone(),
                    });
                } else {
                    members.push(overlap_planner::OverlapScheduleGroupMember {
                        schedule_slot_id: overlap_planner::hint_key(
                            slot_ref.degree_index,
                            &slot_ref.slot_key,
                        ),
                        label: slot_ref.label.clone(),
                        degree_index: slot_ref.degree_index,
                        school: slot_ref.school.clone(),
                        major: slot_ref.major.clone(),
                    });
                }
            }

            if members.len() != 2 {
                continue;
            }

            let group_id = overlap_planner::overlap_group_schedule_id(&pair.slots);
            if resolved_schedulable.len() == 2 {
                for id in &resolved_schedulable {
                    suppressed_overlap_slots.insert(id.clone());
                }
            }

            let combined_label = members
                .iter()
                .map(|m| format!("{} ({})", m.label, m.major))
                .collect::<Vec<_>>()
                .join(" + ");
            slot_labels.insert(group_id.clone(), combined_label);

            overlap_schedule_groups.push(OverlapScheduleGroup {
                group_id,
                members,
                explanation: pair.explanation.clone(),
            });
        }
    }

    all_requirement_slots.retain(|s| !suppressed_overlap_slots.contains(s));
    for group in &overlap_schedule_groups {
        if !all_requirement_slots.contains(&group.group_id) {
            all_requirement_slots.push(group.group_id.clone());
            ug_schedule_items.insert(group.group_id.clone());
        }
    }

    let get_cu = |course_id: &str| -> f64 {
        if requirement::is_requirement_slot_id(course_id) {
            return 1.0;
        }
        *cu_map.get(course_id).unwrap_or(&1.0)
    };

    let place_in_semester = |plan: &mut SemesterPlan, item_id: &str| {
        if requirement::is_requirement_slot_id(item_id) {
            if !requirement::is_schedulable_requirement_slot_id(item_id)
                && !overlap_planner::is_overlap_schedule_group_id(item_id)
            {
                return;
            }
            if !plan.requirement_slots.contains(&item_id.to_string()) {
                plan.requirement_slots.push(item_id.to_string());
                plan.total_cu += get_cu(item_id);
            }
        } else if !plan.courses.contains(&item_id.to_string()) {
            plan.courses.push(item_id.to_string());
            plan.total_cu += get_cu(item_id);
        }
    };

    // Build schedule dynamically — expand semesters until ALL courses fit
    let allow_summer = payload.allow_summer.unwrap_or(true);
    let cu_limits = payload.semester_cu_limits.unwrap_or_default();

    let get_max_cu = |year: i32, semester: &str| -> f64 {
        let key = format!("{}-{}", year, semester);
        if let Some(&limit) = cu_limits.get(&key) {
            return limit;
        }
        default_semester_cu_limit(&degree_schools, year, semester)
    };

    // Helper: ensure schedule has semesters for a given year
    let mut schedule: Vec<SemesterPlan> = Vec::new();

    let ensure_year = |schedule: &mut Vec<SemesterPlan>, year: i32, allow_summer: bool| {
        let has_fall = schedule.iter().any(|p| p.year == year && p.semester == "Fall");
        if !has_fall {
            schedule.push(SemesterPlan {
                year,
                semester: "Fall".to_string(),
                courses: Vec::new(),
                requirement_slots: Vec::new(),
                total_cu: 0.0,
            });
            schedule.push(SemesterPlan {
                year,
                semester: "Spring".to_string(),
                courses: Vec::new(),
                requirement_slots: Vec::new(),
                total_cu: 0.0,
            });
            if allow_summer {
                schedule.push(SemesterPlan {
                    year,
                    semester: "Summer".to_string(),
                    courses: Vec::new(),
                    requirement_slots: Vec::new(),
                    total_cu: 0.0,
                });
            }
            schedule.sort_by(|a, b| {
                let sem_order = |s: &str| match s { "Fall" => 0, "Spring" => 1, "Summer" => 2, _ => 3 };
                a.year.cmp(&b.year).then(sem_order(&a.semester).cmp(&sem_order(&b.semester)))
            });
        }
    };

    let initial_years = undergrad_schedule_years(&degree_schools);
    for yr in 1..=initial_years {
        ensure_year(&mut schedule, yr, allow_summer);
    }

    // Place frozen items first (courses and requirement slots)
    for frozen in &frozen {
        ensure_year(&mut schedule, frozen.year, allow_summer);
        for plan in schedule.iter_mut() {
            if plan.year == frozen.year && plan.semester == frozen.semester {
                place_in_semester(plan, &frozen.course_id);
            }
        }
        all_suggested_courses.retain(|c| c != &frozen.course_id);
        all_requirement_slots.retain(|s| s != &frozen.course_id);
    }

    // Courses and requirement slots share one queue so UG items always compete
    // fairly with MS items regardless of item type.
    let mut remaining_items: Vec<String> = all_suggested_courses;
    for slot in all_requirement_slots {
        if !remaining_items.contains(&slot) {
            remaining_items.push(slot);
        }
    }

    let has_undergrad = payload
        .degrees
        .iter()
        .any(|d| !is_graduate_degree(&d.school));

    let schedule_item_priority = |item: &str| -> u8 {
        if ug_schedule_items.contains(item) {
            0
        } else if !ms_schedule_items.contains(item) {
            1
        } else if ms_grad_schedule_items.contains(item) {
            3
        } else {
            2
        }
    };

    let item_fits_semester =
        |item_id: &str, plan_total_cu: f64, max_cu: f64| -> bool {
            let cu = get_cu(item_id);
            plan_total_cu + cu <= max_cu + CU_EPS
        };

    let find_best_fitting = |remaining: &[String],
                             plan_total_cu: f64,
                             max_cu: f64,
                             skip_ids: &HashSet<String>,
                             only_items: Option<&HashSet<String>>| -> Option<usize> {
        let mut best_idx: Option<usize> = None;
        let mut best_priority = u8::MAX;
        for (idx, item) in remaining.iter().enumerate() {
            if skip_ids.contains(item) {
                continue;
            }
            if only_items.is_some_and(|set| !set.contains(item)) {
                continue;
            }
            if !item_fits_semester(item, plan_total_cu, max_cu) {
                continue;
            }
            let priority = schedule_item_priority(item);
            if priority < best_priority {
                best_priority = priority;
                best_idx = Some(idx);
            }
        }
        best_idx
    };

    let pop_best_fitting = |remaining: &mut Vec<String>,
                            plan_total_cu: f64,
                            max_cu: f64,
                            skip_ids: &HashSet<String>,
                            only_items: Option<&HashSet<String>>| -> Option<String> {
        find_best_fitting(remaining, plan_total_cu, max_cu, skip_ids, only_items)
            .map(|idx| remaining.remove(idx))
    };

    let try_place_item =
        |schedule: &mut Vec<SemesterPlan>, item_id: &str, year: i32, semester: &str| -> bool {
            ensure_year(schedule, year, allow_summer);
            for plan in schedule.iter_mut() {
                if plan.year == year && plan.semester == semester {
                    let already = if requirement::is_requirement_slot_id(item_id) {
                        plan.requirement_slots.contains(&item_id.to_string())
                    } else {
                        plan.courses.contains(&item_id.to_string())
                    };
                    if already {
                        return true;
                    }
                    let cu = get_cu(item_id);
                    let max_cu = get_max_cu(year, semester);
                    if plan.total_cu + cu <= max_cu || plan.total_cu == 0.0 {
                        place_in_semester(plan, item_id);
                        return true;
                    }
                    return false;
                }
            }
            false
        };

    let undergrad_schedule_window = undergrad_schedule_years(&degree_schools);

    let place_with_template =
        |schedule: &mut Vec<SemesterPlan>, item_id: &str, target: &(i32, String)| -> bool {
            let max_year = if has_undergrad && ms_schedule_items.contains(item_id) {
                12
            } else {
                undergrad_schedule_window
            };
            let candidates = if has_undergrad && ms_schedule_items.contains(item_id) {
                ms_grad_placement_candidates(
                    (target.0, target.1.as_str()),
                    undergrad_schedule_window,
                    max_year,
                )
            } else {
                later_semesters((target.0, target.1.as_str()), max_year)
            };
            for (year, semester) in candidates {
                if try_place_item(schedule, item_id, year, &semester) {
                    return true;
                }
            }
            false
        };

    let try_place_greedy =
        |schedule: &mut Vec<SemesterPlan>, item_id: &str, max_year: i32| -> bool {
            let mut best: Option<(i32, String)> = None;
            let mut best_load = f64::MAX;
            let mut best_tie_ord = i32::MIN;
            for year in 1..=max_year {
                for semester in ["Fall", "Spring"] {
                    let max_cu = get_max_cu(year, semester);
                    let load = schedule
                        .iter()
                        .find(|p| p.year == year && p.semester == semester)
                        .map(|p| p.total_cu)
                        .unwrap_or(0.0);
                    if !item_fits_semester(item_id, load, max_cu) {
                        continue;
                    }
                    let tie_ord = semester_order(year, semester);
                    if load < best_load
                        || (load == best_load && tie_ord < best_tie_ord)
                    {
                        best_load = load;
                        best_tie_ord = tie_ord;
                        best = Some((year, semester.to_string()));
                    }
                }
            }
            if let Some((year, semester)) = best {
                return try_place_item(schedule, item_id, year, &semester);
            }
            false
        };

    let sort_schedule_items = |items: &mut [String]| {
        items.sort_by_key(|item| {
            let template_ord = item_targets
                .get(item)
                .map(|(y, s)| semester_order(*y, s))
                .unwrap_or(i32::MAX);
            (schedule_item_priority(item), template_ord)
        });
    };

    let place_schedule_batch =
        |items: &mut Vec<String>, schedule: &mut Vec<SemesterPlan>, greedy_max_year: i32| {
            sort_schedule_items(items);
            let mut overflow = Vec::new();
            for item in items.drain(..) {
                let placed = if let Some(target) = item_targets.get(&item) {
                    place_with_template(schedule, &item, target)
                } else {
                    try_place_greedy(schedule, &item, greedy_max_year)
                };
                if !placed {
                    overflow.push(item);
                }
            }
            overflow
        };

    let partition_and_place = |remaining: &mut Vec<String>, schedule: &mut Vec<SemesterPlan>| {
        let mut items: Vec<String> = remaining.drain(..).collect();
        let mut overflow = Vec::new();
        if has_undergrad {
            let (mut ug_items, mut other_items): (Vec<String>, Vec<String>) =
                items.into_iter().partition(|item| ug_schedule_items.contains(item));
            overflow.extend(place_schedule_batch(
                &mut ug_items,
                schedule,
                undergrad_schedule_window,
            ));
            overflow.extend(place_schedule_batch(&mut other_items, schedule, 12));
        } else {
            overflow.extend(place_schedule_batch(&mut items, schedule, 12));
        }
        *remaining = overflow;
    };

    partition_and_place(&mut remaining_items, &mut schedule);

    if has_undergrad || !ms_schedule_items.is_empty() {
        remaining_items.sort_by_key(|item| schedule_item_priority(item));
    }

    let distribute = |remaining: &mut Vec<String>,
                        schedule: &mut Vec<SemesterPlan>,
                        allow_summer: bool,
                        skip_summer_for: &HashSet<String>,
                        only_items: Option<&HashSet<String>>,
                        year_range: Option<(i32, i32)>|
     -> bool {
        if remaining.is_empty() {
            return false;
        }
        let empty_skip: HashSet<String> = HashSet::new();
        let mut placed_any = false;
        loop {
            if remaining.is_empty() {
                break;
            }
            if only_items.is_some_and(|set| !remaining.iter().any(|item| set.contains(item))) {
                break;
            }

            let mut best_plan_idx: Option<usize> = None;
            let mut best_item_idx: Option<usize> = None;
            let mut best_load = f64::MAX;
            let mut best_tie_ord = i32::MAX;

            for (plan_idx, plan) in schedule.iter().enumerate() {
                if plan.semester == "Summer" && !allow_summer {
                    continue;
                }
                if let Some((min_y, max_y)) = year_range {
                    if plan.year < min_y || plan.year > max_y {
                        continue;
                    }
                }

                let max_cu = get_max_cu(plan.year, &plan.semester);
                let skip_ids = if plan.semester == "Summer" {
                    skip_summer_for
                } else {
                    &empty_skip
                };
                let Some(item_idx) = find_best_fitting(
                    remaining,
                    plan.total_cu,
                    max_cu,
                    skip_ids,
                    only_items,
                ) else {
                    continue;
                };

                let tie_ord = semester_order(plan.year, &plan.semester);
                if plan.total_cu < best_load
                    || (plan.total_cu == best_load && tie_ord < best_tie_ord)
                {
                    best_load = plan.total_cu;
                    best_tie_ord = tie_ord;
                    best_plan_idx = Some(plan_idx);
                    best_item_idx = Some(item_idx);
                }
            }

            let (plan_idx, item_idx) = match (best_plan_idx, best_item_idx) {
                (Some(p), Some(i)) => (p, i),
                _ => break,
            };
            let item = remaining.remove(item_idx);
            place_in_semester(&mut schedule[plan_idx], &item);
            placed_any = true;
        }
        placed_any
    };

    // UG overflow: fill earlier open semesters before any MS placement.
    if has_undergrad && remaining_items.iter().any(|item| ug_schedule_items.contains(item)) {
        loop {
            if !remaining_items
                .iter()
                .any(|item| ug_schedule_items.contains(item))
            {
                break;
            }
            let placed = distribute(
                &mut remaining_items,
                &mut schedule,
                allow_summer,
                &ms_grad_schedule_items,
                Some(&ug_schedule_items),
                Some((1, undergrad_schedule_window)),
            );
            if !placed {
                break;
            }
        }
    }

    // MS courses only after UG: years 3–4, then year 2, then year 5+.
    if has_undergrad {
        for (min_y, max_y) in [(3, 4), (2, 2)] {
            loop {
                if !remaining_items
                    .iter()
                    .any(|item| ms_schedule_items.contains(item))
                {
                    break;
                }
                let placed = distribute(
                    &mut remaining_items,
                    &mut schedule,
                    allow_summer,
                    &ms_grad_schedule_items,
                    Some(&ms_schedule_items),
                    Some((min_y, max_y)),
                );
                if !placed {
                    break;
                }
            }
        }
    }

    if payload.degrees.len() > 1 && has_undergrad && allow_summer {
        let empty_skip: HashSet<String> = HashSet::new();
        for year in 1..=undergrad_schedule_window {
            loop {
                let before = remaining_items.len();
                if before == 0 {
                    break;
                }
                if !distribute(
                    &mut remaining_items,
                    &mut schedule,
                    true,
                    &empty_skip,
                    Some(&ug_schedule_items),
                    Some((year, year)),
                ) {
                    break;
                }
                if remaining_items.len() == before {
                    break;
                }
            }
        }
    }

    let squeeze_undergrad_remaining =
        |remaining: &mut Vec<String>, schedule: &mut Vec<SemesterPlan>| -> bool {
            if remaining.is_empty() || !has_undergrad {
                return false;
            }
            let mut placed_any = false;
            let semesters: Vec<&str> = if allow_summer {
                vec!["Fall", "Spring", "Summer"]
            } else {
                vec!["Fall", "Spring"]
            };
            let mut i = 0;
            while i < remaining.len() {
                let item = remaining[i].clone();
                let mut placed = false;
                'place: for year in 1..=undergrad_schedule_window {
                    for semester in &semesters {
                        if try_place_item(schedule, &item, year, *semester) {
                            placed = true;
                            placed_any = true;
                            break 'place;
                        }
                    }
                }
                if !placed && payload.degrees.len() > 1 {
                    let max_existing = schedule.iter().map(|p| p.year).max().unwrap_or(undergrad_schedule_window);
                    for year in (undergrad_schedule_window + 1)..=(max_existing + 1) {
                        ensure_year(schedule, year, allow_summer);
                        for semester in &semesters {
                            if try_place_item(schedule, &item, year, *semester) {
                                placed = true;
                                placed_any = true;
                                break;
                            }
                        }
                        if placed {
                            break;
                        }
                    }
                }
                if placed {
                    remaining.remove(i);
                } else {
                    i += 1;
                }
            }
            placed_any
        };

    loop {
        if remaining_items.is_empty() {
            break;
        }
        let only_items = if has_undergrad
            && remaining_items
                .iter()
                .any(|item| ms_schedule_items.contains(item))
        {
            Some(&ms_schedule_items)
        } else {
            None
        };
        let placed = distribute(
            &mut remaining_items,
            &mut schedule,
            allow_summer,
            &ms_grad_schedule_items,
            only_items,
            None,
        );
        if remaining_items.is_empty() {
            break;
        }
        if !placed {
            if payload.degrees.len() > 1 {
                for item in &remaining_items {
                    item_targets.remove(item);
                }
                if squeeze_undergrad_remaining(&mut remaining_items, &mut schedule) {
                    continue;
                }
            }
            let max_year = schedule.iter().map(|p| p.year).max().unwrap_or(4);
            ensure_year(&mut schedule, max_year + 1, allow_summer);
        }
    }

    let cross_degree_summary = if degree_schools.len() > 1 {
        cross_degree::enforce_claim_rules(&mut cross_state, &cu_map);

        for (degree_idx, result) in degree_results.iter_mut().enumerate() {
            requirement::filter_mapped_requirements_by_allocation(
                &mut result.fulfilled_requirements,
                degree_idx,
                &cross_state.claims,
            );
            requirement::filter_mapped_requirements_by_allocation(
                &mut result.suggested_for_unfulfilled,
                degree_idx,
                &cross_state.claims,
            );
            requirement::filter_mapped_requirements_by_allocation(
                &mut result.unfulfilled_requirements,
                degree_idx,
                &cross_state.claims,
            );
            requirement::filter_concentration_info_by_claims(
                &mut result.concentration_info,
                degree_idx,
                &cross_state.claims,
            );
        }

        let mut summary = cross_state.to_summary();
        summary.violations = cross_degree::detect_violations(
            &cross_state.claims,
            &degree_schools,
            &cu_map,
        );
        Some(summary)
    } else {
        None
    };

    for plan in schedule.iter_mut() {
        plan.total_cu = plan
            .courses
            .iter()
            .map(|c| get_cu(c))
            .chain(plan.requirement_slots.iter().map(|s| get_cu(s)))
            .sum();
    }

    ScheduleOutput {
        schedule,
        degree_results,
        slot_labels,
        cross_degree_summary,
        overlap_plan,
        overlap_schedule_groups,
        error: None,
    }
}

#[cfg(test)]
mod schedule_integration_tests {
    use super::*;

    #[test]
    fn year_one_fall_cu_limit_is_five_point_five() {
        let seas_wh = vec!["SEAS".to_string(), "WH".to_string()];
        assert_eq!(default_semester_cu_limit(&seas_wh, 1, "Fall"), 5.5);
        assert_eq!(default_semester_cu_limit(&seas_wh, 1, "Spring"), 6.5);
    }

    #[test]
    fn dual_ug_non_cas_gets_six_point_five() {
        let schools = vec!["CAS".to_string(), "WH".to_string()];
        assert_eq!(default_semester_cu_limit(&schools, 2, "Fall"), 6.5);
        assert_eq!(default_semester_cu_limit(&schools, 4, "Spring"), 6.5);
    }

    #[test]
    fn dual_cas_stays_five_point_five_four_years() {
        let schools = vec!["CAS".to_string(), "CAS".to_string()];
        assert_eq!(default_semester_cu_limit(&schools, 2, "Fall"), 5.5);
        assert_eq!(undergrad_schedule_years(&schools), 4);
    }

    #[test]
    fn single_degree_five_point_five() {
        let schools = vec!["SEAS".to_string()];
        assert_eq!(default_semester_cu_limit(&schools, 3, "Spring"), 5.5);
        assert_eq!(undergrad_schedule_years(&schools), 4);
    }

    fn dual_degree_input(
        school1: &str,
        major1: &str,
        school2: &str,
        major2: &str,
    ) -> ScheduleInput {
        let wh_conc = if school2 == "WH" || school1 == "WH" {
            Some("FNCE".to_string())
        } else {
            None
        };
        ScheduleInput {
            taken: vec![],
            degrees: vec![
                DegreeInput {
                    major: major1.to_string(),
                    school: school1.to_string(),
                    concentrations: vec![],
                    concentration: None,
                },
                DegreeInput {
                    major: major2.to_string(),
                    school: school2.to_string(),
                    concentrations: vec![],
                    concentration: wh_conc,
                },
            ],
            frozen: vec![],
            allow_summer: Some(true),
            semester_cu_limits: None,
        }
    }

    fn max_schedule_year(schedule: &[SemesterPlan]) -> i32 {
        schedule.iter().map(|p| p.year).max().unwrap_or(0)
    }

    fn assert_dual_degree_schedule(output: &ScheduleOutput, label: &str) {
        assert!(output.error.is_none(), "{label}: unexpected pipeline error");
        assert_eq!(output.degree_results.len(), 2, "{label}: expected two degree results");
        for result in &output.degree_results {
            assert!(
                result.error.is_none(),
                "{label}: degree {} {} failed: {:?}",
                result.school,
                result.major,
                result.error
            );
        }
        let max_year = max_schedule_year(&output.schedule);
        let schools: Vec<String> = output
            .degree_results
            .iter()
            .map(|r| r.school.clone())
            .collect();
        for plan in &output.schedule {
            let limit = default_semester_cu_limit(&schools, plan.year, &plan.semester);
            assert!(
                plan.total_cu <= limit + CU_EPS,
                "{label}: year {} {} has {:.1} CU (limit {:.1})",
                plan.year,
                plan.semester,
                plan.total_cu,
                limit
            );
        }
        assert!(
            max_year <= 6,
            "{label}: expected finish by year 6 with strict CU limits, schedule extends to year {max_year}"
        );
        let pairs = output
            .overlap_plan
            .as_ref()
            .map(|p| p.pairs.len())
            .unwrap_or(0);
        assert!(
            !output.overlap_schedule_groups.is_empty()
                || pairs > 0,
            "{label}: expected overlap pairs or schedule groups"
        );
    }

    #[test]
    fn neur_wh_dual_degree_finishes_within_five_years() {
        let output = generate_schedule(dual_degree_input("CAS", "NEUR", "WH", "WH_NOFL"));
        assert_dual_degree_schedule(&output, "NEUR + WH_NOFL");
    }

    #[test]
    fn cis_wh_dual_degree_finishes_within_five_years() {
        let output = generate_schedule(dual_degree_input("SEAS", "CIS", "WH", "WH_NOFL"));
        assert_dual_degree_schedule(&output, "CIS + WH_NOFL");
    }
}