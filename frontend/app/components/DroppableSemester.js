"use client";

import { useDroppable } from "@dnd-kit/core";

export default function DroppableSemester({ id, year, semester, style, children }) {
    const { isOver, setNodeRef } = useDroppable({
        id,
        data: { year, semester },
    });

    return (
        <div
            ref={setNodeRef}
            className={`semester-col ${isOver ? "drop-target" : ""}`}
            style={style}
        >
            {children}
        </div>
    );
}
