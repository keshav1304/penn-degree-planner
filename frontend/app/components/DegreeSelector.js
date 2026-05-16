"use client";

import { useState, useMemo, useEffect } from "react";
import { API_BASE } from "@/lib/api";

export default function DegreeSelector({ degreeCatalog, degrees, setDegrees }) {
    const [selectedSchool, setSelectedSchool] = useState("");
    const [selectedMajor, setSelectedMajor] = useState("");
    const [selectedConcentration, setSelectedConcentration] = useState("");
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

    useEffect(() => {
        if (!schoolCode || !majorCode) {
            setConcentrations([]);
            setSelectedConcentration("");
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
            })
            .catch((err) => {
                if (err.name !== "AbortError") setConcentrations([]);
            })
            .finally(() => setConcentrationsLoading(false));

        return () => controller.abort();
    }, [schoolCode, majorCode]);

    const addDegree = () => {
        if (!selectedSchoolEntry || !selectedMajorEntry) return;
        const rawConc = concentrations.length > 0 ? (selectedConcentration || concentrations[0]) : null;
        const conc = rawConc === "None" ? null : rawConc;

        const isDup = degrees.some(
            (d) =>
                d.schoolCode === schoolCode &&
                d.majorCode === majorCode &&
                d.concentration === conc
        );
        if (isDup) return;

        setDegrees((prev) => [
            ...prev,
            {
                schoolCode,
                majorCode,
                concentration: conc,
                displaySchool: selectedSchoolEntry.display_name,
                displayMajor: selectedMajorEntry.display_name,
            },
        ]);

        setSelectedMajor("");
        setSelectedConcentration("");
    };

    const removeDegree = (index) => {
        setDegrees((prev) => prev.filter((_, i) => i !== index));
    };

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

            {degrees.map((d, i) => (
                <div key={i} className="degree-chip fade-in">
                    <div>
                        <div className="degree-chip-label">
                            {d.displayMajor || `${d.schoolCode} — ${d.majorCode}`}
                        </div>
                        {d.concentration && (
                            <div className="degree-chip-sub">
                                {d.displaySchool ? d.displaySchool.split("(")[0].trim() : d.schoolCode}
                                {" · "}Conc: {d.concentration}
                            </div>
                        )}
                        {!d.concentration && d.displaySchool && (
                            <div className="degree-chip-sub">
                                {d.displaySchool.split("(")[0].trim()}
                            </div>
                        )}
                    </div>
                    <button className="remove-btn" onClick={() => removeDegree(i)}>✕</button>
                </div>
            ))}

            <div className="degree-form">
                <select
                    value={selectedSchool}
                    onChange={(e) => {
                        setSelectedSchool(e.target.value);
                        setSelectedMajor("");
                        setSelectedConcentration("");
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
                        onChange={(e) => setSelectedConcentration(e.target.value)}
                        disabled={concentrationsLoading}
                    >
                        {concentrations.map((c) => (
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
