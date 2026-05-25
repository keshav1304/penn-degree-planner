"use client";

import { useState, useEffect } from "react";
import {
    filterValidCourseCodes,
    filterValidPlacements,
    filterFrozenPlacements,
    isValidCourseCode,
} from "@/lib/courseUtils";
import {
    buildCourseCuMap,
    childMatchesAnyOfFulfillment,
    getAnyOfPossibilities,
    getRequirementInstanceId,
    getRequirementStem,
    isExpandableAnyOf,
    parseRequirement,
    scheduleCoursesFulfillingRestriction,
} from "@/lib/requirementText";
import { reqRowDomId, attributeFulfillmentMap } from "@/lib/requirementNav";

// ─── Design tokens ───
const C = {
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

const S = {
    wrap: { display: "flex", flexDirection: "column", gap: 12, minHeight: 0, height: "100%" },
    tabs: { display: "flex", gap: 6, overflowX: "auto", paddingBottom: 2, flexShrink: 0 },
    tab: { padding: "6px 14px", background: "rgba(0,0,0,0.04)", border: "1px solid rgba(0,0,0,0.1)", borderRadius: 8, color: C.gray500, fontSize: "0.76rem", fontWeight: 600, cursor: "pointer", whiteSpace: "nowrap", fontFamily: "inherit" },
    tabActive: { padding: "6px 14px", background: "rgba(5,150,105,0.1)", border: `1px solid ${C.teal600}`, borderRadius: 8, color: C.teal600, fontSize: "0.76rem", fontWeight: 600, cursor: "pointer", whiteSpace: "nowrap", fontFamily: "inherit" },
    error: { padding: "10px 14px", background: "rgba(220,38,38,0.06)", border: "1px solid rgba(220,38,38,0.2)", borderRadius: 8, fontSize: "0.8rem", color: C.red500 },
    summary: { display: "flex", flexDirection: "column", gap: 8, padding: "12px 14px", background: "rgba(0,0,0,0.03)", border: "1px solid rgba(0,0,0,0.08)", borderRadius: 8, flexShrink: 0 },
    summStats: { display: "flex", alignItems: "center", gap: 14, flexWrap: "wrap" },
    statOk: { display: "flex", alignItems: "center", gap: 6, fontSize: "0.78rem", fontWeight: 600, color: C.green600 },
    statPlanned: { display: "flex", alignItems: "center", gap: 6, fontSize: "0.78rem", fontWeight: 600, color: C.amber700 },
    statRemaining: { display: "flex", alignItems: "center", gap: 6, fontSize: "0.78rem", fontWeight: 600, color: C.gray500 },
    statPct: { marginLeft: "auto", fontSize: "0.78rem", fontWeight: 700, color: C.gray500 },
    dot: (color) => ({ width: 7, height: 7, borderRadius: "50%", background: color, flexShrink: 0 }),
    track: { height: 5, background: "rgba(0,0,0,0.08)", borderRadius: 3, overflow: "hidden", display: "flex" },
    progressFulfilled: (pct) => ({ height: "100%", minWidth: pct > 0 ? 4 : 0, width: `${pct}%`, background: `linear-gradient(90deg, ${C.green600}, ${C.teal600})`, transition: "width 0.5s ease" }),
    progressPlanned: (pct) => ({ height: "100%", minWidth: pct > 0 ? 4 : 0, width: `${pct}%`, background: `linear-gradient(90deg, ${C.amber500}, ${C.amber700})`, transition: "width 0.5s ease" }),
    groups: { display: "flex", flexDirection: "column", gap: 8, flex: 1, minHeight: 0, overflowY: "auto", paddingRight: 2 },
    group: (tone) => ({ border: `1px solid ${tone === "fulfilled" ? C.green300 : tone === "frozen" ? C.amber200 : C.gray200}`, borderRadius: 8, overflow: "hidden", flexShrink: 0 }),
    groupHdr: (tone) => ({ display: "flex", alignItems: "center", gap: 8, padding: "10px 14px", background: tone === "fulfilled" ? C.green50 : tone === "frozen" ? C.amber50 : C.gray100, borderBottom: `1px solid ${tone === "fulfilled" ? C.green300 : tone === "frozen" ? C.amber200 : C.gray200}`, cursor: "pointer", userSelect: "none" }),
    groupBadge: (tone) => ({ width: 20, height: 20, borderRadius: "50%", flexShrink: 0, display: "flex", alignItems: "center", justifyContent: "center", background: tone === "fulfilled" ? C.green100 : tone === "frozen" ? C.amber50 : C.gray200, color: tone === "fulfilled" ? C.green600 : tone === "frozen" ? C.amber700 : C.gray400, fontSize: tone === "incomplete" ? "1rem" : "0.65rem", fontWeight: 800, lineHeight: tone === "incomplete" ? 0.85 : 1 }),
    groupName: { flex: 1, fontSize: "0.82rem", fontWeight: 700, color: C.gray900 },
    groupPill: (tone) => ({ fontSize: "0.68rem", fontWeight: 700, padding: "2px 10px", borderRadius: 10, flexShrink: 0, background: tone === "fulfilled" ? C.green100 : tone === "frozen" ? C.amber50 : C.gray100, color: tone === "fulfilled" ? C.green600 : tone === "frozen" ? C.amber700 : C.gray500, border: `1px solid ${tone === "fulfilled" ? C.green300 : tone === "frozen" ? C.amber200 : C.gray300}` }),
    chevron: { fontSize: "0.6rem", color: C.gray400, marginLeft: 2 },
    groupBody: { display: "flex", flexDirection: "column", background: C.white },
    item: (tone, isFirst) => ({ display: "flex", alignItems: "flex-start", gap: 10, padding: "10px 14px", minHeight: 40, flexShrink: 0, background: tone === "fulfilled" ? "#f8fff8" : tone === "frozen" ? C.amber50 : C.white, borderTop: isFirst ? "none" : `1px solid ${C.gray100}`, borderLeft: `3px solid ${tone === "fulfilled" ? C.green300 : tone === "frozen" ? C.amber200 : C.gray200}`, boxSizing: "border-box" }),
    itemIcon: (tone) => ({ flexShrink: 0, fontSize: "0.72rem", fontWeight: 800, marginTop: 2, width: 14, textAlign: "center", color: tone === "fulfilled" ? C.green600 : tone === "frozen" ? C.amber700 : C.gray300 }),
    itemBody: { flex: 1, minWidth: 0 },
    itemLine: { display: "flex", flexWrap: "wrap", gap: 6, alignItems: "center" },
    itemStem: { fontSize: "0.78rem", fontWeight: 600, color: C.gray700, lineHeight: 1.4 },
    itemColon: { fontSize: "0.78rem", fontWeight: 600, color: C.gray500, lineHeight: 1.4 },
    badges: { display: "inline-flex", flexWrap: "wrap", gap: 4, alignItems: "center" },
    chip: (kind) => {
        const map = {
            open: { bg: C.gray100, border: C.gray300, color: C.gray500 },
            fulfilled: { bg: C.green100, border: C.green300, color: C.green700 },
            frozen: { bg: C.amber50, border: C.amber200, color: C.amber700 },
        };
        const t = map[kind] || map.open;
        return { fontSize: "0.67rem", fontWeight: 600, padding: "2px 7px", borderRadius: 4, whiteSpace: "nowrap", background: t.bg, border: `1px solid ${t.border}`, color: t.color, boxSizing: "border-box" };
    },
    empty: { textAlign: "center", padding: "36px 20px", color: C.gray400, fontSize: "0.82rem" },
    anyOfBlock: (tone, isFirst) => ({
        flexShrink: 0,
        borderTop: isFirst ? "none" : `1px solid ${C.gray100}`,
        borderLeft: `3px solid ${tone === "fulfilled" ? C.green300 : tone === "frozen" ? C.amber200 : C.gray200}`,
        background: tone === "fulfilled" ? "#f8fff8" : tone === "frozen" ? C.amber50 : C.white,
        boxSizing: "border-box",
    }),
    anyOfIntro: { padding: "10px 14px 4px 14px", fontSize: "0.78rem", fontWeight: 600, color: C.gray700, lineHeight: 1.4 },
    anyOfChild: (tone) => ({
        display: "flex",
        alignItems: "flex-start",
        gap: 8,
        padding: "6px 14px 8px 32px",
        minHeight: 32,
        background: tone === "fulfilled" ? C.green50 : tone === "frozen" ? C.amber50 : "transparent",
        boxSizing: "border-box",
    }),
    anyOfBullet: (tone) => ({
        flexShrink: 0,
        width: 12,
        marginTop: 2,
        fontSize: "0.85rem",
        fontWeight: 700,
        color: tone === "fulfilled" ? C.green600 : tone === "frozen" ? C.amber700 : C.gray400,
        lineHeight: 1.2,
    }),
};

export default function RequirementsPanel({
    scheduleData,
    degrees,
    allCourses = [],
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
        return <div style={S.empty}><div style={{ fontSize: "2rem", marginBottom: 8 }}>📋</div>Add degrees to see requirement fulfillment</div>;
    }
    if (!scheduleData || !scheduleData.degree_results) {
        return <div style={S.empty}>Loading requirements…</div>;
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
    const cuMap = buildCourseCuMap(allCourses);
    const scheduleCtx = { assignedIds, frozenIds, cuMap, allReqs };

    const totalCount = allReqs.length;
    const fulfilledCount = allReqs.filter((r) => isItemFulfilled(r, scheduleCtx) && !itemIsPlannedOnly(r, scheduleCtx)).length;
    const plannedCount = allReqs.filter((r) => isItemFulfilled(r, scheduleCtx) && itemIsPlannedOnly(r, scheduleCtx)).length;
    const remainingCount = totalCount - fulfilledCount - plannedCount;
    const fulfilledPct = totalCount > 0 ? (fulfilledCount / totalCount) * 100 : 0;
    const plannedPct = totalCount > 0 ? (plannedCount / totalCount) * 100 : 0;
    const pct = totalCount > 0 ? Math.round(((fulfilledCount + plannedCount) / totalCount) * 100) : 0;

    return (
        <div style={S.wrap}>
            {results.length > 1 && (
                <div style={S.tabs}>
                    {results.map((result, i) => (
                        <button key={i} style={tabIndex === i ? S.tabActive : S.tab} onClick={() => setActiveTab(i)}>
                            {degrees[i]?.displayMajor || `${result.school} — ${result.major}`}
                        </button>
                    ))}
                </div>
            )}

            {current.error && <div style={S.error}>⚠️ {current.error}</div>}

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

            <div style={S.groups}>
                {orderedCategories.map((cat) => {
                    const items = categoryMap[cat];
                    if (!items?.length) return null;
                    const done = items.filter((r) => isItemFulfilled(r, scheduleCtx)).length;
                    const catTone = groupTone(items, scheduleCtx);
                    const isCollapsed = collapsedGroups[cat] ?? true;

                    return (
                        <div key={cat} style={S.group(catTone)}>
                            <div style={S.groupHdr(catTone)} onClick={() => setCollapsedGroups((p) => ({ ...p, [cat]: !(p[cat] ?? true) }))}>
                                <span style={S.groupBadge(catTone)}>{done === items.length ? "✓" : "·"}</span>
                                <span style={S.groupName}>{cat}</span>
                                <span style={S.groupPill(catTone)}>{done}/{items.length}</span>
                                <span style={S.chevron}>{isCollapsed ? "▶" : "▾"}</span>
                            </div>
                            {!isCollapsed && (
                                <div style={S.groupBody}>
                                    {items.map((item, rowIdx) => {
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
                                            null,
                                            tabIndex,
                                            flashRowId
                                        );
                                    })}
                                </div>
                            )}
                        </div>
                    );
                })}
                {totalCount === 0 && !current.error && <div style={S.empty}>No requirement data available for this program</div>}
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

function schedulePlacedIds(scheduleCtx) {
    return [...new Set([...scheduleCtx.assignedIds, ...scheduleCtx.frozenIds])];
}

/** Courses already allocated to other fulfilled requirements on this degree tab. */
function coursesAllocatedElsewhere(allReqs, exceptInstanceId) {
    const allocated = new Set();
    (allReqs || []).forEach((other) => {
        if (other.instanceId === exceptInstanceId || !other.fulfilled) return;
        collectFulfillingCourses(other).forEach((c) => allocated.add(c));
    });
    return allocated;
}

function scheduleAvailableForItem(item, scheduleCtx) {
    const excluded = coursesAllocatedElsewhere(scheduleCtx.allReqs, item.instanceId);
    return schedulePlacedIds(scheduleCtx).filter((id) => !excluded.has(id));
}

function scheduleFulfillsItem(item, scheduleCtx) {
    const placed = scheduleAvailableForItem(item, scheduleCtx);
    if (!placed.length) return null;
    const { type, data } = parseRequirement(item.requirement);
    if (type === "Restriction") {
        return scheduleCoursesFulfillingRestriction(
            data,
            placed,
            scheduleCtx.cuMap,
            item.attributeFulfillment
        );
    }
    if (type === "SingleCourse") {
        const opts = new Set(data.possibilities || []);
        const matched = placed.filter((id) => opts.has(id));
        return matched.length ? matched : null;
    }
    return null;
}

function getFulfillingCourses(item, scheduleCtx) {
    const fromApi = collectFulfillingCourses(item);
    if (fromApi.length) return fromApi;
    return scheduleFulfillsItem(item, scheduleCtx) || [];
}

/** Requirement satisfied (API or schedule placement, e.g. two 0.5 CU courses → 1 CU slot). */
function isItemFulfilled(item, scheduleCtx) {
    if (item.fulfilled) return true;
    return (scheduleFulfillsItem(item, scheduleCtx)?.length ?? 0) > 0;
}

/** Fulfilled only via frozen/planned courses on the schedule (for stats, not row color). */
function itemIsPlannedOnly(item, scheduleCtx) {
    if (!isItemFulfilled(item, scheduleCtx)) return false;
    const courses = getFulfillingCourses(item, scheduleCtx);
    if (!courses.length) return false;
    const anyAssigned = courses.some((c) => scheduleCtx.assignedIds.has(c));
    const allFrozen = courses.every((c) => scheduleCtx.frozenIds.has(c));
    return allFrozen && !anyAssigned;
}

function itemTone(item, scheduleCtx) {
    if (!isItemFulfilled(item, scheduleCtx)) return "open";
    return "fulfilled";
}

function groupTone(items, scheduleCtx) {
    const done = items.filter((r) => isItemFulfilled(r, scheduleCtx)).length;
    if (done !== items.length) return "incomplete";
    if (items.some((item) => itemIsPlannedOnly(item, scheduleCtx))) return "frozen";
    return "fulfilled";
}

/** Green/orange only for courses allocated to this requirement; gray otherwise. */
function badgeKindFor(courseId, { assignedIds, frozenIds, fulfillingSet }) {
    if (!fulfillingSet?.has(courseId)) return "open";
    if (assignedIds.has(courseId)) return "fulfilled";
    if (frozenIds.has(courseId)) return "frozen";
    return "fulfilled";
}

function buildRowContent(item, scheduleCtx) {
    const { type, data } = parseRequirement(item.requirement);
    const stem = getRequirementStem(item.requirement);
    const fulfilling = getFulfillingCourses(item, scheduleCtx);
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

function makeAnyOfChildItem(parent, childReq, childIdx, scheduleCtx) {
    const parentFulfilled = isItemFulfilled(parent, scheduleCtx);
    const fulfillingCourses = parentFulfilled ? getFulfillingCourses(parent, scheduleCtx) : [];
    const parentForMatch = {
        ...parent,
        fulfilled: parentFulfilled,
        fulfilledCourses: fulfillingCourses,
    };
    const matched = childMatchesAnyOfFulfillment(childReq, parentForMatch, parentFulfilled);
    const courses = matched ? fulfillingCourses : [];
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

function renderRequirementLine(item, scheduleCtx) {
    const { stem, badges, fulfillingSet } = buildRowContent(item, scheduleCtx);
    const chipCtx = { ...scheduleCtx, fulfillingSet };

    const renderBadge = (badge, key) => {
        if (badge.kind === "attr") {
            const hasCourse = badge.courses.length > 0;
            const label = hasCourse ? `[${badge.code}] - ${badge.courses.join(", ")}` : `[${badge.code}]`;
            const allocated = badge.courses.find((c) => fulfillingSet.has(c));
            const kind = allocated ? badgeKindFor(allocated, chipCtx) : "open";
            return <span key={key} style={S.chip(kind)}>{label}</span>;
        }
        return (
            <span key={key} style={S.chip(badgeKindFor(badge.id, chipCtx))}>
                {badge.id}
            </span>
        );
    };

    return (
        <div style={S.itemLine}>
            {stem && <span style={S.itemStem}>{stem}</span>}
            {stem && badges.length > 0 && <span style={S.itemColon}>:</span>}
            {badges.length > 0 && (
                <span style={S.badges}>{badges.map((b, i) => renderBadge(b, i))}</span>
            )}
        </div>
    );
}

function renderAnyOfGroup(parentItem, idx, scheduleCtx, isFirst, degreeIndex, flashRowId) {
    const possibilities = getAnyOfPossibilities(parentItem.requirement);
    const blockTone = itemTone(parentItem, scheduleCtx);
    const rowDomId = reqRowDomId(degreeIndex, idx);
    const isFlashing = flashRowId === rowDomId;

    return (
        <div
            key={String(idx)}
            id={rowDomId}
            className={isFlashing ? "req-row-flash" : undefined}
            style={S.anyOfBlock(blockTone, isFirst)}
        >
            <div style={S.anyOfIntro}>Choose one of the following:</div>
            {possibilities.map((childReq, childIdx) => {
                const childItem = makeAnyOfChildItem(parentItem, childReq, childIdx, scheduleCtx);
                const childTone = childItem.fulfilled ? itemTone(childItem, scheduleCtx) : "open";
                const childRowId = reqRowDomId(degreeIndex, childItem.instanceId);
                return (
                    <div
                        key={childItem.instanceId}
                        id={childRowId}
                        className={flashRowId === childRowId ? "req-row-flash" : undefined}
                        style={S.anyOfChild(childTone)}
                    >
                        <span style={S.anyOfBullet(childTone)}>{childItem.fulfilled ? "✓" : "•"}</span>
                        <div style={S.itemBody}>
                            {renderRequirementLine(childItem, scheduleCtx)}
                        </div>
                    </div>
                );
            })}
        </div>
    );
}

function renderItem(item, idx, scheduleCtx, isFirst, _stemOverride, degreeIndex, flashRowId) {
    const fulfilled = isItemFulfilled(item, scheduleCtx);
    const rowTone = itemTone(item, scheduleCtx);
    const rowDomId = reqRowDomId(degreeIndex, idx);
    const isFlashing = flashRowId === rowDomId;

    return (
        <div
            key={String(idx)}
            id={rowDomId}
            className={isFlashing ? "req-row-flash" : undefined}
            style={S.item(rowTone, isFirst)}
        >
            <span style={S.itemIcon(rowTone)}>{fulfilled ? "✓" : "○"}</span>
            <div style={S.itemBody}>
                {renderRequirementLine(item, scheduleCtx)}
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
    for (const v of ["SingleCourse", "CourseGroup", "AnyOf", "AllOf", "Concentration", "Restriction", "DoubleCount"]) {
        if (req[v]?.category) return req[v].category;
    }
    return "Other";
}
