/** Lookup key for overlap hints: `{degreeIndex}:{slotKey}`. */
export function overlapHintKey(degreeIndex, slotKey) {
    return `${degreeIndex}:${slotKey}`;
}

export function overlapCoursesForSlot(overlapPlan, degreeIndex, slotKey) {
    if (!overlapPlan?.hints_by_slot || slotKey == null) return [];
    return overlapPlan.hints_by_slot[overlapHintKey(degreeIndex, slotKey)] || [];
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

export function overlapHintTooltip(overlapPlan, degreeIndex, slotKey) {
    const courses = overlapCoursesForSlot(overlapPlan, degreeIndex, slotKey);
    if (!courses.length) return null;

    const peers = overlapPeersForSlot(overlapPlan, degreeIndex, slotKey);
    const lines = [];
    if (peers.length) {
        lines.push("Can overlap with:");
        peers.forEach((p) => {
            lines.push(`• ${p.school}-${p.major}: ${p.label}`);
        });
        lines.push("");
    }
    lines.push("Suggested courses:");
    lines.push(courses.join(", "));
    return lines.join("\n");
}
