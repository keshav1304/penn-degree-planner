/** @typedef {{ schoolCode?: string, majorCode?: string, displaySchool?: string, displayMajor?: string, concentration?: string | null, concentrations?: string[] }} DegreeEntry */

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

function resolveFromCatalog(degreeCatalog, schoolCode, majorCode) {
    if (!degreeCatalog?.length || !schoolCode) {
        return { displaySchool: schoolCode || "", displayMajor: majorCode || "" };
    }
    const schoolEntry = degreeCatalog.find((s) => s.school_code === schoolCode);
    const majorEntry = schoolEntry?.majors?.find((m) => m.api_code === majorCode);
    return {
        displaySchool: schoolEntry?.display_name || schoolCode,
        displayMajor: majorEntry?.display_name || majorCode || "Degree",
    };
}

/**
 * SEAS Masters layout: full major on line 1, full school name on line 2.
 * Undergraduate-style concentration suffix when concentrations are set.
 */
export function formatDegreeDisplay(degree, result, degreeCatalog) {
    const schoolCode = degree?.schoolCode || result?.school || "";
    const majorCode = degree?.majorCode || result?.major || "";
    const fromCatalog = resolveFromCatalog(degreeCatalog, schoolCode, majorCode);

    const major = degree?.displayMajor || fromCatalog.displayMajor || "Degree";
    const school = degree?.displaySchool || fromCatalog.displaySchool || schoolCode;
    const concLabel = formatConcentrationLabel(getDegreeConcentrations(degree));
    const schoolLine = concLabel ? `${school} · Conc: ${concLabel}` : school;

    return { major, school, schoolLine, concLabel };
}
