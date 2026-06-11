export const DEGREE_COLORS = [
    "#a51c30", // Penn red
    "#059669", // teal
    "#d97706", // amber
    "#7c3aed", // purple
];

export function getDegreeColorForIndex(index) {
    return DEGREE_COLORS[index % DEGREE_COLORS.length];
}

export function buildDegreeColorMap(scheduleData) {
    const map = {};
    if (scheduleData?.degree_results) {
        scheduleData.degree_results.forEach((result, i) => {
            const key = `${result.school}-${result.major}`;
            map[key] = getDegreeColorForIndex(i);
        });
    }
    return map;
}
