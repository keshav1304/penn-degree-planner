"use client";

import { useState } from "react";

export default function RequirementsPanel({ scheduleData, degrees }) {
    const [activeTab, setActiveTab] = useState(0);
    const [expandedOptions, setExpandedOptions] = useState({});

    if (!degrees || degrees.length === 0) {
        return (
            <div className="empty-state">
                <div className="emoji">📋</div>
                <div>Add degrees to see requirement fulfillment</div>
            </div>
        );
    }

    if (!scheduleData || !scheduleData.degree_results) {
        return (
            <div className="empty-state">
                <div className="loading-spinner" style={{ width: 24, height: 24 }} />
                <div style={{ marginTop: 10 }}>Loading requirements…</div>
            </div>
        );
    }

    const results = scheduleData.degree_results;
    const tabIndex = Math.min(activeTab, results.length - 1);
    const current = results[tabIndex];

    if (!current) return null;

    // ─── Build a unified list of all requirements with their status ───
    const allReqs = [];

    // Fulfilled requirements
    (current.fulfilled_requirements || []).forEach((mapped) => {
        const cat = getCategory(mapped.requirement);
        allReqs.push({
            category: cat,
            fulfilled: true,
            fulfilledCourses: mapped.course_ids || [],
            requirement: mapped.requirement,
        });
    });

    // Unfulfilled requirements (with suggestions)
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

    // ─── Group by category ───
    const categoryMap = {};
    allReqs.forEach((item) => {
        if (!categoryMap[item.category]) categoryMap[item.category] = [];
        categoryMap[item.category].push(item);
    });

    // ─── Order by category_order from backend ───
    const categoryOrder = current.category_order || [];
    const orderedCategories = [...categoryOrder];
    // Add any categories not in the order (shouldn't happen, but just in case)
    Object.keys(categoryMap).forEach((cat) => {
        if (!orderedCategories.includes(cat)) orderedCategories.push(cat);
    });

    // ─── Summary counts ───
    const fulfilledCount = current.fulfilled_requirements?.length || 0;
    const totalCount = fulfilledCount + (current.unfulfilled_requirements?.length || 0);

    const toggleExpand = (key) => {
        setExpandedOptions((prev) => ({ ...prev, [key]: !prev[key] }));
    };

    return (
        <div className="requirements-section">
            {/* Tabs for each degree */}
            {results.length > 1 && (
                <div className="req-tabs">
                    {results.map((result, i) => (
                        <button
                            key={i}
                            className={`req-tab ${tabIndex === i ? "active" : ""}`}
                            onClick={() => setActiveTab(i)}
                        >
                            {degrees[i]?.displayMajor || `${result.school} — ${result.major}`}
                        </button>
                    ))}
                </div>
            )}

            {/* Error state */}
            {current.error && (
                <div style={{
                    padding: 12,
                    background: "var(--danger-dim)",
                    borderRadius: "var(--radius-sm)",
                    border: "1px solid rgba(239,68,68,0.2)",
                    marginBottom: 12,
                    fontSize: "0.82rem",
                    color: "var(--danger)",
                }}>
                    ⚠️ {current.error}
                </div>
            )}

            {/* Summary bar */}
            {!current.error && totalCount > 0 && (
                <div className="req-summary-bar">
                    <span className="req-summary-fulfilled">
                        ✅ {fulfilledCount} fulfilled
                    </span>
                    <span className="req-summary-unfulfilled">
                        ❌ {totalCount - fulfilledCount} remaining
                    </span>
                    <span className="req-summary-total">
                        / {totalCount} total
                    </span>
                    <div className="req-progress-track">
                        <div
                            className="req-progress-fill"
                            style={{ width: `${(fulfilledCount / totalCount) * 100}%` }}
                        />
                    </div>
                </div>
            )}

            {/* Category groups */}
            <div className="req-list">
                {orderedCategories.map((cat) => {
                    const items = categoryMap[cat];
                    if (!items || items.length === 0) return null;

                    const groupFulfilled = items.filter((r) => r.fulfilled).length;
                    const groupTotal = items.length;
                    const allGroupFulfilled = groupFulfilled === groupTotal;

                    return (
                        <div key={cat} className={`req-category-group ${allGroupFulfilled ? "all-fulfilled" : ""}`}>
                            <div className="req-category-header">
                                <span className="req-category-icon">
                                    {allGroupFulfilled ? "✅" : "📌"}
                                </span>
                                <span className="req-category-name">{cat}</span>
                                <span className="req-category-count">
                                    {groupFulfilled}/{groupTotal}
                                </span>
                            </div>
                            <div className="req-category-items">
                                {items.map((item, idx) => {
                                    const expandKey = `${cat}-${idx}`;
                                    return renderRequirement(item, idx, expandKey, expandedOptions[expandKey], () => toggleExpand(expandKey));
                                })}
                            </div>
                        </div>
                    );
                })}

                {totalCount === 0 && !current.error && (
                    <div className="empty-state" style={{ padding: "20px 0" }}>
                        <div style={{ fontSize: "0.82rem", color: "var(--text-muted)" }}>
                            No requirement data available for this program
                        </div>
                    </div>
                )}
            </div>
        </div>
    );
}

// ─── Render a single requirement item ───
function renderRequirement(item, idx, expandKey, isExpanded, onToggle) {
    const req = item.requirement;
    const { type, data } = parseRequirement(req);
    const options = getOptions(type, data);
    const fulfilled = item.fulfilled;
    const fulfilledSet = new Set(item.fulfilledCourses || []);
    const suggestedSet = new Set(item.suggestedCourses || []);

    const MAX_VISIBLE = 5;
    const hasMore = options.length > MAX_VISIBLE;
    const visibleOptions = isExpanded ? options : options.slice(0, MAX_VISIBLE);

    return (
        <div key={idx} className={`req-item ${fulfilled ? "fulfilled" : "unfulfilled"}`}>
            <span className="req-status-icon">{fulfilled ? "✓" : "○"}</span>
            <div className="req-content">
                <div className="req-description">
                    {getDescription(type, data)}
                </div>
                {options.length > 0 && (
                    <div className="req-options">
                        {visibleOptions.map((opt, i) => {
                            const isFulfilled = fulfilledSet.has(opt);
                            const isSuggested = suggestedSet.has(opt);
                            return (
                                <span
                                    key={i}
                                    className={`req-option-chip ${isFulfilled ? "chip-fulfilled" : ""} ${isSuggested ? "chip-suggested" : ""}`}
                                >
                                    {opt}
                                </span>
                            );
                        })}
                        {hasMore && (
                            <button className="req-expand-btn" onClick={onToggle}>
                                {isExpanded ? "Show less" : `+${options.length - MAX_VISIBLE} more`}
                            </button>
                        )}
                    </div>
                )}
                {/* Show fulfilled courses for Restriction-type items that have description-only suggestions */}
                {fulfilled && item.fulfilledCourses.length > 0 && options.length === 0 && (
                    <div className="req-options">
                        {item.fulfilledCourses.map((c, i) => (
                            <span key={i} className="req-option-chip chip-fulfilled">{c}</span>
                        ))}
                    </div>
                )}
                {!fulfilled && item.suggestedCourses?.length > 0 && options.length === 0 && (
                    <div className="req-options">
                        {item.suggestedCourses.map((c, i) => (
                            <span key={i} className="req-option-chip chip-suggested">{c}</span>
                        ))}
                    </div>
                )}
            </div>
        </div>
    );
}

// ─── Helpers ───

/** Parse the externally-tagged Requirement enum */
function parseRequirement(req) {
    if (!req) return { type: "Unknown", data: {} };
    const variants = ["SingleCourse", "CourseGroup", "AnyOf", "AllOf", "Concentration", "Restriction", "DoubleCount"];
    for (const v of variants) {
        if (req[v] !== undefined) return { type: v, data: req[v] };
    }
    // Direct fields (already unwrapped by serde)
    if (req.possibilities) return { type: "SingleCourse", data: req };
    if (req.department !== undefined || req.attr !== undefined) return { type: "Restriction", data: req };
    return { type: "Unknown", data: req };
}

/** Get a human-readable description of a requirement */
function getDescription(type, data) {
    switch (type) {
        case "SingleCourse":
            return "Complete one of the following:";
        case "CourseGroup":
            return `Complete ${data.number || "N"} of the following:`;
        case "Restriction": {
            let parts = [];
            if (data.number) parts.push(`${data.number} course(s)`);
            if (data.department) parts.push(`from ${Array.isArray(data.department) ? data.department.join("/") : data.department}`);
            if (data.level) parts.push(`level ${data.level}+`);
            if (data.attr) parts.push(`in ${data.attr.join(" or ")}`);
            if (data.excluding) parts.push(`excluding ${data.excluding.join(", ")}`);
            if (data.no_school) parts.push(`not from ${data.no_school}`);
            return parts.join(" ") || "Restriction requirement";
        }
        case "AnyOf":
            return "Complete one of the following options:";
        case "AllOf":
            return "Complete all of the following:";
        case "Concentration":
            return `Complete concentration (${data.number || "N"} courses):`;
        case "DoubleCount":
            return "Double-counted with other requirements";
        default:
            return "Requirement";
    }
}

/** Get the list of course options to display as chips */
function getOptions(type, data) {
    switch (type) {
        case "SingleCourse":
            return data.possibilities || [];
        case "CourseGroup":
            return data.possibilities || [];
        case "Restriction":
            // Show attribute names as "pseudo-options"
            if (data.attr && data.attr.length > 0) {
                return data.attr.map(a => `[${a}]`);
            }
            return [];
        case "AnyOf": {
            // Flatten nested requirements into readable items
            const items = [];
            (data.possibilities || []).forEach((subReq) => {
                const { type: t2, data: d2 } = parseRequirement(subReq);
                if (t2 === "SingleCourse") {
                    items.push(...(d2.possibilities || []));
                } else if (t2 === "AllOf") {
                    const names = [];
                    (d2.requirements || []).forEach((r) => {
                        const { type: t3, data: d3 } = parseRequirement(r);
                        if (t3 === "SingleCourse") names.push(...(d3.possibilities || []));
                    });
                    if (names.length > 0) items.push(names.join(" + "));
                }
            });
            return items;
        }
        case "AllOf": {
            const items = [];
            (data.requirements || []).forEach((subReq) => {
                const { type: t2, data: d2 } = parseRequirement(subReq);
                if (t2 === "SingleCourse") items.push(...(d2.possibilities || []));
            });
            return items;
        }
        default:
            return [];
    }
}

/** Get category from a requirement object */
function getCategory(req) {
    if (!req) return "Other";
    // Direct category field
    if (req.category) return req.category;
    // Externally tagged enum
    const variants = ["SingleCourse", "CourseGroup", "AnyOf", "AllOf", "Concentration", "Restriction", "DoubleCount"];
    for (const v of variants) {
        if (req[v]?.category) return req[v].category;
    }
    return "Other";
}

/** Unique key for matching unfulfilled requirements to their suggestions */
function getReqKey(req) {
    if (!req) return "unknown";
    const { type, data } = parseRequirement(req);
    switch (type) {
        case "SingleCourse":
            return `SC:${(data.possibilities || []).slice(0, 3).join(",")}`;
        case "CourseGroup":
            return `CG:${data.number}:${(data.possibilities || []).slice(0, 3).join(",")}`;
        case "Restriction":
            return `R:${data.number}:${(data.department || []).join(",")}:${(data.attr || []).join(",")}:${data.level || ""}`;
        case "AnyOf":
            return `AO:${(data.possibilities || []).length}`;
        case "AllOf":
            return `AL:${(data.requirements || []).length}`;
        default:
            return JSON.stringify(req).slice(0, 50);
    }
}
