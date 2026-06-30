"use client";

import { useState, useMemo, useEffect, useRef, useCallback } from "react";
import { createPortal } from "react-dom";
import { API_BASE } from "@/lib/api";
import {
    concentrationsFromCatalog,
    formatConcentrationDropdownLabel,
    implementedMajorsForSchool,
    implementedMinorsForSchool,
    implementedSchools,
    implementedSchoolsForMinors,
    normalizeConcentrations,
} from "@/lib/degreeDisplay";

/**
 * @param {{
 *   open: boolean,
 *   mode: "add" | "edit",
 *   kind: "major" | "minor",
 *   catalog: Array,
 *   concentrationCatalog?: Record<string, string[]>,
 *   anchorRef: React.RefObject<HTMLElement | null>,
 *   initial?: { schoolCode?: string, majorCode?: string, concentrations?: string[] },
 *   onClose: () => void,
 *   onSave: (entry: object) => void,
 * }} props
 */
export default function DegreeProgramPopover({
    open,
    mode,
    kind,
    catalog,
    concentrationCatalog = {},
    anchorRef,
    initial,
    onClose,
    onSave,
}) {
    const popoverRef = useRef(null);
    const [selectedSchool, setSelectedSchool] = useState("");
    const [majorQuery, setMajorQuery] = useState("");
    const [highlightedMajorCode, setHighlightedMajorCode] = useState("");
    const [selectedMajorCode, setSelectedMajorCode] = useState("");
    const [selectedConcentration, setSelectedConcentration] = useState("");
    const [selectedConcentration2, setSelectedConcentration2] = useState("");
    const [concentrations, setConcentrations] = useState([]);
    const [concentrationsLoading, setConcentrationsLoading] = useState(false);
    const [listOpen, setListOpen] = useState(false);
    const [mounted, setMounted] = useState(false);

    useEffect(() => {
        setMounted(true);
    }, []);

    const selectableSchools = useMemo(
        () => (kind === "minor" ? implementedSchoolsForMinors(catalog) : implementedSchools(catalog)),
        [catalog, kind],
    );

    const selectedSchoolEntry = useMemo(
        () => selectableSchools.find((s) => s.display_name === selectedSchool),
        [selectableSchools, selectedSchool],
    );

    const selectablePrograms = useMemo(() => {
        if (kind === "minor") {
            return implementedMinorsForSchool(selectedSchoolEntry);
        }
        return implementedMajorsForSchool(selectedSchoolEntry);
    }, [selectedSchoolEntry, kind]);

    const filteredPrograms = useMemo(() => {
        const q = majorQuery.trim().toLowerCase();
        if (!q) return selectablePrograms;
        return selectablePrograms.filter(
            (m) =>
                m.display_name.toLowerCase().includes(q)
                || m.api_code.toLowerCase().includes(q),
        );
    }, [selectablePrograms, majorQuery]);

    const selectedMajorEntry = useMemo(
        () => selectablePrograms.find((m) => m.api_code === selectedMajorCode),
        [selectablePrograms, selectedMajorCode],
    );

    const schoolCode = selectedSchoolEntry?.school_code ?? "";
    const majorCode = selectedMajorEntry?.api_code ?? highlightedMajorCode ?? "";
    const isWharton = schoolCode === "WH";

    const applyConcentrationList = useCallback((list) => {
        setConcentrations(list);
        if (mode === "add" || !initial?.concentrations?.length) {
            setSelectedConcentration(list[0] ?? "");
            setSelectedConcentration2("");
        }
    }, [mode, initial?.concentrations]);

    useEffect(() => {
        if (!open) return;
        const schoolEntry = selectableSchools.find(
            (s) => s.school_code === initial?.schoolCode,
        );
        setSelectedSchool(schoolEntry?.display_name ?? "");
        setSelectedMajorCode(initial?.majorCode ?? "");
        setMajorQuery("");
        setHighlightedMajorCode(initial?.majorCode ?? "");
        const concs = normalizeConcentrations(initial?.concentrations || []);
        setSelectedConcentration(concs[0] ?? "");
        setSelectedConcentration2(concs[1] ?? "");
        setListOpen(false);
    }, [open, initial, selectableSchools]);

    useEffect(() => {
        if (!schoolCode || !majorCode) {
            setConcentrations([]);
            if (!open) return;
            setSelectedConcentration("");
            setSelectedConcentration2("");
            return;
        }

        const cached = concentrationsFromCatalog(concentrationCatalog, schoolCode, majorCode);
        if (cached) {
            setConcentrationsLoading(false);
            applyConcentrationList(cached);
            return;
        }

        const controller = new AbortController();
        setConcentrationsLoading(true);
        const params = new URLSearchParams({ school: schoolCode, major: majorCode, kind });

        fetch(`${API_BASE}/concentrations?${params}`, { signal: controller.signal })
            .then((r) => r.json())
            .then((data) => {
                applyConcentrationList(data.concentrations || []);
            })
            .catch((err) => {
                if (err.name !== "AbortError") setConcentrations([]);
            })
            .finally(() => setConcentrationsLoading(false));

        return () => controller.abort();
    }, [
        schoolCode,
        majorCode,
        kind,
        open,
        concentrationCatalog,
        applyConcentrationList,
    ]);

    useEffect(() => {
        if (!open) return;
        const onDocClick = (e) => {
            if (
                popoverRef.current?.contains(e.target)
                || anchorRef?.current?.contains(e.target)
            ) {
                return;
            }
            onClose();
        };
        const onKey = (e) => {
            if (e.key === "Escape") onClose();
        };
        document.addEventListener("mousedown", onDocClick);
        document.addEventListener("keydown", onKey);
        return () => {
            document.removeEventListener("mousedown", onDocClick);
            document.removeEventListener("keydown", onKey);
        };
    }, [open, onClose, anchorRef]);

    const buildConcentrationsList = useCallback(() => {
        if (concentrations.length === 0) return [];
        const c1 = selectedConcentration || concentrations[0];
        if (!isWharton || !selectedConcentration2 || selectedConcentration2 === c1) {
            return normalizeConcentrations([c1]);
        }
        return normalizeConcentrations([c1, selectedConcentration2]);
    }, [concentrations, selectedConcentration, selectedConcentration2, isWharton]);

    const showConcentrationStep = concentrations.length > 1;
    const effectiveConc = selectedConcentration || concentrations[0] || "";
    const secondConcOptions = concentrations.filter((c) => c !== effectiveConc);

    const canSave = Boolean(selectedSchoolEntry && selectedMajorEntry && !concentrationsLoading);

    const handleSave = () => {
        if (!canSave) return;
        const concList = buildConcentrationsList();
        onSave({
            kind,
            schoolCode,
            majorCode: selectedMajorEntry.api_code,
            concentrations: concList,
            concentration: concList[0] ?? null,
            displaySchool: selectedSchoolEntry.display_name,
            displayMajor: selectedMajorEntry.display_name,
        });
        onClose();
    };

    if (!open || !mounted) return null;

    const anchorRect = anchorRef?.current?.getBoundingClientRect();
    const style = anchorRect
        ? { top: anchorRect.bottom + 6, left: Math.max(8, anchorRect.left) }
        : {};

    return createPortal(
        <div className="degree-popover-backdrop" aria-hidden>
            <div
                ref={popoverRef}
                className="degree-popover"
                style={style}
                role="dialog"
                aria-label={mode === "edit" ? "Edit program" : "Add program"}
            >
                <div className="degree-popover-title">
                    {mode === "edit" ? "Edit" : "Add"}{" "}
                    {kind === "minor" ? "Minor" : "Degree"}
                </div>

                <label className="degree-popover-field">
                    <span className="degree-popover-label">School</span>
                    <select
                        value={selectedSchool}
                        disabled={mode === "edit"}
                        onChange={(e) => {
                            setSelectedSchool(e.target.value);
                            setSelectedMajorCode("");
                            setMajorQuery("");
                            setHighlightedMajorCode("");
                            setSelectedConcentration("");
                            setSelectedConcentration2("");
                        }}
                    >
                        <option value="">Select school…</option>
                        {selectableSchools.map((school) => (
                            <option key={school.school_code} value={school.display_name}>
                                {school.display_name}
                            </option>
                        ))}
                    </select>
                </label>

                {selectedSchool && (
                    <label className="degree-popover-field">
                        <span className="degree-popover-label">
                            {kind === "minor" ? "Minor" : "Major"}
                        </span>
                        <div className="degree-combobox">
                            <input
                                type="text"
                                placeholder="Search…"
                                value={
                                    selectedMajorEntry && !listOpen
                                        ? selectedMajorEntry.display_name
                                        : majorQuery
                                }
                                onChange={(e) => {
                                    setMajorQuery(e.target.value);
                                    setSelectedMajorCode("");
                                    setListOpen(true);
                                }}
                                onFocus={() => setListOpen(true)}
                            />
                            {listOpen && filteredPrograms.length > 0 && (
                                <ul className="degree-combobox-list" role="listbox">
                                    {filteredPrograms.map((m) => (
                                        <li key={m.api_code}>
                                            <button
                                                type="button"
                                                role="option"
                                                className={
                                                    m.api_code === selectedMajorCode
                                                        ? "degree-combobox-option active"
                                                        : "degree-combobox-option"
                                                }
                                                onMouseEnter={() => {
                                                    setHighlightedMajorCode(m.api_code);
                                                }}
                                                onClick={() => {
                                                    setSelectedMajorCode(m.api_code);
                                                    setMajorQuery(m.display_name);
                                                    setListOpen(false);
                                                    setSelectedConcentration("");
                                                    setSelectedConcentration2("");
                                                }}
                                            >
                                                {m.display_name}
                                            </button>
                                        </li>
                                    ))}
                                </ul>
                            )}
                        </div>
                    </label>
                )}

                {showConcentrationStep && selectedMajorEntry && (
                    <label className="degree-popover-field">
                        <span className="degree-popover-label">Concentration</span>
                        {concentrations.length === 2 ? (
                            <div className="degree-segmented">
                                {concentrations.map((c) => (
                                    <button
                                        key={c}
                                        type="button"
                                        className={
                                            (selectedConcentration || concentrations[0]) === c
                                                ? "degree-segment active"
                                                : "degree-segment"
                                        }
                                        onClick={() => {
                                            setSelectedConcentration(c);
                                            if (c === selectedConcentration2) {
                                                setSelectedConcentration2("");
                                            }
                                        }}
                                    >
                                        {formatConcentrationDropdownLabel(c, schoolCode)}
                                    </button>
                                ))}
                            </div>
                        ) : (
                            <select
                                value={selectedConcentration || concentrations[0]}
                                onChange={(e) => {
                                    setSelectedConcentration(e.target.value);
                                    if (e.target.value === selectedConcentration2) {
                                        setSelectedConcentration2("");
                                    }
                                }}
                                disabled={concentrationsLoading}
                            >
                                {concentrations.map((c) => (
                                    <option key={c} value={c}>
                                        {formatConcentrationDropdownLabel(c, schoolCode)}
                                    </option>
                                ))}
                            </select>
                        )}
                    </label>
                )}

                {isWharton && concentrations.length > 0 && selectedMajorEntry && (
                    <label className="degree-popover-field">
                        <span className="degree-popover-label">2nd concentration (optional)</span>
                        <select
                            value={selectedConcentration2}
                            onChange={(e) => setSelectedConcentration2(e.target.value)}
                            disabled={concentrationsLoading}
                        >
                            <option value="">None</option>
                            {secondConcOptions.map((c) => (
                                <option key={c} value={c}>
                                    {formatConcentrationDropdownLabel(c, schoolCode)}
                                </option>
                            ))}
                        </select>
                    </label>
                )}

                <div className="degree-popover-actions">
                    <button type="button" className="btn btn-ghost btn-sm" onClick={onClose}>
                        Cancel
                    </button>
                    <button
                        type="button"
                        className="btn btn-primary btn-sm"
                        onClick={handleSave}
                        disabled={!canSave}
                    >
                        {mode === "edit" ? "Save" : "Add"}
                    </button>
                </div>
            </div>
        </div>,
        document.body,
    );
}
