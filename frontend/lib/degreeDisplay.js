/** @typedef {{ schoolCode?: string, majorCode?: string, displaySchool?: string, displayMajor?: string, concentration?: string | null, concentrations?: string[] }} DegreeEntry */

const WH_CONCENTRATION_DROPDOWN_LABELS = {
    ACCT: "Accounting",
    BEPP: "Business Economics and Public Policy",
    BUAN: "Business Analytics",
    FNCE: "Finance",
    MAOM: "Marketing & Operations Management",
    MGMT: "Management",
    MKTG: "Marketing",
    STAT: "Statistics and Data Science",
    HCMG: "Health Care Management",
};

/** Wharton concentration picker only: "Finance (FNCE)". */
export function formatWhConcentrationDropdownLabel(code) {
    if (!code) return "";
    const full = WH_CONCENTRATION_DROPDOWN_LABELS[code];
    return full ? `${full} (${code})` : code;
}

export function formatConcentrationDropdownLabel(code, schoolCode) {
    if (schoolCode === "WH") return formatWhConcentrationDropdownLabel(code);
    return code || "";
}

export function normalizeConcentrations(list) {
    return [...new Set((list || []).filter((c) => c && c !== "None"))];
}

export function getDegreeConcentrations(degree) {
    if (!degree) return [];
    return normalizeConcentrations(
        degree.concentrations || (degree.concentration ? [degree.concentration] : []),
    );
}

export function formatConcentrationLabel(concList) {
    const normalized = normalizeConcentrations(concList);
    if (!normalized.length) return null;
    return normalized.join(" + ");
}

/** Strip trailing catalog abbreviations like "(CIS)" or "(WH)" from display labels. */
export function stripAbbreviationSuffix(label) {
    if (!label || typeof label !== "string") return label || "";
    return label.replace(/\s*\([A-Z][A-Z0-9_]+\)\s*$/, "").trim();
}

function resolveFromCatalog(degreeCatalog, schoolCode, majorCode) {
    if (!degreeCatalog?.length || !schoolCode) {
        return {
            displaySchool: stripAbbreviationSuffix(schoolCode || ""),
            displayMajor: stripAbbreviationSuffix(majorCode || ""),
        };
    }
    const schoolEntry = degreeCatalog.find((s) => s.school_code === schoolCode);
    const majorEntry = schoolEntry?.majors?.find((m) => m.api_code === majorCode);
    const displaySchool = stripAbbreviationSuffix(schoolEntry?.display_name || schoolCode);
    const displayMajor = stripAbbreviationSuffix(majorEntry?.display_name || majorCode || "Degree");
    return { displaySchool, displayMajor };
}

/** Human-readable label for API keys like `CAS-ECON`. */
export function formatDegreeApiLabel(schoolCode, majorCode, degreeCatalog) {
    const { displayMajor, displaySchool } = resolveFromCatalog(
        degreeCatalog,
        schoolCode,
        majorCode,
    );
    if (displayMajor && displaySchool) {
        return `${displayMajor} · ${displaySchool}`;
    }
    return displayMajor || `${schoolCode}-${majorCode}`;
}

/** Majors the UI should offer (excludes placeholder / not-implemented entries). */
export function implementedMajorsForSchool(schoolEntry) {
    return (schoolEntry?.majors || []).filter(
        (m) => m.api_code && m.api_code !== "NA",
    );
}

/** Schools that have at least one selectable major. */
export function implementedSchools(degreeCatalog) {
    return (degreeCatalog || []).filter(
        (school) => implementedMajorsForSchool(school).length > 0,
    );
}

/** Minors the UI should offer for a school entry. */
export function implementedMinorsForSchool(schoolEntry) {
    return (schoolEntry?.majors || []).filter(
        (m) => m.api_code && m.api_code !== "NA",
    );
}

/** Schools that have at least one selectable minor. */
export function implementedSchoolsForMinors(minorCatalog) {
    return (minorCatalog || []).filter(
        (school) => implementedMinorsForSchool(school).length > 0,
    );
}

/**
 * SEAS Masters layout: full major on line 1, full school name on line 2.
 * Undergraduate-style concentration suffix when concentrations are set.
 */
export function formatDegreeDisplay(degree, result, degreeCatalog) {
    const schoolCode = degree?.schoolCode || result?.school || "";
    const majorCode = degree?.majorCode || result?.major || "";
    const fromCatalog = resolveFromCatalog(degreeCatalog, schoolCode, majorCode);

    const major = fromCatalog.displayMajor
        || stripAbbreviationSuffix(degree?.displayMajor)
        || "Degree";
    const school = fromCatalog.displaySchool
        || stripAbbreviationSuffix(degree?.displaySchool)
        || schoolCode;
    const concLabel = formatConcentrationLabel(getDegreeConcentrations(degree));
    const schoolLine = concLabel ? `${school} · Conc: ${concLabel}` : school;

    return { major, school, schoolLine, concLabel };
}
