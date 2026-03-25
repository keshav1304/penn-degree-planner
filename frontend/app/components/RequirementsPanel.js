"use client";

import { useState } from "react";

export default function RequirementsPanel({ scheduleData, degrees }) {
    const [activeTab, setActiveTab] = useState(0);

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

    // Ensure activeTab is valid
    const tabIndex = Math.min(activeTab, results.length - 1);
    const current = results[tabIndex];

    if (!current) return null;

    const fulfilledCount = current.fulfilled_requirements?.length || 0;
    const unfulfilledCount = current.unfulfilled_requirements?.length || 0;
    const totalCount = fulfilledCount + unfulfilledCount;

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
            {!current.error && (
                <div style={{
                    display: "flex",
                    gap: 12,
                    marginBottom: 14,
                    fontSize: "0.78rem",
                    fontWeight: 600,
                }}>
                    <span style={{ color: "var(--success)" }}>
                        ✅ {fulfilledCount} fulfilled
                    </span>
                    <span style={{ color: "var(--danger)" }}>
                        ❌ {unfulfilledCount} unfulfilled
                    </span>
                    <span style={{ color: "var(--text-muted)" }}>
                        / {totalCount} total
                    </span>
                    {totalCount > 0 && (
                        <div style={{
                            flex: 1,
                            height: 6,
                            background: "var(--bg-secondary)",
                            borderRadius: 3,
                            overflow: "hidden",
                            alignSelf: "center",
                        }}>
                            <div style={{
                                width: `${(fulfilledCount / totalCount) * 100}%`,
                                height: "100%",
                                background: "linear-gradient(90deg, var(--success), var(--accent-teal))",
                                borderRadius: 3,
                                transition: "width 0.5s ease",
                            }} />
                        </div>
                    )}
                </div>
            )}

            {/* Requirements list */}
            <div className="req-list">
                {/* Fulfilled */}
                {current.fulfilled_requirements?.map((req, i) => (
                    <div key={`f-${i}`} className="req-item fulfilled fade-in">
                        <span className="req-status-icon">✅</span>
                        <div className="req-content">
                            <div className="req-category">
                                {req.requirement?.category || getCategoryFromReq(req.requirement)}
                            </div>
                            <div className="req-courses">
                                Fulfilled by: {req.course_ids?.join(", ") || "—"}
                            </div>
                        </div>
                    </div>
                ))}

                {/* Unfulfilled */}
                {current.suggested_for_unfulfilled?.map((suggestion, i) => {
                    return (
                        <div key={`u-${i}`} className="req-item unfulfilled fade-in">
                            <span className="req-status-icon">❌</span>
                            <div className="req-content">
                                <div className="req-category">
                                    {getCategoryFromReq(suggestion.requirement)}
                                </div>
                                {suggestion && suggestion.course_ids?.length > 0 && (
                                    <div className="req-courses">
                                        Suggested: {suggestion.course_ids.slice(0, 5).join(", ")}
                                        {suggestion.course_ids.length > 5 && ` +${suggestion.course_ids.length - 5} more`}
                                    </div>
                                )}
                            </div>
                        </div>
                    );
                })}

                {fulfilledCount === 0 && unfulfilledCount === 0 && !current.error && (
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

// Extract category name from the Rust Requirement enum serialized as tagged JSON
function getCategoryFromReq(req) {
    if (!req) return "Unknown Requirement";

    // The Rust Requirement enum serializes as an object with a variant key
    // e.g., { "SingleCourse": { "category": "...", ... } }
    // or it may have a direct category field
    if (req.category) return req.category;

    // Try to extract from enum variant structure
    const variants = ["SingleCourse", "CourseGroup", "AnyOf", "AllOf", "Concentration", "Restriction", "DoubleCount"];
    for (const v of variants) {
        if (req[v]) {
            return req[v].category || v;
        }
    }

    // If the requirement is directly the variant value
    if (typeof req === "object") {
        const keys = Object.keys(req);
        for (const key of keys) {
            if (typeof req[key] === "object" && req[key]?.category) {
                return req[key].category;
            }
        }
    }

    return "Requirement";
}
