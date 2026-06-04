export const DEGREE_COLORS = [
    "#a51c30", // Penn red
    "#059669", // teal
    "#d97706", // amber
    "#7c3aed", // purple
];

/** Map `${school}-${major}` → stripe color (matches schedule legend). */
export function buildDegreeColorMap(degreeResults = []) {
    const map = {};
    degreeResults.forEach((result, i) => {
        if (result?.error) return;
        map[`${result.school}-${result.major}`] = DEGREE_COLORS[i % DEGREE_COLORS.length];
    });
    return map;
}

export function degreeLabelForResult(result) {
    if (!result) return null;
    return `${result.school}-${result.major}`;
}
