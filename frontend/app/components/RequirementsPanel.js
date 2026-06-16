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
    courseCountsForDegree,
} from "@/lib/crossDegree";
import { buildDegreeColorMap, getDegreeColorForIndex } from "@/lib/degreeColors";
import { reqRowDomId, attributeFulfillmentMap } from "@/lib/requirementNav";
import {
    buildCasSuperSections,
    buildRequirementTabs,
    casGenEdProgress,
    getCategory,
    normalizeCategory,
    poolGroupStats,
    resolveActiveTabIndex,
} from "@/lib/casRequirementsLayout";

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
        if (!navTarget || !scheduleData?.degree_results) return;
        const tabs = buildRequirementTabs(scheduleData.degree_results, degrees, degreeCatalog);
        const resolvedTab = resolveActiveTabIndex(tabs, activeTab, navTarget);
        const { degreeIndex, instanceId, category } = navTarget;
        const rowId = reqRowDomId(degreeIndex ?? 0, instanceId);
        const timer = window.setTimeout(() => {
            document.getElementById(rowId)?.scrollIntoView({ behavior: "smooth", block: "nearest" });
            setFlashRowId(rowId);
            window.setTimeout(() => setFlashRowId(null), 2200);
            setActiveTab(resolvedTab);
            if (category || degreeIndex != null) {
                setCollapsedGroups((prev) => ({
                    ...prev,
                    ...casCollapseKeysForNav(navTarget, scheduleData.degree_results),
                }));
            }
            onNavTargetConsumed?.();
        }, 80);
        return () => window.clearTimeout(timer);
    }, [navTarget, onNavTargetConsumed, scheduleData, degrees, degreeCatalog, activeTab]);

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
    const degreeColorMap = buildDegreeColorMap(scheduleData);
    const tabs = buildRequirementTabs(results, degrees, degreeCatalog);
    const tabIndex = resolveActiveTabIndex(tabs, activeTab, navTarget);
    const activeTabDef = tabs[tabIndex];
    if (!activeTabDef) return null;

    const assignedIds = new Set(filterValidPlacements(assignedCourses).map((a) => a.courseId));
    const frozenIds = new Set(filterFrozenPlacements(frozenCourses).map((f) => f.courseId));
    const crossDegreeChipTitle = (courseId) => crossDegreeViolationsByCourse[courseId] || undefined;

    const isCasLayout = activeTabDef.type === "cas-combined" || activeTabDef.type === "cas-single";

    let current;
    let degreeLabel;
    let allReqs;
    let pools;
    let categoryMap;
    let orderedCategories;
    let scheduleCtx;
    let errors = [];

    if (isCasLayout) {
        const casIndices = activeTabDef.type === "cas-combined"
            ? activeTabDef.indices
            : [activeTabDef.index];
        const superSections = buildCasSuperSections({
            casIndices,
            results,
            degrees,
            degreeCatalog,
            courseDegreesMap,
            combined: activeTabDef.type === "cas-combined",
        });
        allReqs = superSections.flatMap((sec) => {
            if (sec.kind === "writing") return sec.items || [];
            if (sec.kind === "major") {
                return (sec.orderedCategories || []).flatMap(
                    (cat) => sec.categoryMap[cat] || [],
                );
            }
            return [];
        });
        current = results[casIndices[0]];
        degreeLabel = `${current.school}-${current.major}`;
        pools = [];
        categoryMap = {};
        orderedCategories = [];
        errors = casIndices.map((i) => results[i]?.error).filter(Boolean);
        scheduleCtx = {
            assignedIds,
            frozenIds,
            crossDegreeViolationsByCourse,
            crossDegreeChipTitle,
            degreeIndex: casIndices[0],
            superSections,
            isCasCombined: activeTabDef.type === "cas-combined",
            casIndices,
        };
    } else {
        current = results[activeTabDef.index];
        degreeLabel = `${current.school}-${current.major}`;

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

        allReqs = [];
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

        pools = current.pool_coverage_info || [];

        const categoryForItem = (item) => {
            const cat = normalizeCategory(item.category);
            for (const pool of pools) {
                const poolCat = normalizeCategory(pool.category);
                if (cat === poolCat || cat === `${poolCat} - Pool course`) {
                    return poolCat;
                }
            }
            return cat;
        };

        categoryMap = {};
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
        orderedCategories = [...categoryOrder];
        Object.keys(categoryMap).forEach((c) => {
            if (!orderedCategories.includes(c)) orderedCategories.push(c);
        });

        scheduleCtx = {
            assignedIds,
            frozenIds,
            crossDegreeViolationsByCourse,
            crossDegreeChipTitle,
            degreeIndex: activeTabDef.index,
        };
        if (current.error) errors.push(current.error);
    }

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
            {tabs.length > 1 && (
                <div className="req-degree-tabs" role="tablist" aria-label="Degree requirements">
                    {tabs.map((tab, i) => {
                        const isActive = tabIndex === i;
                        const degreeKey = tab.type === "cas-combined"
                            ? "CAS-combined"
                            : `${results[tab.index].school}-${results[tab.index].major}`;
                        const degreeColor = tab.type === "cas-combined"
                            ? getDegreeColorForIndex(tab.indices[0])
                            : (degreeColorMap[degreeKey] || getDegreeColorForIndex(tab.index));
                        return (
                            <button
                                key={tab.id}
                                type="button"
                                role="tab"
                                aria-selected={isActive}
                                className={`req-degree-tab ${isActive ? "active" : ""}`}
                                style={{ "--degree-tab-color": degreeColor }}
                                onClick={() => setActiveTab(i)}
                                title={tab.schoolLine ? `${tab.label} (${tab.schoolLine})` : tab.label}
                            >
                                <span className="req-degree-tab-major">{tab.label}</span>
                                {tab.schoolLine && (
                                    <span className="req-degree-tab-school">{tab.schoolLine}</span>
                                )}
                            </button>
                        );
                    })}
                </div>
            )}

            {errors.map((err) => (
                <div key={err} className="req-error-banner">⚠️ {err}</div>
            ))}

            {!errors.length && totalCount > 0 && (
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
                {isCasLayout ? (
                    renderCasSuperSections({
                        scheduleCtx,
                        collapsedGroups,
                        setCollapsedGroups,
                        navTarget,
                        flashRowId,
                        degreeLabel,
                        courseDegreesMap,
                        results,
                        frozenIds,
                    })
                ) : (
                    orderedCategories.map((cat) => {
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

                        return (
                            <div key={cat} className={groupClass}>
                                <div
                                    className="req-group-header"
                                    onClick={() => setCollapsedGroups((p) => ({ ...p, [cat]: !(p[cat] ?? true) }))}
                                >
                                    <span className={`req-group-badge ${groupDone ? "badge-done" : "badge-pending"}`}>
                                        {groupDone ? "✓" : "·"}
                                    </span>
                                    <span className="req-group-name" title={cat}>{cat}</span>
                                    <span className={`req-group-pill ${groupDone ? "pill-done" : "pill-pending"}`}>
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
                                        {items.map((item, rowIdx) => renderRequirementItem(
                                            item,
                                            item.instanceId ?? String(rowIdx),
                                            scheduleCtx,
                                            rowIdx === 0,
                                            scheduleCtx.degreeIndex,
                                            flashRowId,
                                        ))}
                                        {pool && (pool.constraints || []).length > 0 && (
                                            <>
                                                <div className="req-pool-divider req-item-first">
                                                    {poolStats.slotsTotal} course{poolStats.slotsTotal === 1 ? "" : "s"}
                                                    {" "}to fulfill{" "}
                                                    {poolStats.covTotal} requirement{poolStats.covTotal === 1 ? "" : "s"}
                                                </div>
                                                {(pool.constraints || []).map((constraint, j) =>
                                                    renderPoolConstraintItem(
                                                        constraint,
                                                        `pool-${pool.pool_index ?? cat}-${j}`,
                                                        false,
                                                        scheduleCtx,
                                                    ),
                                                )}
                                            </>
                                        )}
                                    </div>
                                )}
                            </div>
                        );
                    })
                )}
                {totalCount === 0 && !errors.length && (
                    <div className="req-empty-state"><div className="req-empty-text">No requirement data available</div></div>
                )}
            </div>
        </div>
    );
}

function casCollapseKeysForNav(navTarget, results) {
    const keys = {};
    const { degreeIndex, category } = navTarget;
    if (category) keys[normalizeCategory(category)] = false;
    if (degreeIndex == null) return keys;
    const result = results[degreeIndex];
    if (result?.school !== "CAS") return keys;
    const cat = normalizeCategory(category);
    if (cat === "Writing Seminar") keys["cas-writing"] = false;
    else if (
        cat.startsWith("Foundational Approaches")
        || cat.startsWith("Sectors of Knowledge")
        || cat === "General Education"
    ) {
        keys["cas-gened"] = false;
    } else if (cat) {
        keys[`cas-major-${degreeIndex}`] = false;
    }
    return keys;
}

function renderRequirementItem(item, idx, scheduleCtx, isFirst, degreeIndex, flashRowId) {
    if (isExpandableCourseGroup(item.requirement)) {
        return renderCourseGroup(item, idx, scheduleCtx, isFirst, degreeIndex, flashRowId);
    }
    if (isExpandableAnyOf(item.requirement)) {
        return renderAnyOfGroup(item, idx, scheduleCtx, isFirst, degreeIndex, flashRowId);
    }
    return renderItem(item, idx, scheduleCtx, isFirst, degreeIndex, flashRowId);
}

function renderCasSuperSections({
    scheduleCtx,
    collapsedGroups,
    setCollapsedGroups,
    navTarget,
    flashRowId,
    degreeLabel,
    courseDegreesMap,
    results,
    frozenIds,
}) {
    const { superSections, isCasCombined, casIndices } = scheduleCtx;

    return superSections.map((section) => {
        const isCollapsed = collapsedGroups[section.id] ?? true;
        const toggle = () => setCollapsedGroups((p) => ({
            ...p,
            [section.id]: !(p[section.id] ?? true),
        }));

        if (section.kind === "writing") {
            const items = section.items || [];
            const groupDone = items.length > 0
                && items.every((r) => r.fulfilled && itemTone(r, frozenIds) === "fulfilled");
            return (
                <div key={section.id} className={`req-super-group ${groupDone ? "req-super-group-done" : ""}`}>
                    {renderSuperGroupHeader(section.title, groupDone, isCollapsed, toggle, `${items.filter((r) => r.fulfilled).length}/${items.length}`)}
                    {!isCollapsed && (
                        <div className="req-super-group-body">
                            {items.map((item, rowIdx) => renderRequirementItem(
                                item,
                                item.instanceId ?? String(rowIdx),
                                { ...scheduleCtx, degreeIndex: section.degreeIndex },
                                rowIdx === 0,
                                section.degreeIndex,
                                flashRowId,
                            ))}
                        </div>
                    )}
                </div>
            );
        }

        if (section.kind === "genEd") {
            const casGenEd = section.casGenEd;
            const pool = section.pool;
            const casProgress = casGenEd ? casGenEdProgress(casGenEd) : null;
            const poolStats = pool ? poolGroupStats(pool) : null;
            const groupDone = casProgress
                ? casProgress.done === casProgress.total
                    && (!poolStats || poolStats.covDone >= poolStats.covTotal)
                : poolStats
                    ? poolStats.covDone >= poolStats.covTotal
                    : false;
            const pillLabel = casProgress
                ? `${casProgress.done}/${casProgress.total}`
                : poolStats
                    ? `${poolStats.covDone}/${poolStats.covTotal}`
                    : null;

            const genEdDegreeLabel = isCasCombined
                ? casIndices.map((i) => `${results[i].school}-${results[i].major}`).join(", ")
                : degreeLabel;

            return (
                <div key={section.id} className={`req-super-group ${groupDone ? "req-super-group-done" : ""}`}>
                    {renderSuperGroupHeader(section.title, groupDone, isCollapsed, toggle, pillLabel)}
                    {!isCollapsed && (
                        <div className="req-super-group-body">
                            {casGenEd ? (
                                renderCasGenEdPool(
                                    casGenEd,
                                    null,
                                    scheduleCtx,
                                    genEdDegreeLabel,
                                    courseDegreesMap,
                                    section.degreeIndex,
                                    isCasCombined,
                                    casIndices,
                                    results,
                                )
                            ) : pool && (pool.constraints || []).length > 0 && (
                                <>
                                    {poolStats && (
                                        <div className="req-pool-divider req-item-first">
                                            {poolStats.covTotal} gen-ed requirement{poolStats.covTotal === 1 ? "" : "s"}
                                            {poolStats.covTotal > poolStats.slotsTotal
                                                ? " (double-counting permitted)"
                                                : ""}
                                        </div>
                                    )}
                                    {(pool.constraints || []).map((constraint, j) =>
                                        renderPoolConstraintItem(
                                            constraint,
                                            `pool-${pool.pool_index}-c-${j}`,
                                            j === 0,
                                            scheduleCtx,
                                        ),
                                    )}
                                </>
                            )}
                        </div>
                    )}
                </div>
            );
        }

        if (section.kind === "major") {
            const { categoryMap, orderedCategories, degreeIndex } = section;
            const majorItems = orderedCategories.flatMap((cat) => categoryMap[cat] || []);
            const majorDone = majorItems.length > 0
                && majorItems.every((r) => r.fulfilled && itemTone(r, frozenIds) === "fulfilled");
            const majorPartial = majorItems.some((r) => itemTone(r, frozenIds) === "partial");
            const majorFulfilled = majorItems.filter((r) => r.fulfilled).length;

            return (
                <div
                    key={section.id}
                    className={`req-major-section ${majorDone ? "req-major-section-done" : ""}`}
                >
                    {renderMajorSectionTitle(
                        section.title,
                        majorDone && !majorPartial,
                        `${majorFulfilled}/${majorItems.length}`,
                    )}
                    <div className="req-major-body">
                        {orderedCategories.map((cat) => {
                            const items = categoryMap[cat] || [];
                            if (!items.length) return null;
                            const done = items.filter((r) => r.fulfilled).length;
                            const groupDone = done === items.length;
                            const innerCollapsed = (() => {
                                if (navTarget?.category && normalizeCategory(navTarget.category) === cat) {
                                    return false;
                                }
                                return collapsedGroups[cat] ?? true;
                            })();

                            return (
                                <div key={cat} className={`req-group ${groupDone ? "req-group-done" : ""}`}>
                                    <div
                                        className="req-group-header"
                                        onClick={() => setCollapsedGroups((p) => ({
                                            ...p,
                                            [cat]: !(p[cat] ?? true),
                                        }))}
                                    >
                                        <span className={`req-group-badge ${groupDone ? "badge-done" : "badge-pending"}`}>
                                            {groupDone ? "✓" : "·"}
                                        </span>
                                        <span className="req-group-name" title={cat}>{cat}</span>
                                        <span className={`req-group-pill ${groupDone ? "pill-done" : "pill-pending"}`}>
                                            {done}/{items.length}
                                        </span>
                                        <span
                                            className={`req-group-chevron${innerCollapsed ? "" : " req-group-chevron-open"}`}
                                            aria-hidden
                                        >
                                            ▶
                                        </span>
                                    </div>
                                    {!innerCollapsed && (
                                        <div className="req-group-body">
                                            {items.map((item, rowIdx) => renderRequirementItem(
                                                item,
                                                item.instanceId ?? String(rowIdx),
                                                { ...scheduleCtx, degreeIndex },
                                                rowIdx === 0,
                                                degreeIndex,
                                                flashRowId,
                                            ))}
                                        </div>
                                    )}
                                </div>
                            );
                        })}
                    </div>
                </div>
            );
        }

        return null;
    });
}

function renderSuperGroupHeader(title, done, isCollapsed, onToggle, pillLabel) {
    return (
        <div className="req-super-group-header" onClick={onToggle}>
            <span className={`req-super-group-badge ${done ? "badge-done" : "badge-pending"}`}>
                {done ? "✓" : "·"}
            </span>
            <span className="req-super-group-name">{title}</span>
            {pillLabel && (
                <span className={`req-group-pill ${done ? "pill-done" : "pill-pending"}`}>
                    {pillLabel}
                </span>
            )}
            <span
                className={`req-group-chevron${isCollapsed ? "" : " req-group-chevron-open"}`}
                aria-hidden
            >
                ▶
            </span>
        </div>
    );
}

function renderMajorSectionTitle(title, done, pillLabel) {
    return (
        <div className="req-major-title">
            <span className="req-major-title-text">{title}</span>
            {pillLabel && (
                <span className={`req-group-pill ${done ? "pill-done" : "pill-pending"}`}>
                    {pillLabel}
                </span>
            )}
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

function renderCasGenEdPool(
    casGenEd,
    majorDisplayName,
    scheduleCtx,
    degreeLabel,
    courseDegreesMap,
    degreeIndex,
    isCasCombined = false,
    casIndices = [],
    results = [],
) {
    const filterCourses = (courseIds) => {
        if (isCasCombined) {
            return (courseIds || []).filter((id) =>
                casIndices.some((idx) =>
                    courseCountsForDegree(
                        id,
                        `${results[idx].school}-${results[idx].major}`,
                        courseDegreesMap,
                    ),
                ),
            );
        }
        return filterCoursesForDegree(courseIds, degreeLabel, courseDegreesMap);
    };

    return (
        <div className="req-cas-gened">
            <div className="req-cas-gened-section">
                <div className="req-cas-gened-heading">Foundational Approaches</div>
                {(casGenEd.foundational_approaches || []).map((row) =>
                    renderCasGenEdRow(
                        row,
                        null,
                        scheduleCtx,
                        filterCourses,
                        degreeIndex,
                    ),
                )}
            </div>
            <div className="req-cas-gened-section">
                <div className="req-cas-gened-heading">Sectors of Knowledge</div>
                {(casGenEd.sectors || []).map((row) =>
                    renderCasGenEdRow(
                        row,
                        majorDisplayName,
                        scheduleCtx,
                        filterCourses,
                        degreeIndex,
                    ),
                )}
            </div>
        </div>
    );
}

function renderCasGenEdRow(row, majorDisplayName, scheduleCtx, filterCourses, degreeIndex) {
    const rowTone = row.fulfilled ? "fulfilled" : "open";
    const courses = filterCourses(row.matched_courses || []);
    const fulfillingSet = new Set(courses);

    return (
        <div
            key={`${row.attr}-${row.name}`}
            className={`req-cas-gened-row req-cas-gened-row--${rowTone}`}
        >
            <span className={`req-item-icon icon-${rowTone}`}>
                {row.fulfilled ? "✓" : "○"}
            </span>
            <div className="req-item-body">
                <div className="req-item-line">
                    <span className="req-cas-gened-label">
                        {row.name}
                        <span className="req-cas-gened-attr"> [{row.attr}]</span>
                    </span>
                    {row.fulfilled_by_major && majorDisplayName && (
                        <span className="req-cas-gened-note">
                            Fulfilled by {majorDisplayName} major
                        </span>
                    )}
                    {!row.fulfilled_by_major && courses.length > 0 && (
                        <>
                            <span className="req-item-colon">:</span>
                            <span className="req-chips">
                                {courses.map((courseId) => (
                                    <span
                                        key={courseId}
                                        className={chipClass(badgeKindFor(courseId, {
                                            ...scheduleCtx,
                                            fulfillingSet,
                                            partialTone: false,
                                        }))}
                                        title={scheduleCtx.crossDegreeChipTitle?.(courseId)}
                                    >
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

function renderPoolConstraintItem(constraint, rowKey, isFirst, scheduleCtx) {
    const rowTone = constraint.fulfilled ? "fulfilled" : "open";
    const fulfillingSet = new Set(constraint.matched_courses || []);
    return (
        <div
            key={rowKey}
            className={`req-item req-item--${rowTone} ${isFirst ? "req-item-first" : ""}`}
        >
            <span className={`req-item-icon icon-${rowTone}`}>
                {constraint.fulfilled ? "✓" : "○"}
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
                                    <span
                                        key={courseId}
                                        className={chipClass(badgeKindFor(courseId, {
                                            ...scheduleCtx,
                                            fulfillingSet,
                                            partialTone: false,
                                        }))}
                                        title={scheduleCtx.crossDegreeChipTitle?.(courseId)}
                                    >
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
