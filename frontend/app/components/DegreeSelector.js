"use client";

import { useState, useMemo, useEffect } from "react";
import { API_BASE } from "@/lib/api";

function normalizeConcentrations(list) {
    return [...new Set(list.filter((c) => c && c !== "None"))];
}

export default function DegreeSelector({ degreeCatalog, degrees, setDegrees }) {
    const [selectedSchool, setSelectedSchool] = useState("");
    const [selectedMajor, setSelectedMajor] = useState("");
    const [selectedConcentration, setSelectedConcentration] = useState("");
    const [selectedConcentration2, setSelectedConcentration2] = useState("");
    const [concentrations, setConcentrations] = useState([]);
    const [concentrationsLoading, setConcentrationsLoading] = useState(false);

    const selectedSchoolEntry = useMemo(
        () => degreeCatalog?.find((s) => s.display_name === selectedSchool),
        [degreeCatalog, selectedSchool]
    );

    const selectedMajorEntry = useMemo(
        () => selectedSchoolEntry?.majors?.find((m) => m.display_name === selectedMajor),
        [selectedSchoolEntry, selectedMajor]
    );

    const schoolCode = selectedSchoolEntry?.school_code ?? "";
    const majorCode = selectedMajorEntry?.api_code ?? "";
    const isWharton = schoolCode === "WH";

    useEffect(() => {
        if (!schoolCode || !majorCode) {
            setConcentrations([]);
            setSelectedConcentration("");
            setSelectedConcentration2("");
            return;
        }

        const controller = new AbortController();
        setConcentrationsLoading(true);

        const params = new URLSearchParams({ school: schoolCode, major: majorCode });

        fetch(`${API_BASE}/concentrations?${params}`, { signal: controller.signal })
            .then((r) => r.json())
            .then((data) => {
                setConcentrations(data.concentrations || []);
                setSelectedConcentration("");
                setSelectedConcentration2("");
            })
            .catch((err) => {
                if (err.name !== "AbortError") setConcentrations([]);
            })
            .finally(() => setConcentrationsLoading(false));

        return () => controller.abort();
    }, [schoolCode, majorCode]);

    const buildConcentrationsList = () => {
        if (concentrations.length === 0) return [];
        const c1 = selectedConcentration || concentrations[0];
        if (!isWharton || !selectedConcentration2 || selectedConcentration2 === c1) {
            return normalizeConcentrations([c1]);
        }
        return normalizeConcentrations([c1, selectedConcentration2]);
    };

    const formatConcLabel = (concList) => {
        if (!concList?.length) return null;
        return concList.join(" + ");
    };

    const addDegree = () => {
        if (!selectedSchoolEntry || !selectedMajorEntry) return;
        const concList = buildConcentrationsList();
        const concLegacy = concList[0] ?? null;

        const isDup = degrees.some(
            (d) =>
                d.schoolCode === schoolCode &&
                d.majorCode === majorCode &&
                JSON.stringify(normalizeConcentrations(d.concentrations || (d.concentration ? [d.concentration] : []))) ===
                    JSON.stringify(concList)
        );
        if (isDup) return;

        setDegrees((prev) => [
            ...prev,
            {
                schoolCode,
                majorCode,
                concentrations: concList,
                concentration: concLegacy,
                displaySchool: selectedSchoolEntry.display_name,
                displayMajor: selectedMajorEntry.display_name,
            },
        ]);

        setSelectedMajor("");
        setSelectedConcentration("");
        setSelectedConcentration2("");
    };

    const removeDegree = (index) => {
        setDegrees((prev) => prev.filter((_, i) => i !== index));
    };

    const secondConcOptions = concentrations.filter(
        (c) => c !== (selectedConcentration || concentrations[0])
    );

    if (!degreeCatalog?.length) {
        return (
            <div className="degree-bar">
                <span style={{ fontSize: "0.82rem", color: "var(--text-muted)" }}>Loading schools…</span>
            </div>
        );
    }

    return (
        <div className="degree-bar">
            <span style={{ fontSize: "0.82rem", fontWeight: 700, color: "var(--text-secondary)", whiteSpace: "nowrap" }}>
                Degrees:
            </span>

            {degrees.map((d, i) => {
                const concList = normalizeConcentrations(
                    d.concentrations || (d.concentration ? [d.concentration] : [])
                );
                const concLabel = formatConcLabel(concList);
                return (
                    <div key={i} className="degree-chip fade-in">
                        <div>
                            <div className="degree-chip-label">
                                {d.displayMajor || `${d.schoolCode} — ${d.majorCode}`}
                            </div>
                            {concLabel && (
                                <div className="degree-chip-sub">
                                    {d.displaySchool ? d.displaySchool.split("(")[0].trim() : d.schoolCode}
                                    {" · "}Conc: {concLabel}
                                </div>
                            )}
                            {!concLabel && d.displaySchool && (
                                <div className="degree-chip-sub">
                                    {d.displaySchool.split("(")[0].trim()}
                                </div>
                            )}
                        </div>
                        <button className="remove-btn" onClick={() => removeDegree(i)}>✕</button>
                    </div>
                );
            })}

            <div className="degree-form">
                <select
                    value={selectedSchool}
                    onChange={(e) => {
                        setSelectedSchool(e.target.value);
                        setSelectedMajor("");
                        setSelectedConcentration("");
                        setSelectedConcentration2("");
                    }}
                >
                    <option value="">School…</option>
                    {degreeCatalog.map((school) => (
                        <option key={school.school_code} value={school.display_name}>
                            {school.display_name}
                        </option>
                    ))}
                </select>

                {selectedSchool && (
                    <select
                        value={selectedMajor}
                        onChange={(e) => {
                            setSelectedMajor(e.target.value);
                            setSelectedConcentration("");
                            setSelectedConcentration2("");
                        }}
                    >
                        <option value="">Major…</option>
                        {selectedSchoolEntry?.majors?.map((m) => (
                            <option key={m.api_code} value={m.display_name}>
                                {m.display_name}
                            </option>
                        ))}
                    </select>
                )}

                {concentrations.length > 0 && selectedMajor && (
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
                                {c}
                            </option>
                        ))}
                    </select>
                )}

                {isWharton && concentrations.length > 0 && selectedMajor && (
                    <select
                        value={selectedConcentration2}
                        onChange={(e) => setSelectedConcentration2(e.target.value)}
                        disabled={concentrationsLoading}
                        title="Optional second concentration (double concentration)"
                    >
                        <option value="">2nd concentration (optional)…</option>
                        {secondConcOptions.map((c) => (
                            <option key={c} value={c}>
                                {c}
                            </option>
                        ))}
                    </select>
                )}

                <button
                    className="btn btn-primary btn-sm"
                    onClick={addDegree}
                    disabled={!selectedSchool || !selectedMajor || concentrationsLoading}
                >
                    + Add
                </button>
            </div>
        </div>
    );
}
