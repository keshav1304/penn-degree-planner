"use client";

import { useState } from "react";
import DraggableCourse from "./DraggableCourse";
import DroppableSemester from "./DroppableSemester";

const YEAR_NAMES = {};

const DEGREE_COLORS = [
    "#a51c30",  // Penn red
    "#059669",  // teal
    "#d97706",  // amber
    "#7c3aed",  // purple
];

const DC_COLORS = [
    "#6366f1",  // indigo
    "#ec4899",  // pink
    "#14b8a6",  // teal
    "#f59e0b",  // amber
];

export default function ScheduleGrid({
    scheduleData, frozenCourses, assignedCourses,
    onToggleFreeze, onMarkTaken, onUnmarkTaken, degrees,
    courseDegreesMap, courseRequirementMap, allowSummer,
    doubleCountData, courseDoubleCountMap,
}) {
    const [creditsCollapsed, setCreditsCollapsed] = useState(false);
    const [infoPopup, setInfoPopup] = useState(null); // { courseId, x, y }

    if (!degrees || degrees.length === 0) {
        return (
            <div className="empty-state">
                <div className="emoji">🏫</div>
                <div>Add a degree above to generate your schedule</div>
            </div>
        );
    }

    if (!scheduleData || !scheduleData.schedule) {
        return (
            <div className="empty-state">
                <div className="loading-spinner" style={{ width: 24, height: 24 }} />
                <div style={{ marginTop: 10 }}>Generating schedule…</div>
            </div>
        );
    }

    // Derive years and semesters dynamically from schedule data
    const uniqueYears = [...new Set(scheduleData.schedule.map(s => s.year))].sort((a, b) => a - b);
    const uniqueSemesters = [...new Set(scheduleData.schedule.map(s => s.semester))];
    // Maintain Fall, Spring, Summer order
    const semOrder = ["Fall", "Spring", "Summer"];
    const semesters = semOrder.filter(s => uniqueSemesters.includes(s));

    const getSemesterPlan = (year, semester) => {
        return scheduleData.schedule.find(
            s => s.year === year && s.semester === semester
        );
    };

    const isFrozen = (courseId) => frozenCourses.some(f => f.courseId === courseId);
    const isAssigned = (courseId) => assignedCourses?.some(a => a.courseId === courseId);

    // Build degree label → color index map
    const degreeColorMap = {};
    if (scheduleData?.degree_results) {
        scheduleData.degree_results.forEach((result, i) => {
            degreeColorMap[`${result.school}-${result.major}`] = DEGREE_COLORS[i % DEGREE_COLORS.length];
        });
    }

    // Courses assigned to "Credits Received" (year 0)
    const creditsCourses = assignedCourses?.filter(a => a.year === 0) || [];

    const handleInfoClick = (e, courseId) => {
        e.stopPropagation();
        if (infoPopup?.courseId === courseId) {
            setInfoPopup(null);
        } else {
            const rect = e.currentTarget.getBoundingClientRect();
            console.log(rect)
            setInfoPopup({ courseId, x: rect.left, y: rect.bottom });
        }
    };

    const renderDegreeBar = (courseId) => {
        const degs = courseDegreesMap?.[courseId];
        if (!degs || degs.length === 0) return null;
        return (
            <div className="degree-bar-container">
                {degs.map((d, i) => (
                    <div
                        key={d}
                        className="degree-bar-stripe"
                        style={{ background: degreeColorMap[d] || "#888" }}
                        title={d}
                    />
                ))}
            </div>
        );
    };

    const renderInfoButton = (courseId) => {
        const reqs = courseRequirementMap?.[courseId];
        if (!reqs || reqs.length === 0) return null;
        return (
            <button
                className="course-info-btn"
                onClick={(e) => handleInfoClick(e, courseId)}
                title="View requirement info"
            >ℹ️</button>
        );
    };

    const renderDcBadges = (courseId) => {
        const dcEntries = courseDoubleCountMap?.[courseId];
        if (!dcEntries || dcEntries.length === 0) return null;
        return (
            <span className="dc-badges">
                {dcEntries.map((entry, i) => (
                    <span
                        key={i}
                        className={`dc-badge ${entry.isDoubleCountMatch ? "dc-badge-matched" : ""}`}
                        style={{ borderColor: DC_COLORS[entry.dcIndex % DC_COLORS.length] }}
                        title={`${entry.dcCategory} (${entry.dcLabel})${entry.isDoubleCountMatch ? " ✓ double-counted" : ""}`}
                    >
                        {entry.dcLabel}
                    </span>
                ))}
            </span>
        );
    };

    const renderCourseCard = (courseId, year, sem, idx) => {
        const frozen = isFrozen(courseId);
        const assigned = isAssigned(courseId);
        let className = "schedule-course";
        if (assigned) className += " assigned";
        else if (frozen) className += " frozen";

        const handleClick = () => {
            if (assigned) {
                // Green → Orange: freeze in place
                onUnmarkTaken(courseId);
                onToggleFreeze(courseId, year, sem);
            } else if (frozen) {
                // Orange → Default: remove freeze
                onUnmarkTaken(courseId);
            } else {
                // Default → Green: mark as taken
                onMarkTaken(courseId, year, sem);
            }
        };

        return (
            <DraggableCourse
                key={`${courseId}-${idx}`}
                id={`schedule-${year}-${sem}-${courseId}-${idx}`}
                data={{ courseId, source: "schedule", fromYear: year, fromSemester: sem }}
            >
                <div className={className} style={{ position: "relative" }}>
                    {renderDegreeBar(courseId)}
                    <div
                        className="schedule-course-content"
                        onClick={handleClick}
                        title={
                            assigned ? "Click to freeze (orange)"
                                : frozen ? "Click to un-mark (default)"
                                    : "Click to mark taken (green)"
                        }
                    >
                        <span>{courseId}</span>
                        <span className="course-card-actions">
                            {renderDcBadges(courseId)}
                            {renderInfoButton(courseId)}
                            <span className="lock-icon">
                                {assigned ? "📗" : frozen ? "🔒" : "📌"}
                            </span>
                        </span>
                    </div>
                </div>
            </DraggableCourse>
        );
    };

    return (
        <div className="schedule-container" onClick={() => setInfoPopup(null)}>
            {/* Credits Received section — collapsible */}
            <div className="credits-received-row fade-in">
                <div
                    className="credits-received-label"
                    onClick={() => setCreditsCollapsed(prev => !prev)}
                    style={{ cursor: "pointer", userSelect: "none", display: "flex", alignItems: "center", gap: 6 }}
                >
                    <span style={{
                        display: "inline-block",
                        transition: "transform 0.2s ease",
                        transform: creditsCollapsed ? "rotate(-90deg)" : "rotate(0deg)",
                        fontSize: "0.7rem",
                    }}>▼</span>
                    🎓 Credits Received
                    {creditsCourses.length > 0 && (
                        <span style={{ fontSize: "0.7rem", fontWeight: 400, color: "var(--text-muted)" }}>
                            ({creditsCourses.length})
                        </span>
                    )}
                </div>
                {!creditsCollapsed && (
                    <DroppableSemester id="slot-0-Credits" year={0} semester="Credits" style={{ minHeight: "50px" }}>
                        <div className="credits-received-body">
                            {creditsCourses.length > 0 ? (
                                <div className="credits-received-list">
                                    {creditsCourses.map((a, idx) => (
                                        <DraggableCourse
                                            key={`${a.courseId}-${idx}`}
                                            id={`schedule-0-Credits-${a.courseId}-${idx}`}
                                            data={{ courseId: a.courseId, source: "schedule", fromYear: 0, fromSemester: "Credits" }}
                                        >
                                            <div className="schedule-course assigned" style={{ position: "relative" }}>
                                                {renderDegreeBar(a.courseId)}
                                                <div className="schedule-course-content">
                                                    <span>{a.courseId}</span>
                                                    <span className="course-card-actions">
                                                        {renderInfoButton(a.courseId)}
                                                        <span className="lock-icon">📗</span>
                                                    </span>
                                                </div>
                                            </div>
                                        </DraggableCourse>
                                    ))}
                                </div>
                            ) : (
                                <div className="drop-hint">
                                    Assign AP/transfer credits from My Courses, or drag here
                                </div>
                            )}
                        </div>
                    </DroppableSemester>
                )}
            </div>

            {/* Column headers */}
            <div className="year-row" style={{ minHeight: 0, gridTemplateColumns: `90px repeat(${semesters.length}, 1fr)` }}>
                <div />
                {semesters.map(sem => (
                    <div
                        key={sem}
                        style={{
                            textAlign: "center",
                            fontSize: "0.72rem",
                            fontWeight: 700,
                            textTransform: "uppercase",
                            letterSpacing: "1px",
                            color: "var(--text-muted)",
                            padding: "4px 0",
                        }}
                    >
                        {sem}
                    </div>
                ))}
            </div>

            {uniqueYears.map(year => (
                <div key={year} className="year-row fade-in" style={{ gridTemplateColumns: `90px repeat(${semesters.length}, 1fr)` }}>
                    <div className="year-label">{YEAR_NAMES[year] || `Year ${year}`}</div>
                    {semesters.map(sem => {
                        const plan = getSemesterPlan(year, sem);
                        const courses = plan?.courses || [];
                        const droppableId = `slot-${year}-${sem}`;

                        return (
                            <DroppableSemester key={sem} id={droppableId} year={year} semester={sem}>
                                <div className="semester-col-header">
                                    {(YEAR_NAMES[year] || `Year ${year}`)} {sem}
                                    {courses.length > 0 && (
                                        <span style={{ float: "right", fontWeight: 400 }}>{courses.length}</span>
                                    )}
                                </div>
                                {courses.map((courseId, idx) => renderCourseCard(courseId, year, sem, idx))}
                                {courses.length === 0 && (
                                    <div className="drop-hint">Drop courses here</div>
                                )}
                            </DroppableSemester>
                        );
                    })}
                </div>
            ))}

            {/* Degree legend */}
            {Object.keys(degreeColorMap).length > 0 && (
                <div className="degree-legend">
                    {Object.entries(degreeColorMap).map(([label, color]) => (
                        <div key={label} className="degree-legend-item">
                            <span className="degree-legend-swatch" style={{ background: color }} />
                            <span>{label}</span>
                        </div>
                    ))}
                </div>
            )}

            {/* Info popup */}
            {infoPopup && courseRequirementMap?.[infoPopup.courseId] && (
                <div
                    className="course-info-popup"
                    style={{ position: "fixed", left: infoPopup.x, top: infoPopup.y, border: "2px solid red" }}
                    onClick={e => e.stopPropagation()}
                >
                    <div className="course-info-popup-title">{infoPopup.courseId}</div>
                    {courseRequirementMap[infoPopup.courseId].map((entry, i) => (
                        <div key={i} className="course-info-popup-row">{entry}</div>
                    ))}
                </div>
            )}

            {/* Double Count Tracker Bars */}
            {doubleCountData && doubleCountData.length > 0 && (
                <div className="dc-tracker-section">
                    <div className="dc-tracker-title">🔗 Double Count Trackers</div>
                    {doubleCountData.map((dc, i) => {
                        const fulfilledCount = dc.dc_fulfilled?.filter(Boolean).length || 0;
                        const totalCount = dc.dc_fulfilled?.length || 0;
                        const allFulfilled = fulfilledCount === totalCount;
                        const color = DC_COLORS[dc.dcIndex % DC_COLORS.length];
                        return (
                            <div
                                key={i}
                                className={`dc-tracker-bar ${allFulfilled ? "dc-tracker-fulfilled" : ""}`}
                                style={{ borderLeftColor: color }}
                            >
                                <div className="dc-tracker-header">
                                    <span className="dc-tracker-label" style={{ color }}>
                                        {dc.dcLabel}
                                    </span>
                                    <span className="dc-tracker-category">
                                        {dc.category}
                                    </span>
                                    <span className="dc-tracker-progress">
                                        {fulfilledCount}/{totalCount}
                                    </span>
                                </div>
                                <div className="dc-tracker-constraints">
                                    {dc.dc_descriptions?.map((desc, j) => (
                                        <div key={j} className="dc-constraint-row">
                                            <span className="dc-constraint-status">
                                                {dc.dc_fulfilled?.[j] ? "✅" : "❌"}
                                            </span>
                                            <span className="dc-constraint-desc">
                                                {desc}
                                            </span>
                                            {dc.dc_fulfilled?.[j] && dc.dc_matched_courses?.[j]?.length > 0 && (
                                                <span className="dc-constraint-courses">
                                                    {dc.dc_matched_courses[j].map((c, k) => (
                                                        <span key={k} className="dc-course-chip" style={{ borderColor: color }}>{c}</span>
                                                    ))}
                                                </span>
                                            )}
                                        </div>
                                    ))}
                                </div>
                            </div>
                        );
                    })}
                </div>
            )}
        </div>
    );
}
