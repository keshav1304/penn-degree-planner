use std::collections::{BTreeMap, HashMap};

use axum::{
    debug_handler,
    extract::Query,
    http::{header, Method},
    routing::{get, post},
    Json, Router,
};
use degree_planner::course::{self, Course};
use degree_planner::major::{
    all_concentrations, all_majors, concentrations_for, degree_catalog, minor_catalog,
    resolve_major,
};
use degree_planner::requirement::{self, MappedRequirement, PoolCoverageInfo};
use degree_planner::scheduler::{generate_schedule, ScheduleInput, ScheduleOutput};
use serde::{Deserialize, Serialize};
use tower_http::cors::{Any, CorsLayer};

#[tokio::main]
async fn main() {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_headers([header::CONTENT_TYPE, header::AUTHORIZATION]);

    let app = Router::new()
        .route("/", get(root_get))
        .route("/", post(root_post))
        .route("/all_courses", get(all_courses_get))
        .route("/course", get(course_get))
        .route("/all_majors", get(all_majors_get))
        .route("/degree_catalog", get(degree_catalog_get))
        .route("/minor_catalog", get(minor_catalog_get))
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
    error: Option<String>,
}

#[debug_handler]
async fn root_get() -> Json<SimpleResponse> {
    Json(SimpleResponse {
        response_str: "GET does not exist for /".to_string(),
    })
}

#[debug_handler]
async fn root_post(Json(payload): Json<RootPostInput>) -> Json<RootPostOutput> {
    let taken = payload.taken;
    let major = payload.major;
    let school = payload.school;
    let concentrations: Vec<String> = payload.concentration.clone().into_iter().collect();

    if let Some(major_req_unwrapped) = resolve_major(&school, &major, &concentrations) {
        let all_courses = degree_planner::penn_data::courses_data::all_courses();
        let cu_map: HashMap<String, f64> = all_courses
            .iter()
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
            for course_id in &req.course_ids {
                if unapplicable_courses.contains(course_id) {
                    unapplicable_courses.retain(|x| x != course_id);
                }
            }
        }
        Json(RootPostOutput {
            fulfilled_requirements,
            unfulfilled_requirements,
            suggested_for_unfulfilled,
            unapplicable_courses,
            pool_coverage_info,
            error: None,
        })
    } else {
        Json(RootPostOutput {
            fulfilled_requirements: vec![],
            unfulfilled_requirements: vec![],
            suggested_for_unfulfilled: vec![],
            unapplicable_courses: vec![],
            pool_coverage_info: vec![],
            error: Some(
                "Major provided is not valid or has no data associated with it yet!".to_string(),
            ),
        })
    }
}

#[debug_handler]
async fn all_majors_get() -> Json<BTreeMap<String, Vec<String>>> {
    Json(all_majors())
}

#[debug_handler]
async fn degree_catalog_get() -> Json<Vec<degree_planner::major::SchoolCatalogEntry>> {
    Json(degree_catalog())
}

#[debug_handler]
async fn minor_catalog_get() -> Json<Vec<degree_planner::major::SchoolCatalogEntry>> {
    Json(minor_catalog())
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
    Json(ConcentrationsResponse {
        concentrations: concentrations_for(&params.school, &params.major),
    })
}

#[debug_handler]
async fn all_concentrations_get() -> Json<BTreeMap<String, Vec<String>>> {
    Json(all_concentrations())
}

#[debug_handler]
async fn all_courses_get() -> Json<Vec<Course>> {
    Json(degree_planner::penn_data::courses_data::all_courses())
}

#[derive(Debug, Deserialize)]
struct CourseGetParams {
    course_id: String,
}

#[debug_handler]
async fn course_get(Query(params): Query<CourseGetParams>) -> Json<Course> {
    let empty = || Course {
        dept_code: String::new(),
        course_code: String::new(),
        title: String::new(),
        description: None,
        semester: None,
        prereq: None,
        cu: 0.0,
        also_offered_as: None,
        mutually_exclusive: None,
        coreq: None,
    };

    match course::find_course("all_courses.csv", &params.course_id) {
        Ok(Some(val)) => Json(val),
        Ok(None) | Err(_) => Json(empty()),
    }
}

#[debug_handler]
async fn generate_schedule_post(Json(payload): Json<ScheduleInput>) -> Json<ScheduleOutput> {
    Json(generate_schedule(payload))
}
