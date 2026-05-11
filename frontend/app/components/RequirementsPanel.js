"use client";

import { useState } from "react";

const STYLES = `
  .rp-wrap { display: flex; flex-direction: column; gap: 12px; }

  /* ── tabs ── */
  .rp-tabs { display: flex; gap: 6px; overflow-x: auto; padding-bottom: 2px; }
  .rp-tab {
    padding: 6px 14px; background: rgba(0,0,0,0.04);
    border: 1px solid rgba(0,0,0,0.1); border-radius: 8px;
    color: #4a5568; font-size: 0.76rem; font-weight: 600;
    cursor: pointer; white-space: nowrap; font-family: inherit;
    transition: background 0.15s, color 0.15s;
  }
  .rp-tab:hover { background: rgba(0,0,0,0.07); color: #1a1a2e; }
  .rp-tab.rp-tab-active { background: rgba(5,150,105,0.1); border-color: #059669; color: #059669; }

  /* ── error ── */
  .rp-error {
    padding: 10px 14px; background: rgba(220,38,38,0.06);
    border: 1px solid rgba(220,38,38,0.2); border-radius: 8px;
    font-size: 0.8rem; color: #dc2626;
  }

  /* ── summary ── */
  .rp-summary {
    display: flex; flex-direction: column; gap: 8px;
    padding: 12px 14px; background: rgba(0,0,0,0.03);
    border: 1px solid rgba(0,0,0,0.08); border-radius: 8px;
  }
  .rp-summary-stats { display: flex; align-items: center; gap: 14px; flex-wrap: wrap; }
  .rp-stat { display: flex; align-items: center; gap: 6px; font-size: 0.78rem; font-weight: 600; }
  .rp-dot { width: 7px; height: 7px; border-radius: 50%; flex-shrink: 0; }
  .rp-dot-ok { background: #16a34a; }
  .rp-dot-no { background: #dc2626; }
  .rp-stat-ok { color: #16a34a; }
  .rp-stat-no { color: #dc2626; }
  .rp-stat-pct { margin-left: auto; font-size: 0.78rem; font-weight: 700; color: #4a5568; }
  .rp-track { height: 5px; background: rgba(0,0,0,0.08); border-radius: 3px; overflow: hidden; }
  .rp-fill {
    height: 100%; min-width: 4px;
    background: linear-gradient(90deg, #16a34a, #059669);
    border-radius: 3px; transition: width 0.5s ease;
  }

  /* ── groups ── */
  .rp-groups {
    display: flex; flex-direction: column; gap: 8px;
    max-height: 540px; overflow-y: auto; padding-right: 2px;
  }

  .rp-group {
    border: 1px solid #d1d5db;
    border-radius: 8px; overflow: hidden;
  }
  .rp-group-done { border-color: #86efac; }

  /* ── group header ── */
  .rp-group-header {
    display: flex; align-items: center; gap: 8px;
    padding: 9px 12px;
    background: #f1f5f9;
    border-bottom: 1px solid #e2e8f0;
    cursor: pointer; user-select: none;
  }
  .rp-group-done .rp-group-header {
    background: #f0fdf4;
    border-bottom-color: #bbf7d0;
  }

  .rp-group-badge {
    width: 18px; height: 18px; border-radius: 50%;
    display: flex; align-items: center; justify-content: center;
    font-size: 0.65rem; font-weight: 800; flex-shrink: 0;
  }
  .rp-badge-done { background: #dcfce7; color: #16a34a; }
  .rp-badge-pend { background: #e2e8f0; color: #94a3b8; font-size: 1rem; line-height: 0.85; }

  .rp-group-name { flex: 1; font-size: 0.82rem; font-weight: 700; color: #111827; }

  .rp-group-pill {
    font-size: 0.68rem; font-weight: 700;
    padding: 2px 8px; border-radius: 10px; flex-shrink: 0;
  }
  .rp-pill-done { background: #dcfce7; color: #16a34a; border: 1px solid #86efac; }
  .rp-pill-pend { background: #f1f5f9; color: #64748b; border: 1px solid #cbd5e1; }

  /* ── group body ── */
  .rp-group-body { display: flex; flex-direction: column; }

  /* KEY FIX: items alternate slightly so they're distinguishable */
  .rp-item {
    display: flex; align-items: flex-start; gap: 10px;
    padding: 10px 12px;
    border-top: 1px solid #f1f5f9;
    background: #ffffff;
  }
  .rp-item:first-child { border-top: none; }

  .rp-item-ok   { background: #f8fffe; }
  .rp-item-pend { background: #fafafa; }

  .rp-item-ok:hover   { background: #f0fdf4; }
  .rp-item-pend:hover { background: #f8fafc; }

  /* left accent bar per item */
  .rp-item-ok   { border-left: 3px solid #86efac; }
  .rp-item-pend { border-left: 3px solid #e2e8f0; }

  .rp-item-icon {
    flex-shrink: 0; font-size: 0.72rem; font-weight: 800;
    margin-top: 1px; width: 14px; text-align: center;
  }
  .rp-icon-ok   { color: #16a34a; }
  .rp-icon-pend { color: #94a3b8; }

  .rp-item-body { flex: 1; min-width: 0; display: flex; flex-direction: column; gap: 5px; }

  /* DARKER text so it's actually readable */
  .rp-item-desc {
    font-size: 0.78rem; font-weight: 600; color: #374151; line-height: 1.4;
  }

  /* ── chips ── */
  .rp-chips { display: flex; flex-wrap: wrap; gap: 4px; align-items: center; }
  .rp-chip  { font-size: 0.67rem; font-weight: 600; padding: 2px 7px; border-radius: 4px; white-space: nowrap; }
  .rp-chip-default   { background: #f1f5f9; border: 1px solid #cbd5e1; color: #475569; }
  .rp-chip-fulfilled { background: #dcfce7; border: 1px solid #86efac; color: #15803d; }
  .rp-chip-suggested { background: #d1fae5; border: 1px solid #6ee7b7; color: #047857; }

  .rp-expand-btn {
    font-size: 0.65rem; font-weight: 600; padding: 2px 8px; border-radius: 4px;
    background: none; border: 1px dashed #cbd5e1; color: #94a3b8;
    cursor: pointer; font-family: inherit; transition: all 0.15s;
  }
  .rp-expand-btn:hover { color: #059669; border-color: #059669; background: #f0fdf4; }

  /* ── empty ── */
  .rp-empty { text-align: center; padding: 36px 20px; color: #94a3b8; font-size: 0.82rem; }
  .rp-empty-icon { font-size: 2rem; margin-bottom: 8px; }
`;

export default function RequirementsPanel({ scheduleData, degrees }) {
    const [activeTab, setActiveTab] = useState(0);
    const [expandedOptions, setExpandedOptions] = useState({});
    const [collapsedGroups, setCollapsedGroups] = useState({});

    if (!degrees || degrees.length === 0) {
        return (
            <>
                <style>{STYLES}</style>
                <div className="rp-empty">
                    <div className="rp-empty-icon">📋</div>
                    Add degrees to see requirement fulfillment
                </div>
            </>
        );
    }

    if (!scheduleData || !scheduleData.degree_results) {
        return (
            <>
                <style>{STYLES}</style>
                <div className="rp-empty">Loading requirements…</div>
            </>
        );
    }

    const results = scheduleData.degree_results;
    const tabIndex = Math.min(activeTab, results.length - 1);
    const current = results[tabIndex];
    if (!current) return null;

    const allReqs = [];
    (current.fulfilled_requirements || []).forEach((mapped) => {
        allReqs.push({
            category: getCategory(mapped.requirement),
            fulfilled: true,
            fulfilledCourses: mapped.course_ids || [],
            requirement: mapped.requirement,
        });
    });

    const suggestionsMap = {};
    (current.suggested_for_unfulfilled || []).forEach((mapped) => {
        const cat = getCategory(mapped.requirement);
        const key = `${cat}::${getReqKey(mapped.requirement)}`;
        suggestionsMap[key] = mapped.course_ids || [];
    });

    (current.unfulfilled_requirements || []).forEach((req) => {
        const cat = getCategory(req);
        const key = `${cat}::${getReqKey(req)}`;
        allReqs.push({
            category: cat,
            fulfilled: false,
            fulfilledCourses: [],
            suggestedCourses: suggestionsMap[key] || [],
            requirement: req,
        });
    });

    const categoryMap = {};
    allReqs.forEach((item) => {
        if (!categoryMap[item.category]) categoryMap[item.category] = [];
        categoryMap[item.category].push(item);
    });

    const categoryOrder = current.category_order || [];
    const orderedCategories = [...categoryOrder];
    Object.keys(categoryMap).forEach((c) => {
        if (!orderedCategories.includes(c)) orderedCategories.push(c);
    });

    const fulfilledCount = current.fulfilled_requirements?.length || 0;
    const totalCount = fulfilledCount + (current.unfulfilled_requirements?.length || 0);
    const pct = totalCount > 0 ? Math.round((fulfilledCount / totalCount) * 100) : 0;

    const toggleExpand = (key) =>
        setExpandedOptions((prev) => ({ ...prev, [key]: !prev[key] }));

    const toggleGroup = (cat) =>
        setCollapsedGroups((prev) => ({ ...prev, [cat]: !prev[cat] }));

    return (
        <>
            <style>{STYLES}</style>
            <div className="rp-wrap">

                {results.length > 1 && (
                    <div className="rp-tabs">
                        {results.map((result, i) => (
                            <button
                                key={i}
                                className={`rp-tab ${tabIndex === i ? "rp-tab-active" : ""}`}
                                onClick={() => setActiveTab(i)}
                            >
                                {degrees[i]?.displayMajor || `${result.school} — ${result.major}`}
                            </button>
                        ))}
                    </div>
                )}

                {current.error && (
                    <div className="rp-error">⚠️ {current.error}</div>
                )}

                {!current.error && totalCount > 0 && (
                    <div className="rp-summary">
                        <div className="rp-summary-stats">
                            <span className="rp-stat rp-stat-ok">
                                <span className="rp-dot rp-dot-ok" /> {fulfilledCount} fulfilled
                            </span>
                            <span className="rp-stat rp-stat-no">
                                <span className="rp-dot rp-dot-no" /> {totalCount - fulfilledCount} remaining
                            </span>
                            <span className="rp-stat-pct">{pct}%</span>
                        </div>
                        <div className="rp-track">
                            <div className="rp-fill" style={{ width: `${pct}%` }} />
                        </div>
                    </div>
                )}

                <div className="rp-groups">
                    {orderedCategories.map((cat) => {
                        const items = categoryMap[cat];
                        if (!items || items.length === 0) return null;

                        const done = items.filter((r) => r.fulfilled).length;
                        const total = items.length;
                        const allDone = done === total;
                        const isCollapsed = collapsedGroups[cat];

                        return (
                            <div key={cat} className={`rp-group ${allDone ? "rp-group-done" : ""}`}>
                                <div
                                    className="rp-group-header"
                                    onClick={() => toggleGroup(cat)}
                                >
                                    <span className={`rp-group-badge ${allDone ? "rp-badge-done" : "rp-badge-pend"}`}>
                                        {allDone ? "✓" : "·"}
                                    </span>
                                    <span className="rp-group-name">{cat}</span>
                                    <span className={`rp-group-pill ${allDone ? "rp-pill-done" : "rp-pill-pend"}`}>
                                        {done}/{total}
                                    </span>
                                    <span style={{ fontSize: "0.65rem", color: "#94a3b8", marginLeft: 4 }}>
                                        {isCollapsed ? "▶" : "▾"}
                                    </span>
                                </div>
                                {!isCollapsed && (
                                    <div className="rp-group-body">
                                        {items.map((item, idx) => {
                                            const expandKey = `${cat}-${idx}`;
                                            return renderItem(
                                                item, idx, expandKey,
                                                expandedOptions[expandKey],
                                                () => toggleExpand(expandKey)
                                            );
                                        })}
                                    </div>
                                )}
                            </div>
                        );
                    })}

                    {totalCount === 0 && !current.error && (
                        <div className="rp-empty">No requirement data available for this program</div>
                    )}
                </div>
            </div>
        </>
    );
}

function renderItem(item, idx, expandKey, isExpanded, onToggle) {
    const { type, data } = parseRequirement(item.requirement);
    const options = getOptions(type, data);
    const fulfilledSet = new Set(item.fulfilledCourses || []);
    const suggestedSet = new Set(item.suggestedCourses || []);

    const MAX_VISIBLE = 5;
    const hasMore = options.length > MAX_VISIBLE;
    const visible = isExpanded ? options : options.slice(0, MAX_VISIBLE);

    return (
        <div key={idx} className={`rp-item ${item.fulfilled ? "rp-item-ok" : "rp-item-pend"}`}>
            <span className={`rp-item-icon ${item.fulfilled ? "rp-icon-ok" : "rp-icon-pend"}`}>
                {item.fulfilled ? "✓" : "○"}
            </span>
            <div className="rp-item-body">
                <div className="rp-item-desc">{getDescription(type, data)}</div>

                {options.length > 0 && (
                    <div className="rp-chips">
                        {visible.map((opt, i) => (
                            <span key={i} className={`rp-chip ${fulfilledSet.has(opt) ? "rp-chip-fulfilled"
                                    : suggestedSet.has(opt) ? "rp-chip-suggested"
                                        : "rp-chip-default"
                                }`}>{opt}</span>
                        ))}
                        {hasMore && (
                            <button className="rp-expand-btn" onClick={onToggle}>
                                {isExpanded ? "Show less ↑" : `+${options.length - MAX_VISIBLE} more`}
                            </button>
                        )}
                    </div>
                )}

                {item.fulfilled && item.fulfilledCourses.length > 0 && options.length === 0 && (
                    <div className="rp-chips">
                        {item.fulfilledCourses.map((c, i) => (
                            <span key={i} className="rp-chip rp-chip-fulfilled">{c}</span>
                        ))}
                    </div>
                )}

                {!item.fulfilled && item.suggestedCourses?.length > 0 && options.length === 0 && (
                    <div className="rp-chips">
                        {item.suggestedCourses.map((c, i) => (
                            <span key={i} className="rp-chip rp-chip-suggested">{c}</span>
                        ))}
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
    for (const v of variants) {
        if (req[v] !== undefined) return { type: v, data: req[v] };
    }
    if (req.possibilities) return { type: "SingleCourse", data: req };
    if (req.department !== undefined || req.attr !== undefined) return { type: "Restriction", data: req };
    return { type: "Unknown", data: req };
}

function getDescription(type, data) {
    switch (type) {
        case "SingleCourse": return "Complete one of the following:";
        case "CourseGroup": return `Complete ${data.number || "N"} of the following:`;
        case "Restriction": {
            const parts = [];
            if (data.number) parts.push(`${data.number} course(s)`);
            if (data.department) parts.push(`from ${Array.isArray(data.department) ? data.department.join("/") : data.department}`);
            if (data.level) parts.push(`level ${data.level}+`);
            if (data.attr) parts.push(`in ${data.attr.join(" or ")}`);
            if (data.excluding) parts.push(`excluding ${data.excluding.join(", ")}`);
            if (data.no_school) parts.push(`not from ${data.no_school}`);
            return parts.join(" ") || "Restriction requirement";
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
            const items = [];
            (data.possibilities || []).forEach((sub) => {
                const { type: t2, data: d2 } = parseRequirement(sub);
                if (t2 === "SingleCourse") {
                    items.push(...(d2.possibilities || []));
                } else if (t2 === "AllOf") {
                    const names = [];
                    (d2.requirements || []).forEach((r) => {
                        const { type: t3, data: d3 } = parseRequirement(r);
                        if (t3 === "SingleCourse") names.push(...(d3.possibilities || []));
                    });
                    if (names.length) items.push(names.join(" + "));
                }
            });
            return items;
        }
        case "AllOf": {
            const items = [];
            (data.requirements || []).forEach((sub) => {
                const { type: t2, data: d2 } = parseRequirement(sub);
                if (t2 === "SingleCourse") items.push(...(d2.possibilities || []));
            });
            return items;
        }
        default: return [];
    }
}

function getCategory(req) {
    if (!req) return "Other";
    if (req.category) return req.category;
    const variants = ["SingleCourse", "CourseGroup", "AnyOf", "AllOf", "Concentration", "Restriction", "DoubleCount"];
    for (const v of variants) {
        if (req[v]?.category) return req[v].category;
    }
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