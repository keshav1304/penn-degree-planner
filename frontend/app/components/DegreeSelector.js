"use client";

import { useState, useRef } from "react";
import {
    formatDegreeDisplay,
    normalizeConcentrations,
} from "@/lib/degreeDisplay";
import DegreeProgramPopover from "./DegreeProgramPopover";

function programKey(d) {
    const concList = normalizeConcentrations(
        d.concentrations || (d.concentration ? [d.concentration] : []),
    );
    return `${d.schoolCode}:${d.majorCode}:${JSON.stringify(concList)}`;
}

export default function DegreeSelector({
    degreeCatalog,
    degrees,
    setDegrees,
}) {
    const [popover, setPopover] = useState(null);
    const anchorRef = useRef(null);

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

    const renderChip = (d, index) => {
        const { major, schoolLine } = formatDegreeDisplay(d, null, degreeCatalog);
        return (
            <div key={index} className="degree-chip fade-in">
                <button
                    type="button"
                    className="degree-chip-body"
                    onClick={(e) =>
                        openPopover(
                            {
                                mode: "edit",
                                editIndex: index,
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
                    onClick={() => removeAt(index)}
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

    return (
        <div className="degree-bar degree-selector-row">
            <div className="degree-bar-section">
                <span
                    style={{
                        fontSize: "0.82rem",
                        fontWeight: 700,
                        color: "var(--text-secondary)",
                        whiteSpace: "nowrap",
                    }}
                >
                    Degrees:
                </span>

                {degrees.map((d, index) => renderChip(d, index))}

                <button
                    type="button"
                    className="degree-add-btn"
                    aria-label="Add degree"
                    onClick={(e) =>
                        openPopover({ mode: "add" }, e.currentTarget)
                    }
                >
                    +
                </button>
            </div>

            {popover && (
                <DegreeProgramPopover
                    open
                    mode={popover.mode}
                    catalog={degreeCatalog}
                    anchorRef={anchorRef}
                    initial={popover.initial}
                    onClose={closePopover}
                    onSave={handleSave}
                />
            )}
        </div>
    );
}
