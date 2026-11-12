"use client";

import { useEffect, useRef, useState } from "react";

export default function ScheduleSwitcher({
  schedules,
  activeId,
  onSwitch,
  onCreate,
  onRename,
  onDuplicate,
  onDelete,
}) {
  const [menuOpen, setMenuOpen] = useState(false);
  const menuRef = useRef(null);

  useEffect(() => {
    if (!menuOpen) return undefined;
    const onPointerDown = (event) => {
      if (menuRef.current && !menuRef.current.contains(event.target)) {
        setMenuOpen(false);
      }
    };
    const onKeyDown = (event) => {
      if (event.key === "Escape") setMenuOpen(false);
    };
    document.addEventListener("pointerdown", onPointerDown);
    document.addEventListener("keydown", onKeyDown);
    return () => {
      document.removeEventListener("pointerdown", onPointerDown);
      document.removeEventListener("keydown", onKeyDown);
    };
  }, [menuOpen]);

  const closeAnd = (action) => {
    setMenuOpen(false);
    action();
  };

  return (
    <div className="schedule-switcher">
      <select
        id="schedule-switcher"
        className="schedule-switcher-select"
        value={activeId || ""}
        aria-label="Choose a schedule"
        disabled={!activeId}
        onChange={(event) => onSwitch(event.target.value)}
      >
        {schedules.map((plan) => (
          <option key={plan.id} value={plan.id}>{plan.name}</option>
        ))}
      </select>
      <button type="button" className="btn btn-ghost btn-sm" onClick={onCreate} disabled={!activeId}>
        New
      </button>
      <div className="export-menu schedule-switcher-menu" ref={menuRef}>
        <button
          type="button"
          className="btn btn-ghost btn-sm schedule-switcher-more"
          aria-expanded={menuOpen}
          aria-haspopup="menu"
          aria-label="Schedule actions"
          disabled={!activeId}
          onClick={() => setMenuOpen((open) => !open)}
        >
          ···
        </button>
        {menuOpen && (
          <div className="export-menu-dropdown" role="menu">
            <button type="button" role="menuitem" className="export-menu-item" onClick={() => closeAnd(onRename)}>
              Rename
            </button>
            <button type="button" role="menuitem" className="export-menu-item" onClick={() => closeAnd(onDuplicate)}>
              Duplicate
            </button>
            <button type="button" role="menuitem" className="export-menu-item" onClick={() => closeAnd(onDelete)}>
              Delete
            </button>
          </div>
        )}
      </div>
    </div>
  );
}
