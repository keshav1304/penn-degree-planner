"use client";

import { useState, useRef } from "react";
import {
    formatDegreeDisplay,
    normalizeConcentrations,
} from "@/lib/degreeDisplay";
import DegreeProgramPopover from "./DegreeProgramPopover";

function isMinor(degree) {
    return degree?.kind === "minor";
}

function programKey(d) {
    const concList = normalizeConcentrations(
        d.concentrations || (d.concentration ? [d.concentration] : []),
    );
    return `${d.kind || "major"}:${d.schoolCode}:${d.majorCode}:${JSON.stringify(concList)}`;
}

export default function DegreeSelector({
    degreeCatalog,
    minorCatalog = [],
    degrees,
    setDegrees,
}) {
    const [popover, setPopover] = useState(null);
    const anchorRef = useRef(null);

    const majors = degrees.filter((d) => !isMinor(d));
    const minors = degrees.filter((d) => isMinor(d));

    const openPopover = (config, anchorEl) => {
        anchorRef.current = anchorEl;
        setPopover(config);
    };

    const closePopover = () => setPopover(null);

    const handleSave = (entry) => {
        if (popover?.mode === "edit" && popover.editIndex != null) {
            setDegrees((prev) =>
                prev.map((d, i) => (i === popover.editIndex ? { ...d, ...entry } : d)),
            );
            return;
        }

        const isDup = degrees.some((d) => programKey(d) === programKey(entry));
        if (isDup) return;

        setDegrees((prev) => [...prev, entry]);
    };

    const removeAt = (index) => {
        setDegrees((prev) => prev.filter((_, i) => i !== index));
    };

    const renderChip = (d, globalIndex, isMinorChip) => {
        const catalog = isMinorChip ? minorCatalog : degreeCatalog;
        const { major, schoolLine } = formatDegreeDisplay(d, null, catalog);
        return (
            <div
                key={globalIndex}
                className={`degree-chip fade-in${isMinorChip ? " degree-chip-minor" : ""}`}
            >
                <button
                    type="button"
                    className="degree-chip-body"
                    onClick={(e) =>
                        openPopover(
                            {
                                mode: "edit",
                                kind: isMinorChip ? "minor" : "major",
                                editIndex: globalIndex,
                                initial: {
                                    schoolCode: d.schoolCode,
                                    majorCode: d.majorCode,
                                    concentrations: d.concentrations
                                        || (d.concentration ? [d.concentration] : []),
                                },
                            },
                            e.currentTarget.closest(".degree-chip"),
                        )
                    }
                >
                    <div className="degree-chip-label">{major}</div>
                    {schoolLine && <div className="degree-chip-sub">{schoolLine}</div>}
                </button>
                <button
                    type="button"
                    className="remove-btn"
                    aria-label={`Remove ${major}`}
                    onClick={() => removeAt(globalIndex)}
                >
                    ✕
                </button>
            </div>
        );
    };

    if (!degreeCatalog?.length) {
        return (
            <div className="degree-bar">
                <span style={{ fontSize: "0.82rem", color: "var(--text-muted)" }}>
                    Loading schools…
                </span>
            </div>
        );
    }

    const labelStyle = {
        fontSize: "0.82rem",
        fontWeight: 700,
        color: "var(--text-secondary)",
        whiteSpace: "nowrap",
    };

    return (
        <div className="degree-bar degree-selector-row">
            <div className="degree-bar-section">
                <span style={labelStyle}>Degrees:</span>

                {majors.map((d) => {
                    const globalIndex = degrees.indexOf(d);
                    return renderChip(d, globalIndex, false);
                })}

                <button
                    type="button"
                    className="degree-add-btn"
                    aria-label="Add degree"
                    onClick={(e) =>
                        openPopover({ mode: "add", kind: "major" }, e.currentTarget)
                    }
                >
                    +
                </button>
            </div>

            <div className="degree-bar-section degree-bar-section-minors">
                <span style={labelStyle}>Minors:</span>

                {minors.map((d) => {
                    const globalIndex = degrees.indexOf(d);
                    return renderChip(d, globalIndex, true);
                })}

                <button
                    type="button"
                    className="degree-add-btn"
                    aria-label="Add minor"
                    disabled={majors.length === 0}
                    title={majors.length === 0 ? "Add a degree first" : "Add minor"}
                    onClick={(e) =>
                        openPopover({ mode: "add", kind: "minor" }, e.currentTarget)
                    }
                >
                    +
                </button>
            </div>

            {popover && (
                <DegreeProgramPopover
                    open
                    mode={popover.mode}
                    kind={popover.kind}
                    catalog={popover.kind === "minor" ? minorCatalog : degreeCatalog}
                    anchorRef={anchorRef}
                    initial={popover.initial}
                    onClose={closePopover}
                    onSave={handleSave}
                />
            )}
        </div>
    );
}
