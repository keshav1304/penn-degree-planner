"use client";

import { useState, useMemo } from "react";
import DraggableCourse from "./DraggableCourse";

const SEMESTER_OPTIONS = [
    { label: "—", value: "" },
    { label: "Credits Received", value: "Credits-0" },
    { label: "Freshman Fall", value: "Fall-1" },
    { label: "Freshman Spring", value: "Spring-1" },
    { label: "Freshman Summer", value: "Summer-1" },
    { label: "Sophomore Fall", value: "Fall-2" },
    { label: "Sophomore Spring", value: "Spring-2" },
    { label: "Sophomore Summer", value: "Summer-2" },
    { label: "Junior Fall", value: "Fall-3" },
    { label: "Junior Spring", value: "Spring-3" },
    { label: "Junior Summer", value: "Summer-3" },
    { label: "Senior Fall", value: "Fall-4" },
    { label: "Senior Spring", value: "Spring-4" },
    { label: "Senior Summer", value: "Summer-4" },
];

export default function CourseSearch({ allCourses, takenCourses, assignedCourses, onAdd, onRemove, onAssign }) {
    const [search, setSearch] = useState("");

    const filteredCourses = useMemo(() => {
        if (!search.trim()) return [];
        const q = search.toLowerCase().trim();
        return allCourses
            .filter(c =>
                c.course_code?.toLowerCase().includes(q) ||
                c.title?.toLowerCase().includes(q) ||
                c.dept_code?.toLowerCase().includes(q)
            )
            .slice(0, 50);
    }, [search, allCourses]);

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

            {search.trim() && (
                <div className="course-list">
                    {filteredCourses.length === 0 && (
                        <div style={{ padding: 12, fontSize: "0.8rem", color: "var(--text-muted)" }}>
                            No courses found
                        </div>
                    )}
                    {filteredCourses.map(course => {
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
                                            {inCart && <span style={{ marginLeft: 6, fontSize: "0.7rem" }}>✓</span>}
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
                    <span className="cart-count">{takenCourses.length}</span>
                </div>
                <div style={{ fontSize: "0.68rem", color: "var(--text-muted)", marginBottom: 8 }}>
                    Drag courses to a semester slot, or use the dropdown
                </div>
                {takenCourses.length === 0 ? (
                    <div style={{ fontSize: "0.78rem", color: "var(--text-muted)", padding: "8px 0" }}>
                        Search and add courses above
                    </div>
                ) : (
                    <div className="cart-list">
                        {takenCourses.map(code => {
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
                                            {SEMESTER_OPTIONS.map(opt => (
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
