const YEAR_LABELS = ["Freshman", "Sophomore", "Junior", "Senior", "Fifth Year", "Sixth Year"];

export const DEFAULT_SEMESTER_CU_LIMIT = 5.5;
export const DEFAULT_SUMMER_CU_LIMIT = 2.0;

export function defaultSemesterCuLimit(semester) {
    return semester === "Summer" ? DEFAULT_SUMMER_CU_LIMIT : DEFAULT_SEMESTER_CU_LIMIT;
}

export function buildSemesterOptions(maxYear = 4, allowSummer = true) {
    const options = [
        { label: "—", value: "" },
        { label: "Credits Received", value: "Credits-0" },
    ];

    const years = Math.max(4, maxYear);
    for (let y = 1; y <= years; y++) {
        const name = YEAR_LABELS[y - 1] || `Year ${y}`;
        options.push({ label: `${name} Fall`, value: `Fall-${y}` });
        options.push({ label: `${name} Spring`, value: `Spring-${y}` });
        if (allowSummer) {
            options.push({ label: `${name} Summer`, value: `Summer-${y}` });
        }
    }

    return options;
}

export function maxYearFromSchedule(schedule) {
    if (!schedule?.length) return 4;
    const years = schedule.map((s) => s.year).filter((y) => y > 0);
    if (years.length === 0) return 4;
    return Math.max(4, ...years);
}
