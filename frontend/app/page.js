"use client";

import { useState, useEffect, useCallback, useRef } from "react";
import { DndContext, DragOverlay, PointerSensor, useSensor, useSensors } from "@dnd-kit/core";
import DegreeSelector from "./components/DegreeSelector";
import CourseSearch from "./components/CourseSearch";
import ScheduleGrid from "./components/ScheduleGrid";
import RequirementsPanel from "./components/RequirementsPanel";

const API_BASE = "https://degree-planner.fly.dev";

const STORAGE_KEY = "penn_degree_planner_state";

function loadSavedState() {
  if (typeof window === "undefined") return null;
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    return raw ? JSON.parse(raw) : null;
  } catch { return null; }
}

function saveState(state) {
  if (typeof window === "undefined") return;
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify({
      degrees: state.degrees,
      takenCourses: state.takenCourses,
      frozenCourses: state.frozenCourses,
      assignedCourses: state.assignedCourses,
    }));
  } catch { }
}

export default function Home() {
  const [allCourses, setAllCourses] = useState([]);
  const [allMajors, setAllMajors] = useState({});
  const [degrees, setDegrees] = useState([]);
  const [takenCourses, setTakenCourses] = useState([]);
  const [frozenCourses, setFrozenCourses] = useState([]);
  const [assignedCourses, setAssignedCourses] = useState([]);
  const [scheduleData, setScheduleData] = useState(null);
  const [loading, setLoading] = useState(false);
  const [coursesLoading, setCoursesLoading] = useState(true);
  const [activeDragId, setActiveDragId] = useState(null);
  const debounceRef = useRef(null);

  // Require 8px movement before starting drag (so clicks still work)
  const sensors = useSensors(
    useSensor(PointerSensor, {
      activationConstraint: { distance: 8 },
    })
  );

  // Load data on mount
  useEffect(() => {
    const saved = loadSavedState();
    if (saved) {
      setDegrees(saved.degrees || []);
      setTakenCourses(saved.takenCourses || []);
      setFrozenCourses(saved.frozenCourses || []);
      setAssignedCourses(saved.assignedCourses || []);
    }

    fetch(`${API_BASE}/all_courses`)
      .then(r => r.json())
      .then(data => { setAllCourses(data); setCoursesLoading(false); })
      .catch(() => setCoursesLoading(false));

    fetch(`${API_BASE}/all_majors`)
      .then(r => r.json())
      .then(data => setAllMajors(data))
      .catch(() => { });
  }, []);

  // Auto-save on changes
  useEffect(() => {
    saveState({ degrees, takenCourses, frozenCourses, assignedCourses });
  }, [degrees, takenCourses, frozenCourses, assignedCourses]);

  // Generate schedule when inputs change (debounced)
  const generateSchedule = useCallback(async () => {
    if (degrees.length === 0) {
      setScheduleData(null);
      return;
    }

    const allFrozen = [
      ...frozenCourses.map(f => ({
        course_id: f.courseId,
        year: f.year,
        semester: f.semester,
      })),
      ...assignedCourses.filter(a => a.year > 0).map(a => ({
        course_id: a.courseId,
        year: a.year,
        semester: a.semester,
      })),
    ];

    setLoading(true);
    try {
      const response = await fetch(`${API_BASE}/generate_schedule`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          taken: takenCourses,
          degrees: degrees.map(d => ({
            major: d.majorCode,
            school: d.schoolCode,
            concentration: d.concentration || null,
          })),
          frozen: allFrozen,
        }),
      });
      const data = await response.json();
      console.log(data);
      setScheduleData(data);
    } catch (err) {
      console.error("Schedule generation failed:", err);
    }
    setLoading(false);
  }, [degrees, takenCourses, frozenCourses, assignedCourses]);

  useEffect(() => {
    if (debounceRef.current) clearTimeout(debounceRef.current);
    debounceRef.current = setTimeout(generateSchedule, 500);
    return () => clearTimeout(debounceRef.current);
  }, [generateSchedule]);

  const addCourse = (courseCode) => {
    if (!takenCourses.includes(courseCode)) {
      setTakenCourses(prev => [...prev, courseCode]);
    }
  };

  const removeCourse = (courseCode) => {
    setTakenCourses(prev => prev.filter(c => c !== courseCode));
    setFrozenCourses(prev => prev.filter(f => f.courseId !== courseCode));
    setAssignedCourses(prev => prev.filter(a => a.courseId !== courseCode));
  };

  const assignCourse = (courseId, year, semester) => {
    setAssignedCourses(prev => {
      const filtered = prev.filter(a => a.courseId !== courseId);
      if (year === null || semester === null) return filtered;
      return [...filtered, { courseId, year, semester }];
    });
  };

  const toggleFreeze = (courseId, year, semester) => {
    setFrozenCourses(prev => {
      const existing = prev.find(f => f.courseId === courseId);
      if (existing) {
        return prev.filter(f => f.courseId !== courseId);
      }
      return [...prev, { courseId, year, semester }];
    });
  };

  const moveFrozenCourse = (courseId, newYear, newSemester) => {
    setFrozenCourses(prev => {
      const filtered = prev.filter(f => f.courseId !== courseId);
      return [...filtered, { courseId, year: newYear, semester: newSemester }];
    });
  };

  const clearAll = () => {
    setDegrees([]);
    setTakenCourses([]);
    setFrozenCourses([]);
    setAssignedCourses([]);
    setScheduleData(null);
    localStorage.removeItem(STORAGE_KEY);
  };

  // ─── Drag & Drop handlers ───
  const handleDragStart = (event) => {
    setActiveDragId(event.active.data.current?.courseId || null);
  };

  const handleDragEnd = (event) => {
    setActiveDragId(null);
    const { active, over } = event;

    if (!over) return;

    const dragData = active.data.current;
    const dropData = over.data.current;
    const courseId = dragData?.courseId;
    const targetYear = dropData?.year;
    const targetSemester = dropData?.semester;

    if (!courseId || targetYear == null || !targetSemester) return;

    if (dragData.source === "cart") {
      // Cart → Schedule: freeze in the slot (orange — planned but not yet taken)
      // Remove any existing assignment first, then freeze
      setAssignedCourses(prev => prev.filter(a => a.courseId !== courseId));
      setFrozenCourses(prev => {
        const filtered = prev.filter(f => f.courseId !== courseId);
        return [...filtered, { courseId, year: targetYear, semester: targetSemester }];
      });
    } else if (dragData.source === "search") {
      // Search → Schedule: add to cart AND assign in one action
      if (!takenCourses.includes(courseId)) {
        setTakenCourses(prev => [...prev, courseId]);
      }
      assignCourse(courseId, targetYear, targetSemester);
    } else if (dragData.source === "schedule") {
      // Schedule → Schedule: move course to new slot
      const isUserAssigned = assignedCourses.some(a => a.courseId === courseId);
      const isUserFrozen = frozenCourses.some(f => f.courseId === courseId);

      if (isUserAssigned) {
        // Move assigned course to new slot
        assignCourse(courseId, targetYear, targetSemester);
      } else if (isUserFrozen) {
        // Move frozen course to new slot
        moveFrozenCourse(courseId, targetYear, targetSemester);
      } else {
        // It's a suggested course — freeze it in the new slot
        setFrozenCourses(prev => [...prev, { courseId, year: targetYear, semester: targetSemester }]);
      }
    }
  };

  const handleDragCancel = () => {
    setActiveDragId(null);
  };

  return (
    <DndContext
      sensors={sensors}
      onDragStart={handleDragStart}
      onDragEnd={handleDragEnd}
      onDragCancel={handleDragCancel}
    >
      <div className="app-container">
        <header className="header">
          <h1>🎓 Penn Degree Planner</h1>
          <div className="header-actions">
            {loading && <div className="loading-spinner" />}
            <button className="btn btn-ghost btn-sm" onClick={clearAll}>
              Clear All
            </button>
          </div>
        </header>

        <DegreeSelector
          allMajors={allMajors}
          degrees={degrees}
          setDegrees={setDegrees}
        />

        <div className="main-layout">
          <div className="panel">
            <div className="panel-header">
              <h2>📚 Courses</h2>
              {coursesLoading && <div className="loading-spinner" />}
            </div>
            <div className="panel-body">
              <CourseSearch
                allCourses={allCourses}
                takenCourses={takenCourses}
                assignedCourses={assignedCourses}
                onAdd={addCourse}
                onRemove={removeCourse}
                onAssign={assignCourse}
              />
            </div>
          </div>

          <div style={{ display: "flex", flexDirection: "column", gap: 20 }}>
            <div className="panel" style={{ flex: 1 }}>
              <div className="panel-header">
                <h2>📅 4-Year Schedule</h2>
                {degrees.length > 0 && (
                  <span style={{ fontSize: "0.72rem", color: "var(--text-muted)" }}>
                    {assignedCourses.length} placed · {frozenCourses.length} frozen
                  </span>
                )}
              </div>
              <div className="panel-body">
                <ScheduleGrid
                  scheduleData={scheduleData}
                  frozenCourses={frozenCourses}
                  assignedCourses={assignedCourses}
                  onToggleFreeze={toggleFreeze}
                  degrees={degrees}
                />
              </div>
            </div>

            <div className="panel">
              <div className="panel-header">
                <h2>✅ Requirements</h2>
              </div>
              <div className="panel-body">
                <RequirementsPanel
                  scheduleData={scheduleData}
                  degrees={degrees}
                />
              </div>
            </div>
          </div>
        </div>
      </div>

      {/* Drag overlay showing what's being dragged */}
      <DragOverlay>
        {activeDragId ? (
          <div className="drag-overlay-card">
            {activeDragId}
          </div>
        ) : null}
      </DragOverlay>
    </DndContext>
  );
}
