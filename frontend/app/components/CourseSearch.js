"use client";

import { useState, useMemo, useEffect, useRef } from "react";
import DraggableCourse from "./DraggableCourse";
import { API_BASE } from "@/lib/api";
import { buildSemesterOptions } from "@/lib/semesterOptions";
import { filterValidCourseCodes } from "@/lib/courseUtils";
import { sortCourseCodesBySemester } from "@/lib/courseOrdering";

const SEARCH_DEBOUNCE_MS = 300;
const SEARCH_LIMIT = 50;

export default function CourseSearch({
    takenCourses, assignedCourses, frozenCourses = [],
    onAdd, onRemove, onAssign,
    maxScheduleYear = 4, allowSummer = true,
}) {
    const [search, setSearch] = useState("");
    const [results, setResults] = useState([]);
    const [searching, setSearching] = useState(false);
    const abortRef = useRef(null);

    const semesterOptions = useMemo(
        () => buildSemesterOptions(maxScheduleYear, allowSummer),
        [maxScheduleYear, allowSummer]
    );

    const sortedCartCourses = useMemo(
        () =>
            sortCourseCodesBySemester(filterValidCourseCodes(takenCourses), {
                assignedCourses,
                frozenCourses,
                semesterOptions,
            }),
        [takenCourses, assignedCourses, frozenCourses, semesterOptions]
    );

    useEffect(() => {
        const q = search.trim();
        if (!q) {
            setResults([]);
            setSearching(false);
            return undefined;
        }

        const timer = setTimeout(() => {
            abortRef.current?.abort();
            const controller = new AbortController();
            abortRef.current = controller;
            setSearching(true);

            const params = new URLSearchParams({ q, limit: String(SEARCH_LIMIT) });
            fetch(`${API_BASE}/search_courses?${params}`, { signal: controller.signal })
                .then((r) => r.json())
                .then((data) => {
                    setResults(Array.isArray(data?.courses) ? data.courses : []);
                })
                .catch((err) => {
                    if (err.name !== "AbortError") setResults([]);
                })
                .finally(() => {
                    if (!controller.signal.aborted) setSearching(false);
                });
        }, SEARCH_DEBOUNCE_MS);

        return () => {
            clearTimeout(timer);
            abortRef.current?.abort();
        };
    }, [search]);

    const getAssignment = (courseId) => {
        const a = assignedCourses?.find(ac => ac.courseId === courseId);
        return a ? `${a.semester}-${a.year}` : "";
    };

    const handleAssign = (courseId, value) => {
        if (!value) {
            onAssign(courseId, null, null);
            return;
        }
        const [semester, yearStr] = value.split("-");
        onAssign(courseId, parseInt(yearStr), semester);
    };

    const showResults = search.trim().length > 0;

    return (
        <>
            <div className="search-box">
                <span className="icon">🔍</span>
                <input
                    className="input"
                    placeholder="Search courses (e.g. MATH 1400)…"
                    value={search}
                    onChange={(e) => setSearch(e.target.value)}
                />
            </div>

            {showResults && (
                <div className="course-list">
                    {searching && (
                        <div style={{ padding: 12, fontSize: "0.8rem", color: "var(--text-muted)" }}>
                            Searching…
                        </div>
                    )}
                    {!searching && results.length === 0 && (
                        <div style={{ padding: 12, fontSize: "0.8rem", color: "var(--text-muted)" }}>
                            No courses found
                        </div>
                    )}
                    {results.map(course => {
                        const inCart = takenCourses.includes(course.course_code);
                        return (
                            <DraggableCourse
                                key={course.course_code}
                                id={`search-${course.course_code}`}
                                data={{ courseId: course.course_code, source: "search" }}
                            >
                                <div
                                    className={`course-item ${inCart ? "in-cart" : ""}`}
                                    onClick={() => inCart ? onRemove(course.course_code) : onAdd(course.course_code)}
                                >
                                    <div>
                                        <div className="course-code">
                                            {course.course_code}
                                            {inCart && <span className="course-in-cart-mark">✓</span>}
                                        </div>
                                        <div className="course-title">{course.title}</div>
                                    </div>
                                    <div className="course-cu">{course.cu} CU</div>
                                </div>
                            </DraggableCourse>
                        );
                    })}
                </div>
            )}

            <div className="cart-section">
                <div className="cart-header">
                    <h3>My Courses</h3>
                    <span className="cart-count">{filterValidCourseCodes(takenCourses).length}</span>
                </div>
                <div className="cart-hint">
                    Drag courses to a semester slot, or use the dropdown
                </div>
                {takenCourses.length === 0 ? (
                    <div className="cart-empty">
                        Search and add courses above
                    </div>
                ) : (
                    <div className="cart-list">
                        {sortedCartCourses.map(code => {
                            const assignValue = getAssignment(code);
                            return (
                                <DraggableCourse
                                    key={code}
                                    id={`cart-${code}`}
                                    data={{ courseId: code, source: "cart" }}
                                >
                                    <div className="cart-item fade-in">
                                        <span className="cart-item-drag-handle">⠿</span>
                                        <span style={{ flex: 1 }}>{code}</span>
                                        <select
                                            className="semester-assign-select"
                                            value={assignValue}
                                            onChange={(e) => handleAssign(code, e.target.value)}
                                            onClick={(e) => e.stopPropagation()}
                                            onPointerDown={(e) => e.stopPropagation()}
                                            title="Assign to semester"
                                        >
                                            {semesterOptions.map(opt => (
                                                <option key={opt.value} value={opt.value}>{opt.label}</option>
                                            ))}
                                        </select>
                                        <button
                                            className="btn-icon"
                                            style={{ width: 22, height: 22, fontSize: "0.65rem" }}
                                            onClick={(e) => { e.stopPropagation(); onRemove(code); }}
                                            onPointerDown={(e) => e.stopPropagation()}
                                        >
                                            ✕
                                        </button>
                                    </div>
                                </DraggableCourse>
                            );
                        })}
                    </div>
                )}
            </div>
        </>
    );
}
