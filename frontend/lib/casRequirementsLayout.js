import {
    isPoolConstraintInstanceId,
    isPoolFlexibleSlotInstanceId,
} from "@/lib/courseUtils";
import { courseCountsForDegree, filterAttributeFulfillmentForDegree } from "@/lib/crossDegree";
import { formatDegreeDisplay } from "@/lib/degreeDisplay";
import { attributeFulfillmentMap } from "@/lib/requirementNav";
import { getRequirementInstanceId } from "@/lib/requirementText";

export const CAS_WRITING_HEADING = "Writing Seminar";
export const CAS_GENED_HEADING = "General Education";

export function isCasSchool(school) {
    return school === "CAS";
}

export function casDegreeIndices(results) {
    return results
        .map((r, i) => (isCasSchool(r?.school) ? i : -1))
        .filter((i) => i >= 0);
}

/** Tab descriptors for the requirements panel. */
export function buildRequirementTabs(results, degrees, degreeCatalog, minorCatalog = []) {
    const catalogFor = (degree) => (
        degree?.kind === "minor" && minorCatalog?.length ? minorCatalog : degreeCatalog
    );

    const entries = degrees
        .map((degree, index) => ({ degree, index, result: results[index] }))
        .filter(({ result }) => result);

    const majorEntries = entries.filter(({ degree }) => degree?.kind !== "minor");
    const casIndices = majorEntries
        .filter(({ result }) => isCasSchool(result.school))
        .map(({ index }) => index);
    const dualCas = casIndices.length >= 2;
    let casCombinedEmitted = false;

    const makeTab = (entry, type) => {
        const { major, schoolLine } = formatDegreeDisplay(
            entry.degree,
            entry.result,
            catalogFor(entry.degree),
        );
        return {
            id: `deg-${entry.index}`,
            type,
            index: entry.index,
            label: major,
            schoolLine,
        };
    };

    const tabs = [];

    for (const entry of entries) {
        const { degree, result } = entry;
        const isMinor = degree?.kind === "minor";
        const isCasMajor = !isMinor && isCasSchool(result.school);

        if (isMinor) {
            tabs.push(makeTab(entry, "degree"));
            continue;
        }

        if (dualCas && isCasMajor) {
            if (!casCombinedEmitted) {
                const majorLabels = casIndices.map((i) => {
                    const { major } = formatDegreeDisplay(degrees[i], results[i], degreeCatalog);
                    return major;
                });
                tabs.push({
                    id: "cas-combined",
                    type: "cas-combined",
                    indices: casIndices,
                    label: majorLabels.join(" + "),
                    schoolLine: "College of Arts & Sciences",
                });
                casCombinedEmitted = true;
            }
            continue;
        }

        const type = isCasMajor ? "cas-single" : "degree";
        tabs.push(makeTab(entry, type));
    }

    return tabs;
}

export function resolveActiveTabIndex(tabs, activeTab, navTarget) {
    if (navTarget?.degreeIndex != null) {
        const navIdx = navTarget.degreeIndex;
        const casTab = tabs.find((t) => t.type === "cas-combined");
        if (casTab?.indices?.includes(navIdx)) {
            return tabs.indexOf(casTab);
        }
        const direct = tabs.findIndex((t) => t.index === navIdx);
        if (direct >= 0) return direct;
    }
    return Math.min(activeTab, Math.max(0, tabs.length - 1));
}

export function normalizeCategory(cat) {
    if (!cat || typeof cat !== "string" || !cat.trim()) return "Other";
    return cat.trim();
}

export function getCategory(req) {
    if (!req) return "Other";
    if (req.category) return req.category;
    for (const v of [
        "SingleCourse",
        "CourseGroup",
        "AnyOf",
        "AllOf",
        "Concentration",
        "Restriction",
        "CoursePool",
    ]) {
        if (req[v]?.category) return req[v].category;
    }
    return "Other";
}

/** Classify a CAS requirement row into a super-section. */
export function casRequirementSection(item) {
    const cat = normalizeCategory(item.category);
    const id = item.instanceId || "";

    if (cat === CAS_WRITING_HEADING || id === "0") {
        return "writing";
    }
    if (
        cat.startsWith("Foundational Approaches")
        || cat.startsWith("Sectors of Knowledge")
        || cat === CAS_GENED_HEADING
        || isPoolConstraintInstanceId(id)
        || isPoolFlexibleSlotInstanceId(id)
    ) {
        return "genEd";
    }
    if (/^1:f/.test(id)) {
        return "major";
    }
    return "major";
}

function degreeLabelFor(results, index) {
    const r = results[index];
    return `${r.school}-${r.major}`;
}

function coursesForAnyCasLabel(courseIds, casIndices, results, courseDegreesMap) {
    return (courseIds || []).filter((id) =>
        casIndices.some((idx) =>
            courseCountsForDegree(id, degreeLabelFor(results, idx), courseDegreesMap),
        ),
    );
}

export function mapRequirementForDegreeLabel(
    mapped,
    degreeLabel,
    courseDegreesMap,
    { fulfilledDefault, partialDefault },
) {
    const fulfilledCourses = (mapped.course_ids || []).filter((id) =>
        courseCountsForDegree(id, degreeLabel, courseDegreesMap),
    );
    const attributeFulfillment = filterAttributeFulfillmentForDegree(
        attributeFulfillmentMap(mapped),
        degreeLabel,
        courseDegreesMap,
    );
    const hasAllocatedFulfillment =
        fulfilledCourses.length > 0
        || (attributeFulfillment
            && [...attributeFulfillment.values()].some((ids) => ids.length > 0));
    return {
        category: normalizeCategory(getCategory(mapped.requirement)),
        fulfilled: fulfilledDefault && hasAllocatedFulfillment,
        partial: partialDefault && hasAllocatedFulfillment,
        committedAnyofBranch: mapped.committed_anyof_branch ?? null,
        fulfilledCourses,
        requirement: mapped.requirement,
        instanceId: getRequirementInstanceId(mapped),
        attributeFulfillment,
    };
}

export function mapRequirementForCasCombined(
    mapped,
    casIndices,
    results,
    courseDegreesMap,
    { fulfilledDefault, partialDefault },
) {
    const fulfilledCourses = coursesForAnyCasLabel(
        mapped.course_ids,
        casIndices,
        results,
        courseDegreesMap,
    );
    let attributeFulfillment;
    for (const idx of casIndices) {
        const label = degreeLabelFor(results, idx);
        const filtered = filterAttributeFulfillmentForDegree(
            attributeFulfillmentMap(mapped),
            label,
            courseDegreesMap,
        );
        if (filtered?.size) {
            attributeFulfillment = filtered;
            break;
        }
    }
    const hasAllocatedFulfillment =
        fulfilledCourses.length > 0
        || (attributeFulfillment
            && [...attributeFulfillment.values()].some((ids) => ids.length > 0));
    return {
        category: normalizeCategory(getCategory(mapped.requirement)),
        fulfilled: fulfilledDefault && hasAllocatedFulfillment,
        partial: partialDefault && hasAllocatedFulfillment,
        committedAnyofBranch: mapped.committed_anyof_branch ?? null,
        fulfilledCourses,
        requirement: mapped.requirement,
        instanceId: getRequirementInstanceId(mapped),
        attributeFulfillment,
    };
}

function collectMappedItems(result, mapFn) {
    const items = [];
    const pushIfSchedulable = (mapped, opts) => {
        if (isPoolConstraintInstanceId(getRequirementInstanceId(mapped))) return;
        if (isPoolFlexibleSlotInstanceId(getRequirementInstanceId(mapped))) return;
        items.push(mapFn(mapped, opts));
    };
    (result.fulfilled_requirements || []).forEach((mapped) => {
        pushIfSchedulable(mapped, { fulfilledDefault: true, partialDefault: false });
    });
    (result.unfulfilled_requirements || []).forEach((mapped, rowIdx) => {
        const req = mapped?.requirement ?? mapped;
        if (isPoolConstraintInstanceId(getRequirementInstanceId(mapped))) return;
        if (isPoolFlexibleSlotInstanceId(getRequirementInstanceId(mapped))) return;
        const item = mapFn(
            { ...mapped, requirement: req },
            { fulfilledDefault: false, partialDefault: Boolean(mapped.partial) },
        );
        items.push({
            ...item,
            instanceId: item.instanceId ?? `u-${rowIdx}`,
        });
    });
    return items;
}

export function collectDegreeRequirements(
    result,
    degreeIndex,
    degreeLabel,
    courseDegreesMap,
) {
    const mapFn = (mapped, opts) =>
        mapRequirementForDegreeLabel(mapped, degreeLabel, courseDegreesMap, opts);
    return collectMappedItems(result, mapFn).map((item) => ({
        ...item,
        degreeIndex,
    }));
}

function categoryForItem(item, pools) {
    const cat = normalizeCategory(item.category);
    for (const pool of pools) {
        const poolCat = normalizeCategory(pool.category);
        if (cat === poolCat || cat === `${poolCat} - Pool course`) {
            return poolCat;
        }
    }
    return cat;
}

function groupItemsByCategory(items, pools) {
    const categoryMap = {};
    items.forEach((item) => {
        const cat = categoryForItem(item, pools);
        if (!categoryMap[cat]) categoryMap[cat] = [];
        categoryMap[cat].push(item);
    });
    return categoryMap;
}

function orderedCategoriesForResult(result, categoryMap) {
    const order = (result.category_order || []).map(normalizeCategory);
    const ordered = [...order];
    Object.keys(categoryMap).forEach((c) => {
        if (!ordered.includes(c)) ordered.push(c);
    });
    return ordered;
}

function mergeCasGenEdRow(rows, casIndices, results, courseDegreesMap) {
    if (!rows.length) return null;
    const template = rows[0];
    const fulfilled = rows.some((r) => r.fulfilled);
    const fulfilledByMajor = rows.some((r) => r.fulfilled_by_major);
    const matched = [
        ...new Set(
            rows.flatMap((r) =>
                coursesForAnyCasLabel(r.matched_courses, casIndices, results, courseDegreesMap),
            ),
        ),
    ];
    return {
        ...template,
        fulfilled: fulfilled || matched.length > 0,
        fulfilled_by_major: fulfilledByMajor,
        matched_courses: matched,
    };
}

export function mergeCasGenEdInfo(casIndices, results, courseDegreesMap) {
    const primary = results[casIndices[0]];
    if (!primary?.cas_gen_ed) return null;

    const allGenEd = casIndices.map((i) => results[i]?.cas_gen_ed).filter(Boolean);
    if (!allGenEd.length) return null;

    const faNames = primary.cas_gen_ed.foundational_approaches?.map((r) => r.name) || [];
    const sectorNames = primary.cas_gen_ed.sectors?.map((r) => r.name) || [];

    const foundational_approaches = faNames.map((name, idx) => {
        const rows = allGenEd
            .map((g) => g.foundational_approaches?.[idx])
            .filter(Boolean);
        return mergeCasGenEdRow(rows, casIndices, results, courseDegreesMap);
    }).filter(Boolean);

    const sectors = sectorNames.map((name, idx) => {
        const rows = allGenEd.map((g) => g.sectors?.[idx]).filter(Boolean);
        return mergeCasGenEdRow(rows, casIndices, results, courseDegreesMap);
    }).filter(Boolean);

    return { foundational_approaches, sectors };
}

export function mergeCasPoolCoverage(casIndices, results, courseDegreesMap) {
    const primaryIdx = casIndices[0];
    const primary = results[primaryIdx];
    const pool = (primary.pool_coverage_info || []).find(
        (p) => normalizeCategory(p.category) === CAS_GENED_HEADING,
    );
    if (!pool) return null;

    const constraints = (pool.constraints || []).map((constraint, j) => {
        let fulfilled = false;
        const matched = new Set();
        for (const idx of casIndices) {
            const p = (results[idx].pool_coverage_info || []).find(
                (x) => normalizeCategory(x.category) === CAS_GENED_HEADING,
            );
            const c = p?.constraints?.[j];
            if (c?.fulfilled) fulfilled = true;
            (c?.matched_courses || []).forEach((courseId) => {
                if (courseCountsForDegree(
                    courseId,
                    degreeLabelFor(results, idx),
                    courseDegreesMap,
                )) {
                    matched.add(courseId);
                }
            });
        }
        return {
            ...constraint,
            fulfilled: fulfilled || matched.size > 0,
            matched_courses: [...matched],
        };
    });

    return { ...pool, constraints };
}

/**
 * Build super-sections for one or more CAS degrees.
 * @returns {{ id: string, title: string, kind: string, degreeIndex?: number, writingItems?: [], genEd?: {}, majorCategories?: [] }[]}
 */
export function buildCasSuperSections({
    casIndices,
    results,
    degrees,
    degreeCatalog,
    courseDegreesMap,
    combined,
}) {
    const sections = [];

    const allItems = casIndices.flatMap((idx) => {
        const result = results[idx];
        const label = degreeLabelFor(results, idx);
        if (combined) {
            return collectMappedItems(result, (mapped, opts) => ({
                ...mapRequirementForCasCombined(
                    mapped,
                    casIndices,
                    results,
                    courseDegreesMap,
                    opts,
                ),
                degreeIndex: idx,
            }));
        }
        return collectDegreeRequirements(result, idx, label, courseDegreesMap);
    });

    const writingItems = allItems.filter((i) => casRequirementSection(i) === "writing");
    if (writingItems.length) {
        const deduped = dedupeByInstanceId(writingItems);
        sections.push({
            id: "cas-writing",
            kind: "writing",
            title: CAS_WRITING_HEADING,
            items: deduped,
            degreeIndex: combined ? casIndices[0] : casIndices[0],
        });
    }

    const casGenEd = combined
        ? mergeCasGenEdInfo(casIndices, results, courseDegreesMap)
        : results[casIndices[0]]?.cas_gen_ed;
    const pool = combined
        ? mergeCasPoolCoverage(casIndices, results, courseDegreesMap)
        : (results[casIndices[0]].pool_coverage_info || []).find(
            (p) => normalizeCategory(p.category) === CAS_GENED_HEADING,
        );

    sections.push({
        id: "cas-gened",
        kind: "genEd",
        title: CAS_GENED_HEADING,
        casGenEd,
        pool,
        degreeIndex: casIndices[0],
    });

    for (const idx of casIndices) {
        const result = results[idx];
        const { major: majorName } = formatDegreeDisplay(
            degrees[idx],
            result,
            degreeCatalog,
        );
        const label = degreeLabelFor(results, idx);
        const items = combined
            ? allItems.filter((i) => i.degreeIndex === idx && casRequirementSection(i) === "major")
            : collectDegreeRequirements(result, idx, label, courseDegreesMap).filter(
                (i) => casRequirementSection(i) === "major",
            );

        const pools = result.pool_coverage_info || [];
        const categoryMap = groupItemsByCategory(items, pools);
        const orderedCategories = orderedCategoriesForResult(result, categoryMap).filter(
            (cat) =>
                cat !== CAS_WRITING_HEADING
                && cat !== CAS_GENED_HEADING
                && (categoryMap[cat]?.length ?? 0) > 0,
        );

        sections.push({
            id: `cas-major-${idx}`,
            kind: "major",
            title: `Major — ${majorName}`,
            degreeIndex: idx,
            categoryMap,
            orderedCategories,
            pools,
        });
    }

    return sections;
}

function dedupeByInstanceId(items) {
    const seen = new Set();
    return items.filter((item) => {
        const key = item.instanceId || item.category;
        if (seen.has(key)) return false;
        seen.add(key);
        return true;
    });
}

export function casGenEdProgress(casGenEd) {
    const rows = [
        ...(casGenEd.foundational_approaches || []),
        ...(casGenEd.sectors || []),
    ];
    return {
        done: rows.filter((r) => r.fulfilled).length,
        total: rows.length,
    };
}

export function poolGroupStats(pool) {
    const slotsFilled = (pool.fixed_slots_filled || 0) + (pool.flexible_slots_filled || 0);
    const slotsTotal = (pool.fixed_slots_total || 0) + (pool.flexible_slots_total || 0);
    const covDone = (pool.constraints || []).filter((c) => c.fulfilled).length;
    const covTotal = (pool.constraints || []).length;
    return { slotsFilled, slotsTotal, covDone, covTotal };
}

/** Pool is complete when every slot has a real course and every coverage constraint is met. */
export function isPoolComplete(poolStats) {
    if (!poolStats) return false;
    return poolStats.slotsFilled >= poolStats.slotsTotal
        && poolStats.covDone >= poolStats.covTotal;
}

/** Header pill for a course pool — prefer coverage constraint progress when present. */
export function poolProgressLabel(poolStats) {
    if (!poolStats) return null;
    if (poolStats.covTotal > 0) {
        return `${poolStats.covDone}/${poolStats.covTotal}`;
    }
    return `${poolStats.slotsFilled}/${poolStats.slotsTotal}`;
}
