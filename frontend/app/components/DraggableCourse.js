"use client";

import { useDraggable } from "@dnd-kit/core";
import { CSS } from "@dnd-kit/utilities";

export default function DraggableCourse({ id, children, data, disabled = false }) {
    const { attributes, listeners, setNodeRef, transform, isDragging } = useDraggable({
        id,
        data,
        disabled,
    });

    const style = {
        transform: CSS.Translate.toString(transform),
        opacity: isDragging ? 0.4 : 1,
        cursor: disabled ? "default" : "grab",
        // Allow vertical page scroll; TouchSensor delay activates drag without blocking pan.
        touchAction: "pan-y",
    };

    return (
        <div
            ref={setNodeRef}
            style={style}
            {...(disabled ? {} : listeners)}
            {...attributes}
        >
            {children}
        </div>
    );
}
