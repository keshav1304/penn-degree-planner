/** Lookup key for overlap hints: `{degreeIndex}:{slotKey}`. */
export function overlapHintKey(degreeIndex, slotKey) {
    return `${degreeIndex}:${slotKey}`;
}

export function overlapCoursesForSlot(overlapPlan, degreeIndex, slotKey) {
    if (!overlapPlan?.hints_by_slot || slotKey == null) return [];
    return overlapPlan.hints_by_slot[overlapHintKey(degreeIndex, slotKey)] || [];
}

export function overlapExplanationForSlot(overlapPlan, degreeIndex, slotKey) {
    if (!overlapPlan?.slot_explanations || slotKey == null) return null;
    return overlapPlan.slot_explanations[overlapHintKey(degreeIndex, slotKey)] || null;
}

export function overlapPeersForSlot(overlapPlan, degreeIndex, slotKey) {
    if (!overlapPlan?.opportunities?.length || slotKey == null) return [];
    const peers = [];
    for (const opp of overlapPlan.opportunities) {
        const mine = opp.slots?.some(
            (s) => s.degree_index === degreeIndex && s.slot_key === slotKey,
        );
        if (!mine) continue;
        for (const s of opp.slots) {
            if (s.degree_index === degreeIndex && s.slot_key === slotKey) continue;
            peers.push(s);
        }
    }
    return peers;
}

export function overlapAssignmentForSlot(overlapPlan, degreeIndex, slotKey) {
    if (!overlapPlan?.assignments?.length || slotKey == null) return null;
    for (const a of overlapPlan.assignments) {
        const matches = a.slots?.some(
            (s) => s.degree_index === degreeIndex && s.slot_key === slotKey,
        );
        if (matches) return a;
    }
    return null;
}

export function overlapHintTooltip(overlapPlan, degreeIndex, slotKey) {
    const courses = overlapCoursesForSlot(overlapPlan, degreeIndex, slotKey);
    if (!courses.length) return null;

    const lines = [];
    const explanation = overlapExplanationForSlot(overlapPlan, degreeIndex, slotKey);
    const assignment = overlapAssignmentForSlot(overlapPlan, degreeIndex, slotKey);

    if (assignment) {
        lines.push("Planned overlap:");
        lines.push(assignment.explanation);
        lines.push("");
    } else if (explanation) {
        lines.push(explanation);
        lines.push("");
    } else {
        const peers = overlapPeersForSlot(overlapPlan, degreeIndex, slotKey);
        if (peers.length) {
            lines.push("Can overlap with:");
            peers.forEach((p) => {
                lines.push(`• ${p.school}-${p.major}: ${p.label}`);
            });
            lines.push("");
        }
    }

    lines.push("Suggested courses:");
    lines.push(courses.join(", "));
    return lines.join("\n");
}
