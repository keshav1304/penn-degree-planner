"use client";

import { useState, useEffect, useCallback, useRef, useMemo } from "react";
import { DndContext, DragOverlay, PointerSensor, useSensor, useSensors } from "@dnd-kit/core";
import DegreeSelector from "./components/DegreeSelector";
import CourseSearch from "./components/CourseSearch";
import ScheduleGrid from "./components/ScheduleGrid";
import RequirementsPanel from "./components/RequirementsPanel";
import { API_BASE } from "@/lib/api";
import { maxYearFromSchedule } from "@/lib/semesterOptions";
import {
  isValidCourseCode,
  isRequirementSlotId,
  isSchedulePlacementId,
  filterValidCourseCodes,
  filterValidPlacements,
  filterFrozenPlacements,
} from "@/lib/courseUtils";
import { getSlotLabel } from "@/lib/requirementText";

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
      allowSummer: state.allowSummer,
      semesterCuLimits: state.semesterCuLimits,
    }));
  } catch { }
}

export default function Home() {
  const [allCourses, setAllCourses] = useState([]);
  const [degreeCatalog, setDegreeCatalog] = useState([]);
  const [degrees, setDegrees] = useState([]);
  const [takenCourses, setTakenCourses] = useState([]);
  const [frozenCourses, setFrozenCourses] = useState([]);
  const [assignedCourses, setAssignedCourses] = useState([]);
  const [scheduleData, setScheduleData] = useState(null);
  const [loading, setLoading] = useState(false);
  const [coursesLoading, setCoursesLoading] = useState(true);
  const [activeDragId, setActiveDragId] = useState(null);
  const [allowSummer, setAllowSummer] = useState(true);
  const [semesterCuLimits, setSemesterCuLimits] = useState({});
  const debounceRef = useRef(null);
  const scheduleRequestId = useRef(0);

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
      setTakenCourses(filterValidCourseCodes(saved.takenCourses || []));
      setFrozenCourses(filterFrozenPlacements(saved.frozenCourses || []));
      setAssignedCourses(filterValidPlacements(saved.assignedCourses || []));
      if (saved.allowSummer !== undefined) setAllowSummer(saved.allowSummer);
      if (saved.semesterCuLimits) setSemesterCuLimits(saved.semesterCuLimits);
    }

    fetch(`${API_BASE}/all_courses`)
      .then(r => r.json())
      .then(data => { setAllCourses(data); setCoursesLoading(false); })
      .catch(() => setCoursesLoading(false));

    fetch(`${API_BASE}/degree_catalog`)
      .then((r) => (r.ok ? r.json() : []))
      .then((data) => setDegreeCatalog(Array.isArray(data) ? data : []))
      .catch(() => setDegreeCatalog([]));
  }, []);

  const maxScheduleYear = useMemo(
    () => maxYearFromSchedule(scheduleData?.schedule),
    [scheduleData?.schedule]
  );

  // Auto-save on changes
  useEffect(() => {
    saveState({ degrees, takenCourses, frozenCourses, assignedCourses, allowSummer, semesterCuLimits });
  }, [degrees, takenCourses, frozenCourses, assignedCourses, allowSummer, semesterCuLimits]);

  // Generate schedule when inputs change (debounced)
  const generateSchedule = useCallback(async () => {
    if (degrees.length === 0) {
      setScheduleData(null);
      return;
    }

    const pinnedOnSchedule = [
      ...filterFrozenPlacements(frozenCourses),
      ...filterValidPlacements(assignedCourses.filter((a) => a.year > 0)),
    ];
    const allFrozen = pinnedOnSchedule.map((p) => ({
      course_id: p.courseId,
      year: p.year,
      semester: p.semester,
    }));

    const requestId = ++scheduleRequestId.current;
    setLoading(true);
    try {
      const response = await fetch(`${API_BASE}/generate_schedule`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          taken: filterValidCourseCodes(takenCourses),
          degrees: degrees.map(d => {
            const concentrations = (d.concentrations?.length
              ? d.concentrations
              : d.concentration
                ? [d.concentration]
                : []
            ).filter(Boolean);
            return {
              major: d.majorCode,
              school: d.schoolCode,
              concentrations,
              concentration: concentrations[0] || null,
            };
          }),
          frozen: allFrozen,
          allow_summer: allowSummer,
          semester_cu_limits: Object.keys(semesterCuLimits).length > 0 ? semesterCuLimits : null,
        }),
      });
      const data = await response.json();
      if (requestId !== scheduleRequestId.current) return;
      setScheduleData(data);
    } catch (err) {
      if (requestId !== scheduleRequestId.current) return;
      console.error("Schedule generation failed:", err);
    }
    if (requestId === scheduleRequestId.current) {
      setLoading(false);
    }
  }, [degrees, takenCourses, frozenCourses, assignedCourses, allowSummer, semesterCuLimits]);

  useEffect(() => {
    if (debounceRef.current) clearTimeout(debounceRef.current);
    debounceRef.current = setTimeout(generateSchedule, 500);
    return () => clearTimeout(debounceRef.current);
  }, [generateSchedule]);

  // Drop any legacy invalid entries (e.g. requirement description strings in My Courses)
  useEffect(() => {
    setTakenCourses((prev) => {
      const filtered = filterValidCourseCodes(prev);
      return filtered.length === prev.length ? prev : filtered;
    });
    setFrozenCourses((prev) => {
      const openSlotIds = new Set();
      scheduleData?.degree_results?.forEach((result) => {
        result.suggested_for_unfulfilled?.forEach((mapped) => {
          mapped.course_ids?.forEach((id) => {
            if (isRequirementSlotId(id)) openSlotIds.add(id);
          });
        });
      });
      const filtered = filterFrozenPlacements(prev).filter(
        (f) => !isRequirementSlotId(f.courseId) || openSlotIds.has(f.courseId)
      );
      return filtered.length === prev.length ? prev : filtered;
    });
    setAssignedCourses((prev) => {
      const filtered = filterValidPlacements(prev);
      return filtered.length === prev.length ? prev : filtered;
    });
  }, [scheduleData]);

  const addCourse = (courseCode) => {
    if (!isValidCourseCode(courseCode)) return;
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
    if (!isValidCourseCode(courseId)) return;
    setAssignedCourses(prev => {
      const filtered = prev.filter(a => a.courseId !== courseId);
      if (year === null || semester === null) return filtered;
      return [...filtered, { courseId, year, semester }];
    });
  };

  const toggleFreeze = (courseId, year, semester) => {
    if (!isSchedulePlacementId(courseId)) return;
    setFrozenCourses(prev => {
      const existing = prev.find(f => f.courseId === courseId);
      if (existing) {
        return prev.filter(f => f.courseId !== courseId);
      }
      return [...prev, { courseId, year, semester }];
    });
  };

  // Orange → Green: mark a frozen course as taken (locked in place)
  const markTaken = (courseId, year, semester) => {
    if (!isValidCourseCode(courseId)) return;
    // Add to taken courses if not already there
    if (!takenCourses.includes(courseId)) {
      setTakenCourses(prev => [...prev, courseId]);
    }
    // Remove from frozen
    setFrozenCourses(prev => prev.filter(f => f.courseId !== courseId));
    // Add to assigned
    setAssignedCourses(prev => {
      const filtered = prev.filter(a => a.courseId !== courseId);
      return [...filtered, { courseId, year, semester }];
    });
  };

  // Green → Default: un-mark a taken course (return to auto-suggested)
  const unmarkTaken = (courseId) => {
    setAssignedCourses(prev => prev.filter(a => a.courseId !== courseId));
    setFrozenCourses(prev => prev.filter(f => f.courseId !== courseId));
    setTakenCourses(prev => prev.filter(t => t !== courseId));
  };

  const moveFrozenCourse = (courseId, newYear, newSemester) => {
    if (!isSchedulePlacementId(courseId)) return;
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
      if (!isValidCourseCode(courseId)) return;
      // Cart → Credits Received: assign directly
      if (targetYear === 0) {
        assignCourse(courseId, targetYear, targetSemester);
      } else {
        // Cart → Schedule: freeze in the slot (orange — planned but not yet taken)
        setAssignedCourses(prev => prev.filter(a => a.courseId !== courseId));
        setFrozenCourses(prev => {
          const filtered = prev.filter(f => f.courseId !== courseId);
          return [...filtered, { courseId, year: targetYear, semester: targetSemester }];
        });
      }
    } else if (dragData.source === "search") {
      if (!isValidCourseCode(courseId)) return;
      // Search → Schedule: add to cart AND assign in one action
      if (!takenCourses.includes(courseId)) {
        setTakenCourses(prev => [...prev, courseId]);
      }
      assignCourse(courseId, targetYear, targetSemester);
    } else if (dragData.source === "schedule") {
      if (!isSchedulePlacementId(courseId)) return;
      // Schedule → Credits Received: assign to year 0
      if (targetYear === 0) {
        if (!isValidCourseCode(courseId)) return;
        setFrozenCourses(prev => prev.filter(f => f.courseId !== courseId));
        assignCourse(courseId, targetYear, targetSemester);
      } else {
        // Schedule → Schedule: move course or requirement slot
        const isUserAssigned = assignedCourses.some(a => a.courseId === courseId);
        const isUserFrozen = frozenCourses.some(f => f.courseId === courseId);

        if (isRequirementSlotId(courseId)) {
          moveFrozenCourse(courseId, targetYear, targetSemester);
        } else if (isUserAssigned) {
          assignCourse(courseId, targetYear, targetSemester);
        } else if (isUserFrozen) {
          moveFrozenCourse(courseId, targetYear, targetSemester);
        } else {
          setFrozenCourses(prev => [...prev, { courseId, year: targetYear, semester: targetSemester }]);
        }
      }
    }
  };

  const handleDragCancel = () => {
    setActiveDragId(null);
  };

  const requirementSlotLabels = useMemo(() => {
    const apiLabels = scheduleData?.slot_labels || {};
    const labels = { ...apiLabels };
    scheduleData?.degree_results?.forEach((result) => {
      result.suggested_for_unfulfilled?.forEach((mapped) => {
        mapped.course_ids?.forEach((id) => {
          if (isRequirementSlotId(id) && mapped.requirement) {
            labels[id] = getSlotLabel(mapped.requirement, id, apiLabels);
          }
        });
      });
    });
    return labels;
  }, [scheduleData]);

  // ─── Build course → degree and course → requirement maps ───
  const { courseDegreesMap, courseRequirementMap } = (() => {
    const degMap = {};
    const reqMap = {};
    if (scheduleData?.degree_results) {
      scheduleData.degree_results.forEach((result, i) => {
        const degreeLabel = `${result.school}-${result.major}`;
        const addScheduleItem = (itemId, category) => {
          if (!isValidCourseCode(itemId) && !isRequirementSlotId(itemId)) return;
          if (!degMap[itemId]) degMap[itemId] = [];
          if (!degMap[itemId].includes(degreeLabel)) degMap[itemId].push(degreeLabel);
          if (category) {
            if (!reqMap[itemId]) reqMap[itemId] = [];
            const entry = `${degreeLabel}: ${category}`;
            if (!reqMap[itemId].includes(entry)) reqMap[itemId].push(entry);
          }
        };
        // Fulfilled requirements
        result.fulfilled_requirements?.forEach(req => {
          const cat = req.requirement?.category || getCategoryFromReq(req.requirement);
          req.course_ids?.forEach(c => addScheduleItem(c, cat));
        });
        // Suggested for unfulfilled
        result.suggested_for_unfulfilled?.forEach(req => {
          const cat = getCategoryFromReq(req.requirement);
          req.course_ids?.forEach(c => addScheduleItem(c, cat));
        });
      });
    }
    return { courseDegreesMap: degMap, courseRequirementMap: reqMap };
  })();

  // ─── Build double-count tracker data ───
  const { doubleCountData, courseDoubleCountMap } = (() => {
    const dcList = [];
    const dcCourseMap = {}; // courseId → [{dcIndex, dcLabel, isDoubleCountMatch}]
    if (scheduleData?.degree_results) {
      let globalDcIndex = 0;
      scheduleData.degree_results.forEach((result) => {
        const degreeLabel = `${result.school}-${result.major}`;
        if (result.double_count_info) {
          result.double_count_info.forEach((dc) => {
            const dcIdx = globalDcIndex++;
            const dcLabel = `DC-${dcIdx + 1}`;
            dcList.push({ ...dc, dcLabel, degreeLabel, dcIndex: dcIdx });

            // Map base courses → this DC tracker
            const matchedSet = new Set(dc.dc_matched_courses?.flat() || []);
            (dc.base_courses || []).forEach((courseId) => {
              if (!dcCourseMap[courseId]) dcCourseMap[courseId] = [];
              if (!dcCourseMap[courseId].some(course => course.dcLabel === dcLabel)) {
                dcCourseMap[courseId].push({
                  dcIndex: dcIdx,
                  dcLabel,
                  dcCategory: dc.category,
                  isDoubleCountMatch: matchedSet.has(courseId),
                });
              }
            });
          });
        }
      });
    }
    return { doubleCountData: dcList, courseDoubleCountMap: dcCourseMap };
  })();

  // ─── Build concentration tracker data ───
  const { concentrationData, courseConcentrationMap } = (() => {
    const concList = [];
    const concCourseMap = {};
    if (scheduleData?.degree_results) {
      scheduleData.degree_results.forEach((result) => {
        const degreeLabel = `${result.school}-${result.major}`;
        if (result.concentration_info) {
          result.concentration_info.forEach((ci) => {
            if (ci.is_core) return; // core concentrations are handled via normal requirements
            concList.push({ ...ci, degreeLabel });

            // Map matched courses to this concentration tracker
            (ci.matched_courses || []).flat().forEach((courseId) => {
              if (!concCourseMap[courseId]) concCourseMap[courseId] = [];
              if (!concCourseMap[courseId].some(e => e.name === ci.name)) {
                concCourseMap[courseId].push({
                  name: ci.name,
                  degreeLabel,
                });
              }
            });
          });
        }
      });
    }
    return { concentrationData: concList, courseConcentrationMap: concCourseMap };
  })();

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
            <a
              href="https://forms.gle/tFzvnx3iNFVWk8PX8"
              target="_blank"
              rel="noopener noreferrer"
              className="btn btn-ghost btn-sm"
              style={{ textDecoration: "none" }}
            >
              📝 Feedback / Bug Report
            </a>
            <button className="btn btn-ghost btn-sm" onClick={clearAll}>
              Clear All
            </button>
          </div>
        </header>

        <DegreeSelector
          degreeCatalog={degreeCatalog}
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
                maxScheduleYear={maxScheduleYear}
                allowSummer={allowSummer}
              />
            </div>
          </div>

          <div style={{ display: "flex", flexDirection: "column", gap: 20 }}>
            <div className="panel" style={{ flex: 1 }}>
              <div className="panel-header">
                <h2>📅 Schedule</h2>
                <div style={{ display: "flex", alignItems: "center", gap: 12 }}>
                  <label className="summer-toggle" style={{ display: "flex", alignItems: "center", gap: 6, fontSize: "0.75rem", color: "var(--text-secondary)", cursor: "pointer", userSelect: "none" }}>
                    <input
                      type="checkbox"
                      checked={allowSummer}
                      onChange={e => setAllowSummer(e.target.checked)}
                      style={{ accentColor: "var(--accent-teal)" }}
                    />
                    ☀️ Summer courses
                  </label>
                  {degrees.length > 0 && (
                    <span style={{ fontSize: "0.72rem", color: "var(--text-muted)" }}>
                      {assignedCourses.length} placed · {frozenCourses.length} frozen
                    </span>
                  )}
                </div>
              </div>
              <div className="panel-body">
                <ScheduleGrid
                  scheduleData={scheduleData}
                  requirementSlotLabels={requirementSlotLabels}
                  frozenCourses={frozenCourses}
                  assignedCourses={assignedCourses}
                  onToggleFreeze={toggleFreeze}
                  onMarkTaken={markTaken}
                  onUnmarkTaken={unmarkTaken}
                  degrees={degrees}
                  courseDegreesMap={courseDegreesMap}
                  courseRequirementMap={courseRequirementMap}
                  allowSummer={allowSummer}
                  doubleCountData={doubleCountData}
                  courseDoubleCountMap={courseDoubleCountMap}
                  concentrationData={concentrationData}
                  courseConcentrationMap={courseConcentrationMap}
                  allCourses={allCourses}
                  semesterCuLimits={semesterCuLimits}
                  onSemesterCuLimitChange={(key, value) => {
                    setSemesterCuLimits(prev => ({ ...prev, [key]: value }));
                  }}
                />
              </div>
            </div>

            <div className="panel">
              <div className="panel-header">
                <h2>✅ Requirements</h2>
              </div>
              <div className="panel-body panel-body-requirements">
                <RequirementsPanel
                  scheduleData={scheduleData}
                  degrees={degrees}
                  frozenCourses={frozenCourses}
                  assignedCourses={assignedCourses}
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
            {isRequirementSlotId(activeDragId)
              ? (requirementSlotLabels[activeDragId] || "Open requirement")
              : activeDragId}
          </div>
        ) : null}
      </DragOverlay>
    </DndContext>
  );
}

// Extract category name from serialized Rust Requirement enum
function getCategoryFromReq(req) {
  if (!req) return "Unknown Requirement";
  if (req.category) return req.category;
  const variants = ["SingleCourse", "CourseGroup", "AnyOf", "AllOf", "Concentration", "Restriction", "DoubleCount"];
  for (const v of variants) {
    if (req[v]) return req[v].category || v;
  }
  if (typeof req === "object") {
    for (const key of Object.keys(req)) {
      if (typeof req[key] === "object" && req[key]?.category) return req[key].category;
    }
  }
  return "Requirement";
}

