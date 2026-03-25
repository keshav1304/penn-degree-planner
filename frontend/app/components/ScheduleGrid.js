"use client";

import DraggableCourse from "./DraggableCourse";
import DroppableSemester from "./DroppableSemester";

const YEAR_NAMES = { 1: "Freshman", 2: "Sophomore", 3: "Junior", 4: "Senior" };

export default function ScheduleGrid({ scheduleData, frozenCourses, assignedCourses, onToggleFreeze, degrees }) {
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

    const years = [1, 2, 3, 4];
    const semesters = ["Fall", "Spring", "Summer"];

    const getSemesterPlan = (year, semester) => {
        return scheduleData.schedule.find(
            s => s.year === year && s.semester === semester
        );
    };

    const isFrozen = (courseId) => frozenCourses.some(f => f.courseId === courseId);
    const isAssigned = (courseId) => assignedCourses?.some(a => a.courseId === courseId);

    // Courses assigned to "Credits Received" (year 0)
    const creditsCourses = assignedCourses?.filter(a => a.year === 0) || [];

    const renderCourseCard = (courseId, year, sem) => {
        const frozen = isFrozen(courseId);
        const assigned = isAssigned(courseId);
        let className = "schedule-course";
        if (assigned) className += " assigned";
        else if (frozen) className += " frozen";

        return (
            <DraggableCourse
                key={courseId}
                id={`schedule-${year}-${sem}-${courseId}`}
                data={{ courseId, source: "schedule", fromYear: year, fromSemester: sem }}
            >
                <div
                    className={className}
                    onClick={() => { if (!assigned) onToggleFreeze(courseId, year, sem); }}
                    title={
                        assigned ? "Placed from My Courses — drag to move"
                            : frozen ? "Click to unfreeze — drag to move"
                                : "Click to freeze — drag to move"
                    }
                >
                    <span>{courseId}</span>
                    <span className="lock-icon">
                        {assigned ? "📗" : frozen ? "🔒" : "📌"}
                    </span>
                </div>
            </DraggableCourse>
        );
    };

    return (
        <div className="schedule-container">
            {/* Credits Received section */}
            <div className="credits-received-row fade-in">
                <div className="credits-received-label">🎓 Credits Received</div>
                <DroppableSemester id="slot-0-Credits" year={0} semester="Credits" style={{ minHeight: "50px" }}>
                    <div className="credits-received-body">
                        {creditsCourses.length > 0 ? (
                            <div className="credits-received-list">
                                {creditsCourses.map(a => (
                                    <DraggableCourse
                                        key={a.courseId}
                                        id={`schedule-0-Credits-${a.courseId}`}
                                        data={{ courseId: a.courseId, source: "schedule", fromYear: 0, fromSemester: "Credits" }}
                                    >
                                        <div className="schedule-course assigned">
                                            <span>{a.courseId}</span>
                                            <span className="lock-icon">📗</span>
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
            </div>

            {/* Column headers */}
            <div className="year-row" style={{ minHeight: 0 }}>
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

            {
                years.map(year => (
                    <div key={year} className="year-row fade-in">
                        <div className="year-label">{YEAR_NAMES[year]}</div>
                        {semesters.map(sem => {
                            const plan = getSemesterPlan(year, sem);
                            const courses = plan?.courses || [];
                            const droppableId = `slot-${year}-${sem}`;

                            return (
                                <DroppableSemester key={sem} id={droppableId} year={year} semester={sem}>
                                    <div className="semester-col-header">
                                        {YEAR_NAMES[year]} {sem}
                                        {courses.length > 0 && (
                                            <span style={{ float: "right", fontWeight: 400 }}>{courses.length}</span>
                                        )}
                                    </div>
                                    {courses.map(courseId => renderCourseCard(courseId, year, sem))}
                                    {courses.length === 0 && (
                                        <div className="drop-hint">Drop courses here</div>
                                    )}
                                </DroppableSemester>
                            );
                        })}
                    </div>
                ))
            }
        </div >
    );
}
