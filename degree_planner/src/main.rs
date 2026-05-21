use std::vec;

pub mod course;
pub mod major;
pub mod requirement;

pub mod attributes_data;
pub mod seas_data;
pub mod wharton_data;
pub mod courses_data;

use std::collections::{BTreeMap, HashMap};

use course::Course;
use requirement::Requirement;
use requirement::MappedRequirement;
use requirement::DoubleCountInfo;
use requirement::ConcentrationInfo;
use major::Major;

use axum:: {
    http::{header, Method, HeaderValue},
    Json, Router, routing::{delete, get, patch, post, put},
    debug_handler
};
use serde::{Serialize, Deserialize};
use tower_http::cors::{Any, CorsLayer};

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
        let all_courses = courses_data::all_courses();
        let cu_map: HashMap<String, f64> = all_courses.iter()
            .map(|c| (c.course_code.clone(), c.cu))
            .collect();
        let (mut fulfilled_requirements, unfulfilled_requirements) =
            requirement::validate_courses_for_degree(
                major_req_unwrapped.requirements.clone(),
                &taken,
                &cu_map,
            );

        fulfilled_requirements.sort_by_key(|r| r.requirement.get_category());
        let suggested_for_unfulfilled = requirement::suggest_courses_for_requirements(
            &unfulfilled_requirements,
            &taken,
            &cu_map,
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
            error: None,
        };
    } else {
        response = RootPostOutput { 
            fulfilled_requirements: vec![], unfulfilled_requirements: vec![], 
            suggested_for_unfulfilled: vec![], unapplicable_courses: vec![],
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

    let all_courses = courses_data::all_courses();

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


#[derive(Debug, Deserialize)]
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
            if self.school == "WH" {
                return wharton_data::normalize_wh_concentrations(&self.concentrations);
            }
            return self.concentrations.clone();
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
    double_count_info: Vec<DoubleCountInfo>,
    concentration_info: Vec<ConcentrationInfo>,
    available_concentrations: Vec<String>,
    has_core_concentration: bool,
    category_order: Vec<String>,
    error: Option<String>,
}

#[derive(Serialize)]
struct ScheduleOutput {
    schedule: Vec<SemesterPlan>,
    degree_results: Vec<DegreeResult>,
    /// Maps requirement slot id → human-readable description for the schedule UI.
    slot_labels: HashMap<String, String>,
    error: Option<String>,
}

#[debug_handler]
async fn generate_schedule_post(Json(payload): Json<ScheduleInput>) -> Json<ScheduleOutput> {
    println!("POST /generate_schedule request made");

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

    // Build a CU lookup map from all courses
    let all_courses = courses_data::all_courses();
    let cu_map: HashMap<String, f64> = all_courses.iter()
        .map(|c| (c.course_code.clone(), c.cu))
        .collect();

    // Process each degree
    for degree in &payload.degrees {
        let concs = degree.effective_concentrations();
        let major_req = resolve_major(&degree.school, &degree.major, &concs);

        if let Some(major_data) = major_req {
            let (mut fulfilled, mut unfulfilled) = requirement::validate_courses_for_degree(
                major_data.requirements.clone(),
                &courses_for_validation,
                &cu_map,
            );
            if degree.school == "WH" && concs.len() >= 2 {
                requirement::apply_wharton_double_concentration_bb_overlap(
                    &concs,
                    &mut fulfilled,
                    &mut unfulfilled,
                );
            }
            for mapped in &mut fulfilled {
                mapped.course_ids = requirement::filter_valid_course_ids(mapped.course_ids.clone());
            }
            fulfilled.retain(|m| !m.course_ids.is_empty());
            fulfilled.sort_by_key(|r| r.requirement.get_category());
            let suggested = requirement::suggest_courses_for_requirements(
                &unfulfilled,
                &courses_for_validation,
                &cu_map,
            );

            // Collect unique suggested courses and requirement slots for the schedule
            for mapped in &suggested {
                for course_id in &mapped.course_ids {
                    if course::is_valid_course_code(course_id)
                        && !all_suggested_courses.contains(course_id)
                        && !courses_for_validation.contains(course_id)
                    {
                        all_suggested_courses.push(course_id.clone());
                    } else if requirement::is_requirement_slot_id(course_id)
                        && !all_requirement_slots.contains(course_id)
                    {
                        all_requirement_slots.push(course_id.clone());
                        slot_labels.insert(
                            course_id.clone(),
                            mapped.requirement.slot_label_for_id(course_id),
                        );
                    }
                }
            }

            let mut unapplicable = courses_for_validation.clone();
            for req in &fulfilled {
                for course in &req.course_ids {
                    unapplicable.retain(|x| x != course);
                }
            }

            // Extract double-count metadata
            let dc_info = requirement::extract_double_count_info(
                &major_data.requirements,
                &courses_for_validation,
                &fulfilled,
                &suggested,
                &cu_map,
            );

            // Extract concentration info
            let conc_info = requirement::extract_concentration_info(
                &major_data.requirements,
                &major_data.concentrations,
                &concs,
                &courses_for_validation,
                &cu_map,
            );

            // Available concentration names
            let available_concs: Vec<String> = major_data.concentrations.as_ref()
                .map(|m| m.keys().cloned().collect())
                .unwrap_or_default();

            // Check if this major uses core concentrations
            let has_core = major_data.requirements.iter()
                .any(|r| matches!(r, Requirement::Concentration { .. }));

            // Extract category order from requirement definition
            let mut category_order: Vec<String> = Vec::new();
            for req in &major_data.requirements {
                let cat = req.get_category();
                if !cat.is_empty() && !category_order.contains(&cat) {
                    category_order.push(cat);
                }
            }

            degree_results.push(DegreeResult {
                school: degree.school.clone(),
                major: degree.major.clone(),
                fulfilled_requirements: fulfilled,
                unfulfilled_requirements: unfulfilled,
                suggested_for_unfulfilled: suggested,
                unapplicable_courses: unapplicable,
                double_count_info: dc_info,
                concentration_info: conc_info,
                available_concentrations: available_concs,
                has_core_concentration: has_core,
                category_order,
                error: None,
            });
        } else {
            degree_results.push(DegreeResult {
                school: degree.school.clone(),
                major: degree.major.clone(),
                fulfilled_requirements: vec![],
                unfulfilled_requirements: vec![],
                suggested_for_unfulfilled: vec![],
                unapplicable_courses: vec![],
                double_count_info: vec![],
                concentration_info: vec![],
                available_concentrations: vec![],
                has_core_concentration: false,
                category_order: vec![],
                error: Some(format!("Major '{}' in school '{}' is not implemented yet.", degree.major, degree.school)),
            });
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
        // Defaults
        match semester {
            "Summer" => 2.0,
            _ => 5.0,
        }
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

    // Start with 4 years
    for yr in 1..=4 {
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

    // Distribute remaining courses and requirement slots
    let mut remaining_courses: Vec<String> = all_suggested_courses;
    let mut remaining_slots: Vec<String> = all_requirement_slots;

    let distribute = |remaining: &mut Vec<String>, schedule: &mut Vec<SemesterPlan>, allow_summer: bool| -> bool {
        if remaining.is_empty() {
            return false;
        }
        let mut placed_any = false;
        for plan in schedule.iter_mut() {
            if plan.semester == "Summer" || remaining.is_empty() {
                continue;
            }
            let max_cu = get_max_cu(plan.year, &plan.semester);
            let has_items = !plan.courses.is_empty() || !plan.requirement_slots.is_empty();
            while !remaining.is_empty() {
                let cu = get_cu(&remaining[0]);
                if plan.total_cu + cu > max_cu && has_items {
                    break;
                }
                let item = remaining.remove(0);
                place_in_semester(plan, &item);
                placed_any = true;
            }
        }
        if allow_summer && !remaining.is_empty() {
            for plan in schedule.iter_mut() {
                if plan.semester != "Summer" || remaining.is_empty() {
                    continue;
                }
                let max_cu = get_max_cu(plan.year, &plan.semester);
                let has_items = !plan.courses.is_empty() || !plan.requirement_slots.is_empty();
                while !remaining.is_empty() {
                    let cu = get_cu(&remaining[0]);
                    if plan.total_cu + cu > max_cu && has_items {
                        break;
                    }
                    let item = remaining.remove(0);
                    place_in_semester(plan, &item);
                    placed_any = true;
                }
            }
        }
        placed_any
    };

    loop {
        if remaining_courses.is_empty() && remaining_slots.is_empty() {
            break;
        }
        let placed_courses = distribute(&mut remaining_courses, &mut schedule, allow_summer);
        let placed_slots = distribute(&mut remaining_slots, &mut schedule, allow_summer);
        if remaining_courses.is_empty() && remaining_slots.is_empty() {
            break;
        }
        if !placed_courses && !placed_slots {
            let max_year = schedule.iter().map(|p| p.year).max().unwrap_or(4);
            ensure_year(&mut schedule, max_year + 1, allow_summer);
        }
    }

    Json(ScheduleOutput {
        schedule,
        degree_results,
        slot_labels,
        error: None,
    })
}