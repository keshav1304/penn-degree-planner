"use client";

import { useState } from "react";
import { filterValidCourseCodes } from "@/lib/courseUtils";

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
    statNo: { display: "flex", alignItems: "center", gap: 6, fontSize: "0.78rem", fontWeight: 600, color: C.red500 },
    statPct: { marginLeft: "auto", fontSize: "0.78rem", fontWeight: 700, color: C.gray500 },
    dot: (color) => ({ width: 7, height: 7, borderRadius: "50%", background: color, flexShrink: 0 }),
    track: { height: 5, background: "rgba(0,0,0,0.08)", borderRadius: 3, overflow: "hidden" },
    fill: (pct) => ({ height: "100%", minWidth: 4, width: `${pct}%`, background: `linear-gradient(90deg, ${C.green600}, ${C.teal600})`, borderRadius: 3, transition: "width 0.5s ease" }),

    // groups — flexShrink: 0 on children prevents accordion rows squashing to ~2px
    groups: { display: "flex", flexDirection: "column", gap: 8, flex: 1, minHeight: 0, overflowY: "auto", paddingRight: 2 },
    group: (done) => ({
        border: `1px solid ${done ? C.green300 : C.gray200}`,
        borderRadius: 8,
        overflow: "hidden",
        flexShrink: 0,
    }),

    // group header
    groupHdr: (done) => ({
        display: "flex", alignItems: "center", gap: 8,
        padding: "10px 14px",               // ← explicit padding, not from CSS
        background: done ? C.green50 : C.gray100,
        borderBottom: `1px solid ${done ? C.green300 : C.gray200}`,
        cursor: "pointer", userSelect: "none",
    }),
    groupBadge: (done) => ({
        width: 20, height: 20, borderRadius: "50%", flexShrink: 0,
        display: "flex", alignItems: "center", justifyContent: "center",
        background: done ? C.green100 : C.gray200,
        color: done ? C.green600 : C.gray400,
        fontSize: done ? "0.65rem" : "1rem",
        fontWeight: 800, lineHeight: done ? 1 : 0.85,
    }),
    groupName: { flex: 1, fontSize: "0.82rem", fontWeight: 700, color: C.gray900 },
    groupPill: (done) => ({
        fontSize: "0.68rem", fontWeight: 700, padding: "2px 10px", borderRadius: 10, flexShrink: 0,
        background: done ? C.green100 : C.gray100,
        color: done ? C.green600 : C.gray500,
        border: `1px solid ${done ? C.green300 : C.gray300}`,
    }),
    chevron: { fontSize: "0.6rem", color: C.gray400, marginLeft: 2 },

    // group body
    groupBody: { display: "flex", flexDirection: "column", background: C.white },

    // items — ALL INLINE so global reset cannot collapse them
    item: (fulfilled, isFirst) => ({
        display: "flex",
        alignItems: "flex-start",
        gap: 10,
        padding: "10px 14px",              // ← explicit, immune to CSS reset
        minHeight: 40,                     // ← floor so rows are never invisible
        flexShrink: 0,
        background: fulfilled ? "#f8fff8" : C.white,
        borderTop: isFirst ? "none" : `1px solid ${C.gray100}`,
        borderLeft: `3px solid ${fulfilled ? C.green300 : C.gray200}`,
        boxSizing: "border-box",
    }),
    itemIcon: (fulfilled) => ({
        flexShrink: 0, fontSize: "0.72rem", fontWeight: 800,
        marginTop: 2, width: 14, textAlign: "center",
        color: fulfilled ? C.green600 : C.gray300,
    }),
    itemBody: { flex: 1, minWidth: 0, display: "flex", flexDirection: "column", gap: 5 },
    itemDesc: { fontSize: "0.78rem", fontWeight: 600, color: C.gray700, lineHeight: 1.4 },

    // chips
    chips: { display: "flex", flexWrap: "wrap", gap: 4, alignItems: "center" },
    chip: (kind) => {
        const map = {
            default: { bg: C.gray100, border: C.gray300, color: C.gray500 },
            fulfilled: { bg: C.green100, border: C.green300, color: C.green700 },
            suggested: { bg: "#d1fae5", border: "#6ee7b7", color: "#047857" },
        };
        const t = map[kind] || map.default;
        return { fontSize: "0.67rem", fontWeight: 600, padding: "2px 7px", borderRadius: 4, whiteSpace: "nowrap", background: t.bg, border: `1px solid ${t.border}`, color: t.color, boxSizing: "border-box" };
    },
    expandBtn: { fontSize: "0.65rem", fontWeight: 600, padding: "2px 8px", borderRadius: 4, background: "none", border: `1px dashed ${C.gray300}`, color: C.gray400, cursor: "pointer", fontFamily: "inherit" },

    // empty
    empty: { textAlign: "center", padding: "36px 20px", color: C.gray400, fontSize: "0.82rem" },
};

export default function RequirementsPanel({ scheduleData, degrees }) {
    const [activeTab, setActiveTab] = useState(0);
    const [expandedOptions, setExpandedOptions] = useState({});
    const [collapsedGroups, setCollapsedGroups] = useState({});

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
        allReqs.push({ category: cat, fulfilled: true, fulfilledCourses: mapped.course_ids || [], requirement: mapped.requirement });
    });
    const suggestionsMap = {};
    (current.suggested_for_unfulfilled || []).forEach((mapped) => {
        const cat = normalizeCategory(getCategory(mapped.requirement));
        suggestionsMap[`${cat}::${getReqKey(mapped.requirement)}`] = mapped.course_ids || [];
    });
    (current.unfulfilled_requirements || []).forEach((req) => {
        const cat = normalizeCategory(getCategory(req));
        allReqs.push({ category: cat, fulfilled: false, fulfilledCourses: [], suggestedCourses: suggestionsMap[`${cat}::${getReqKey(req)}`] || [], requirement: req });
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

    const fulfilledCount = current.fulfilled_requirements?.length || 0;
    const totalCount = fulfilledCount + (current.unfulfilled_requirements?.length || 0);
    const pct = totalCount > 0 ? Math.round((fulfilledCount / totalCount) * 100) : 0;

    const toggleExpand = (key) => setExpandedOptions((p) => ({ ...p, [key]: !p[key] }));
    const toggleGroup = (cat) => setCollapsedGroups((p) => ({ ...p, [cat]: !p[cat] }));

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
                        <span style={S.statNo}><span style={S.dot(C.red500)} />{totalCount - fulfilledCount} remaining</span>
                        <span style={S.statPct}>{pct}%</span>
                    </div>
                    <div style={S.track}><div style={S.fill(pct)} /></div>
                </div>
            )}

            {/* Category groups */}
            <div style={S.groups}>
                {orderedCategories.map((cat) => {
                    const items = categoryMap[cat];
                    if (!items || items.length === 0) return null;

                    const done = items.filter((r) => r.fulfilled).length;
                    const total = items.length;
                    const allDone = done === total;
                    const isCollapsed = collapsedGroups[cat];

                    return (
                        <div key={cat} style={S.group(allDone)}>
                            <div style={S.groupHdr(allDone)} onClick={() => toggleGroup(cat)}>
                                <span style={S.groupBadge(allDone)}>{allDone ? "✓" : "·"}</span>
                                <span style={S.groupName}>{cat}</span>
                                <span style={S.groupPill(allDone)}>{done}/{total}</span>
                                <span style={S.chevron}>{isCollapsed ? "▶" : "▾"}</span>
                            </div>

                            {!isCollapsed && (
                                <div style={S.groupBody}>
                                    {items.map((item, idx) => {
                                        const expandKey = `${cat}-${idx}`;
                                        return renderItem(item, idx, expandKey, expandedOptions[expandKey], () => toggleExpand(expandKey));
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

function renderItem(item, idx, expandKey, isExpanded, onToggle) {
    const { type, data } = parseRequirement(item.requirement);
    const options = getOptions(type, data);
    const fulfilledCourses = filterValidCourseCodes(item.fulfilledCourses || []);
    const suggestedCourses = filterValidCourseCodes(item.suggestedCourses || []);
    const fulfilledSet = new Set(fulfilledCourses);
    const suggestedSet = new Set(suggestedCourses);

    const MAX_VISIBLE = 5;
    const visible = isExpanded ? options : options.slice(0, MAX_VISIBLE);

    return (
        <div key={idx} style={S.item(item.fulfilled, idx === 0)}>
            <span style={S.itemIcon(item.fulfilled)}>{item.fulfilled ? "✓" : "○"}</span>
            <div style={S.itemBody}>
                <div style={S.itemDesc}>{getDescription(type, data)}</div>

                {options.length > 0 && (
                    <div style={S.chips}>
                        {visible.map((opt, i) => (
                            <span key={i} style={S.chip(fulfilledSet.has(opt) ? "fulfilled" : suggestedSet.has(opt) ? "suggested" : "default")}>
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

                {item.fulfilled && fulfilledCourses.length > 0 && options.length === 0 && (
                    <div style={S.chips}>
                        {fulfilledCourses.map((c, i) => <span key={i} style={S.chip("fulfilled")}>{c}</span>)}
                    </div>
                )}

                {!item.fulfilled && suggestedCourses.length > 0 && options.length === 0 && (
                    <div style={S.chips}>
                        {suggestedCourses.map((c, i) => <span key={i} style={S.chip("suggested")}>{c}</span>)}
                    </div>
                )}
            </div>
        </div>
    );
}

// ─── Helpers (logic unchanged) ───

function parseRequirement(req) {
    if (!req) return { type: "Unknown", data: {} };
    const variants = ["SingleCourse", "CourseGroup", "AnyOf", "AllOf", "Concentration", "Restriction", "DoubleCount"];
    for (const v of variants) { if (req[v] !== undefined) return { type: v, data: req[v] }; }
    if (req.possibilities) return { type: "SingleCourse", data: req };
    if (req.department !== undefined || req.attr !== undefined) return { type: "Restriction", data: req };
    return { type: "Unknown", data: req };
}

function getDescription(type, data) {
    switch (type) {
        case "SingleCourse": return "Complete one of the following:";
        case "CourseGroup": return `Complete ${data.number || "N"} of the following:`;
        case "Restriction": {
            const p = [];
            if (data.number) p.push(`${data.number} course(s)`);
            if (data.department) p.push(`from ${Array.isArray(data.department) ? data.department.join("/") : data.department}`);
            if (data.level) p.push(`level ${data.level}+`);
            if (data.attr) p.push(`in ${data.attr.join(" or ")}`);
            if (data.excluding) p.push(`excluding ${data.excluding.join(", ")}`);
            if (data.no_school) p.push(`not from ${data.no_school}`);
            return p.join(" ") || "Restriction requirement";
        }
        case "AnyOf": return "Complete one of the following options:";
        case "AllOf": return "Complete all of the following:";
        case "Concentration": return `Complete concentration (${data.number || "N"} courses):`;
        case "DoubleCount": return "Double-counted with other requirements";
        default: return "Requirement";
    }
}

function getOptions(type, data) {
    switch (type) {
        case "SingleCourse":
        case "CourseGroup": return data.possibilities || [];
        case "Restriction": return data.attr?.length > 0 ? data.attr.map(a => `[${a}]`) : [];
        case "AnyOf": {
            const out = [];
            (data.possibilities || []).forEach((sub) => {
                const { type: t2, data: d2 } = parseRequirement(sub);
                if (t2 === "SingleCourse") out.push(...(d2.possibilities || []));
                else if (t2 === "AllOf") {
                    const names = [];
                    (d2.requirements || []).forEach((r) => {
                        const { type: t3, data: d3 } = parseRequirement(r);
                        if (t3 === "SingleCourse") names.push(...(d3.possibilities || []));
                    });
                    if (names.length) out.push(names.join(" + "));
                }
            });
            return out;
        }
        case "AllOf": {
            const out = [];
            (data.requirements || []).forEach((sub) => {
                const { type: t2, data: d2 } = parseRequirement(sub);
                if (t2 === "SingleCourse") out.push(...(d2.possibilities || []));
            });
            return out;
        }
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

function getReqKey(req) {
    if (!req) return "unknown";
    const { type, data } = parseRequirement(req);
    switch (type) {
        case "SingleCourse": return `SC:${(data.possibilities || []).slice(0, 3).join(",")}`;
        case "CourseGroup": return `CG:${data.number}:${(data.possibilities || []).slice(0, 3).join(",")}`;
        case "Restriction": return `R:${data.number}:${(data.department || []).join(",")}:${(data.attr || []).join(",")}:${data.level || ""}`;
        case "AnyOf": return `AO:${(data.possibilities || []).length}`;
        case "AllOf": return `AL:${(data.requirements || []).length}`;
        default: return JSON.stringify(req).slice(0, 50);
    }
}