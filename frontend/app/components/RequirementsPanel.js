"use client";

import { useState, useEffect } from "react";
import {
    filterValidCourseCodes,
    filterValidPlacements,
    filterFrozenPlacements,
    isValidCourseCode,
} from "@/lib/courseUtils";
import {
    childMatchesAnyOfFulfillment,
    getAnyOfPossibilities,
    getRequirementInstanceId,
    getRequirementStem,
    isExpandableAnyOf,
    parseRequirement,
} from "@/lib/requirementText";
import { reqRowDomId, attributeFulfillmentMap } from "@/lib/requirementNav";

export default function RequirementsPanel({
    scheduleData,
    degrees,
    frozenCourses = [],
    assignedCourses = [],
    navTarget = null,
    onNavTargetConsumed,
}) {
    const [activeTab, setActiveTab] = useState(0);
    const [collapsedGroups, setCollapsedGroups] = useState({});
    const [flashRowId, setFlashRowId] = useState(null);

    useEffect(() => {
        if (!navTarget) return;
        const { degreeIndex, instanceId, category } = navTarget;
        if (degreeIndex != null) setActiveTab(degreeIndex);
        if (category) setCollapsedGroups((prev) => ({ ...prev, [category]: false }));
        const rowId = reqRowDomId(degreeIndex ?? 0, instanceId);
        const timer = window.setTimeout(() => {
            document.getElementById(rowId)?.scrollIntoView({ behavior: "smooth", block: "nearest" });
            setFlashRowId(rowId);
            window.setTimeout(() => setFlashRowId(null), 2200);
            onNavTargetConsumed?.();
        }, 80);
        return () => window.clearTimeout(timer);
    }, [navTarget, onNavTargetConsumed]);

    if (!degrees || degrees.length === 0) {
        return (
            <div className="req-empty-state">
                <div className="req-empty-icon">📋</div>
                <div className="req-empty-text">Add degrees to see requirement fulfillment</div>
            </div>
        );
    }
    if (!scheduleData || !scheduleData.degree_results) {
        return <div className="req-empty-state"><div className="req-empty-text">Loading requirements…</div></div>;
    }

    const results = scheduleData.degree_results;
    const tabIndex = Math.min(activeTab, results.length - 1);
    const current = results[tabIndex];
    if (!current) return null;

    const allReqs = [];
    (current.fulfilled_requirements || []).forEach((mapped) => {
        allReqs.push({
            category: normalizeCategory(getCategory(mapped.requirement)),
            fulfilled: true,
            fulfilledCourses: mapped.course_ids || [],
            requirement: mapped.requirement,
            instanceId: getRequirementInstanceId(mapped),
            attributeFulfillment: attributeFulfillmentMap(mapped),
        });
    });
    (current.unfulfilled_requirements || []).forEach((mapped, rowIdx) => {
        const req = mapped?.requirement ?? mapped;
        const cat = normalizeCategory(getCategory(req));
        const id = getRequirementInstanceId(mapped) ?? `u-${rowIdx}`;
        allReqs.push({
            category: cat,
            fulfilled: false,
            fulfilledCourses: [],
            requirement: req,
            instanceId: id,
            attributeFulfillment: attributeFulfillmentMap(mapped),
        });
    });

    const categoryMap = {};
    allReqs.forEach((item) => {
        const cat = normalizeCategory(item.category);
        if (!categoryMap[cat]) categoryMap[cat] = [];
        categoryMap[cat].push(item);
    });

    const categoryOrder = (current.category_order || []).map(normalizeCategory);
    const orderedCategories = [...categoryOrder];
    Object.keys(categoryMap).forEach((c) => { if (!orderedCategories.includes(c)) orderedCategories.push(c); });

    const assignedIds = new Set(filterValidPlacements(assignedCourses).map((a) => a.courseId));
    const frozenIds = new Set(filterFrozenPlacements(frozenCourses).map((f) => f.courseId));
    const scheduleCtx = { assignedIds, frozenIds };

    const totalCount = allReqs.length;
    const fulfilledCount = allReqs.filter((r) => r.fulfilled && itemTone(r, frozenIds) === "fulfilled").length;
    const plannedCount = allReqs.filter((r) => r.fulfilled && itemTone(r, frozenIds) === "frozen").length;
    const remainingCount = totalCount - fulfilledCount - plannedCount;
    const fulfilledPct = totalCount > 0 ? (fulfilledCount / totalCount) * 100 : 0;
    const plannedPct = totalCount > 0 ? (plannedCount / totalCount) * 100 : 0;
    const pct = totalCount > 0 ? Math.round(((fulfilledCount + plannedCount) / totalCount) * 100) : 0;

    return (
        <div className="req-panel">
            {results.length > 1 && (
                <div className="req-degree-tabs" role="tablist" aria-label="Degree requirements">
                    {results.map((result, i) => {
                        const { major, school } = degreeTabLabel(degrees[i], result);
                        const isActive = tabIndex === i;
                        return (
                            <button
                                key={i}
                                type="button"
                                role="tab"
                                aria-selected={isActive}
                                className={`req-degree-tab ${isActive ? "active" : ""}`}
                                onClick={() => setActiveTab(i)}
                                title={school ? `${major} (${school})` : major}
                            >
                                <span className="req-degree-tab-major">{major}</span>
                                {school && <span className="req-degree-tab-school">{school}</span>}
                            </button>
                        );
                    })}
                </div>
            )}

            {current.error && <div className="req-error-banner">⚠️ {current.error}</div>}

            {!current.error && totalCount > 0 && (
                <div className="req-summary">
                    <div className="req-summary-stats">
                        <span className="req-stat req-stat-fulfilled">
                            <span className="req-stat-dot req-stat-dot-fulfilled" />
                            {fulfilledCount}
                        </span>
                        <span className="req-stat req-stat-planned">
                            <span className="req-stat-dot req-stat-dot-planned" />
                            {plannedCount}
                        </span>
                        <span className="req-stat req-stat-remaining">
                            <span className="req-stat-dot req-stat-dot-remaining" />
                            {remainingCount}
                        </span>
                        <span className="req-stat-pct">{pct}%</span>
                    </div>
                    <div className="req-progress-track">
                        {fulfilledPct > 0 && (
                            <div
                                className="req-progress-fill"
                                style={{ width: `${fulfilledPct}%`, background: "linear-gradient(90deg, var(--success), var(--accent-teal))" }}
                            />
                        )}
                        {plannedPct > 0 && (
                            <div
                                className="req-progress-fill"
                                style={{ width: `${plannedPct}%`, background: "var(--accent-amber)" }}
                            />
                        )}
                    </div>
                </div>
            )}

            <div className="req-groups">
                {orderedCategories.map((cat) => {
                    const items = categoryMap[cat];
                    if (!items?.length) return null;
                    const done = items.filter((r) => r.fulfilled).length;
                    const catTone = groupTone(items, frozenIds);
                    const isCollapsed = collapsedGroups[cat] ?? true;
                    const groupClass = catTone === "fulfilled" ? "req-group req-group-done" : "req-group";

                    return (
                        <div key={cat} className={groupClass}>
                            <div
                                className="req-group-header"
                                onClick={() => setCollapsedGroups((p) => ({ ...p, [cat]: !(p[cat] ?? true) }))}
                            >
                                <span className={`req-group-badge ${done === items.length ? "badge-done" : "badge-pending"}`}>
                                    {done === items.length ? "✓" : "·"}
                                </span>
                                <span className="req-group-name" title={cat}>{cat}</span>
                                <span className={`req-group-pill ${done === items.length ? "pill-done" : "pill-pending"}`}>
                                    {done}/{items.length}
                                </span>
                                <span className="req-group-chevron">{isCollapsed ? "▶" : "▾"}</span>
                            </div>
                            {!isCollapsed && (
                                <div className="req-group-body">
                                    <table className="req-fulfillment-table">
                                        <thead>
                                            <tr>
                                                <th className="req-col-status" scope="col" />
                                                <th className="req-col-req" scope="col">Req</th>
                                                <th className="req-col-courses" scope="col">Courses</th>
                                            </tr>
                                        </thead>
                                        <tbody>
                                            {items.map((item, rowIdx) => {
                                                if (isExpandableAnyOf(item.requirement)) {
                                                    return renderAnyOfRows(
                                                        item,
                                                        item.instanceId ?? String(rowIdx),
                                                        scheduleCtx,
                                                        tabIndex,
                                                        flashRowId
                                                    );
                                                }
                                                return renderItemRow(
                                                    item,
                                                    item.instanceId ?? String(rowIdx),
                                                    scheduleCtx,
                                                    tabIndex,
                                                    flashRowId
                                                );
                                            })}
                                        </tbody>
                                    </table>
                                </div>
                            )}
                        </div>
                    );
                })}
                {totalCount === 0 && !current.error && (
                    <div className="req-empty-state"><div className="req-empty-text">No requirement data available</div></div>
                )}
            </div>
        </div>
    );
}

function degreeTabLabel(degree, result) {
    const major = degree?.displayMajor || result?.major || "Degree";
    const school = degree?.schoolCode || result?.school || "";
    return { major, school };
}

function collectFulfillingCourses(item) {
    const courses = filterValidCourseCodes(item.fulfilledCourses || []);
    item.attributeFulfillment?.forEach((ids) => {
        ids.forEach((c) => { if (isValidCourseCode(c)) courses.push(c); });
    });
    return [...new Set(courses)];
}

function itemHasFrozenCourse(courses, frozenIds) {
    return courses.some((c) => frozenIds.has(c));
}

function itemTone(item, frozenIds) {
    if (!item.fulfilled) return "open";
    if (itemHasFrozenCourse(collectFulfillingCourses(item), frozenIds)) return "frozen";
    return "fulfilled";
}

function groupTone(items, frozenIds) {
    const done = items.filter((r) => r.fulfilled).length;
    if (done !== items.length) return "incomplete";
    if (items.some((item) => itemTone(item, frozenIds) === "frozen")) return "frozen";
    return "fulfilled";
}

function badgeKindFor(courseId, { assignedIds, frozenIds, fulfillingSet }) {
    if (!fulfillingSet?.has(courseId)) return "open";
    if (frozenIds.has(courseId)) return "frozen";
    if (assignedIds.has(courseId)) return "fulfilled";
    return "fulfilled";
}

function chipClass(kind) {
    if (kind === "fulfilled") return "req-chip chip-fulfilled";
    if (kind === "frozen") return "req-chip chip-frozen";
    return "req-chip chip-default";
}

function buildRowContent(item) {
    const { type, data } = parseRequirement(item.requirement);
    const stem = getRequirementStem(item.requirement);
    const fulfilling = collectFulfillingCourses(item);
    const fulfillingSet = new Set(fulfilling);

    if (type === "SingleCourse") {
        const possibilities = (data.possibilities || []).filter(Boolean);
        if (possibilities.length <= 1) {
            const ids = possibilities.length ? possibilities : fulfilling;
            return { stem: null, badges: ids.map((id) => ({ kind: "course", id })), fulfillingSet };
        }
        return { stem, badges: possibilities.map((id) => ({ kind: "course", id })), fulfillingSet };
    }

    if (type === "CourseGroup") {
        const ids = fulfilling.length ? fulfilling : (data.possibilities || []).filter(Boolean);
        return { stem, badges: ids.map((id) => ({ kind: "course", id })), fulfillingSet };
    }

    if (type === "Restriction" && data.attr?.length > 0) {
        return {
            stem,
            badges: data.attr.map((code) => ({
                kind: "attr",
                code,
                courses: (item.attributeFulfillment?.get(code) || []).filter(isValidCourseCode),
            })),
            fulfillingSet,
        };
    }

    if (type === "Restriction") {
        return { stem, badges: fulfilling.map((id) => ({ kind: "course", id })), fulfillingSet };
    }

    return { stem, badges: fulfilling.map((id) => ({ kind: "course", id })), fulfillingSet };
}

function renderBadges(item, scheduleCtx) {
    const { badges, fulfillingSet } = buildRowContent(item);
    const chipCtx = { ...scheduleCtx, fulfillingSet };

    if (!badges.length) return <span className="req-chips"><span className="req-chip chip-default">—</span></span>;

    return (
        <span className="req-chips">
            {badges.map((badge, i) => {
                if (badge.kind === "attr") {
                    const fulfillingForAttr = badge.courses.filter((c) => fulfillingSet.has(c));
                    const hasCourse = fulfillingForAttr.length > 0;
                    const label = hasCourse
                        ? `[${badge.code}] ${fulfillingForAttr.join(", ")}`
                        : `[${badge.code}]`;
                    const kind = hasCourse ? badgeKindFor(fulfillingForAttr[0], chipCtx) : "open";
                    return <span key={i} className={chipClass(kind)}>{label}</span>;
                }
                return (
                    <span key={i} className={chipClass(badgeKindFor(badge.id, chipCtx))}>
                        {badge.id}
                    </span>
                );
            })}
        </span>
    );
}

function renderStemCell(item) {
    const { stem } = buildRowContent(item);
    if (!stem) return null;
    return <span className="req-stem-text">{stem}</span>;
}

function makeAnyOfChildItem(parent, childReq, childIdx) {
    const matched = childMatchesAnyOfFulfillment(childReq, parent);
    const courses = matched ? collectFulfillingCourses(parent) : [];
    let attrFulfillment = parent.attributeFulfillment;
    if (matched && attrFulfillment) {
        const { type, data } = parseRequirement(childReq);
        if (type === "Restriction" && data.attr?.length) {
            const filtered = new Map();
            data.attr.forEach((code) => {
                const ids = attrFulfillment.get(code);
                if (ids?.length) filtered.set(code, ids);
            });
            attrFulfillment = filtered.size > 0 ? filtered : attrFulfillment;
        }
    }
    return {
        category: parent.category,
        fulfilled: matched,
        fulfilledCourses: courses,
        requirement: childReq,
        instanceId: `${parent.instanceId}::${childIdx}`,
        attributeFulfillment: attrFulfillment,
    };
}

function renderAnyOfRows(parentItem, idx, scheduleCtx, degreeIndex, flashRowId) {
    const possibilities = getAnyOfPossibilities(parentItem.requirement);
    const rows = [];

    rows.push(
        <tr key={`${idx}-intro`} className="req-anyof-intro">
            <td colSpan={3}>Choose one:</td>
        </tr>
    );

    possibilities.forEach((childReq, childIdx) => {
        const childItem = makeAnyOfChildItem(parentItem, childReq, childIdx);
        const childTone = childItem.fulfilled ? itemTone(childItem, scheduleCtx.frozenIds) : "open";
        const childRowId = reqRowDomId(degreeIndex, childItem.instanceId);
        const statusIcon = childItem.fulfilled ? "✓" : "○";

        rows.push(
            <tr
                key={childItem.instanceId}
                id={childRowId}
                className={`req-table-row req-table-row--${childTone} req-anyof-child ${flashRowId === childRowId ? "req-row-flash" : ""}`}
            >
                <td className="req-col-status">
                    <span className="req-table-status">{statusIcon}</span>
                </td>
                <td className="req-col-req">{renderStemCell(childItem)}</td>
                <td className="req-cell-courses">{renderBadges(childItem, scheduleCtx)}</td>
            </tr>
        );
    });

    return rows;
}

function renderItemRow(item, idx, scheduleCtx, degreeIndex, flashRowId) {
    const rowTone = itemTone(item, scheduleCtx.frozenIds);
    const rowDomId = reqRowDomId(degreeIndex, idx);
    const statusIcon = item.fulfilled ? "✓" : "○";
    const { stem } = buildRowContent(item);

    return (
        <tr
            key={String(idx)}
            id={rowDomId}
            className={`req-table-row req-table-row--${rowTone} ${flashRowId === rowDomId ? "req-row-flash" : ""}`}
        >
            <td className="req-col-status">
                <span className="req-table-status">{statusIcon}</span>
            </td>
            <td className="req-col-req">
                {stem ? renderStemCell(item) : null}
            </td>
            <td className="req-cell-courses">{renderBadges(item, scheduleCtx)}</td>
        </tr>
    );
}

function normalizeCategory(cat) {
    if (!cat || typeof cat !== "string" || !cat.trim()) return "Other";
    return cat.trim();
}

function getCategory(req) {
    if (!req) return "Other";
    if (req.category) return req.category;
    for (const v of ["SingleCourse", "CourseGroup", "AnyOf", "AllOf", "Concentration", "Restriction", "DoubleCount"]) {
        if (req[v]?.category) return req[v].category;
    }
    return "Other";
}
