"use client";

import { useState, useMemo, useEffect } from "react";
import { API_BASE } from "@/lib/api";

// Map display school names to API school codes
const SCHOOL_CODE_MAP = {
    "College of Arts and Sciences (CAS)": "CAS",
    "School of Engineering and Applied Science (SEAS)": "SEAS",
    "The Wharton School (WH)": "WH",
    "School of Nursing (NURS)": "NURS",
};

// Parse major code from display name like "Electrical Engineering (EE)" → "EE"
function parseMajorCode(displayName) {
    const match = displayName.match(/\(([^)]+)\)$/);
    return match ? match[1] : displayName;
}

// Maps for major code → API major code
const MAJOR_API_MAP = {
    CE: "CE",
    EE: "EE",
    CIS: "CIS",
    MEAM: "MEAM",
    MSE: "MSE",
    CBE: "CBE",
    AI: "AI",
    FL: "WH_FL",
    NO_FL: "WH_NOFL",
    NOFL_MT: "WH_NOFL_MT",
    NA: "NA",
};

export default function DegreeSelector({ allMajors, degrees, setDegrees }) {
    const [selectedSchool, setSelectedSchool] = useState("");
    const [selectedMajor, setSelectedMajor] = useState("");
    const [selectedConcentration, setSelectedConcentration] = useState("");
    const [concentrations, setConcentrations] = useState([]);
    const [concentrationsLoading, setConcentrationsLoading] = useState(false);

    const schoolNames = useMemo(() => Object.keys(allMajors), [allMajors]);

    const majorOptions = useMemo(() => {
        if (!selectedSchool || !allMajors[selectedSchool]) return [];
        return allMajors[selectedSchool];
    }, [selectedSchool, allMajors]);

    const currentMajorApiCode = useMemo(() => {
        if (!selectedMajor) return "";
        const code = parseMajorCode(selectedMajor);
        return MAJOR_API_MAP[code] || code;
    }, [selectedMajor]);

    const schoolCode = useMemo(() => {
        return SCHOOL_CODE_MAP[selectedSchool] || selectedSchool;
    }, [selectedSchool]);

    useEffect(() => {
        if (!selectedSchool || !currentMajorApiCode) {
            setConcentrations([]);
            setSelectedConcentration("");
            return;
        }

        const controller = new AbortController();
        setConcentrationsLoading(true);

        const params = new URLSearchParams({
            school: schoolCode,
            major: currentMajorApiCode,
        });

        fetch(`${API_BASE}/concentrations?${params}`, { signal: controller.signal })
            .then(r => r.json())
            .then(data => {
                setConcentrations(data.concentrations || []);
                setSelectedConcentration("");
            })
            .catch(err => {
                if (err.name !== "AbortError") setConcentrations([]);
            })
            .finally(() => setConcentrationsLoading(false));

        return () => controller.abort();
    }, [selectedSchool, currentMajorApiCode, schoolCode]);

    const addDegree = () => {
        if (!selectedSchool || !selectedMajor) return;
        const majorCode = currentMajorApiCode;
        const rawConc = concentrations.length > 0 ? (selectedConcentration || concentrations[0]) : null;
        const conc = rawConc === "None" ? null : rawConc;

        // Check for duplicate
        const isDup = degrees.some(
            d => d.schoolCode === schoolCode && d.majorCode === majorCode && d.concentration === conc
        );
        if (isDup) return;

        setDegrees(prev => [
            ...prev,
            {
                schoolCode,
                majorCode,
                concentration: conc,
                displaySchool: selectedSchool,
                displayMajor: selectedMajor,
            },
        ]);

        setSelectedMajor("");
        setSelectedConcentration("");
    };

    const removeDegree = (index) => {
        setDegrees(prev => prev.filter((_, i) => i !== index));
    };

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
                    {schoolNames.map(name => (
                        <option key={name} value={name}>{name}</option>
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
                        {majorOptions.map(m => (
                            <option key={m} value={m}>{m}</option>
                        ))}
                    </select>
                )}

                {concentrations.length > 0 && selectedMajor && (
                    <select
                        value={selectedConcentration || concentrations[0]}
                        onChange={(e) => setSelectedConcentration(e.target.value)}
                        disabled={concentrationsLoading}
                    >
                        {concentrations.map(c => (
                            <option key={c} value={c}>{c}</option>
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
