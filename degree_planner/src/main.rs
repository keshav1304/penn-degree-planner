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
    unfulfilled_requirements: Vec<Requirement>,
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
    let concentration = payload.concentration; 

    let major_req: Option<Major> = resolve_major(&school, &major, &concentration);

    let response: RootPostOutput;

    if let Some(major_req_unwrapped) = major_req {
        let (mut fulfilled_requirements, unfulfilled_requirements) = requirement::validate_courses_for_degree(major_req_unwrapped.requirements.clone(), &taken);
        
        fulfilled_requirements.sort_by_key(|r| r.requirement.get_category());        
        let suggested_for_unfulfilled = requirement::suggest_courses_for_requirements(&unfulfilled_requirements, &taken);
        
        let mut unapplicable_courses = taken.clone();
        for req in &fulfilled_requirements {
            for course in &req.course_ids {
                if unapplicable_courses.contains(course) {
                    unapplicable_courses.retain(|x| x != course);
                }
            }
        }
        response = RootPostOutput {
            fulfilled_requirements, unfulfilled_requirements, suggested_for_unfulfilled, unapplicable_courses,
            error: None
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
async fn all_courses_get() -> Json<Vec<Course>> {
    println!("GET /all_courses request made");

    let all_courses = courses_data::all_courses();

    return Json(all_courses);
}

use axum::{extract::Query};

use crate::course::find_course;
use crate::major::all_majors;
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
    concentration: Option<String>,
}

#[derive(Debug, Deserialize)]
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
<<<<<<< HEAD
    max_cu_per_semester: Option<f64>,
=======
    semester_cu_limits: Option<HashMap<String, f64>>,
>>>>>>> 0dc7dc2 (cu stuff changes)
}

#[derive(Serialize)]
struct SemesterPlan {
    year: i32,
    semester: String,
    courses: Vec<String>,
    total_cu: f64,
}

#[derive(Serialize)]
struct DegreeResult {
    school: String,
    major: String,
    fulfilled_requirements: Vec<MappedRequirement>,
    unfulfilled_requirements: Vec<Requirement>,
    suggested_for_unfulfilled: Vec<MappedRequirement>,
    unapplicable_courses: Vec<String>,
    double_count_info: Vec<DoubleCountInfo>,
    error: Option<String>,
}

#[derive(Serialize)]
struct ScheduleOutput {
    schedule: Vec<SemesterPlan>,
    degree_results: Vec<DegreeResult>,
    error: Option<String>,
}

#[debug_handler]
async fn generate_schedule_post(Json(payload): Json<ScheduleInput>) -> Json<ScheduleOutput> {
    println!("POST /generate_schedule request made");

    let mut taken = payload.taken.clone();
    let frozen = &payload.frozen;
    taken.extend(frozen.iter().map(|x| x.course_id.clone()).collect::<Vec<_>>());
    let mut degree_results: Vec<DegreeResult> = Vec::new();
    let mut all_suggested_courses: Vec<String> = Vec::new();

    // Process each degree
    for degree in &payload.degrees {
        let major_req = resolve_major(&degree.school, &degree.major, &degree.concentration);

        if let Some(major_data) = major_req {
            let (mut fulfilled, unfulfilled) = requirement::validate_courses_for_degree(
                major_data.requirements.clone(), &taken
            );
            fulfilled.sort_by_key(|r| r.requirement.get_category());
            let suggested = requirement::suggest_courses_for_requirements(&unfulfilled, &taken);

            // Collect unique suggested courses
            for mapped in &suggested {
                for course_id in &mapped.course_ids {
                    if !course::is_valid_course_code(course_id) {
                        all_suggested_courses.push(course_id.clone());
                    } else if !all_suggested_courses.contains(course_id) && !taken.contains(course_id) {
                        all_suggested_courses.push(course_id.clone());
                    }
                }
            }

            let mut unapplicable = taken.clone();
            for req in &fulfilled {
                for course in &req.course_ids {
                    unapplicable.retain(|x| x != course);
                }
            }

            // Extract double-count metadata
            let dc_info = requirement::extract_double_count_info(
                &major_data.requirements, &taken, &fulfilled, &suggested
            );

            degree_results.push(DegreeResult {
                school: degree.school.clone(),
                major: degree.major.clone(),
                fulfilled_requirements: fulfilled,
                unfulfilled_requirements: unfulfilled,
                suggested_for_unfulfilled: suggested,
                unapplicable_courses: unapplicable,
                double_count_info: dc_info,
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
                error: Some(format!("Major '{}' in school '{}' is not implemented yet.", degree.major, degree.school)),
            });
        }
    }

    // Build a CU lookup map from all courses
    let all_courses = courses_data::all_courses();
    let cu_map: HashMap<String, f64> = all_courses.iter()
        .map(|c| (c.course_code.clone(), c.cu))
        .collect();

    let get_cu = |course_id: &str| -> f64 {
        *cu_map.get(course_id).unwrap_or(&1.0)
    };

    // Build schedule dynamically — expand semesters until ALL courses fit
    let allow_summer = payload.allow_summer.unwrap_or(true);
<<<<<<< HEAD
    let max_cu_fall_spring = payload.max_cu_per_semester.unwrap_or(5.0);
    let max_cu_summer = 2.0_f64;
=======
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
>>>>>>> 0dc7dc2 (cu stuff changes)

    // Helper: ensure schedule has semesters for a given year
    let mut schedule: Vec<SemesterPlan> = Vec::new();

    let ensure_year = |schedule: &mut Vec<SemesterPlan>, year: i32, allow_summer: bool| {
        let has_fall = schedule.iter().any(|p| p.year == year && p.semester == "Fall");
        if !has_fall {
            schedule.push(SemesterPlan { year, semester: "Fall".to_string(), courses: Vec::new(), total_cu: 0.0 });
            schedule.push(SemesterPlan { year, semester: "Spring".to_string(), courses: Vec::new(), total_cu: 0.0 });
            if allow_summer {
                schedule.push(SemesterPlan { year, semester: "Summer".to_string(), courses: Vec::new(), total_cu: 0.0 });
            }
<<<<<<< HEAD
            // Re-sort so semesters are in order
=======
>>>>>>> 0dc7dc2 (cu stuff changes)
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

<<<<<<< HEAD
    // Place frozen courses first (expand schedule if frozen course is in a later year)
=======
    // Place frozen courses first
>>>>>>> 0dc7dc2 (cu stuff changes)
    for frozen in &payload.frozen {
        ensure_year(&mut schedule, frozen.year, allow_summer);
        for plan in schedule.iter_mut() {
            if plan.year == frozen.year && plan.semester == frozen.semester {
                if !plan.courses.contains(&frozen.course_id) {
                    let cu = get_cu(&frozen.course_id);
                    plan.courses.push(frozen.course_id.clone());
                    plan.total_cu += cu;
                }
            }
        }
<<<<<<< HEAD
        // Remove from suggested pool if present
        all_suggested_courses.retain(|c| c != &frozen.course_id);
    }

    // Distribute remaining courses — keep adding years until everything fits
=======
        all_suggested_courses.retain(|c| c != &frozen.course_id);
    }

    // Distribute remaining courses
>>>>>>> 0dc7dc2 (cu stuff changes)
    let mut remaining: Vec<String> = all_suggested_courses;

    loop {
        if remaining.is_empty() {
            break;
        }

        let mut placed_any = false;

<<<<<<< HEAD
        // First pass: fill Fall/Spring slots (up to max CU each)
=======
        // First pass: fill Fall/Spring slots
>>>>>>> 0dc7dc2 (cu stuff changes)
        for plan in schedule.iter_mut() {
            if plan.semester == "Summer" || remaining.is_empty() {
                continue;
            }
<<<<<<< HEAD
            while !remaining.is_empty() {
                let cu = get_cu(&remaining[0]);
                if plan.total_cu + cu > max_cu_fall_spring && !plan.courses.is_empty() {
                    break; // Would exceed limit (but always allow at least 1 course)
=======
            let max_cu = get_max_cu(plan.year, &plan.semester);
            while !remaining.is_empty() {
                let cu = get_cu(&remaining[0]);
                if plan.total_cu + cu > max_cu && !plan.courses.is_empty() {
                    break;
>>>>>>> 0dc7dc2 (cu stuff changes)
                }
                let course = remaining.remove(0);
                plan.total_cu += cu;
                plan.courses.push(course);
                placed_any = true;
            }
        }

<<<<<<< HEAD
        // Second pass: fill Summer slots if allowed (up to max CU)
=======
        // Second pass: fill Summer slots if allowed
>>>>>>> 0dc7dc2 (cu stuff changes)
        if allow_summer && !remaining.is_empty() {
            for plan in schedule.iter_mut() {
                if plan.semester != "Summer" || remaining.is_empty() {
                    continue;
                }
<<<<<<< HEAD
                while !remaining.is_empty() {
                    let cu = get_cu(&remaining[0]);
                    if plan.total_cu + cu > max_cu_summer && !plan.courses.is_empty() {
=======
                let max_cu = get_max_cu(plan.year, &plan.semester);
                while !remaining.is_empty() {
                    let cu = get_cu(&remaining[0]);
                    if plan.total_cu + cu > max_cu && !plan.courses.is_empty() {
>>>>>>> 0dc7dc2 (cu stuff changes)
                        break;
                    }
                    let course = remaining.remove(0);
                    plan.total_cu += cu;
                    plan.courses.push(course);
                    placed_any = true;
                }
            }
        }

        // If we still have remaining courses, add another year and loop
        if !remaining.is_empty() {
            let max_year = schedule.iter().map(|p| p.year).max().unwrap_or(4);
            ensure_year(&mut schedule, max_year + 1, allow_summer);
        } else {
            break;
        }

        // Safety: if nothing was placed and we still have courses, add a year
        if !placed_any && !remaining.is_empty() {
            let max_year = schedule.iter().map(|p| p.year).max().unwrap_or(4);
            ensure_year(&mut schedule, max_year + 1, allow_summer);
        }
    }

    Json(ScheduleOutput {
        schedule,
        degree_results,
        error: None,
    })
}