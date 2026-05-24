"use client";

import { useState, useEffect } from "react";
import {
    filterValidCourseCodes,
    filterValidPlacements,
    filterFrozenPlacements,
    isValidCourseCode,
} from "@/lib/courseUtils";
import {
    createRequirementDescription,
    createRequirementPanelDescription,
    getRequirementInstanceId,
    parseRequirement,
} from "@/lib/requirementText";
import { reqRowDomId, attributeFulfillmentMap } from "@/lib/requirementNav";

// ─── Design tokens ───
const C = {
    gray50: "#f8fafc",
    gray100: "#f1f5f9",
    gray200: "#e2e8f0",
    gray300: "#cbd5e1",
    gray400: "#94a3b8",
    gray500: "#64748b",
    gray700: "#374151",
    gray900: "#111827",
    green50: "#f0fdf4",
    green100: "#dcfce7",
    green300: "#86efac",
    green400: "#4ade80",
    green600: "#16a34a",
    green700: "#15803d",
    teal600: "#059669",
    red500: "#dc2626",
    amber50: "#fffbeb",
    amber200: "#fde68a",
    amber500: "#f59e0b",
    amber700: "#b45309",
    white: "#ffffff",
};

// ─── Inline style objects ───
const S = {
    wrap: { display: "flex", flexDirection: "column", gap: 12, minHeight: 0, height: "100%" },

    // tabs
    tabs: { display: "flex", gap: 6, overflowX: "auto", paddingBottom: 2, flexShrink: 0 },
    tab: { padding: "6px 14px", background: "rgba(0,0,0,0.04)", border: "1px solid rgba(0,0,0,0.1)", borderRadius: 8, color: C.gray500, fontSize: "0.76rem", fontWeight: 600, cursor: "pointer", whiteSpace: "nowrap", fontFamily: "inherit" },
    tabActive: { padding: "6px 14px", background: "rgba(5,150,105,0.1)", border: `1px solid ${C.teal600}`, borderRadius: 8, color: C.teal600, fontSize: "0.76rem", fontWeight: 600, cursor: "pointer", whiteSpace: "nowrap", fontFamily: "inherit" },

    // error
    error: { padding: "10px 14px", background: "rgba(220,38,38,0.06)", border: "1px solid rgba(220,38,38,0.2)", borderRadius: 8, fontSize: "0.8rem", color: C.red500 },

    // summary
    summary: { display: "flex", flexDirection: "column", gap: 8, padding: "12px 14px", background: "rgba(0,0,0,0.03)", border: "1px solid rgba(0,0,0,0.08)", borderRadius: 8, flexShrink: 0 },
    summStats: { display: "flex", alignItems: "center", gap: 14, flexWrap: "wrap" },
    statOk: { display: "flex", alignItems: "center", gap: 6, fontSize: "0.78rem", fontWeight: 600, color: C.green600 },
    statPlanned: { display: "flex", alignItems: "center", gap: 6, fontSize: "0.78rem", fontWeight: 600, color: C.amber700 },
    statRemaining: { display: "flex", alignItems: "center", gap: 6, fontSize: "0.78rem", fontWeight: 600, color: C.gray500 },
    statPct: { marginLeft: "auto", fontSize: "0.78rem", fontWeight: 700, color: C.gray500 },
    dot: (color) => ({ width: 7, height: 7, borderRadius: "50%", background: color, flexShrink: 0 }),
    track: { height: 5, background: "rgba(0,0,0,0.08)", borderRadius: 3, overflow: "hidden", display: "flex" },
    progressFulfilled: (pct) => ({
        height: "100%",
        minWidth: pct > 0 ? 4 : 0,
        width: `${pct}%`,
        background: `linear-gradient(90deg, ${C.green600}, ${C.teal600})`,
        transition: "width 0.5s ease",
    }),
    progressPlanned: (pct) => ({
        height: "100%",
        minWidth: pct > 0 ? 4 : 0,
        width: `${pct}%`,
        background: `linear-gradient(90deg, ${C.amber500}, ${C.amber700})`,
        transition: "width 0.5s ease",
    }),

    // groups — flexShrink: 0 on children prevents accordion rows squashing to ~2px
    groups: { display: "flex", flexDirection: "column", gap: 8, flex: 1, minHeight: 0, overflowY: "auto", paddingRight: 2 },
    group: (tone) => ({
        border: `1px solid ${tone === "fulfilled" ? C.green300 : tone === "frozen" ? C.amber200 : C.gray200}`,
        borderRadius: 8,
        overflow: "hidden",
        flexShrink: 0,
    }),

    // group header
    groupHdr: (tone) => ({
        display: "flex", alignItems: "center", gap: 8,
        padding: "10px 14px",               // ← explicit padding, not from CSS
        background: tone === "fulfilled" ? C.green50 : tone === "frozen" ? C.amber50 : C.gray100,
        borderBottom: `1px solid ${tone === "fulfilled" ? C.green300 : tone === "frozen" ? C.amber200 : C.gray200}`,
        cursor: "pointer", userSelect: "none",
    }),
    groupBadge: (tone) => ({
        width: 20, height: 20, borderRadius: "50%", flexShrink: 0,
        display: "flex", alignItems: "center", justifyContent: "center",
        background: tone === "fulfilled" ? C.green100 : tone === "frozen" ? C.amber50 : C.gray200,
        color: tone === "fulfilled" ? C.green600 : tone === "frozen" ? C.amber700 : C.gray400,
        fontSize: tone === "incomplete" ? "1rem" : "0.65rem",
        fontWeight: 800, lineHeight: tone === "incomplete" ? 0.85 : 1,
    }),
    groupName: { flex: 1, fontSize: "0.82rem", fontWeight: 700, color: C.gray900 },
    groupPill: (tone) => ({
        fontSize: "0.68rem", fontWeight: 700, padding: "2px 10px", borderRadius: 10, flexShrink: 0,
        background: tone === "fulfilled" ? C.green100 : tone === "frozen" ? C.amber50 : C.gray100,
        color: tone === "fulfilled" ? C.green600 : tone === "frozen" ? C.amber700 : C.gray500,
        border: `1px solid ${tone === "fulfilled" ? C.green300 : tone === "frozen" ? C.amber200 : C.gray300}`,
    }),
    chevron: { fontSize: "0.6rem", color: C.gray400, marginLeft: 2 },

    // group body
    groupBody: { display: "flex", flexDirection: "column", background: C.white },

    // items — ALL INLINE so global reset cannot collapse them
    item: (tone, isFirst) => ({
        display: "flex",
        alignItems: "flex-start",
        gap: 10,
        padding: "10px 14px",              // ← explicit, immune to CSS reset
        minHeight: 40,                     // ← floor so rows are never invisible
        flexShrink: 0,
        background: tone === "fulfilled" ? "#f8fff8" : tone === "frozen" ? C.amber50 : C.white,
        borderTop: isFirst ? "none" : `1px solid ${C.gray100}`,
        borderLeft: `3px solid ${tone === "fulfilled" ? C.green300 : tone === "frozen" ? C.amber200 : C.gray200}`,
        boxSizing: "border-box",
    }),
    itemIcon: (tone) => ({
        flexShrink: 0, fontSize: "0.72rem", fontWeight: 800,
        marginTop: 2, width: 14, textAlign: "center",
        color: tone === "fulfilled" ? C.green600 : tone === "frozen" ? C.amber700 : C.gray300,
    }),
    itemBody: { flex: 1, minWidth: 0, display: "flex", flexDirection: "column", gap: 5 },
    itemDescRow: { display: "flex", flexWrap: "wrap", gap: 6, alignItems: "center" },
    itemDesc: { fontSize: "0.78rem", fontWeight: 600, color: C.gray700, lineHeight: 1.4 },

    // chips
    chips: { display: "flex", flexWrap: "wrap", gap: 4, alignItems: "center" },
    chip: (kind) => {
        const map = {
            default: { bg: C.gray100, border: C.gray300, color: C.gray500 },
            fulfilled: { bg: C.green100, border: C.green300, color: C.green700 },
            suggested: { bg: "#d1fae5", border: "#6ee7b7", color: "#047857" },
            frozen: { bg: C.amber50, border: C.amber200, color: C.amber700 },
        };
        const t = map[kind] || map.default;
        return { fontSize: "0.67rem", fontWeight: 600, padding: "2px 7px", borderRadius: 4, whiteSpace: "nowrap", background: t.bg, border: `1px solid ${t.border}`, color: t.color, boxSizing: "border-box" };
    },
    expandBtn: { fontSize: "0.65rem", fontWeight: 600, padding: "2px 8px", borderRadius: 4, background: "none", border: `1px dashed ${C.gray300}`, color: C.gray400, cursor: "pointer", fontFamily: "inherit" },

    // empty
    empty: { textAlign: "center", padding: "36px 20px", color: C.gray400, fontSize: "0.82rem" },
};

export default function RequirementsPanel({
    scheduleData,
    degrees,
    frozenCourses = [],
    assignedCourses = [],
    navTarget = null,
    onNavTargetConsumed,
}) {
    const [activeTab, setActiveTab] = useState(0);
    const [expandedOptions, setExpandedOptions] = useState({});
    const [collapsedGroups, setCollapsedGroups] = useState({});
    const [flashRowId, setFlashRowId] = useState(null);

    useEffect(() => {
        if (!navTarget) return;
        const { degreeIndex, instanceId, category } = navTarget;
        if (degreeIndex != null) setActiveTab(degreeIndex);
        if (category) {
            setCollapsedGroups((prev) => ({ ...prev, [category]: false }));
        }
        const rowId = reqRowDomId(degreeIndex ?? 0, instanceId);
        const timer = window.setTimeout(() => {
            const el = document.getElementById(rowId);
            el?.scrollIntoView({ behavior: "smooth", block: "nearest" });
            setFlashRowId(rowId);
            window.setTimeout(() => setFlashRowId(null), 2200);
            onNavTargetConsumed?.();
        }, 80);
        return () => window.clearTimeout(timer);
    }, [navTarget, onNavTargetConsumed]);

    if (!degrees || degrees.length === 0) {
        return <div style={S.empty}><div style={{ fontSize: "2rem", marginBottom: 8 }}>📋</div>Add degrees to see requirement fulfillment</div>;
    }
    if (!scheduleData || !scheduleData.degree_results) {
        return <div style={S.empty}>Loading requirements…</div>;
    }

    const results = scheduleData.degree_results;
    const tabIndex = Math.min(activeTab, results.length - 1);
    const current = results[tabIndex];
    if (!current) return null;

    // Build requirement list
    const allReqs = [];
    (current.fulfilled_requirements || []).forEach((mapped) => {
        const cat = normalizeCategory(getCategory(mapped.requirement));
        allReqs.push({
            category: cat,
            fulfilled: true,
            fulfilledCourses: mapped.course_ids || [],
            requirement: mapped.requirement,
            instanceId: getRequirementInstanceId(mapped),
            attributeFulfillment: attributeFulfillmentMap(mapped),
        });
    });
    const suggestionsMap = {};
    (current.suggested_for_unfulfilled || []).forEach((mapped) => {
        const cat = normalizeCategory(getCategory(mapped.requirement));
        const id = getRequirementInstanceId(mapped);
        suggestionsMap[`${cat}::${id}`] = mapped.course_ids || [];
    });
    (current.unfulfilled_requirements || []).forEach((mapped, rowIdx) => {
        const req = mapped?.requirement ?? mapped;
        const cat = normalizeCategory(getCategory(req));
        const id = getRequirementInstanceId(mapped) ?? `u-${rowIdx}`;
        allReqs.push({
            category: cat,
            fulfilled: false,
            fulfilledCourses: [],
            suggestedCourses: suggestionsMap[`${cat}::${id}`] || [],
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

    const assignedIds = new Set(
        filterValidPlacements(assignedCourses).map((a) => a.courseId)
    );
    const frozenIds = new Set(
        filterFrozenPlacements(frozenCourses).map((f) => f.courseId)
    );

    const totalCount = allReqs.length;
    const fulfilledCount = allReqs.filter(
        (r) => reqIsFulfilled(r) && itemTone(r, frozenIds) === "fulfilled"
    ).length;
    const plannedCount = allReqs.filter(
        (r) => reqIsFulfilled(r) && itemTone(r, frozenIds) === "frozen"
    ).length;
    const remainingCount = totalCount - fulfilledCount - plannedCount;
    const fulfilledPct = totalCount > 0 ? (fulfilledCount / totalCount) * 100 : 0;
    const plannedPct = totalCount > 0 ? (plannedCount / totalCount) * 100 : 0;
    const pct = totalCount > 0 ? Math.round(((fulfilledCount + plannedCount) / totalCount) * 100) : 0;

    const toggleExpand = (key) => setExpandedOptions((p) => ({ ...p, [key]: !p[key] }));
    const isGroupCollapsed = (cat) => collapsedGroups[cat] ?? true;
    const toggleGroup = (cat) =>
        setCollapsedGroups((p) => ({ ...p, [cat]: !(p[cat] ?? true) }));

    return (
        <div style={S.wrap}>

            {/* Tabs */}
            {results.length > 1 && (
                <div style={S.tabs}>
                    {results.map((result, i) => (
                        <button key={i} style={tabIndex === i ? S.tabActive : S.tab} onClick={() => setActiveTab(i)}>
                            {degrees[i]?.displayMajor || `${result.school} — ${result.major}`}
                        </button>
                    ))}
                </div>
            )}

            {/* Error */}
            {current.error && <div style={S.error}>⚠️ {current.error}</div>}

            {/* Progress summary */}
            {!current.error && totalCount > 0 && (
                <div style={S.summary}>
                    <div style={S.summStats}>
                        <span style={S.statOk}><span style={S.dot(C.green600)} />{fulfilledCount} fulfilled</span>
                        <span style={S.statPlanned}><span style={S.dot(C.amber500)} />{plannedCount} planned</span>
                        <span style={S.statRemaining}><span style={S.dot(C.gray400)} />{remainingCount} remaining</span>
                        <span style={S.statPct}>{pct}%</span>
                    </div>
                    <div style={S.track}>
                        {fulfilledPct > 0 && <div style={S.progressFulfilled(fulfilledPct)} />}
                        {plannedPct > 0 && <div style={S.progressPlanned(plannedPct)} />}
                    </div>
                </div>
            )}

            {/* Category groups */}
            <div style={S.groups}>
                {orderedCategories.map((cat) => {
                    const items = categoryMap[cat];
                    if (!items || items.length === 0) return null;

                    const done = items.filter((r) => reqIsFulfilled(r)).length;
                    const total = items.length;
                    const catTone = groupTone(items, frozenIds);
                    const isCollapsed = isGroupCollapsed(cat);

                    return (
                        <div key={cat} style={S.group(catTone)}>
                            <div style={S.groupHdr(catTone)} onClick={() => toggleGroup(cat)}>
                                <span style={S.groupBadge(catTone)}>{done === total ? "✓" : "·"}</span>
                                <span style={S.groupName}>{cat}</span>
                                <span style={S.groupPill(catTone)}>{done}/{total}</span>
                                <span style={S.chevron}>{isCollapsed ? "▶" : "▾"}</span>
                            </div>

                            {!isCollapsed && (
                                <div style={S.groupBody}>
                                    {items.map((item, rowIdx) => {
                                        const expandKey = `${cat}-${item.instanceId ?? rowIdx}`;
                                        const desc = getItemDescription(item);
                                        const sameDescCount = items.filter(
                                            (o) => getItemDescription(o) === desc
                                        ).length;
                                        const sameDescIndex = items
                                            .filter((o) => getItemDescription(o) === desc)
                                            .indexOf(item);
                                        return renderItem(
                                            item,
                                            item.instanceId ?? String(rowIdx),
                                            expandKey,
                                            expandedOptions[expandKey],
                                            () => toggleExpand(expandKey),
                                            { assignedIds, frozenIds },
                                            rowIdx === 0,
                                            sameDescCount > 1
                                                ? `${desc} (${sameDescIndex + 1}/${sameDescCount})`
                                                : desc,
                                            tabIndex,
                                            flashRowId
                                        );
                                    })}
                                </div>
                            )}
                        </div>
                    );
                })}

                {totalCount === 0 && !current.error && (
                    <div style={S.empty}>No requirement data available for this program</div>
                )}
            </div>
        </div>
    );
}


/** API fulfillment (includes frozen/planned courses); chip colors still distinguish taken vs frozen. */
function reqIsFulfilled(item) {
    return item.fulfilled;
}

/** @typedef {"open" | "frozen" | "fulfilled"} ReqVisualTone */

function collectFulfillingCourses(item) {
    const courses = filterValidCourseCodes(item.fulfilledCourses || []);
    if (item.attributeFulfillment) {
        item.attributeFulfillment.forEach((ids) => {
            ids.forEach((c) => {
                if (isValidCourseCode(c)) courses.push(c);
            });
        });
    }
    return [...new Set(courses)];
}

function itemHasFrozenCourse(courses, frozenIds) {
    return courses.some((c) => frozenIds.has(c));
}

/** Row / group styling: orange when fulfilled via a frozen (planned) course. */
function itemTone(item, frozenIds) {
    if (!reqIsFulfilled(item)) return "open";
    if (itemHasFrozenCourse(collectFulfillingCourses(item), frozenIds)) return "frozen";
    return "fulfilled";
}

function groupTone(items, frozenIds) {
    const fulfilledCount = items.filter((r) => reqIsFulfilled(r)).length;
    if (fulfilledCount !== items.length) return "incomplete";
    if (items.some((item) => itemTone(item, frozenIds) === "frozen")) return "frozen";
    return "fulfilled";
}

function chipKindFor(courseId, { assignedIds, frozenIds, fulfilledSet, suggestedSet }) {
    if (assignedIds.has(courseId)) return "fulfilled";
    if (frozenIds.has(courseId)) return "frozen";
    if (fulfilledSet.has(courseId)) return "fulfilled";
    if (suggestedSet.has(courseId)) return "suggested";
    return "default";
}

function getItemDescription(item) {
    return createRequirementPanelDescription(item.requirement, { fulfilled: item.fulfilled });
}

function renderItem(
    item,
    idx,
    expandKey,
    isExpanded,
    onToggle,
    { assignedIds, frozenIds },
    isFirst = false,
    descriptionOverride = null,
    degreeIndex = 0,
    flashRowId = null
) {
    const { type, data } = parseRequirement(item.requirement);
    const options = getOptions(type, data);
    const fulfilledCourses = filterValidCourseCodes(item.fulfilledCourses || []);
    const suggestedCourses = filterValidCourseCodes(item.suggestedCourses || []);
    const fulfilledSet = new Set(fulfilledCourses);
    const suggestedSet = new Set(suggestedCourses);
    const chipCtx = { assignedIds, frozenIds, fulfilledSet, suggestedSet };
    const rowFulfilled = reqIsFulfilled(item);
    const rowTone = itemTone(item, frozenIds);
    const attrFulfillment = item.attributeFulfillment;
    const hasAttrFulfillment = attrFulfillment && attrFulfillment.size > 0;
    const rowDomId = reqRowDomId(degreeIndex, idx);
    const isFlashing = flashRowId === rowDomId;

    const MAX_VISIBLE = 5;
    const visible = isExpanded ? options : options.slice(0, MAX_VISIBLE);
    const fulfillingCourses = rowFulfilled ? collectFulfillingCourses(item) : [];
    const showInlineCourses = rowFulfilled && fulfillingCourses.length > 0 && !hasAttrFulfillment;
    const showOptionChips = !rowFulfilled && options.length > 0 && !(type === "Restriction" && data.attr?.length > 0);

    const renderCourseChip = (courseId, key) => (
        <span key={key ?? courseId} style={S.chip(chipKindFor(courseId, chipCtx))}>
            {courseId}
        </span>
    );

    const renderAttrChip = (attrCode) => {
        const matchedCourses = (attrFulfillment?.get(attrCode) || []).filter(isValidCourseCode);
        const isAttrFulfilled = matchedCourses.length > 0;
        const label = isAttrFulfilled
            ? `[${attrCode}] - ${matchedCourses.join(", ")}`
            : `[${attrCode}]`;
        return (
            <span
                key={attrCode}
                style={S.chip(isAttrFulfilled ? chipKindFor(matchedCourses[0], chipCtx) : "default")}
            >
                {label}
            </span>
        );
    };

    return (
        <div
            key={String(idx)}
            id={rowDomId}
            className={isFlashing ? "req-row-flash" : undefined}
            style={S.item(rowTone, isFirst)}
        >
            <span style={S.itemIcon(rowTone)}>{rowFulfilled ? "✓" : "○"}</span>
            <div style={S.itemBody}>
                <div style={S.itemDescRow}>
                    <span style={S.itemDesc}>
                        {descriptionOverride ?? createRequirementPanelDescription(item.requirement, { fulfilled: rowFulfilled })}
                    </span>
                    {showInlineCourses && fulfillingCourses.map((c) => renderCourseChip(c))}
                </div>

                {type === "Restriction" && data.attr?.length > 0 && (
                    <div style={S.chips}>
                        {data.attr.map(renderAttrChip)}
                    </div>
                )}

                {showOptionChips && (
                    <div style={S.chips}>
                        {visible.map((opt, i) => (
                            <span key={i} style={S.chip(chipKindFor(opt, chipCtx))}>
                                {opt}
                            </span>
                        ))}
                        {options.length > MAX_VISIBLE && (
                            <button style={S.expandBtn} onClick={onToggle}>
                                {isExpanded ? "Show less ↑" : `+${options.length - MAX_VISIBLE} more`}
                            </button>
                        )}
                    </div>
                )}

                {!rowFulfilled && suggestedCourses.length > 0 && options.length === 0 && (
                    <div style={S.chips}>
                        {suggestedCourses.map((c, i) => renderCourseChip(c, i))}
                    </div>
                )}
            </div>
        </div>
    );
}

// ─── Helpers ───

function formatAnyOfOptionLabel(sub) {
    return createRequirementDescription(sub);
}

function getOptions(type, data) {
    switch (type) {
        case "SingleCourse":
            return (data.possibilities || []).filter(Boolean);
        case "CourseGroup": return data.possibilities || [];
        case "Restriction": return data.attr?.length > 0 ? data.attr.map(a => `[${a}]`) : [];
        case "AnyOf":
            return (data.possibilities || []).map(formatAnyOfOptionLabel).filter(Boolean);
        case "AllOf":
            return (data.requirements || [])
                .map((sub) => createRequirementDescription(sub))
                .filter(Boolean);
        default: return [];
    }
}

function normalizeCategory(cat) {
    if (!cat || typeof cat !== "string" || !cat.trim()) return "Other";
    return cat.trim();
}

function getCategory(req) {
    if (!req) return "Other";
    if (req.category) return req.category;
    const variants = ["SingleCourse", "CourseGroup", "AnyOf", "AllOf", "Concentration", "Restriction", "DoubleCount"];
    for (const v of variants) { if (req[v]?.category) return req[v].category; }
    return "Other";
}
