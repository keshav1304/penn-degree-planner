"use client";

import { useState, useMemo } from "react";
import DraggableCourse from "./DraggableCourse";
import { buildSemesterOptions } from "@/lib/semesterOptions";
import { filterValidCourseCodes } from "@/lib/courseUtils";
import { sortCourseCodesBySemester } from "@/lib/courseOrdering";
import { searchCourses } from "@/lib/courseCatalog";

const SEARCH_LIMIT = 50;

export default function CourseSearch({
    courseCatalog,
    takenCourses, assignedCourses, frozenCourses = [],
    onAdd, onRemove, onAssign,
    maxScheduleYear = 4, allowSummer = true,
}) {
    const [search, setSearch] = useState("");

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

    const results = useMemo(() => {
        const q = search.trim();
        if (!q || !courseCatalog) return [];
        return searchCourses(courseCatalog, q, SEARCH_LIMIT);
    }, [courseCatalog, search]);

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
    const catalogLoading = !courseCatalog;

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
                    {catalogLoading && (
                        <div style={{ padding: 12, fontSize: "0.8rem", color: "var(--text-muted)" }}>
                            Loading catalog…
                        </div>
                    )}
                    {!catalogLoading && results.length === 0 && (
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
                                    <div className="course-item-main">
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
