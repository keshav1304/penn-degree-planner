"use client";

import { useState, useEffect } from "react";
import {
    filterValidCourseCodes,
    filterValidPlacements,
    filterFrozenPlacements,
    isValidCourseCode,
    isPoolConstraintInstanceId,
    isPoolFlexibleSlotInstanceId,
} from "@/lib/courseUtils";
import {
    childMatchesAnyOfFulfillment,
    courseGroupHeaderLabel,
    coursesMatchingChildLeaf,
    createRequirementDescription,
    evaluateCourseGroupChildren,
    getAnyOfPossibilities,
    getCourseGroupAreaLabel,
    getRequirementInstanceId,
    getRequirementStem,
    isExpandableAnyOf,
    isExpandableCourseGroup,
    parseRequirement,
} from "@/lib/requirementText";
import {
    filterAttributeFulfillmentForDegree,
    filterCoursesForDegree,
} from "@/lib/crossDegree";
import { formatDegreeDisplay } from "@/lib/degreeDisplay";
import { reqRowDomId, attributeFulfillmentMap } from "@/lib/requirementNav";

export default function RequirementsPanel({
    scheduleData,
    degrees,
    degreeCatalog = [],
    frozenCourses = [],
    assignedCourses = [],
    courseDegreesMap = {},
    crossDegreeViolationsByCourse = {},
    navTarget = null,
    onNavTargetConsumed,
}) {
    const [activeTab, setActiveTab] = useState(0);
    const [collapsedGroups, setCollapsedGroups] = useState({});
    const [flashRowId, setFlashRowId] = useState(null);

    useEffect(() => {
        if (!navTarget) return;
        const { degreeIndex, instanceId, category } = navTarget;
        const rowId = reqRowDomId(degreeIndex ?? 0, instanceId);
        const timer = window.setTimeout(() => {
            document.getElementById(rowId)?.scrollIntoView({ behavior: "smooth", block: "nearest" });
            setFlashRowId(rowId);
            window.setTimeout(() => setFlashRowId(null), 2200);
            if (degreeIndex != null) setActiveTab(degreeIndex);
            if (category) setCollapsedGroups((prev) => ({ ...prev, [category]: false }));
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
    const tabIndex = navTarget?.degreeIndex != null
        ? Math.min(navTarget.degreeIndex, results.length - 1)
        : Math.min(activeTab, results.length - 1);
    const current = results[tabIndex];
    if (!current) return null;

    const degreeLabel = `${current.school}-${current.major}`;

    const mapRequirementForDegree = (mapped, { fulfilledDefault, partialDefault }) => {
        const fulfilledCourses = filterCoursesForDegree(
            mapped.course_ids || [],
            degreeLabel,
            courseDegreesMap,
        );
        const attributeFulfillment = filterAttributeFulfillmentForDegree(
            attributeFulfillmentMap(mapped),
            degreeLabel,
            courseDegreesMap,
        );
        const hasAllocatedFulfillment =
            fulfilledCourses.length > 0
            || (attributeFulfillment
                && [...attributeFulfillment.values()].some((ids) => ids.length > 0));
        return {
            category: normalizeCategory(getCategory(mapped.requirement)),
            fulfilled: fulfilledDefault && hasAllocatedFulfillment,
            partial: partialDefault && hasAllocatedFulfillment,
            committedAnyofBranch: mapped.committed_anyof_branch ?? null,
            fulfilledCourses,
            requirement: mapped.requirement,
            instanceId: getRequirementInstanceId(mapped),
            attributeFulfillment,
        };
    };

    const allReqs = [];
    const pushIfSchedulable = (mapped, opts) => {
        if (isPoolConstraintInstanceId(getRequirementInstanceId(mapped))) return;
        if (isPoolFlexibleSlotInstanceId(getRequirementInstanceId(mapped))) return;
        allReqs.push(mapRequirementForDegree(mapped, opts));
    };
    (current.fulfilled_requirements || []).forEach((mapped) => {
        pushIfSchedulable(mapped, { fulfilledDefault: true, partialDefault: false });
    });
    (current.unfulfilled_requirements || []).forEach((mapped, rowIdx) => {
        const req = mapped?.requirement ?? mapped;
        if (isPoolConstraintInstanceId(getRequirementInstanceId(mapped))) return;
        if (isPoolFlexibleSlotInstanceId(getRequirementInstanceId(mapped))) return;
        const item = mapRequirementForDegree(
            { ...mapped, requirement: req },
            { fulfilledDefault: false, partialDefault: Boolean(mapped.partial) },
        );
        allReqs.push({
            ...item,
            instanceId: item.instanceId ?? `u-${rowIdx}`,
        });
    });

    const pools = current.pool_coverage_info || [];

    const categoryForItem = (item) => {
        const cat = normalizeCategory(item.category);
        for (const pool of pools) {
            const poolCat = normalizeCategory(pool.category);
            if (cat === poolCat || cat === `${poolCat} — Pool course`) {
                return poolCat;
            }
        }
        return cat;
    };

    const categoryMap = {};
    allReqs.forEach((item) => {
        const cat = categoryForItem(item);
        if (!categoryMap[cat]) categoryMap[cat] = [];
        categoryMap[cat].push(item);
    });
    pools.forEach((pool) => {
        const cat = normalizeCategory(pool.category);
        if (!categoryMap[cat]) categoryMap[cat] = [];
    });

    const categoryOrder = (current.category_order || []).map(normalizeCategory);
    const orderedCategories = [...categoryOrder];
    Object.keys(categoryMap).forEach((c) => { if (!orderedCategories.includes(c)) orderedCategories.push(c); });

    const assignedIds = new Set(filterValidPlacements(assignedCourses).map((a) => a.courseId));
    const frozenIds = new Set(filterFrozenPlacements(frozenCourses).map((f) => f.courseId));
    const crossDegreeChipTitle = (courseId) => {
        return crossDegreeViolationsByCourse[courseId] || undefined;
    };

    const scheduleCtx = {
        assignedIds,
        frozenIds,
        crossDegreeViolationsByCourse,
        crossDegreeChipTitle,
    };

    const totalCount = allReqs.length;
    const fulfilledCount = allReqs.filter((r) => r.fulfilled && itemTone(r, frozenIds) === "fulfilled").length;
    const plannedCount = allReqs.filter((r) => r.fulfilled && itemTone(r, frozenIds) === "frozen").length;
    const partialCount = allReqs.filter((r) => itemTone(r, frozenIds) === "partial").length;
    const remainingCount = totalCount - fulfilledCount - plannedCount - partialCount;
    const fulfilledPct = totalCount > 0 ? (fulfilledCount / totalCount) * 100 : 0;
    const plannedPct = totalCount > 0 ? (plannedCount / totalCount) * 100 : 0;
    const pct = totalCount > 0 ? Math.round(((fulfilledCount + plannedCount) / totalCount) * 100) : 0;

    return (
        <div className="req-panel">
            {results.length > 1 && (
                <div className="req-degree-tabs" role="tablist" aria-label="Degree requirements">
                    {results.map((result, i) => {
                        const { major, schoolLine } = formatDegreeDisplay(
                            degrees[i],
                            result,
                            degreeCatalog,
                        );
                        const isActive = tabIndex === i;
                        return (
                            <button
                                key={i}
                                type="button"
                                role="tab"
                                aria-selected={isActive}
                                className={`req-degree-tab ${isActive ? "active" : ""}`}
                                onClick={() => setActiveTab(i)}
                                title={schoolLine ? `${major} (${schoolLine})` : major}
                            >
                                <span className="req-degree-tab-major">{major}</span>
                                {schoolLine && (
                                    <span className="req-degree-tab-school">{schoolLine}</span>
                                )}
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
                                className="req-progress-fill req-progress-fill-fulfilled"
                                style={{ width: `${fulfilledPct}%` }}
                            />
                        )}
                        {plannedPct > 0 && (
                            <div
                                className="req-progress-fill req-progress-fill-planned"
                                style={{ width: `${plannedPct}%` }}
                            />
                        )}
                    </div>
                </div>
            )}

            <div className="req-groups">
                {orderedCategories.map((cat) => {
                    const items = categoryMap[cat] || [];
                    const pool = pools.find((p) => normalizeCategory(p.category) === cat);
                    if (!items.length && !pool) return null;

                    const poolStats = pool ? poolGroupStats(pool) : null;
                    const done = items.filter((r) => r.fulfilled).length;
                    const groupDone = poolStats
                        ? poolStats.slotsFilled >= poolStats.slotsTotal
                            && poolStats.covDone >= poolStats.covTotal
                        : done === items.length;
                    const isCollapsed = (() => {
                        if (navTarget?.category && normalizeCategory(navTarget.category) === cat) {
                            return false;
                        }
                        return collapsedGroups[cat] ?? true;
                    })();
                    const groupClass = groupDone ? "req-group req-group-done" : "req-group";
                    const pillLabel = poolStats
                        ? `${poolStats.slotsFilled}/${poolStats.slotsTotal}`
                        : `${done}/${items.length}`;
                    const poolSlotsDone = poolStats
                        ? poolStats.slotsFilled >= poolStats.slotsTotal
                        : false;

                    return (
                        <div key={cat} className={groupClass}>
                            <div
                                className="req-group-header"
                                onClick={() => setCollapsedGroups((p) => ({ ...p, [cat]: !(p[cat] ?? true) }))}
                            >
                                {!pool && (
                                    <span className={`req-group-badge ${groupDone ? "badge-done" : "badge-pending"}`}>
                                        {groupDone ? "✓" : "·"}
                                    </span>
                                )}
                                <span className="req-group-name" title={cat}>{cat}</span>
                                <span className={`req-group-pill ${pool ? (poolSlotsDone ? "pill-done" : "pill-pending") : (groupDone ? "pill-done" : "pill-pending")}`}>
                                    {pillLabel}
                                </span>
                                <span
                                    className={`req-group-chevron${isCollapsed ? "" : " req-group-chevron-open"}`}
                                    aria-hidden
                                >
                                    ▶
                                </span>
                            </div>
                            {!isCollapsed && (
                                <div className="req-group-body">
                                    {!pool && items.map((item, rowIdx) => {
                                        if (isExpandableCourseGroup(item.requirement)) {
                                            return renderCourseGroup(
                                                item,
                                                item.instanceId ?? String(rowIdx),
                                                scheduleCtx,
                                                rowIdx === 0,
                                                tabIndex,
                                                flashRowId
                                            );
                                        }
                                        if (isExpandableAnyOf(item.requirement)) {
                                            return renderAnyOfGroup(
                                                item,
                                                item.instanceId ?? String(rowIdx),
                                                scheduleCtx,
                                                rowIdx === 0,
                                                tabIndex,
                                                flashRowId
                                            );
                                        }
                                        return renderItem(
                                            item,
                                            item.instanceId ?? String(rowIdx),
                                            scheduleCtx,
                                            rowIdx === 0,
                                            tabIndex,
                                            flashRowId
                                        );
                                    })}
                                    {pool && (pool.constraints || []).length > 0 && (
                                        <>
                                            <div className="req-pool-divider req-item-first">
                                                {poolStats.slotsTotal} course{poolStats.slotsTotal === 1 ? "" : "s"}
                                                {" "}cover{poolStats.slotsTotal === 1 ? "s" : ""}
                                                {" "}{poolStats.covTotal} requirement{poolStats.covTotal === 1 ? "" : "s"}
                                                {poolStats.covTotal > poolStats.slotsTotal
                                                    ? " (double-counting allowed)"
                                                    : ""}
                                            </div>
                                            {(pool.constraints || []).map((constraint, j) =>
                                                renderPoolConstraintItem(
                                                    constraint,
                                                    `pool-${pool.pool_index ?? cat}-${j}`,
                                                    false,
                                                ),
                                            )}
                                        </>
                                    )}
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
    if (item.partial && collectFulfillingCourses(item).length > 0) return "partial";
    if (!item.fulfilled) return "open";
    if (itemHasFrozenCourse(collectFulfillingCourses(item), frozenIds)) return "frozen";
    return "fulfilled";
}

function groupTone(items, frozenIds) {
    const done = items.filter((r) => r.fulfilled).length;
    const partial = items.some((r) => itemTone(r, frozenIds) === "partial");
    if (partial) return "incomplete";
    if (done !== items.length) return "incomplete";
    if (items.some((item) => itemTone(item, frozenIds) === "frozen")) return "frozen";
    return "fulfilled";
}

function badgeKindFor(courseId, { assignedIds, frozenIds, fulfillingSet, partialTone }) {
    if (!fulfillingSet?.has(courseId)) return "open";
    if (frozenIds.has(courseId)) return "frozen";
    if (partialTone) return "partial";
    if (assignedIds.has(courseId)) return "fulfilled";
    return "fulfilled";
}

function chipClass(kind) {
    if (kind === "fulfilled") return "req-chip chip-fulfilled";
    if (kind === "frozen") return "req-chip chip-frozen";
    if (kind === "partial") return "req-chip chip-partial";
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
        return { stem: null, badges: [], fulfillingSet };
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

    if (type === "AllOf") {
        const parts = (data.requirements || [])
            .map((sub) => createRequirementDescription(sub))
            .filter(Boolean);
        if (parts.length > 0) {
            return { stem: null, badges: [], fulfillingSet };
        }
    }

    return { stem, badges: fulfilling.map((id) => ({ kind: "course", id })), fulfillingSet };
}

function renderRequirementLine(item, scheduleCtx, crossDegreeChipTitle = () => undefined) {
    const { stem, badges, fulfillingSet } = buildRowContent(item);
    const fullDesc = createRequirementDescription(item.requirement);
    const chipCtx = {
        ...scheduleCtx,
        fulfillingSet,
        partialTone: scheduleCtx.partialTone ?? item.partial,
    };

    const renderBadge = (badge, key) => {
        if (badge.kind === "attr") {
            const fulfillingForAttr = badge.courses.filter((c) => fulfillingSet.has(c));
            const hasCourse = fulfillingForAttr.length > 0;
            const label = hasCourse
                ? `[${badge.code}] ${fulfillingForAttr.join(", ")}`
                : `[${badge.code}]`;
            const kind = hasCourse ? badgeKindFor(fulfillingForAttr[0], chipCtx) : "open";
            return <span key={key} className={chipClass(kind)}>{label}</span>;
        }
        const chipTitle = crossDegreeChipTitle(badge.id);
        return (
            <span
                key={key}
                className={chipClass(badgeKindFor(badge.id, chipCtx))}
                title={chipTitle}
            >
                {badge.id}
            </span>
        );
    };

    if (stem && badges.length > 0) {
        return (
            <div className="req-item-line">
                <span className="req-stem-text">{stem}</span>
                <span className="req-item-colon">:</span>
                <span className="req-chips">{badges.map((b, i) => renderBadge(b, i))}</span>
            </div>
        );
    }

    return (
        <div className="req-item-line">
            <span className="req-stem-text">{fullDesc}</span>
        </div>
    );
}

function makeAnyOfChildItem(parent, childReq, childIdx) {
    const matched = childMatchesAnyOfFulfillment(childReq, parent, childIdx);
    const partialActive =
        parent.partial
        && parent.committedAnyofBranch === childIdx
        && !matched;
    const parentCourses = collectFulfillingCourses(parent);
    let courses = matched ? parentCourses : [];
    if (partialActive) {
        courses = coursesMatchingChildLeaf(childReq, parentCourses);
    }
    let attrFulfillment = parent.attributeFulfillment;
    if ((matched || partialActive) && attrFulfillment) {
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
        partial: partialActive,
        fulfilledCourses: courses,
        requirement: childReq,
        instanceId: `${parent.instanceId}::${childIdx}`,
        attributeFulfillment: attrFulfillment,
        committedAnyofBranch: null,
    };
}

function renderAllOfPartialRows(item, scheduleCtx) {
    const { type, data } = parseRequirement(item.requirement);
    if (type !== "AllOf") return null;
    const parentCourses = collectFulfillingCourses(item);

    return (
        <div className="req-allof-partial">
            {(data.requirements || []).map((subReq, subIdx) => {
                const subMatches = coursesMatchingChildLeaf(subReq, parentCourses);
                const subFulfilling = new Set(subMatches);
                const subItem = {
                    requirement: subReq,
                    fulfilledCourses: subMatches,
                    fulfilled: subMatches.length > 0,
                    partial: subMatches.length > 0,
                };
                const subTone = subMatches.length > 0
                    ? itemTone(subItem, scheduleCtx.frozenIds)
                    : "open";
                return (
                    <div key={subIdx} className={`req-allof-partial-row req-allof-partial-row--${subTone}`}>
                        <span className={`req-item-icon icon-${subTone}`}>
                            {subMatches.length > 0 ? "◐" : "•"}
                        </span>
                        <div className="req-item-body">
                            {renderRequirementLine(
                                { ...subItem, attributeFulfillment: item.attributeFulfillment },
                                {
                                    ...scheduleCtx,
                                    fulfillingSet: subFulfilling,
                                    partialTone: subTone === "partial",
                                },
                            )}
                        </div>
                    </div>
                );
            })}
        </div>
    );
}

function choiceGroupHeaderLabel(requirement) {
    const { type } = parseRequirement(requirement);
    return type === "AllOf"
        ? "Choose all of the following:"
        : "Choose one of the following:";
}

function requirementStatusIcon(item, tone) {
    if (item.fulfilled) return "✓";
    if (item.partial || tone === "partial") return "◐";
    return "○";
}

function renderChoiceGroupHeader(item, tone, { isFirst, label }) {
    return (
        <div className={`req-item req-item--${tone} req-choice-header ${isFirst ? "req-item-first" : ""}`}>
            <span className={`req-item-icon icon-${tone}`}>
                {requirementStatusIcon(item, tone)}
            </span>
            <div className="req-item-body">
                <div className="req-item-line">
                    <span className="req-stem-text">{label}</span>
                </div>
            </div>
        </div>
    );
}

function courseOptionsForArea(childReq) {
    const { type, data } = parseRequirement(childReq);
    if (type === "SingleCourse") {
        return (data.possibilities || []).filter(Boolean);
    }
    if (type === "AllOf") {
        return (data.requirements || []).flatMap((sub) => courseOptionsForArea(sub));
    }
    return [];
}

function renderCourseGroupAreaLine(childReq, areaTone, fulfilledCourses, scheduleCtx) {
    const options = courseOptionsForArea(childReq);
    const fulfillingSet = new Set(fulfilledCourses);
    const chipCtx = {
        ...scheduleCtx,
        fulfillingSet,
        partialTone: areaTone === "partial",
    };

    return (
        <div className="req-item-line">
            <span className="req-stem-text">{getCourseGroupAreaLabel(childReq)}</span>
            {options.length > 0 && (
                <>
                    <span className="req-item-colon">:</span>
                    <span className="req-chips">
                        {options.map((id, i) => (
                            <span
                                key={i}
                                className={chipClass(badgeKindFor(id, chipCtx))}
                                title={scheduleCtx.crossDegreeChipTitle?.(id)}
                            >
                                {id}
                            </span>
                        ))}
                    </span>
                </>
            )}
        </div>
    );
}

function renderCourseGroup(parentItem, idx, scheduleCtx, isFirst, degreeIndex, flashRowId) {
    const areaRows = evaluateCourseGroupChildren(parentItem);
    const blockTone = itemTone(parentItem, scheduleCtx.frozenIds);
    const rowDomId = reqRowDomId(degreeIndex, idx);

    return (
        <div
            key={String(idx)}
            id={rowDomId}
            className={`req-anyof-block ${flashRowId === rowDomId ? "req-row-flash" : ""}`}
        >
            {renderChoiceGroupHeader(parentItem, blockTone, {
                isFirst,
                label: courseGroupHeaderLabel(parentItem.requirement),
            })}
            <div className="req-choice-children">
                {areaRows.map(({ childReq, childIdx, fulfilledCourses, tone }) => {
                    const childRowId = reqRowDomId(degreeIndex, `${parentItem.instanceId ?? idx}::${childIdx}`);
                    return (
                        <div
                            key={childRowId}
                            id={childRowId}
                            className={`req-item req-item--${tone} req-anyof-child ${flashRowId === childRowId ? "req-row-flash" : ""}`}
                        >
                            <span className={`req-item-icon icon-${tone}`}>
                                {tone === "fulfilled" ? "✓" : tone === "partial" ? "◐" : "•"}
                            </span>
                            <div className="req-item-body">
                                {renderCourseGroupAreaLine(
                                    childReq,
                                    tone,
                                    fulfilledCourses,
                                    scheduleCtx,
                                )}
                            </div>
                        </div>
                    );
                })}
            </div>
        </div>
    );
}

function renderAnyOfGroup(parentItem, idx, scheduleCtx, isFirst, degreeIndex, flashRowId) {
    const possibilities = getAnyOfPossibilities(parentItem.requirement);
    const blockTone = itemTone(parentItem, scheduleCtx.frozenIds);
    const rowDomId = reqRowDomId(degreeIndex, idx);

    return (
        <div
            key={String(idx)}
            id={rowDomId}
            className={`req-anyof-block ${flashRowId === rowDomId ? "req-row-flash" : ""}`}
        >
            {renderChoiceGroupHeader(parentItem, blockTone, {
                isFirst,
                label: choiceGroupHeaderLabel(parentItem.requirement),
            })}
            <div className="req-choice-children">
            {possibilities.map((childReq, childIdx) => {
                const childItem = makeAnyOfChildItem(parentItem, childReq, childIdx);
                const isCommittedBranch =
                    parentItem.partial && parentItem.committedAnyofBranch === childIdx;
                const isInactiveBranch =
                    parentItem.partial
                    && parentItem.committedAnyofBranch != null
                    && parentItem.committedAnyofBranch !== childIdx;
                const childTone = childItem.fulfilled || childItem.partial
                    ? itemTone(childItem, scheduleCtx.frozenIds)
                    : "open";
                const childRowId = reqRowDomId(degreeIndex, childItem.instanceId);
                const { type } = parseRequirement(childReq);
                return (
                    <div
                        key={childItem.instanceId}
                        id={childRowId}
                        className={`req-item req-item--${childTone} req-anyof-child ${isInactiveBranch ? "req-anyof-child--inactive" : ""} ${flashRowId === childRowId ? "req-row-flash" : ""}`}
                    >
                        <span className={`req-item-icon icon-${childTone}`}>
                            {childItem.fulfilled ? "✓" : childItem.partial ? "◐" : "•"}
                        </span>
                        <div className="req-item-body">
                            {type === "AllOf" && isCommittedBranch && childItem.partial
                                ? renderAllOfPartialRows(childItem, scheduleCtx)
                                : renderRequirementLine(
                                    childItem,
                                    {
                                        ...scheduleCtx,
                                        partialTone: childTone === "partial",
                                    },
                                    scheduleCtx.crossDegreeChipTitle,
                                )}
                        </div>
                    </div>
                );
            })}
            </div>
        </div>
    );
}

function renderItem(item, idx, scheduleCtx, isFirst, degreeIndex, flashRowId) {
    const rowTone = itemTone(item, scheduleCtx.frozenIds);
    const rowDomId = reqRowDomId(degreeIndex, idx);

    return (
        <div
            key={String(idx)}
            id={rowDomId}
            className={`req-item req-item--${rowTone} ${flashRowId === rowDomId ? "req-row-flash" : ""} ${isFirst ? "req-item-first" : ""}`}
        >
            <span className={`req-item-icon icon-${rowTone}`}>
                {requirementStatusIcon(item, rowTone)}
            </span>
            <div className="req-item-body">
                {renderRequirementLine(
                    item,
                    { ...scheduleCtx, partialTone: rowTone === "partial" },
                    scheduleCtx.crossDegreeChipTitle,
                )}
            </div>
        </div>
    );
}

function poolGroupStats(pool) {
    const slotsFilled = (pool.fixed_slots_filled || 0) + (pool.flexible_slots_filled || 0);
    const slotsTotal = (pool.fixed_slots_total || 0) + (pool.flexible_slots_total || 0);
    const covDone = (pool.constraints || []).filter((c) => c.fulfilled).length;
    const covTotal = (pool.constraints || []).length;
    return { slotsFilled, slotsTotal, covDone, covTotal };
}

function renderPoolConstraintItem(constraint, rowKey, isFirst) {
    const rowTone = constraint.fulfilled ? "fulfilled" : "open";
    return (
        <div
            key={rowKey}
            className={`req-item req-item--${rowTone} ${isFirst ? "req-item-first" : ""}`}
        >
            <span className={`req-item-icon icon-${rowTone}`}>
                {constraint.fulfilled ? "✓" : "•"}
            </span>
            <div className="req-item-body">
                <div className="req-item-line">
                    <span className="req-stem-text">
                        {constraint.description || constraint.label}
                    </span>
                    {constraint.matched_courses?.length > 0 && (
                        <>
                            <span className="req-item-colon">:</span>
                            <span className="req-chips">
                                {constraint.matched_courses.map((courseId) => (
                                    <span key={courseId} className={chipClass("fulfilled")}>
                                        {courseId}
                                    </span>
                                ))}
                            </span>
                        </>
                    )}
                </div>
            </div>
        </div>
    );
}

function normalizeCategory(cat) {
    if (!cat || typeof cat !== "string" || !cat.trim()) return "Other";
    return cat.trim();
}

function getCategory(req) {
    if (!req) return "Other";
    if (req.category) return req.category;
    for (const v of ["SingleCourse", "CourseGroup", "AnyOf", "AllOf", "Concentration", "Restriction", "CoursePool"]) {
        if (req[v]?.category) return req[v].category;
    }
    return "Other";
}
