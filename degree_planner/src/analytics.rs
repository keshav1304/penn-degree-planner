//! Optional product analytics: record compact `/generate_schedule` summaries to Postgres.
//! Planning always succeeds even if inserts fail or `DATABASE_URL` is unset.

use serde_json::{json, Value};
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;

use crate::scheduler::{ScheduleInput, ScheduleOutput};

const SCHEMA_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS schedule_generates (
  id                bigserial PRIMARY KEY,
  created_at        timestamptz NOT NULL DEFAULT now(),
  ok                boolean NOT NULL,
  latency_ms        integer,
  degree_combo_key  text NOT NULL,
  degrees           jsonb NOT NULL,
  taken_count       integer NOT NULL DEFAULT 0,
  frozen_count      integer NOT NULL DEFAULT 0,
  allow_summer      boolean NOT NULL DEFAULT false,
  has_cu_overrides  boolean NOT NULL DEFAULT false,
  violation_types   text[] NOT NULL DEFAULT '{}',
  degree_count      integer NOT NULL DEFAULT 0,
  major_count       integer NOT NULL DEFAULT 0,
  minor_count       integer NOT NULL DEFAULT 0,
  has_concentration boolean NOT NULL DEFAULT false,
  total_cu          double precision,
  semester_count    integer,
  has_overlap       boolean NOT NULL DEFAULT false,
  error_kinds       text[] NOT NULL DEFAULT '{}',
  anon_session_id   text
);

ALTER TABLE schedule_generates ADD COLUMN IF NOT EXISTS degree_count integer NOT NULL DEFAULT 0;
ALTER TABLE schedule_generates ADD COLUMN IF NOT EXISTS major_count integer NOT NULL DEFAULT 0;
ALTER TABLE schedule_generates ADD COLUMN IF NOT EXISTS minor_count integer NOT NULL DEFAULT 0;
ALTER TABLE schedule_generates ADD COLUMN IF NOT EXISTS has_concentration boolean NOT NULL DEFAULT false;
ALTER TABLE schedule_generates ADD COLUMN IF NOT EXISTS total_cu double precision;
ALTER TABLE schedule_generates ADD COLUMN IF NOT EXISTS semester_count integer;
ALTER TABLE schedule_generates ADD COLUMN IF NOT EXISTS has_overlap boolean NOT NULL DEFAULT false;
ALTER TABLE schedule_generates ADD COLUMN IF NOT EXISTS error_kinds text[] NOT NULL DEFAULT '{}';
ALTER TABLE schedule_generates ADD COLUMN IF NOT EXISTS anon_session_id text;

CREATE INDEX IF NOT EXISTS schedule_generates_created_at_idx
  ON schedule_generates (created_at);
CREATE INDEX IF NOT EXISTS schedule_generates_degree_combo_key_idx
  ON schedule_generates (degree_combo_key);
CREATE INDEX IF NOT EXISTS schedule_generates_anon_session_id_idx
  ON schedule_generates (anon_session_id);
"#;

pub async fn connect(database_url: &str) -> Result<PgPool, sqlx::Error> {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(std::time::Duration::from_secs(5))
        .connect(database_url)
        .await?;
    ensure_schema(&pool).await?;
    Ok(pool)
}

pub async fn ensure_schema(pool: &PgPool) -> Result<(), sqlx::Error> {
    sqlx::raw_sql(SCHEMA_SQL).execute(pool).await?;
    Ok(())
}

pub struct ScheduleGenerateEvent {
    pub ok: bool,
    pub latency_ms: i32,
    pub degree_combo_key: String,
    pub degrees: Value,
    pub taken_count: i32,
    pub frozen_count: i32,
    pub allow_summer: bool,
    pub has_cu_overrides: bool,
    pub violation_types: Vec<String>,
    pub degree_count: i32,
    pub major_count: i32,
    pub minor_count: i32,
    pub has_concentration: bool,
    pub total_cu: Option<f64>,
    pub semester_count: Option<i32>,
    pub has_overlap: bool,
    pub error_kinds: Vec<String>,
    pub anon_session_id: Option<String>,
}

impl ScheduleGenerateEvent {
    pub fn from_request_and_output(
        input: &ScheduleInput,
        output: &ScheduleOutput,
        latency_ms: i32,
    ) -> Self {
        let mut degree_parts: Vec<String> = Vec::with_capacity(input.degrees.len());
        let mut degrees_json: Vec<Value> = Vec::with_capacity(input.degrees.len());
        let mut major_count = 0_i32;
        let mut minor_count = 0_i32;
        let mut has_concentration = false;

        for d in &input.degrees {
            let mut concentrations = if !d.concentrations.is_empty() {
                d.concentrations.clone()
            } else {
                d.concentration.clone().into_iter().collect()
            };
            concentrations.sort();
            if !concentrations.is_empty() {
                has_concentration = true;
            }
            if d.kind == "minor" {
                minor_count += 1;
            } else {
                major_count += 1;
            }

            let mut part = format!("{}:{}:{}", d.school, d.major, d.kind);
            if !concentrations.is_empty() {
                part.push('[');
                part.push_str(&concentrations.join(","));
                part.push(']');
            }
            degree_parts.push(part);

            degrees_json.push(json!({
                "school": d.school,
                "major": d.major,
                "kind": d.kind,
                "concentrations": concentrations,
            }));
        }
        degree_parts.sort();

        let violation_types = output
            .cross_degree_summary
            .as_ref()
            .map(|s| {
                let mut kinds: Vec<String> = s
                    .violations
                    .iter()
                    .filter_map(|v| serde_json::to_value(&v.kind).ok())
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect();
                kinds.sort();
                kinds.dedup();
                kinds
            })
            .unwrap_or_default();

        let mut error_kinds: Vec<String> = Vec::new();
        if let Some(err) = &output.error {
            error_kinds.push(truncate_error(err));
        }
        for r in &output.degree_results {
            if let Some(err) = &r.error {
                error_kinds.push(format!("{}:{}:{}", r.school, r.major, truncate_error(err)));
            }
        }
        error_kinds.sort();
        error_kinds.dedup();

        let degree_errors = output.degree_results.iter().any(|r| r.error.is_some());
        let ok = output.error.is_none() && !degree_errors;

        let total_cu = if output.schedule.is_empty() {
            None
        } else {
            Some(output.schedule.iter().map(|s| s.total_cu).sum())
        };
        let semester_count = if output.schedule.is_empty() {
            None
        } else {
            Some(output.schedule.len() as i32)
        };

        Self {
            ok,
            latency_ms,
            degree_combo_key: degree_parts.join("+"),
            degrees: Value::Array(degrees_json),
            taken_count: input.taken.len() as i32,
            frozen_count: input.frozen.len() as i32,
            allow_summer: input.allow_summer.unwrap_or(false),
            has_cu_overrides: input
                .semester_cu_limits
                .as_ref()
                .map(|m| !m.is_empty())
                .unwrap_or(false),
            violation_types,
            degree_count: input.degrees.len() as i32,
            major_count,
            minor_count,
            has_concentration,
            total_cu,
            semester_count,
            has_overlap: output.overlap_plan.is_some(),
            error_kinds,
            anon_session_id: sanitize_session_id(input.anon_session_id.as_deref()),
        }
    }
}

fn sanitize_session_id(raw: Option<&str>) -> Option<String> {
    let s = raw?.trim();
    if s.is_empty() || s.len() > 80 {
        return None;
    }
    if !s
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        return None;
    }
    Some(s.to_string())
}

fn truncate_error(err: &str) -> String {
    const MAX: usize = 120;
    let cleaned = err.split('\n').next().unwrap_or(err).trim();
    if cleaned.chars().count() <= MAX {
        cleaned.to_string()
    } else {
        let truncated: String = cleaned.chars().take(MAX).collect();
        format!("{truncated}…")
    }
}

pub async fn insert_schedule_generate(
    pool: &PgPool,
    event: &ScheduleGenerateEvent,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO schedule_generates (
            ok, latency_ms, degree_combo_key, degrees,
            taken_count, frozen_count, allow_summer, has_cu_overrides, violation_types,
            degree_count, major_count, minor_count, has_concentration,
            total_cu, semester_count, has_overlap, error_kinds, anon_session_id
        ) VALUES (
            $1, $2, $3, $4, $5, $6, $7, $8, $9,
            $10, $11, $12, $13, $14, $15, $16, $17, $18
        )
        "#,
    )
    .bind(event.ok)
    .bind(event.latency_ms)
    .bind(&event.degree_combo_key)
    .bind(&event.degrees)
    .bind(event.taken_count)
    .bind(event.frozen_count)
    .bind(event.allow_summer)
    .bind(event.has_cu_overrides)
    .bind(&event.violation_types)
    .bind(event.degree_count)
    .bind(event.major_count)
    .bind(event.minor_count)
    .bind(event.has_concentration)
    .bind(event.total_cu)
    .bind(event.semester_count)
    .bind(event.has_overlap)
    .bind(&event.error_kinds)
    .bind(&event.anon_session_id)
    .execute(pool)
    .await?;
    Ok(())
}
