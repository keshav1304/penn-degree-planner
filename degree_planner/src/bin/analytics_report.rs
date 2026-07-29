//! Print a local analytics report from Neon Postgres.
//!
//! Usage:
//!   DATABASE_URL='postgresql://…' cargo run --bin analytics_report
//!   DATABASE_URL='postgresql://…' cargo run --bin analytics_report -- --days 7
//!   DATABASE_URL='postgresql://…' cargo run --bin analytics_report -- --session <uuid>
//!
//! Never commit DATABASE_URL. Export it in your shell or use a gitignored env file.

use std::env;

use sqlx::postgres::PgPoolOptions;
use sqlx::Row;

#[tokio::main]
async fn main() {
    let days = parse_days(env::args().skip(1));
    let url = match env::var("DATABASE_URL") {
        Ok(u) if !u.is_empty() => u,
        _ => {
            eprintln!(
                "DATABASE_URL is not set.\n\
                 Example:\n\
                   DATABASE_URL='postgresql://…' cargo run --bin analytics_report -- --days {days}"
            );
            std::process::exit(1);
        }
    };

    let pool = PgPoolOptions::new()
        .max_connections(2)
        .connect(&url)
        .await
        .unwrap_or_else(|err| {
            eprintln!("Failed to connect to Postgres: {err}");
            std::process::exit(1);
        });

    if let Err(err) = degree_planner::analytics::ensure_schema(&pool).await {
        eprintln!("Warning: could not ensure schema: {err}");
    }

    println!("Penn Degree Planner — schedule analytics");
    println!("Window: last {days} day(s)\n");

    run("Overview", print_overview(&pool, days)).await;
    run("Sessions", print_session_overview(&pool, days)).await;
    run("Repeat sessions", print_repeat_sessions(&pool, days)).await;
    run("Session changes", print_session_changes(&pool, days)).await;
    run("Daily volume", print_daily_volume(&pool, days)).await;
    run("When people generate (UTC)", print_time_patterns(&pool, days)).await;
    run("Degree-count mix", print_degree_count_mix(&pool, days)).await;
    run("Top schools", print_top_schools(&pool, days)).await;
    run("Top programs", print_top_majors(&pool, days)).await;
    run("Top combinations", print_top_combos(&pool, days)).await;
    run("Cross-school combinations", print_cross_school(&pool, days)).await;
    run("Major + minor patterns", print_major_minor(&pool, days)).await;
    run("Concentrations", print_concentrations(&pool, days)).await;
    run("Feature usage", print_feature_flags(&pool, days)).await;
    run("Taken / frozen intensity", print_taken_frozen_buckets(&pool, days)).await;
    run("Latency by complexity", print_latency_by_complexity(&pool, days)).await;
    run("Plan size (CU / semesters)", print_plan_size(&pool, days)).await;
    run("Failures", print_failures(&pool, days)).await;
    run("Violations", print_violations(&pool, days)).await;
    run("Recent generates", print_recent(&pool, 15)).await;

    if let Some(session_id) = parse_session(env::args().skip(1)) {
        run(
            "Session timeline",
            print_one_session_timeline(&pool, &session_id),
        )
        .await;
    }
}

async fn run(label: &str, fut: impl std::future::Future<Output = Result<(), sqlx::Error>>) {
    if let Err(err) = fut.await {
        eprintln!("  [{label}] query failed: {err}\n");
    }
}

fn parse_days(mut args: impl Iterator<Item = String>) -> i32 {
    let mut days = 30;
    while let Some(arg) = args.next() {
        if arg == "--days" {
            if let Some(v) = args.next() {
                days = v.parse().unwrap_or(30).max(1);
            }
        } else if let Some(v) = arg.strip_prefix("--days=") {
            days = v.parse().unwrap_or(30).max(1);
        }
    }
    days
}

fn parse_session(mut args: impl Iterator<Item = String>) -> Option<String> {
    while let Some(arg) = args.next() {
        if arg == "--session" {
            return args.next();
        }
        if let Some(v) = arg.strip_prefix("--session=") {
            return Some(v.to_string());
        }
    }
    None
}

fn pct(n: i64, total: i64) -> f64 {
    if total == 0 {
        0.0
    } else {
        (n as f64) * 100.0 / (total as f64)
    }
}

async fn print_session_overview(pool: &sqlx::PgPool, days: i32) -> Result<(), sqlx::Error> {
    let row = sqlx::query(
        r#"
        SELECT
          count(*)::bigint AS generates,
          count(*) FILTER (WHERE anon_session_id IS NOT NULL)::bigint AS with_session,
          count(DISTINCT anon_session_id)::bigint AS distinct_sessions
        FROM schedule_generates
        WHERE created_at > now() - make_interval(days => $1)
        "#,
    )
    .bind(days)
    .fetch_one(pool)
    .await?;

    let generates: i64 = row.get("generates");
    let with_session: i64 = row.get("with_session");
    let sessions: i64 = row.get("distinct_sessions");

    println!("## Sessions (anonymous browser IDs)");
    if generates == 0 {
        println!("  (no rows yet)");
    } else {
        println!(
            "  rows with session id: {:.1}% ({}/{})",
            pct(with_session, generates),
            with_session,
            generates
        );
        println!("  distinct sessions:   {sessions}");
        if with_session > 0 && sessions > 0 {
            println!(
                "  generates / session: {:.1}",
                with_session as f64 / sessions as f64
            );
        }
        if with_session == 0 {
            println!("  (redeploy frontend+API to start attaching anon_session_id)");
        }
    }
    println!();
    Ok(())
}

async fn print_repeat_sessions(pool: &sqlx::PgPool, days: i32) -> Result<(), sqlx::Error> {
    let summary = sqlx::query(
        r#"
        WITH per_session AS (
          SELECT
            anon_session_id,
            count(*)::bigint AS n,
            min(created_at) AS first_at,
            max(created_at) AS last_at
          FROM schedule_generates
          WHERE created_at > now() - make_interval(days => $1)
            AND anon_session_id IS NOT NULL
          GROUP BY 1
        )
        SELECT
          count(*)::bigint AS sessions,
          count(*) FILTER (WHERE n >= 2)::bigint AS multi_gen,
          count(*) FILTER (WHERE n >= 5)::bigint AS heavy,
          count(*) FILTER (
            WHERE last_at - first_at > interval '1 hour'
          )::bigint AS span_1h,
          count(*) FILTER (
            WHERE last_at::date > first_at::date
          )::bigint AS multi_day
        FROM per_session
        "#,
    )
    .bind(days)
    .fetch_one(pool)
    .await?;

    let rows = sqlx::query(
        r#"
        SELECT
          left(anon_session_id, 8) AS session_short,
          anon_session_id,
          count(*)::bigint AS n,
          count(DISTINCT degree_combo_key)::bigint AS combos,
          to_char(min(created_at) AT TIME ZONE 'UTC', 'MM-DD HH24:MI') AS first_ts,
          to_char(max(created_at) AT TIME ZONE 'UTC', 'MM-DD HH24:MI') AS last_ts
        FROM schedule_generates
        WHERE created_at > now() - make_interval(days => $1)
          AND anon_session_id IS NOT NULL
        GROUP BY anon_session_id
        HAVING count(*) >= 2
        ORDER BY n DESC
        LIMIT 15
        "#,
    )
    .bind(days)
    .fetch_all(pool)
    .await?;

    let sessions: i64 = summary.get("sessions");
    println!("## Repeat sessions (same browser, multiple generates)");
    if sessions == 0 {
        println!("  (no session-tagged rows yet)");
    } else {
        println!(
            "  2+ generates:     {} / {} sessions ({:.1}%)",
            summary.get::<i64, _>("multi_gen"),
            sessions,
            pct(summary.get("multi_gen"), sessions)
        );
        println!(
            "  5+ generates:     {} ({:.1}%)",
            summary.get::<i64, _>("heavy"),
            pct(summary.get("heavy"), sessions)
        );
        println!(
            "  span > 1 hour:    {} ({:.1}%)",
            summary.get::<i64, _>("span_1h"),
            pct(summary.get("span_1h"), sessions)
        );
        println!(
            "  multi-day:        {} ({:.1}%)",
            summary.get::<i64, _>("multi_day"),
            pct(summary.get("multi_day"), sessions)
        );
        if !rows.is_empty() {
            println!("  top active sessions:");
            for row in rows {
                let short: String = row.get("session_short");
                let n: i64 = row.get("n");
                let combos: i64 = row.get("combos");
                let first_ts: String = row.get("first_ts");
                let last_ts: String = row.get("last_ts");
                println!(
                    "    {short}…  gens={n:<4} combos={combos}  {first_ts} → {last_ts}"
                );
            }
            println!("  tip: cargo run --bin analytics_report -- --session <full-id>");
        }
    }
    println!();
    Ok(())
}

async fn print_session_changes(pool: &sqlx::PgPool, days: i32) -> Result<(), sqlx::Error> {
    let rows = sqlx::query(
        r#"
        WITH ordered AS (
          SELECT
            anon_session_id,
            created_at,
            degree_combo_key,
            taken_count,
            frozen_count,
            allow_summer,
            has_cu_overrides,
            lag(degree_combo_key) OVER w AS prev_combo,
            lag(taken_count) OVER w AS prev_taken,
            lag(frozen_count) OVER w AS prev_frozen,
            lag(allow_summer) OVER w AS prev_summer,
            lag(has_cu_overrides) OVER w AS prev_cu
          FROM schedule_generates
          WHERE created_at > now() - make_interval(days => $1)
            AND anon_session_id IS NOT NULL
          WINDOW w AS (PARTITION BY anon_session_id ORDER BY created_at, id)
        )
        SELECT
          to_char(created_at AT TIME ZONE 'UTC', 'MM-DD HH24:MI') AS ts,
          left(anon_session_id, 8) AS session_short,
          prev_combo,
          degree_combo_key,
          prev_taken,
          taken_count,
          prev_frozen,
          frozen_count,
          prev_summer,
          allow_summer,
          prev_cu,
          has_cu_overrides
        FROM ordered
        WHERE prev_combo IS NOT NULL
          AND (
            prev_combo IS DISTINCT FROM degree_combo_key
            OR prev_taken IS DISTINCT FROM taken_count
            OR prev_frozen IS DISTINCT FROM frozen_count
            OR prev_summer IS DISTINCT FROM allow_summer
            OR prev_cu IS DISTINCT FROM has_cu_overrides
          )
        ORDER BY created_at DESC
        LIMIT 25
        "#,
    )
    .bind(days)
    .fetch_all(pool)
    .await?;

    println!("## Session changes (what people edited between generates)");
    if rows.is_empty() {
        println!("  (no diffs yet — need 2+ generates from the same session)");
    } else {
        for row in rows {
            let ts: String = row.get("ts");
            let short: String = row.get("session_short");
            let mut parts: Vec<String> = Vec::new();

            let prev_combo: String = row.get("prev_combo");
            let combo: String = row.get("degree_combo_key");
            if prev_combo != combo {
                parts.push(format!("degrees: {prev_combo} → {combo}"));
            }

            let prev_taken: i32 = row.get("prev_taken");
            let taken: i32 = row.get("taken_count");
            if prev_taken != taken {
                parts.push(format!("taken {prev_taken}→{taken}"));
            }

            let prev_frozen: i32 = row.get("prev_frozen");
            let frozen: i32 = row.get("frozen_count");
            if prev_frozen != frozen {
                parts.push(format!("frozen {prev_frozen}→{frozen}"));
            }

            let prev_summer: bool = row.get("prev_summer");
            let summer: bool = row.get("allow_summer");
            if prev_summer != summer {
                parts.push(format!("summer {prev_summer}→{summer}"));
            }

            let prev_cu: bool = row.get("prev_cu");
            let cu: bool = row.get("has_cu_overrides");
            if prev_cu != cu {
                parts.push(format!("cu_overrides {prev_cu}→{cu}"));
            }

            println!("  {ts}  {short}…  {}", parts.join("; "));
        }
    }
    println!();
    Ok(())
}

async fn print_one_session_timeline(
    pool: &sqlx::PgPool,
    session_id: &str,
) -> Result<(), sqlx::Error> {
    let rows = sqlx::query(
        r#"
        SELECT
          to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD HH24:MI:SS') AS ts,
          ok,
          latency_ms,
          taken_count,
          frozen_count,
          allow_summer,
          degree_combo_key
        FROM schedule_generates
        WHERE anon_session_id = $1
        ORDER BY created_at, id
        "#,
    )
    .bind(session_id)
    .fetch_all(pool)
    .await?;

    println!("## Session timeline: {session_id}");
    if rows.is_empty() {
        println!("  (no rows for that session id)");
    } else {
        let mut prev_combo: Option<String> = None;
        let mut prev_taken: Option<i32> = None;
        let mut prev_frozen: Option<i32> = None;
        for row in rows {
            let ts: String = row.get("ts");
            let ok: bool = row.get("ok");
            let latency_ms: Option<i32> = row.get("latency_ms");
            let taken: i32 = row.get("taken_count");
            let frozen: i32 = row.get("frozen_count");
            let summer: bool = row.get("allow_summer");
            let combo: String = row.get("degree_combo_key");
            let status = if ok { "ok " } else { "ERR" };
            let ms = latency_ms
                .map(|v| format!("{v}ms"))
                .unwrap_or_else(|| "-".into());

            let mut marks = Vec::new();
            if prev_combo.as_ref().is_some_and(|p| p != &combo) {
                marks.push("degrees★");
            }
            if prev_taken.is_some_and(|p| p != taken) {
                marks.push("taken★");
            }
            if prev_frozen.is_some_and(|p| p != frozen) {
                marks.push("frozen★");
            }
            let mark = if marks.is_empty() {
                String::new()
            } else {
                format!("  [{}]", marks.join(","))
            };

            println!(
                "  {ts}  {status}  {ms:>6}  taken={taken:<3} frozen={frozen:<3} summer={summer}  {combo}{mark}"
            );
            prev_combo = Some(combo);
            prev_taken = Some(taken);
            prev_frozen = Some(frozen);
        }
    }
    println!();
    Ok(())
}

async fn print_overview(pool: &sqlx::PgPool, days: i32) -> Result<(), sqlx::Error> {
    let row = sqlx::query(
        r#"
        SELECT
          count(*)::bigint AS total,
          count(*) FILTER (WHERE ok)::bigint AS ok_count,
          coalesce(round(avg(latency_ms))::bigint, 0) AS avg_latency_ms,
          coalesce(
            (percentile_cont(0.95) WITHIN GROUP (ORDER BY latency_ms))::bigint,
            0
          ) AS p95_latency_ms,
          count(*) FILTER (WHERE jsonb_array_length(degrees) > 1)::bigint AS multi_degree,
          count(*) FILTER (WHERE ok AND has_overlap)::bigint AS with_overlap,
          count(DISTINCT date_trunc('day', created_at))::bigint AS active_days
        FROM schedule_generates
        WHERE created_at > now() - make_interval(days => $1)
        "#,
    )
    .bind(days)
    .fetch_one(pool)
    .await?;

    let total: i64 = row.get("total");
    let ok_count: i64 = row.get("ok_count");
    println!("## Overview");
    println!("  generates:     {total}");
    println!(
        "  success rate:  {:.1}% ({}/{})",
        pct(ok_count, total),
        ok_count,
        total
    );
    println!(
        "  multi-degree:  {:.1}% ({}/{})",
        pct(row.get("multi_degree"), total),
        row.get::<i64, _>("multi_degree"),
        total
    );
    println!(
        "  with overlap:  {:.1}% ({}/{})",
        pct(row.get("with_overlap"), total),
        row.get::<i64, _>("with_overlap"),
        total
    );
    println!("  active days:   {}", row.get::<i64, _>("active_days"));
    println!(
        "  latency avg:   {} ms",
        row.get::<i64, _>("avg_latency_ms")
    );
    println!(
        "  latency p95:   {} ms",
        row.get::<i64, _>("p95_latency_ms")
    );
    println!();
    Ok(())
}

async fn print_daily_volume(pool: &sqlx::PgPool, days: i32) -> Result<(), sqlx::Error> {
    let rows = sqlx::query(
        r#"
        SELECT
          to_char(date_trunc('day', created_at AT TIME ZONE 'UTC'), 'YYYY-MM-DD') AS day,
          count(*)::bigint AS n,
          count(*) FILTER (WHERE ok)::bigint AS ok_n
        FROM schedule_generates
        WHERE created_at > now() - make_interval(days => $1)
        GROUP BY 1
        ORDER BY 1
        "#,
    )
    .bind(days)
    .fetch_all(pool)
    .await?;

    println!("## Daily volume (UTC)");
    if rows.is_empty() {
        println!("  (no rows yet)");
    } else {
        for row in rows {
            let day: String = row.get("day");
            let n: i64 = row.get("n");
            let ok_n: i64 = row.get("ok_n");
            let bar = "█".repeat(n.min(40) as usize);
            println!("  {day}  {n:>4} ok={ok_n:<4}  {bar}");
        }
    }
    println!();
    Ok(())
}

async fn print_time_patterns(pool: &sqlx::PgPool, days: i32) -> Result<(), sqlx::Error> {
    let dow = sqlx::query(
        r#"
        SELECT
          to_char(created_at AT TIME ZONE 'UTC', 'Dy') AS dow,
          extract(dow FROM created_at AT TIME ZONE 'UTC')::int AS dow_n,
          count(*)::bigint AS n
        FROM schedule_generates
        WHERE created_at > now() - make_interval(days => $1)
        GROUP BY 1, 2
        ORDER BY 2
        "#,
    )
    .bind(days)
    .fetch_all(pool)
    .await?;

    let hours = sqlx::query(
        r#"
        SELECT
          extract(hour FROM created_at AT TIME ZONE 'UTC')::int AS hour,
          count(*)::bigint AS n
        FROM schedule_generates
        WHERE created_at > now() - make_interval(days => $1)
        GROUP BY 1
        ORDER BY n DESC
        LIMIT 8
        "#,
    )
    .bind(days)
    .fetch_all(pool)
    .await?;

    println!("## When people generate (UTC)");
    if dow.is_empty() {
        println!("  (no rows yet)");
    } else {
        print!("  by weekday:");
        for row in &dow {
            let name: String = row.get("dow");
            let n: i64 = row.get("n");
            print!("  {name}={n}");
        }
        println!();
        println!("  busiest hours (UTC):");
        for row in hours {
            let hour: i32 = row.get("hour");
            let n: i64 = row.get("n");
            println!("    {hour:02}:00  {n}");
        }
    }
    println!();
    Ok(())
}

async fn print_degree_count_mix(pool: &sqlx::PgPool, days: i32) -> Result<(), sqlx::Error> {
    let rows = sqlx::query(
        r#"
        SELECT
          jsonb_array_length(degrees) AS n_degrees,
          count(*)::bigint AS n
        FROM schedule_generates
        WHERE created_at > now() - make_interval(days => $1) AND ok
        GROUP BY 1
        ORDER BY 1
        "#,
    )
    .bind(days)
    .fetch_all(pool)
    .await?;

    println!("## Degree-count mix (ok only)");
    if rows.is_empty() {
        println!("  (no rows yet)");
    } else {
        let total: i64 = rows.iter().map(|r| r.get::<i64, _>("n")).sum();
        for row in rows {
            let n_degrees: i32 = row.get("n_degrees");
            let n: i64 = row.get("n");
            println!(
                "  {n_degrees} program(s): {n:>5}  ({:.1}%)",
                pct(n, total)
            );
        }
    }
    println!();
    Ok(())
}

async fn print_top_schools(pool: &sqlx::PgPool, days: i32) -> Result<(), sqlx::Error> {
    let rows = sqlx::query(
        r#"
        SELECT d->>'school' AS school, count(*)::bigint AS n
        FROM schedule_generates,
             jsonb_array_elements(degrees) AS d
        WHERE created_at > now() - make_interval(days => $1) AND ok
        GROUP BY 1
        ORDER BY n DESC
        "#,
    )
    .bind(days)
    .fetch_all(pool)
    .await?;

    println!("## Top schools");
    if rows.is_empty() {
        println!("  (no rows yet)");
    } else {
        for row in rows {
            let school: String = row.get("school");
            let n: i64 = row.get("n");
            println!("  {n:>5}  {school}");
        }
    }
    println!();
    Ok(())
}

async fn print_top_majors(pool: &sqlx::PgPool, days: i32) -> Result<(), sqlx::Error> {
    let rows = sqlx::query(
        r#"
        SELECT
          d->>'school' AS school,
          d->>'major' AS major,
          d->>'kind' AS kind,
          count(*)::bigint AS n
        FROM schedule_generates,
             jsonb_array_elements(degrees) AS d
        WHERE created_at > now() - make_interval(days => $1) AND ok
        GROUP BY 1, 2, 3
        ORDER BY n DESC
        LIMIT 25
        "#,
    )
    .bind(days)
    .fetch_all(pool)
    .await?;

    println!("## Top programs (counted once per generate they appear in)");
    if rows.is_empty() {
        println!("  (no rows yet)");
    } else {
        for row in rows {
            let school: String = row.get("school");
            let major: String = row.get("major");
            let kind: String = row.get("kind");
            let n: i64 = row.get("n");
            println!("  {n:>5}  {school}:{major} ({kind})");
        }
    }
    println!();
    Ok(())
}

async fn print_top_combos(pool: &sqlx::PgPool, days: i32) -> Result<(), sqlx::Error> {
    let rows = sqlx::query(
        r#"
        SELECT degree_combo_key, count(*)::bigint AS n
        FROM schedule_generates
        WHERE created_at > now() - make_interval(days => $1) AND ok
        GROUP BY 1
        ORDER BY n DESC
        LIMIT 20
        "#,
    )
    .bind(days)
    .fetch_all(pool)
    .await?;

    println!("## Top degree combinations (ok only)");
    if rows.is_empty() {
        println!("  (no rows yet)");
    } else {
        for row in rows {
            let key: String = row.get("degree_combo_key");
            let n: i64 = row.get("n");
            println!("  {n:>5}  {key}");
        }
    }
    println!();
    Ok(())
}

async fn print_cross_school(pool: &sqlx::PgPool, days: i32) -> Result<(), sqlx::Error> {
    let rows = sqlx::query(
        r#"
        SELECT school_set, count(*)::bigint AS n
        FROM (
          SELECT
            id,
            string_agg(DISTINCT d->>'school', '+' ORDER BY d->>'school') AS school_set
          FROM schedule_generates,
               jsonb_array_elements(degrees) AS d
          WHERE created_at > now() - make_interval(days => $1) AND ok
          GROUP BY id
          HAVING count(DISTINCT d->>'school') > 1
        ) t
        GROUP BY 1
        ORDER BY n DESC
        LIMIT 15
        "#,
    )
    .bind(days)
    .fetch_all(pool)
    .await?;

    println!("## Cross-school combinations");
    if rows.is_empty() {
        println!("  (none — all single-school so far)");
    } else {
        for row in rows {
            let school_set: String = row.get("school_set");
            let n: i64 = row.get("n");
            println!("  {n:>5}  {school_set}");
        }
    }
    println!();
    Ok(())
}

async fn print_major_minor(pool: &sqlx::PgPool, days: i32) -> Result<(), sqlx::Error> {
    let row = sqlx::query(
        r#"
        SELECT
          count(*) FILTER (
            WHERE EXISTS (
              SELECT 1 FROM jsonb_array_elements(degrees) d WHERE d->>'kind' = 'major'
            )
            AND EXISTS (
              SELECT 1 FROM jsonb_array_elements(degrees) d WHERE d->>'kind' = 'minor'
            )
          )::bigint AS major_and_minor,
          count(*) FILTER (
            WHERE NOT EXISTS (
              SELECT 1 FROM jsonb_array_elements(degrees) d WHERE d->>'kind' = 'minor'
            )
            AND jsonb_array_length(degrees) = 1
          )::bigint AS single_major,
          count(*) FILTER (
            WHERE NOT EXISTS (
              SELECT 1 FROM jsonb_array_elements(degrees) d WHERE d->>'kind' = 'minor'
            )
            AND jsonb_array_length(degrees) > 1
          )::bigint AS multi_major_no_minor,
          count(*)::bigint AS total
        FROM schedule_generates
        WHERE created_at > now() - make_interval(days => $1) AND ok
        "#,
    )
    .bind(days)
    .fetch_one(pool)
    .await?;

    let total: i64 = row.get("total");
    println!("## Major + minor patterns (ok only)");
    if total == 0 {
        println!("  (no rows yet)");
    } else {
        println!(
            "  single major only:     {:>5}  ({:.1}%)",
            row.get::<i64, _>("single_major"),
            pct(row.get("single_major"), total)
        );
        println!(
            "  multi-major, no minor: {:>5}  ({:.1}%)",
            row.get::<i64, _>("multi_major_no_minor"),
            pct(row.get("multi_major_no_minor"), total)
        );
        println!(
            "  major + minor:         {:>5}  ({:.1}%)",
            row.get::<i64, _>("major_and_minor"),
            pct(row.get("major_and_minor"), total)
        );
    }
    println!();
    Ok(())
}

async fn print_concentrations(pool: &sqlx::PgPool, days: i32) -> Result<(), sqlx::Error> {
    let summary = sqlx::query(
        r#"
        SELECT
          count(*)::bigint AS total,
          count(*) FILTER (
            WHERE EXISTS (
              SELECT 1 FROM jsonb_array_elements(degrees) d
              WHERE jsonb_typeof(d->'concentrations') = 'array'
                AND jsonb_array_length(d->'concentrations') > 0
            )
          )::bigint AS with_conc
        FROM schedule_generates
        WHERE created_at > now() - make_interval(days => $1) AND ok
        "#,
    )
    .bind(days)
    .fetch_one(pool)
    .await?;

    let rows = sqlx::query(
        r#"
        SELECT
          d->>'school' AS school,
          d->>'major' AS major,
          conc AS concentration,
          count(*)::bigint AS n
        FROM schedule_generates,
             jsonb_array_elements(degrees) AS d,
             jsonb_array_elements_text(d->'concentrations') AS conc
        WHERE created_at > now() - make_interval(days => $1) AND ok
        GROUP BY 1, 2, 3
        ORDER BY n DESC
        LIMIT 20
        "#,
    )
    .bind(days)
    .fetch_all(pool)
    .await?;

    let total: i64 = summary.get("total");
    let with_conc: i64 = summary.get("with_conc");
    println!("## Concentrations");
    println!(
        "  generates with any concentration: {:.1}% ({}/{})",
        pct(with_conc, total),
        with_conc,
        total
    );
    if rows.is_empty() {
        println!("  (no concentration selections yet)");
    } else {
        println!("  top concentration picks:");
        for row in rows {
            let school: String = row.get("school");
            let major: String = row.get("major");
            let concentration: String = row.get("concentration");
            let n: i64 = row.get("n");
            println!("    {n:>5}  {school}:{major} → {concentration}");
        }
    }
    println!();
    Ok(())
}

async fn print_feature_flags(pool: &sqlx::PgPool, days: i32) -> Result<(), sqlx::Error> {
    let row = sqlx::query(
        r#"
        SELECT
          count(*)::bigint AS total,
          count(*) FILTER (WHERE allow_summer)::bigint AS summer,
          count(*) FILTER (WHERE has_cu_overrides)::bigint AS cu_overrides,
          count(*) FILTER (WHERE frozen_count > 0)::bigint AS with_frozen,
          count(*) FILTER (WHERE taken_count > 0)::bigint AS with_taken,
          count(*) FILTER (WHERE has_overlap)::bigint AS with_overlap,
          coalesce(round(avg(taken_count))::bigint, 0) AS avg_taken,
          coalesce(round(avg(frozen_count))::bigint, 0) AS avg_frozen
        FROM schedule_generates
        WHERE created_at > now() - make_interval(days => $1)
        "#,
    )
    .bind(days)
    .fetch_one(pool)
    .await?;

    let total: i64 = row.get("total");
    println!("## Feature usage (share of generates)");
    println!(
        "  allow summer:     {:>5.1}%  ({}/{})",
        pct(row.get("summer"), total),
        row.get::<i64, _>("summer"),
        total
    );
    println!(
        "  CU overrides:     {:>5.1}%  ({}/{})",
        pct(row.get("cu_overrides"), total),
        row.get::<i64, _>("cu_overrides"),
        total
    );
    println!(
        "  has frozen:       {:>5.1}%  ({}/{})",
        pct(row.get("with_frozen"), total),
        row.get::<i64, _>("with_frozen"),
        total
    );
    println!(
        "  has taken:        {:>5.1}%  ({}/{})",
        pct(row.get("with_taken"), total),
        row.get::<i64, _>("with_taken"),
        total
    );
    println!(
        "  overlap plan:     {:>5.1}%  ({}/{})",
        pct(row.get("with_overlap"), total),
        row.get::<i64, _>("with_overlap"),
        total
    );
    println!(
        "  avg taken/frozen: {} / {}",
        row.get::<i64, _>("avg_taken"),
        row.get::<i64, _>("avg_frozen")
    );
    println!();
    Ok(())
}

async fn print_taken_frozen_buckets(pool: &sqlx::PgPool, days: i32) -> Result<(), sqlx::Error> {
    let rows = sqlx::query(
        r#"
        SELECT
          CASE
            WHEN taken_count = 0 THEN '0 taken'
            WHEN taken_count BETWEEN 1 AND 4 THEN '1–4 taken'
            WHEN taken_count BETWEEN 5 AND 12 THEN '5–12 taken'
            ELSE '13+ taken'
          END AS bucket,
          count(*)::bigint AS n
        FROM schedule_generates
        WHERE created_at > now() - make_interval(days => $1)
        GROUP BY 1
        ORDER BY min(taken_count)
        "#,
    )
    .bind(days)
    .fetch_all(pool)
    .await?;

    println!("## Taken-course intensity (how far along are planners?)");
    if rows.is_empty() {
        println!("  (no rows yet)");
    } else {
        for row in rows {
            let bucket: String = row.get("bucket");
            let n: i64 = row.get("n");
            println!("  {n:>5}  {bucket}");
        }
    }
    println!();
    Ok(())
}

async fn print_latency_by_complexity(pool: &sqlx::PgPool, days: i32) -> Result<(), sqlx::Error> {
    let rows = sqlx::query(
        r#"
        SELECT
          jsonb_array_length(degrees) AS n_degrees,
          count(*)::bigint AS n,
          coalesce(round(avg(latency_ms))::bigint, 0) AS avg_ms,
          coalesce(
            (percentile_cont(0.95) WITHIN GROUP (ORDER BY latency_ms))::bigint,
            0
          ) AS p95_ms
        FROM schedule_generates
        WHERE created_at > now() - make_interval(days => $1)
        GROUP BY 1
        ORDER BY 1
        "#,
    )
    .bind(days)
    .fetch_all(pool)
    .await?;

    println!("## Latency by # of programs");
    if rows.is_empty() {
        println!("  (no rows yet)");
    } else {
        for row in rows {
            let n_degrees: i32 = row.get("n_degrees");
            let n: i64 = row.get("n");
            let avg_ms: i64 = row.get("avg_ms");
            let p95_ms: i64 = row.get("p95_ms");
            println!("  {n_degrees} program(s): n={n:<5} avg={avg_ms}ms  p95={p95_ms}ms");
        }
    }
    println!();
    Ok(())
}

async fn print_plan_size(pool: &sqlx::PgPool, days: i32) -> Result<(), sqlx::Error> {
    let row = sqlx::query(
        r#"
        SELECT
          count(*) FILTER (WHERE total_cu IS NOT NULL)::bigint AS with_cu,
          coalesce(round(avg(total_cu)::numeric, 1), 0)::float8 AS avg_cu,
          coalesce(
            round((percentile_cont(0.5) WITHIN GROUP (ORDER BY total_cu))::numeric, 1),
            0
          )::float8 AS p50_cu,
          coalesce(round(avg(semester_count))::bigint, 0) AS avg_semesters
        FROM schedule_generates
        WHERE created_at > now() - make_interval(days => $1) AND ok
        "#,
    )
    .bind(days)
    .fetch_one(pool)
    .await?;

    println!("## Plan size (newer rows only — needs redeployed API)");
    let with_cu: i64 = row.get("with_cu");
    if with_cu == 0 {
        println!("  (no total_cu yet — deploy updated API to start collecting)");
    } else {
        println!("  rows with CU:    {with_cu}");
        println!("  avg total CU:    {:.1}", row.get::<f64, _>("avg_cu"));
        println!("  median total CU: {:.1}", row.get::<f64, _>("p50_cu"));
        println!(
            "  avg semesters:   {}",
            row.get::<i64, _>("avg_semesters")
        );
    }
    println!();
    Ok(())
}

async fn print_failures(pool: &sqlx::PgPool, days: i32) -> Result<(), sqlx::Error> {
    let combos = sqlx::query(
        r#"
        SELECT degree_combo_key, count(*)::bigint AS n
        FROM schedule_generates
        WHERE created_at > now() - make_interval(days => $1) AND NOT ok
        GROUP BY 1
        ORDER BY n DESC
        LIMIT 15
        "#,
    )
    .bind(days)
    .fetch_all(pool)
    .await?;

    let kinds = sqlx::query(
        r#"
        SELECT k AS error_kind, count(*)::bigint AS n
        FROM schedule_generates,
             unnest(error_kinds) AS k
        WHERE created_at > now() - make_interval(days => $1) AND NOT ok
        GROUP BY 1
        ORDER BY n DESC
        LIMIT 15
        "#,
    )
    .bind(days)
    .fetch_all(pool)
    .await?;

    println!("## Failures");
    if combos.is_empty() {
        println!("  (no failed generates in window)");
    } else {
        println!("  by combination:");
        for row in &combos {
            let key: String = row.get("degree_combo_key");
            let n: i64 = row.get("n");
            println!("    {n:>5}  {key}");
        }
        if !kinds.is_empty() {
            println!("  by error kind (newer rows):");
            for row in kinds {
                let error_kind: String = row.get("error_kind");
                let n: i64 = row.get("n");
                println!("    {n:>5}  {error_kind}");
            }
        }
    }
    println!();
    Ok(())
}

async fn print_violations(pool: &sqlx::PgPool, days: i32) -> Result<(), sqlx::Error> {
    let rows = sqlx::query(
        r#"
        SELECT v AS violation, count(*)::bigint AS n
        FROM schedule_generates,
             unnest(violation_types) AS v
        WHERE created_at > now() - make_interval(days => $1)
        GROUP BY 1
        ORDER BY n DESC
        "#,
    )
    .bind(days)
    .fetch_all(pool)
    .await?;

    println!("## Cross-degree violation types");
    if rows.is_empty() {
        println!("  (none recorded)");
    } else {
        for row in rows {
            let violation: String = row.get("violation");
            let n: i64 = row.get("n");
            println!("  {n:>5}  {violation}");
        }
    }
    println!();
    Ok(())
}

async fn print_recent(pool: &sqlx::PgPool, limit: i64) -> Result<(), sqlx::Error> {
    let rows = sqlx::query(
        r#"
        SELECT
          to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD HH24:MI') AS ts,
          ok,
          latency_ms,
          left(coalesce(anon_session_id, ''), 8) AS session_short,
          degree_combo_key
        FROM schedule_generates
        ORDER BY created_at DESC
        LIMIT $1
        "#,
    )
    .bind(limit)
    .fetch_all(pool)
    .await?;

    println!("## Recent generates (UTC)");
    if rows.is_empty() {
        println!("  (no rows yet — deploy API with DATABASE_URL, then generate a schedule)");
    } else {
        for row in rows {
            let ts: String = row.get("ts");
            let ok: bool = row.get("ok");
            let latency_ms: Option<i32> = row.get("latency_ms");
            let session_short: String = row.get("session_short");
            let key: String = row.get("degree_combo_key");
            let status = if ok { "ok " } else { "ERR" };
            let ms = latency_ms
                .map(|v| format!("{v}ms"))
                .unwrap_or_else(|| "-".into());
            let sess = if session_short.is_empty() {
                "--------".to_string()
            } else {
                format!("{session_short}…")
            };
            println!("  {ts}  {status}  {ms:>6}  {sess}  {key}");
        }
    }
    println!();
    Ok(())
}
