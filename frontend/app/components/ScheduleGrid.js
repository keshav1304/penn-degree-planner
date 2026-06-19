"use client";

import { useState } from "react";
import DraggableCourse from "./DraggableCourse";
import DroppableSemester from "./DroppableSemester";
import { isValidCourseCode, isRequirementSlotId, isSchedulableRequirementSlotId, isOverlapScheduleGroupId } from "@/lib/courseUtils";
import { resolveSemesterCuLimit } from "@/lib/semesterOptions";
import { buildDegreeOrder, sortCourseCodesByDegree } from "@/lib/courseOrdering";
import { formatDegreeApiLabel } from "@/lib/degreeDisplay";
import { buildDegreeColorMap, getDegreeColorForIndex } from "@/lib/degreeColors";
import { formatOverlapMemberLabel } from "@/lib/requirementText";

const YEAR_NAMES = {};

export default function ScheduleGrid({
    scheduleData, requirementSlotLabels = {}, frozenCourses, assignedCourses,
    onToggleFreeze, onMarkTaken, onUnmarkTaken, degrees,
    courseDegreesMap, courseRequirementLinks,
    crossDegreeViolationsByCourse = {},
    onNavigateToRequirement, allowSummer,
    concentrationData, courseConcentrationMap,
    allCourses,
    degreeCatalog = [],
    semesterCuLimits, onSemesterCuLimitChange,
}) {
    const [creditsCollapsed, setCreditsCollapsed] = useState(false);
    const [reqNavCycle, setReqNavCycle] = useState({});

    // Build CU lookup from allCourses
    const cuMap = {};
    (allCourses || []).forEach(c => { cuMap[c.course_code] = c.cu; });
    const getCu = (courseId) => cuMap[courseId] ?? 1.0;

    if (!degrees || degrees.length === 0) {
        return (
            <div className="req-empty-state">
                <div className="req-empty-icon">🏫</div>
                <div className="req-empty-text">Add a degree above to generate your schedule</div>
            </div>
        );
    }

    if (!scheduleData || !scheduleData.schedule) {
        return (
            <div className="req-empty-state panel-loading-state">
                <div className="loading-spinner" />
                <div className="req-empty-text">Generating schedule…</div>
            </div>
        );
    }

    // Derive years from API schedule and user-pinned placements
    const pinnedYears = [
        ...frozenCourses.map((f) => f.year),
        ...(assignedCourses || []).filter((a) => a.year > 0).map((a) => a.year),
    ];
    const uniqueYears = [...new Set([
        ...scheduleData.schedule.map((s) => s.year),
        ...pinnedYears,
    ])].filter((y) => y > 0).sort((a, b) => a - b);
    const semOrder = ["Fall", "Spring", "Summer"];

    const getSemesterPlan = (year, semester) => {
        return scheduleData.schedule.find(
            s => s.year === year && s.semester === semester
        );
    };

    // User-pinned items win over API auto-placement (including Credits Received)
    const pinnedIds = new Set([
        ...frozenCourses.map((f) => f.courseId),
        ...(assignedCourses || []).map((a) => a.courseId),
    ]);

    const getDisplayCourses = (year, semester) => {
        const plan = getSemesterPlan(year, semester);
        const apiCourses = (plan?.courses || []).filter((id) => !pinnedIds.has(id) && isValidCourseCode(id));
        const pinnedHere = [
            ...frozenCourses.filter((f) => f.year === year && f.semester === semester).map((f) => f.courseId),
            ...(assignedCourses || [])
                .filter((a) => a.year === year && a.semester === semester)
                .map((a) => a.courseId),
        ];
        return [...new Set([...pinnedHere, ...apiCourses])].filter(isValidCourseCode);
    };

    const overlapScheduleGroups = scheduleData?.overlap_schedule_groups ?? [];
    const overlapGroupById = Object.fromEntries(
        overlapScheduleGroups.map((g) => [g.group_id, g]),
    );
    const overlapMemberSlotIds = new Set(
        overlapScheduleGroups.flatMap((g) =>
            g.members.map((m) => m.schedule_slot_id).filter(isRequirementSlotId),
        ),
    );

    const getDisplayOverlapGroups = (year, semester) => {
        const plan = getSemesterPlan(year, semester);
        const apiGroups = (plan?.requirement_slots || []).filter(
            (id) => isOverlapScheduleGroupId(id) && !pinnedIds.has(id),
        );
        const pinnedHere = frozenCourses
            .filter(
                (f) =>
                    f.year === year
                    && f.semester === semester
                    && isOverlapScheduleGroupId(f.courseId),
            )
            .map((f) => f.courseId);
        return [...new Set([...pinnedHere, ...apiGroups])];
    };

    // Requirement slots still open (unfulfilled) — hide once a frozen/taken course satisfies them
    const openRequirementSlotIds = new Set();
    scheduleData?.degree_results?.forEach((result) => {
        result.suggested_for_unfulfilled?.forEach((mapped) => {
            mapped.course_ids?.forEach((id) => {
                if (isSchedulableRequirementSlotId(id)) openRequirementSlotIds.add(id);
            });
        });
    });

    const getDisplayRequirementSlots = (year, semester) => {
        const plan = getSemesterPlan(year, semester);
        const apiSlots = (plan?.requirement_slots || []).filter(
            (id) => !pinnedIds.has(id) && openRequirementSlotIds.has(id)
        );
        const pinnedHere = [
            ...frozenCourses
                .filter(
                    (f) =>
                        f.year === year
                        && f.semester === semester
                        && isSchedulableRequirementSlotId(f.courseId)
                        && openRequirementSlotIds.has(f.courseId)
                )
                .map((f) => f.courseId),
        ];
        return [...new Set([...pinnedHere, ...apiSlots])]
            .filter(isSchedulableRequirementSlotId)
            .filter((id) => !isOverlapScheduleGroupId(id))
            .filter((id) => !overlapMemberSlotIds.has(id));
    };

    const semesterHasContent = (year, semester) => (
        getDisplayCourses(year, semester).length > 0
        || getDisplayOverlapGroups(year, semester).length > 0
        || getDisplayRequirementSlots(year, semester).length > 0
    );

    // Years 1–4 always show as drop targets; year 5+ only when a term has content
    const visibleYears = uniqueYears.filter((year) => {
        if (year <= 4) return true;
        return semOrder.some((sem) => {
            if (sem === "Summer" && !allowSummer) return false;
            return semesterHasContent(year, sem);
        });
    });

    const visibleSemesters = semOrder.filter((s) => s !== "Summer" || allowSummer);

    const getSlotLabel = (slotId) => requirementSlotLabels[slotId] || "Open requirement";

    const isFrozen = (courseId) => frozenCourses.some(f => f.courseId === courseId);
    const isAssigned = (courseId) => assignedCourses?.some(a => a.courseId === courseId);

    const degreeColorMap = buildDegreeColorMap(scheduleData);
    const degreeDisplayLabels = {};
    const degreeOrder = buildDegreeOrder(scheduleData);
    if (scheduleData?.degree_results) {
        scheduleData.degree_results.forEach((result) => {
            const key = `${result.school}-${result.major}`;
            degreeDisplayLabels[key] = formatDegreeApiLabel(
                result.school,
                result.major,
                degreeCatalog,
            );
        });
    }

    const sortSemesterCourses = (courseIds) =>
        sortCourseCodesByDegree(courseIds, degreeOrder, courseDegreesMap);

    // Courses assigned to "Credits Received" (year 0)
    const creditsCourses = assignedCourses?.filter(a => a.year === 0) || [];
    const sortedCreditsCourses = (() => {
        const ids = sortSemesterCourses(creditsCourses.map((a) => a.courseId));
        const byId = new Map(creditsCourses.map((a) => [a.courseId, a]));
        return ids.map((id) => byId.get(id)).filter(Boolean);
    })();

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

    const renderReqNavButton = (courseId) => {
        const links = courseRequirementLinks?.[courseId];
        if (!links?.length) return null;
        const cycleIdx = reqNavCycle[courseId] ?? 0;
        const link = links[cycleIdx % links.length];
        const title = links.length > 1
            ? `View in requirements (${(cycleIdx % links.length) + 1}/${links.length}): ${link.label}. Click again for the other degree.`
            : `View in requirements: ${link.label}`;
        return (
            <button
                type="button"
                className="course-req-nav-btn"
                title={title}
                aria-label={title}
                onClick={(e) => {
                    e.preventDefault();
                    e.stopPropagation();
                    onNavigateToRequirement?.({
                        degreeIndex: link.degreeIndex,
                        instanceId: link.instanceId,
                        category: link.category,
                    });
                    if (links.length > 1) {
                        setReqNavCycle((prev) => ({
                            ...prev,
                            [courseId]: (cycleIdx + 1) % links.length,
                        }));
                    }
                }}
            >
                <svg
                    className="course-req-nav-arrow"
                    viewBox="0 0 16 16"
                    width="12"
                    height="12"
                    aria-hidden="true"
                >
                    <path
                        d="M6 4l4 4-4 4"
                        fill="none"
                        stroke="currentColor"
                        strokeWidth="2"
                        strokeLinecap="round"
                        strokeLinejoin="round"
                    />
                </svg>
            </button>
        );
    };

    const renderConcBadges = (courseId) => {
        const concEntries = courseConcentrationMap?.[courseId];
        if (!concEntries?.length) return null;
        return (
            <span className="dc-badges">
                {concEntries.map((entry, i) => (
                    <span
                        key={`${entry.name}-${entry.degreeLabel}-${i}`}
                        className="dc-badge conc-badge"
                        style={{ borderColor: degreeColorMap[entry.degreeLabel] || "#888" }}
                        title={`Concentration: ${entry.name} (${degreeDisplayLabels[entry.degreeLabel] || entry.degreeLabel})`}
                    >
                        {entry.name}
                    </span>
                ))}
            </span>
        );
    };

    const renderOverlapGroupCard = (groupId, year, sem, idx) => {
        const group = overlapGroupById[groupId];
        const frozen = isFrozen(groupId);
        let className = "schedule-course schedule-requirement schedule-requirement-overlap";
        if (frozen) className += " frozen";

        const handleClick = () => {
            onToggleFreeze(groupId, year, sem);
        };

        const members = group?.members?.length
            ? group.members
            : [{ schedule_slot_id: groupId, label: getSlotLabel(groupId), school: "", major: "", degree_index: 0 }];

        return (
            <DraggableCourse
                key={`${groupId}-${idx}`}
                id={`schedule-${year}-${sem}-${groupId}-${idx}`}
                data={{ courseId: groupId, source: "schedule", fromYear: year, fromSemester: sem }}
            >
                <div className={className} style={{ position: "relative" }}>
                    <div className="degree-bar-container">
                        {members.map((m, i) => {
                            const degKey = m.school && m.major ? `${m.school}-${m.major}` : null;
                            return (
                                <div
                                    key={`${groupId}-bar-${i}`}
                                    className="degree-bar-stripe"
                                    style={{ background: degKey ? (degreeColorMap[degKey] || "#888") : "#888" }}
                                    title={degKey || m.label}
                                />
                            );
                        })}
                    </div>
                    <div
                        className="schedule-course-content"
                        onClick={handleClick}
                        title={
                            group?.explanation
                                ? `${group.explanation}\n\n${frozen ? "Click to unfreeze" : "Click to freeze"}`
                                : frozen ? "Click to unfreeze (white)" : "Click to freeze in this semester (orange)"
                        }
                    >
                        <div className="schedule-overlap-inline">
                            {members.map((m, i) => {
                                const degKey = m.school && m.major ? `${m.school}-${m.major}` : null;
                                const color = degKey ? (degreeColorMap[degKey] || "#888") : "#888";
                                const slotText = m.schedule_slot_id
                                    ? getSlotLabel(m.schedule_slot_id)
                                    : "";
                                const text = formatOverlapMemberLabel(slotText, m.label);
                                return (
                                    <div key={i} className="schedule-overlap-line">
                                        <span
                                            className="schedule-overlap-req-label"
                                            style={{ borderBottomColor: color }}
                                        >
                                            {text}
                                        </span>
                                        {i < members.length - 1 && (
                                            <span className="schedule-overlap-slash"> /</span>
                                        )}
                                    </div>
                                );
                            })}
                        </div>
                        <span className="course-card-actions">
                            {renderReqNavButton(groupId)}
                            <span className="course-cu-label">1.0 CU</span>
                        </span>
                    </div>
                </div>
            </DraggableCourse>
        );
    };

    const renderRequirementSlotCard = (slotId, year, sem, idx) => {
        const frozen = isFrozen(slotId);
        let className = "schedule-course schedule-requirement";
        if (frozen) className += " frozen";
        const slotLabel = getSlotLabel(slotId).split(/\n↳/)[0].trim();

        const handleClick = () => {
            onToggleFreeze(slotId, year, sem);
        };

        return (
            <DraggableCourse
                key={`${slotId}-${idx}`}
                id={`schedule-${year}-${sem}-${slotId}-${idx}`}
                data={{ courseId: slotId, source: "schedule", fromYear: year, fromSemester: sem }}
            >
                <div className={className} style={{ position: "relative" }}>
                    {renderDegreeBar(slotId)}
                    <div
                        className="schedule-course-content"
                        onClick={handleClick}
                        title={frozen ? "Click to unfreeze (white)" : "Click to freeze in this semester (orange)"}
                    >
                        <span className="schedule-requirement-label">{slotLabel}</span>
                        <span className="course-card-actions">
                            {renderReqNavButton(slotId)}
                            <span className="course-cu-label">1.0 CU</span>
                        </span>
                    </div>
                </div>
            </DraggableCourse>
        );
    };

    const renderCourseCard = (courseId, year, sem, idx) => {
        const frozen = isFrozen(courseId);
        const assigned = isAssigned(courseId);
        const violation = crossDegreeViolationsByCourse?.[courseId];
        let className = "schedule-course";
        if (assigned) className += " assigned";
        else if (frozen) className += " frozen";
        if (violation) className += " cross-degree-violation";

        const handleClick = () => {
            if (!isValidCourseCode(courseId)) return;
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
                            violation
                                ? violation
                                : assigned ? "Click to freeze (orange)"
                                : frozen ? "Click to un-mark (default)"
                                    : "Click to mark taken (green)"
                        }
                    >
                        <span>{courseId}</span>
                        <span className="course-card-actions">
                            {renderConcBadges(courseId)}
                            {renderReqNavButton(courseId)}
                            <span className="course-cu-label">{getCu(courseId).toFixed(1)} CU</span>
                        </span>
                    </div>
                </div>
            </DraggableCourse>
        );
    };

    return (
        <div className="schedule-container">
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
                                    {sortedCreditsCourses.map((a, idx) => (
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
                                        {renderConcBadges(a.courseId)}
                                        {renderReqNavButton(a.courseId)}
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
            <div className="year-row" style={{ minHeight: 0, gridTemplateColumns: `48px repeat(${visibleSemesters.length}, 1fr)` }}>
                <div />
                {visibleSemesters.map(sem => (
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

            {visibleYears.map(year => (
                <div key={year} className="year-row fade-in" style={{ gridTemplateColumns: `48px repeat(${visibleSemesters.length}, 1fr)` }}>
                    <div className="year-label">{YEAR_NAMES[year] || `Year ${year}`}</div>
                    {visibleSemesters.map(sem => {
                        const plan = getSemesterPlan(year, sem);
                        const courses = sortSemesterCourses(getDisplayCourses(year, sem));
                        const overlapGroups = getDisplayOverlapGroups(year, sem);
                        const requirementSlots = getDisplayRequirementSlots(year, sem);
                        const itemCount = courses.length + overlapGroups.length + requirementSlots.length;
                        const droppableId = `slot-${year}-${sem}`;

                        return (
                            <DroppableSemester key={sem} id={droppableId} year={year} semester={sem}>
                                <div className="semester-col-header">
                                    {(YEAR_NAMES[year] || `Year ${year}`)} {sem}
                                    {itemCount > 0 && (
                                        <span style={{ float: "right", fontWeight: 400 }}>{itemCount}</span>
                                    )}
                                </div>
                                {courses.map((courseId, idx) => renderCourseCard(courseId, year, sem, idx))}
                                {overlapGroups.map((groupId, idx) => renderOverlapGroupCard(groupId, year, sem, idx))}
                                {requirementSlots.map((slotId, idx) => renderRequirementSlotCard(slotId, year, sem, idx))}
                                {itemCount === 0 && (
                                    <div className="drop-hint">Drop courses here</div>
                                )}
                                {(() => {
                                    const semKey = `${year}-${sem}`;
                                    const slotCu = (id) => (
                                        isOverlapScheduleGroupId(id) || isRequirementSlotId(id)
                                            ? 1.0
                                            : getCu(id)
                                    );
                                    const actualCu =
                                        courses.reduce((s, c) => s + getCu(c), 0)
                                        + overlapGroups.reduce((s, id) => s + slotCu(id), 0)
                                        + requirementSlots.reduce((s, id) => s + slotCu(id), 0);
                                    const limitValue = resolveSemesterCuLimit(sem, year, degrees, semesterCuLimits);
                                    const overLimit = actualCu > limitValue + 0.001;
                                    return (
                                        <div className={`semester-cu-total${overLimit ? " semester-cu-over" : ""}`}>
                                            <span>{actualCu.toFixed(1)} /</span>
                                            <input
                                                type="number"
                                                min="1"
                                                max="10"
                                                step="0.5"
                                                value={limitValue}
                                                onClick={e => e.stopPropagation()}
                                                onChange={e => {
                                                    const val = parseFloat(e.target.value);
                                                    if (!isNaN(val) && val > 0) {
                                                        onSemesterCuLimitChange(semKey, val);
                                                    }
                                                }}
                                                className="semester-cu-input"
                                            />
                                            <span>CU</span>
                                        </div>
                                    );
                                })()}
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
                            <span>{degreeDisplayLabels[label] || label}</span>
                        </div>
                    ))}
                </div>
            )}

            {/* Concentration Tracker Bars */}
            {concentrationData && concentrationData.length > 0 && (
                <div className="dc-tracker-section">
                    <div className="dc-tracker-title">🎯 Concentration Tracker</div>
                    {concentrationData.map((ci, i) => {
                        const fulfilledCount = ci.requirements_fulfilled || 0;
                        const totalCount = ci.requirements_total || 0;
                        const allFulfilled = fulfilledCount === totalCount && totalCount > 0;
                        const color = degreeColorMap[ci.degreeLabel] || getDegreeColorForIndex(i);
                        return (
                            <div
                                key={i}
                                className={`dc-tracker-bar ${allFulfilled ? "dc-tracker-fulfilled" : ""}`}
                                style={{ borderLeftColor: color }}
                            >
                                <div className="dc-tracker-header">
                                    <span className="dc-tracker-label" style={{ color }}>
                                        {ci.name}
                                    </span>
                                    <span className="dc-tracker-category">
                                        {degreeDisplayLabels[ci.degreeLabel] || ci.degreeLabel}
                                    </span>
                                    <span className="dc-tracker-progress">
                                        {fulfilledCount}/{totalCount}
                                    </span>
                                </div>
                                <div className="dc-tracker-constraints">
                                    {ci.requirement_descriptions?.map((desc, j) => (
                                        <div key={j} className="dc-constraint-row">
                                            <span className="dc-constraint-status">
                                                {ci.requirement_fulfilled?.[j] ? "✅" : "❌"}
                                            </span>
                                            <span className="dc-constraint-desc">
                                                {desc}
                                            </span>
                                            {ci.requirement_fulfilled?.[j] && ci.matched_courses?.[j]?.length > 0 && (
                                                <span className="dc-constraint-courses">
                                                    {ci.matched_courses[j].map((c, k) => (
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
