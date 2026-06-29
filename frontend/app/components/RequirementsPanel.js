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
    courseGroupIsComplete,
    itemEffectiveFulfilled,
    formatCategoryProgress,
    categoryProgressCounts,
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
import { reqRowDomId, attributeFulfillmentMap, poolConstraintInstanceId } from "@/lib/requirementNav";
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
        return (
            <div className="req-empty-state panel-loading-state">
                <div className="loading-spinner" />
                <div className="req-empty-text">Loading requirements…</div>
            </div>
        );
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
    const fulfilledCount = allReqs.filter(
        (r) => itemEffectiveFulfilled(r) && itemTone(r, frozenIds) === "fulfilled",
    ).length;
    const plannedCount = allReqs.filter((r) => itemTone(r, frozenIds) === "frozen").length;
    const partialCount = allReqs.filter(
        (r) => !itemEffectiveFulfilled(r) && itemTone(r, frozenIds) === "partial",
    ).length;
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
                        const groupStatusTone = computeGroupTone(items, pool, frozenIds);
                        const isCollapsed = (() => {
                            if (navTarget?.category && normalizeCategory(navTarget.category) === cat) {
                                return false;
                            }
                            return collapsedGroups[cat] ?? true;
                        })();
                        const groupClass = groupClassForTone(groupStatusTone);
                        const pillLabel = poolStats
                            ? `${poolStats.slotsFilled}/${poolStats.slotsTotal}`
                            : formatCategoryProgress(items);

                        return (
                            <div key={cat} className={groupClass}>
                                <div
                                    className="req-group-header"
                                    onClick={() => setCollapsedGroups((p) => ({ ...p, [cat]: !(p[cat] ?? true) }))}
                                >
                                    <span className={`req-group-badge ${badgeClassForGroupTone(groupStatusTone)}`}>
                                        {groupBadgeIcon(groupStatusTone)}
                                    </span>
                                    <span className="req-group-name" title={cat}>{cat}</span>
                                    <span className={`req-group-pill ${pillClassForGroupTone(groupStatusTone)}`}>
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
                                                        scheduleCtx.degreeIndex,
                                                        pool.pool_index,
                                                        j,
                                                        flashRowId,
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
            const groupStatusTone = computeGroupTone(items, null, frozenIds);
            return (
                <div key={section.id} className={groupClassForTone(groupStatusTone)}>
                    {renderSuperGroupHeader(
                        section.title,
                        groupStatusTone,
                        isCollapsed,
                        toggle,
                        formatCategoryProgress(items),
                    )}
                    {!isCollapsed && (
                        <div className="req-group-body">
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
            const genEdItems = casGenEd
                ? [...(casGenEd.foundational_approaches || []), ...(casGenEd.sectors || [])].map((row) => ({
                    fulfilled: row.fulfilled,
                    fulfilledCourses: row.matched_courses || [],
                    requirement: row.name,
                }))
                : [];
            const groupStatusTone = computeGroupTone(genEdItems, pool, frozenIds);
            const pillLabel = casProgress
                ? `${casProgress.done}/${casProgress.total}`
                : poolStats
                    ? `${poolStats.covDone}/${poolStats.covTotal}`
                    : null;

            const genEdDegreeLabel = isCasCombined
                ? casIndices.map((i) => `${results[i].school}-${results[i].major}`).join(", ")
                : degreeLabel;

            return (
                <div key={section.id} className={groupClassForTone(groupStatusTone)}>
                    {renderSuperGroupHeader(section.title, groupStatusTone, isCollapsed, toggle, pillLabel)}
                    {!isCollapsed && (
                        <div className="req-group-body">
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
                                            section.degreeIndex,
                                            pool.pool_index,
                                            j,
                                            flashRowId,
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
            const majorTone = computeGroupTone(majorItems, null, frozenIds);

            return (
                <div
                    key={section.id}
                    className={`req-major-section ${majorSectionClassForTone(majorTone)}`}
                >
                    {renderMajorSectionTitle(
                        section.title,
                        majorTone,
                        formatCategoryProgress(majorItems),
                    )}
                    <div className="req-major-body">
                        {orderedCategories.map((cat) => {
                            const items = categoryMap[cat] || [];
                            if (!items.length) return null;
                            const groupStatusTone = computeGroupTone(items, null, frozenIds);
                            const innerCollapsed = (() => {
                                if (navTarget?.category && normalizeCategory(navTarget.category) === cat) {
                                    return false;
                                }
                                return collapsedGroups[cat] ?? true;
                            })();

                            return (
                                <div key={cat} className={groupClassForTone(groupStatusTone)}>
                                    <div
                                        className="req-group-header"
                                        onClick={() => setCollapsedGroups((p) => ({
                                            ...p,
                                            [cat]: !(p[cat] ?? true),
                                        }))}
                                    >
                                        <span className={`req-group-badge ${badgeClassForGroupTone(groupStatusTone)}`}>
                                            {groupBadgeIcon(groupStatusTone)}
                                        </span>
                                        <span className="req-group-name" title={cat}>{cat}</span>
                                        <span className={`req-group-pill ${pillClassForGroupTone(groupStatusTone)}`}>
                                            {formatCategoryProgress(items)}
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

function renderSuperGroupHeader(title, tone, isCollapsed, onToggle, pillLabel) {
    return (
        <div className="req-group-header" onClick={onToggle}>
            <span className={`req-group-badge ${badgeClassForGroupTone(tone)}`}>
                {groupBadgeIcon(tone)}
            </span>
            <span className="req-group-name">{title}</span>
            {pillLabel && (
                <span className={`req-group-pill ${pillClassForGroupTone(tone)}`}>
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

function renderMajorSectionTitle(title, tone, pillLabel) {
    return (
        <div className="req-major-title">
            <span className="req-major-title-text">{title}</span>
            {pillLabel && (
                <span className={`req-group-pill ${pillClassForGroupTone(tone)}`}>
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

function requirementHasFrozenCourse(item, frozenIds) {
    if (itemHasFrozenCourse(collectFulfillingCourses(item), frozenIds)) return true;

    if (isExpandableCourseGroup(item.requirement)) {
        return evaluateCourseGroupChildren(item).some(
            ({ fulfilledCourses, tone }) =>
                tone !== "open" && itemHasFrozenCourse(fulfilledCourses, frozenIds),
        );
    }

    if (isExpandableAnyOf(item.requirement)) {
        return getAnyOfPossibilities(item.requirement).some((childReq, childIdx) => {
            const child = makeAnyOfChildItem(item, childReq, childIdx);
            if (!child.fulfilled && !child.partial) return false;
            return itemHasFrozenCourse(collectFulfillingCourses(child), frozenIds);
        });
    }

    return false;
}

function itemTone(item, frozenIds) {
    if (isExpandableCourseGroup(item.requirement)) {
        if (requirementHasFrozenCourse(item, frozenIds)) return "frozen";
        if (!courseGroupIsComplete(item)) {
            const hasProgress = evaluateCourseGroupChildren(item).some((row) => row.tone !== "open");
            return hasProgress ? "partial" : "open";
        }
        return "fulfilled";
    }

    if (item.partial && collectFulfillingCourses(item).length > 0) return "partial";
    if (!item.fulfilled) return "open";
    if (requirementHasFrozenCourse(item, frozenIds)) return "frozen";
    return "fulfilled";
}

function computeGroupTone(items, pool, frozenIds) {
    const poolStats = pool ? poolGroupStats(pool) : null;
    const slotsOk = !poolStats || poolStats.slotsFilled >= poolStats.slotsTotal;
    const covOk = !poolStats || poolStats.covDone >= poolStats.covTotal;

    if (items.some((item) => requirementHasFrozenCourse(item, frozenIds))) return "frozen";
    if ((pool?.constraints || []).some(
        (c) => itemHasFrozenCourse(c.matched_courses || [], frozenIds),
    )) return "frozen";

    const allComplete = items.length === 0 || items.every((item) => itemEffectiveFulfilled(item));
    if (!allComplete || !slotsOk || !covOk) return "incomplete";
    if (items.some((item) => itemTone(item, frozenIds) === "partial")) return "incomplete";
    return "fulfilled";
}

function groupClassForTone(tone) {
    if (tone === "fulfilled") return "req-group req-group-done";
    if (tone === "frozen") return "req-group req-group-planned";
    return "req-group";
}

function majorSectionClassForTone(tone) {
    if (tone === "fulfilled") return "req-major-section-done";
    if (tone === "frozen") return "req-major-section-planned";
    return "";
}

function badgeClassForGroupTone(tone) {
    if (tone === "fulfilled") return "badge-done";
    if (tone === "frozen") return "badge-planned";
    return "badge-pending";
}

function pillClassForGroupTone(tone) {
    if (tone === "fulfilled") return "pill-done";
    if (tone === "frozen") return "pill-planned";
    return "pill-pending";
}

const SINGLE_COURSE_SCROLL_THRESHOLD = 3;

function courseDisplayPriority(courseId, { frozenIds, assignedIds, fulfillingSet }) {
    if (frozenIds.has(courseId)) return 0;
    if (fulfillingSet?.has(courseId) || assignedIds.has(courseId)) return 1;
    return 2;
}

function prioritizeCourseIds(ids, chipCtx) {
    return ids
        .map((id, index) => ({ id, index }))
        .sort((a, b) => {
            const priorityDiff =
                courseDisplayPriority(a.id, chipCtx) - courseDisplayPriority(b.id, chipCtx);
            if (priorityDiff !== 0) return priorityDiff;
            return a.index - b.index;
        })
        .map(({ id }) => id);
}

function ScrollableCourseChips({ itemCount, children }) {
    if (itemCount <= SINGLE_COURSE_SCROLL_THRESHOLD) {
        return <span className="req-chips">{children}</span>;
    }

    return (
        <span className="req-chips-scroll-wrap">
            <span className="req-chips req-chips--scrollable">
                {children}
            </span>
        </span>
    );
}

function renderCourseChipBadges(badges, chipCtx, crossDegreeChipTitle) {
    const renderBadge = (badge, key) => {
        const chipTitle = crossDegreeChipTitle?.(badge.id);
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

    const chips = badges.map((badge, i) => renderBadge(badge, badge.id ?? i));
    return (
        <ScrollableCourseChips itemCount={badges.length}>
            {chips}
        </ScrollableCourseChips>
    );
}

function groupBadgeIcon(tone) {
    if (tone === "fulfilled") return "✓";
    if (tone === "frozen") return "◐";
    return "·";
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

function buildRowContent(item, sortCtx = null) {
    const { type, data } = parseRequirement(item.requirement);
    const stem = getRequirementStem(item.requirement);
    const fulfilling = collectFulfillingCourses(item);
    const fulfillingSet = new Set(fulfilling);
    const chipSortCtx = sortCtx
        ? { ...sortCtx, fulfillingSet, partialTone: sortCtx.partialTone ?? item.partial }
        : null;

    if (type === "SingleCourse") {
        const possibilities = (data.possibilities || []).filter(Boolean);
        if (possibilities.length <= 1) {
            const ids = possibilities.length ? possibilities : fulfilling;
            return { stem: null, badges: ids.map((id) => ({ kind: "course", id })), fulfillingSet, scrollableChips: false };
        }
        const sorted = chipSortCtx
            ? prioritizeCourseIds(possibilities, chipSortCtx)
            : possibilities;
        return {
            stem,
            badges: sorted.map((id) => ({ kind: "course", id })),
            fulfillingSet,
            scrollableChips: sorted.length > SINGLE_COURSE_SCROLL_THRESHOLD,
        };
    }

    if (type === "CourseGroup") {
        return { stem: null, badges: [], fulfillingSet, scrollableChips: false };
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
            scrollableChips: false,
        };
    }

    if (type === "Restriction") {
        return { stem, badges: fulfilling.map((id) => ({ kind: "course", id })), fulfillingSet, scrollableChips: false };
    }

    if (type === "AllOf") {
        const parts = (data.requirements || [])
            .map((sub) => createRequirementDescription(sub))
            .filter(Boolean);
        if (parts.length > 0) {
            return { stem: null, badges: [], fulfillingSet, scrollableChips: false };
        }
    }

    return { stem, badges: fulfilling.map((id) => ({ kind: "course", id })), fulfillingSet, scrollableChips: false };
}

function renderRequirementLine(item, scheduleCtx, crossDegreeChipTitle = () => undefined) {
    const chipCtx = {
        ...scheduleCtx,
        partialTone: scheduleCtx.partialTone ?? item.partial,
    };
    const { stem, badges, fulfillingSet, scrollableChips } = buildRowContent(item, chipCtx);
    chipCtx.fulfillingSet = fulfillingSet;
    const fullDesc = createRequirementDescription(item.requirement);

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
                {scrollableChips
                    ? renderCourseChipBadges(badges, chipCtx, crossDegreeChipTitle)
                    : (
                        <span className="req-chips">
                            {badges.map((b, i) => renderBadge(b, i))}
                        </span>
                    )}
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
    if (tone === "frozen") return "◐";
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
    const { type } = parseRequirement(childReq);
    const options = courseOptionsForArea(childReq);
    const fulfillingSet = new Set(fulfilledCourses);
    const chipCtx = {
        ...scheduleCtx,
        fulfillingSet,
        partialTone: areaTone === "partial",
    };
    const sortedOptions = type === "SingleCourse" && options.length > 1
        ? prioritizeCourseIds(options, chipCtx)
        : options;
    const badges = sortedOptions.map((id) => ({ kind: "course", id }));
    const scrollableChips = type === "SingleCourse"
        && sortedOptions.length > SINGLE_COURSE_SCROLL_THRESHOLD;

    return (
        <div className="req-item-line">
            <span className="req-stem-text">{getCourseGroupAreaLabel(childReq)}</span>
            {options.length > 0 && (
                <>
                    <span className="req-item-colon">:</span>
                    {scrollableChips
                        ? renderCourseChipBadges(badges, chipCtx, scheduleCtx.crossDegreeChipTitle)
                        : (
                            <span className="req-chips">
                                {sortedOptions.map((id, i) => (
                                    <span
                                        key={i}
                                        className={chipClass(badgeKindFor(id, chipCtx))}
                                        title={scheduleCtx.crossDegreeChipTitle?.(id)}
                                    >
                                        {id}
                                    </span>
                                ))}
                            </span>
                        )}
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
                    const childItem = {
                        fulfilled: tone === "fulfilled",
                        partial: tone === "partial",
                        fulfilledCourses,
                        requirement: childReq,
                    };
                    const childTone = tone === "open"
                        ? "open"
                        : itemTone(childItem, scheduleCtx.frozenIds);
                    return (
                        <div
                            key={childRowId}
                            id={childRowId}
                            className={`req-item req-item--${childTone} req-anyof-child ${flashRowId === childRowId ? "req-row-flash" : ""}`}
                        >
                            <span className={`req-item-icon icon-${childTone}`}>
                                {childTone === "fulfilled" ? "✓" : childTone === "partial" ? "◐" : childTone === "frozen" ? "◐" : "•"}
                            </span>
                            <div className="req-item-body">
                                {renderCourseGroupAreaLine(
                                    childReq,
                                    childTone,
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
        <>
            <div className="req-pool-divider req-item-first">Foundational Approaches</div>
            {(casGenEd.foundational_approaches || []).map((row, idx) =>
                renderCasGenEdRow(
                    row,
                    null,
                    scheduleCtx,
                    filterCourses,
                    degreeIndex,
                    idx === 0,
                ),
            )}
            <div className="req-pool-divider">Sectors of Knowledge</div>
            {(casGenEd.sectors || []).map((row) =>
                renderCasGenEdRow(
                    row,
                    majorDisplayName,
                    scheduleCtx,
                    filterCourses,
                    degreeIndex,
                    false,
                ),
            )}
        </>
    );
}

function renderCasGenEdRow(row, majorDisplayName, scheduleCtx, filterCourses, degreeIndex, isFirst) {
    const rowTone = row.fulfilled ? "fulfilled" : "open";
    const courses = filterCourses(row.matched_courses || []);
    const fulfillingSet = new Set(courses);

    return (
        <div
            key={`${row.attr}-${row.name}`}
            className={`req-item req-item--${rowTone} ${isFirst ? "req-item-first" : ""}`}
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

function renderPoolConstraintItem(
    constraint,
    rowKey,
    isFirst,
    scheduleCtx,
    degreeIndex,
    poolIndex,
    constraintIndex,
    flashRowId,
) {
    const instanceId = poolConstraintInstanceId(poolIndex, constraintIndex);
    const rowDomId = reqRowDomId(degreeIndex, instanceId);
    const rowTone = constraint.fulfilled ? "fulfilled" : "open";
    const fulfillingSet = new Set(constraint.matched_courses || []);
    const chipCtx = {
        ...scheduleCtx,
        fulfillingSet,
        partialTone: false,
    };

    const fauxItem = {
        requirement: constraint.requirement,
        fulfilledCourses: constraint.matched_courses || [],
        attributeFulfillment: null,
    };
    const lineContent = constraint.requirement
        ? buildPoolConstraintLineContent(constraint, fauxItem, chipCtx, scheduleCtx.crossDegreeChipTitle)
        : buildPoolConstraintFallbackLine(constraint, chipCtx, scheduleCtx.crossDegreeChipTitle);

    return (
        <div
            id={rowDomId}
            key={rowKey}
            className={`req-item req-item--${rowTone} ${isFirst ? "req-item-first" : ""} ${flashRowId === rowDomId ? "req-row-flash" : ""}`}
        >
            <span className={`req-item-icon icon-${rowTone}`}>
                {constraint.fulfilled ? "✓" : "○"}
            </span>
            <div className="req-item-body">
                {lineContent}
            </div>
        </div>
    );
}

function buildPoolConstraintFallbackLine(constraint, chipCtx, crossDegreeChipTitle) {
    return (
        <div className="req-item-line">
            <span className="req-stem-text">
                {constraint.description || constraint.label}
            </span>
            {constraint.matched_courses?.length > 0 && (
                <>
                    <span className="req-item-colon">:</span>
                    {renderCourseChipBadges(
                        constraint.matched_courses.map((id) => ({ kind: "course", id })),
                        chipCtx,
                        crossDegreeChipTitle,
                    )}
                </>
            )}
        </div>
    );
}

function buildPoolConstraintLineContent(constraint, fauxItem, chipCtx, crossDegreeChipTitle) {
    const { type, data } = parseRequirement(constraint.requirement);
    const stem = (constraint.label && constraint.label !== "constraint")
        ? constraint.label
        : getRequirementStem(constraint.requirement);

    if (type === "SingleCourse") {
        const possibilities = (data.possibilities || []).filter(Boolean);
        if (possibilities.length > 1) {
            const sorted = prioritizeCourseIds(possibilities, chipCtx);
            const badges = sorted.map((id) => ({ kind: "course", id }));
            const scrollable = sorted.length > SINGLE_COURSE_SCROLL_THRESHOLD;
            return (
                <div className="req-item-line">
                    <span className="req-stem-text">{stem || "One of"}</span>
                    <span className="req-item-colon">:</span>
                    {scrollable
                        ? renderCourseChipBadges(badges, chipCtx, crossDegreeChipTitle)
                        : (
                            <span className="req-chips">
                                {badges.map((b, i) => (
                                    <span
                                        key={b.id ?? i}
                                        className={chipClass(badgeKindFor(b.id, chipCtx))}
                                        title={crossDegreeChipTitle?.(b.id)}
                                    >
                                        {b.id}
                                    </span>
                                ))}
                            </span>
                        )}
                </div>
            );
        }
    }

    if (type === "Restriction" && data.attr?.length > 0) {
        const badges = data.attr.map((code) => ({
            kind: "attr",
            code,
            courses: (constraint.matched_courses || []).filter(
                (c) => chipCtx.fulfillingSet?.has(c),
            ),
        }));
        const renderBadge = (badge, key) => {
            const fulfillingForAttr = badge.courses.filter((c) => chipCtx.fulfillingSet.has(c));
            const hasCourse = fulfillingForAttr.length > 0;
            const label = hasCourse
                ? `[${badge.code}] ${fulfillingForAttr.join(", ")}`
                : `[${badge.code}]`;
            const kind = hasCourse ? badgeKindFor(fulfillingForAttr[0], chipCtx) : "open";
            return <span key={key} className={chipClass(kind)}>{label}</span>;
        };
        return (
            <div className="req-item-line">
                <span className="req-stem-text">{stem || createRequirementDescription(constraint.requirement)}</span>
                <span className="req-item-colon">:</span>
                <span className="req-chips">
                    {badges.map((b, i) => renderBadge(b, i))}
                </span>
            </div>
        );
    }

    const { stem: builtStem, badges, scrollableChips } = buildRowContent(fauxItem, chipCtx);
    if (builtStem && badges.length > 0) {
        return (
            <div className="req-item-line">
                <span className="req-stem-text">{stem || builtStem}</span>
                <span className="req-item-colon">:</span>
                {scrollableChips
                    ? renderCourseChipBadges(badges, chipCtx, crossDegreeChipTitle)
                    : (
                        <span className="req-chips">
                            {badges.map((b, i) => (
                                <span
                                    key={b.id ?? i}
                                    className={chipClass(badgeKindFor(b.id, chipCtx))}
                                    title={crossDegreeChipTitle?.(b.id)}
                                >
                                    {b.id}
                                </span>
                            ))}
                        </span>
                    )}
            </div>
        );
    }

    return buildPoolConstraintFallbackLine(constraint, chipCtx, crossDegreeChipTitle);
}
