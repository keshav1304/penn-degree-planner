use std::vec;

pub mod course;
pub mod major;
pub mod requirement;

pub mod attributes_data;
pub mod seas_data;
pub mod wharton_data;

use std::collections::BTreeMap;

use course::Course;
use requirement::Requirement;
use requirement::MappedRequirement;
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
struct SampleInput {
    taken: Vec<String>,
    major: String,
    school: String,
    concentration: Option<String>,
}

#[derive(Serialize)]
struct ApiOutput {
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
async fn root_post(Json(payload): Json<SampleInput>) -> Json<ApiOutput> {
    println!("POST / request made");

    let taken = payload.taken;
    let major = payload.major;
    let school = payload.school;
    let concentration = payload.concentration; 

    let major_req: Option<Major> = resolve_major(&school, &major, &concentration);

    let response: ApiOutput;

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
        response = ApiOutput {
            fulfilled_requirements, unfulfilled_requirements, suggested_for_unfulfilled, unapplicable_courses,
            error: None
        };
    } else {
        response = ApiOutput { 
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

    let all_courses_result = read_courses("all_courses.csv");

    match all_courses_result {
        Ok(res) => {
            return Json(res);
        }
        Err(e) => {
            println!("Error: {}", e);
            return Json(Vec::new());
        }
    }

}

use axum::{extract::Query};

use crate::course::find_course;
use crate::course::read_courses;
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
                        cu: "0.0".to_string(),
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
                cu: "0.0".to_string(),
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
}

#[derive(Serialize)]
struct SemesterPlan {
    year: i32,
    semester: String,
    courses: Vec<String>,
}

#[derive(Serialize)]
struct DegreeResult {
    school: String,
    major: String,
    fulfilled_requirements: Vec<MappedRequirement>,
    unfulfilled_requirements: Vec<Requirement>,
    suggested_for_unfulfilled: Vec<MappedRequirement>,
    unapplicable_courses: Vec<String>,
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

    let taken = &payload.taken;
    let mut degree_results: Vec<DegreeResult> = Vec::new();
    let mut all_suggested_courses: Vec<String> = Vec::new();

    // Process each degree
    for degree in &payload.degrees {
        let major_req = resolve_major(&degree.school, &degree.major, &degree.concentration);

        if let Some(major_data) = major_req {
            let (mut fulfilled, unfulfilled) = requirement::validate_courses_for_degree(
                major_data.requirements.clone(), taken
            );
            fulfilled.sort_by_key(|r| r.requirement.get_category());
            let suggested = requirement::suggest_courses_for_requirements(&unfulfilled, taken);

            // Collect unique suggested courses
            for mapped in &suggested {
                for course_id in &mapped.course_ids {
                    if !all_suggested_courses.contains(course_id) && !taken.contains(course_id) {
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

            degree_results.push(DegreeResult {
                school: degree.school.clone(),
                major: degree.major.clone(),
                fulfilled_requirements: fulfilled,
                unfulfilled_requirements: unfulfilled,
                suggested_for_unfulfilled: suggested,
                unapplicable_courses: unapplicable,
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
                error: Some(format!("Major '{}' in school '{}' is not implemented yet.", degree.major, degree.school)),
            });
        }
    }

    // Build schedule from frozen + suggested courses
    let semesters = vec![
        ("Fall", 1), ("Spring", 1), ("Summer", 1),
        ("Fall", 2), ("Spring", 2), ("Summer", 2),
        ("Fall", 3), ("Spring", 3), ("Summer", 3),
        ("Fall", 4), ("Spring", 4), ("Summer", 4),
    ];

    let mut schedule: Vec<SemesterPlan> = semesters.iter().map(|(sem, yr)| {
        SemesterPlan {
            year: *yr,
            semester: sem.to_string(),
            courses: Vec::new(),
        }
    }).collect();

    // Place frozen courses first
    for frozen in &payload.frozen {
        for plan in schedule.iter_mut() {
            if plan.year == frozen.year && plan.semester == frozen.semester {
                if !plan.courses.contains(&frozen.course_id) {
                    plan.courses.push(frozen.course_id.clone());
                }
            }
        }
        // Remove from suggested pool if present
        all_suggested_courses.retain(|c| c != &frozen.course_id);
    }

    // Distribute suggested courses across Fall/Spring semesters (skip summer unless needed)
    let max_per_semester = 5;
    let mut course_iter = all_suggested_courses.into_iter();

    for plan in schedule.iter_mut() {
        if plan.semester == "Summer" {
            continue; // skip summer for auto-distribution
        }
        while plan.courses.len() < max_per_semester {
            match course_iter.next() {
                Some(course) => plan.courses.push(course),
                None => break,
            }
        }
    }

    // Overflow into summer if needed
    for plan in schedule.iter_mut() {
        if plan.semester != "Summer" {
            continue;
        }
        while plan.courses.len() < 2 {
            match course_iter.next() {
                Some(course) => plan.courses.push(course),
                None => break,
            }
        }
    }

    Json(ScheduleOutput {
        schedule,
        degree_results,
        error: None,
    })
}