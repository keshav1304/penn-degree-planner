"use client";

import { useDraggable } from "@dnd-kit/core";
import { CSS } from "@dnd-kit/utilities";

export default function DraggableCourse({ id, children, data }) {
    const { attributes, listeners, setNodeRef, transform, isDragging } = useDraggable({
        id,
        data,
    });

    const style = {
        transform: CSS.Translate.toString(transform),
        opacity: isDragging ? 0.4 : 1,
        cursor: "grab",
        touchAction: "none",
    };

    return (
        <div ref={setNodeRef} style={style} {...listeners} {...attributes}>
            {children}
        </div>
    );
}
