"use client";

import { useState, useEffect, useCallback, useRef, useMemo } from "react";
import appIcon from "./logo.png";
import { DndContext, DragOverlay, PointerSensor, TouchSensor, useSensor, useSensors } from "@dnd-kit/core";
import DegreeSelector from "./components/DegreeSelector";
import CourseSearch from "./components/CourseSearch";
import ScheduleGrid from "./components/ScheduleGrid";
import RequirementsPanel from "./components/RequirementsPanel";
import { API_BASE } from "@/lib/api";
import { perfLog } from "@/lib/perfLog";
import { prepareCourseCatalog, cuMapFromCatalog } from "@/lib/courseCatalog";
import { buildCourseRelations } from "@/lib/courseRelations";
import { maxYearFromSchedule, buildSemesterCuLimitsMap, degreeCuPolicyKey, undergradScheduleYears, gapSemesterKeys } from "@/lib/semesterOptions";
import {
  isValidCourseCode,
  isRequirementSlotId,
  isOverlapScheduleGroupId,
  isSchedulableRequirementSlotId,
  isSchedulePlacementId,
  filterValidCourseCodes,
  filterValidPlacements,
  filterFrozenPlacements,
} from "@/lib/courseUtils";
import { getSlotLabel, getRequirementInstanceId } from "@/lib/requirementText";
import { reqRowDomId, parseOverlapGroupSlots, overlapSlotsEqual, resolveOverlapSlotNav, poolConstraintInstanceId, requirementSlotScope } from "@/lib/requirementNav";
import {
  buildCourseDegreesMapFromAllocations,
  courseViolationMap,
  filterConcentrationInfoForDegree,
} from "@/lib/crossDegree";
import { exportScheduleJpeg } from "@/lib/exportScheduleImage";
import { exportScheduleExcel } from "@/lib/exportScheduleExcel";
import { getOrCreateAnonSessionId } from "@/lib/anonSession";
import {
  clearCachedSchedule,
  clearPlanPersistence,
  loadCachedCatalogs,
  loadCachedSchedule,
  loadSavedState,
  saveCachedCatalogs,
  saveCachedSchedule,
  savePlanState,
  scheduleInputKey,
} from "@/lib/planPersistence";
import { applyCatalogNamesToDegrees } from "@/lib/degreeDisplay";

export default function Home() {
    const [courseCuMap, setCourseCuMap] = useState({});
  const [courseCatalog, setCourseCatalog] = useState(null);
  const [courseRelations, setCourseRelations] = useState(null);
  const [degreeCatalog, setDegreeCatalog] = useState([]);
  const [minorCatalog, setMinorCatalog] = useState([]);
  const [concentrationCatalog, setConcentrationCatalog] = useState({});
  const [degrees, setDegrees] = useState([]);
  const [takenCourses, setTakenCourses] = useState([]);
  const [frozenCourses, setFrozenCourses] = useState([]);
  const [assignedCourses, setAssignedCourses] = useState([]);
  const [scheduleData, setScheduleData] = useState(null);
    const [loading, setLoading] = useState(false);
  const [activeDragId, setActiveDragId] = useState(null);
  const [reqNavTarget, setReqNavTarget] = useState(null);
  const [requirementsOpen, setRequirementsOpen] = useState(true);
  const [allowSummer, setAllowSummer] = useState(false);
  const [semesterCuLimits, setSemesterCuLimits] = useState({});
  const [gapSemesters, setGapSemesters] = useState({});
  const [exportOpen, setExportOpen] = useState(false);
  const [exporting, setExporting] = useState(false);
  const [planReady, setPlanReady] = useState(false);
  const debounceRef = useRef(null);
  const scheduleRequestId = useRef(0);
  const exportMenuRef = useRef(null);
  const firstGenerateRef = useRef(true);
  const prevCuPolicyKey = useRef(null);

  // Pointer: 8px movement before drag. Touch: short delay so page scroll wins.
  const sensors = useSensors(
    useSensor(PointerSensor, {
      activationConstraint: { distance: 8 },
    }),
    useSensor(TouchSensor, {
      activationConstraint: { delay: 200, tolerance: 8 },
    })
  );

  // Load data on mount
  useEffect(() => {
    const bootstrapStart = typeof performance !== "undefined" ? performance.now() : Date.now();
    const elapsed = () => (
        typeof performance !== "undefined" ? performance.now() : Date.now()
    ) - bootstrapStart;

    const saved = loadSavedState();
    const cachedCatalogs = loadCachedCatalogs();
    const cachedDegreeCatalog = cachedCatalogs?.degreeCatalog || [];
    const cachedMinorCatalog = cachedCatalogs?.minorCatalog || [];
    if (cachedDegreeCatalog.length) setDegreeCatalog(cachedDegreeCatalog);
    if (cachedMinorCatalog.length) setMinorCatalog(cachedMinorCatalog);
    if (cachedCatalogs?.concentrationCatalog
        && Object.keys(cachedCatalogs.concentrationCatalog).length) {
      setConcentrationCatalog(cachedCatalogs.concentrationCatalog);
    }

    if (saved) {
      const takenCourses = filterValidCourseCodes(saved.takenCourses || []);
      const frozenCourses = filterFrozenPlacements(saved.frozenCourses || []);
      const assignedCourses = filterValidPlacements(saved.assignedCourses || []);
      const allowSummer = saved.allowSummer !== undefined ? saved.allowSummer : false;
      const semesterCuLimits = saved.semesterCuLimits || {};
      const gapSemesters = saved.gapSemesters || {};
      const degrees = applyCatalogNamesToDegrees(
        saved.degrees || [],
        cachedDegreeCatalog,
        cachedMinorCatalog,
      );
      setDegrees(degrees);
      setTakenCourses(takenCourses);
      setFrozenCourses(frozenCourses);
      setAssignedCourses(assignedCourses);
      if (saved.allowSummer !== undefined) setAllowSummer(saved.allowSummer);
      if (saved.semesterCuLimits) setSemesterCuLimits(saved.semesterCuLimits);
      if (saved.gapSemesters) setGapSemesters(saved.gapSemesters);
      const cached = loadCachedSchedule(scheduleInputKey({
        degrees,
        takenCourses,
        frozenCourses,
        assignedCourses,
        allowSummer,
        semesterCuLimits,
        gapSemesters,
      }));
      if (cached) setScheduleData(cached);
    }
    setPlanReady(true);
    perfLog("bootstrap.localStorage", elapsed());

    const trackFetch = (step, url, onData) => {
      const start = typeof performance !== "undefined" ? performance.now() : Date.now();
      const sinceStart = () => (
        typeof performance !== "undefined" ? performance.now() : Date.now()
      ) - start;
      return fetch(url)
        .then((r) => {
          perfLog(`${step}.headers`, sinceStart(), { ok: r.ok, status: r.status });
          const parseStart = typeof performance !== "undefined" ? performance.now() : Date.now();
          return r.json().then((data) => {
            perfLog(`${step}.json`, (
              typeof performance !== "undefined" ? performance.now() : Date.now()
            ) - parseStart);
            onData(data);
            perfLog(`${step}.total`, sinceStart(), { url });
          });
        })
        .catch((err) => {
          perfLog(`${step}.error`, sinceStart(), { url, message: err?.message });
          onData(null);
        });
    };

    trackFetch("bootstrap.course_index", "/course_index.json", (data) => {
      const apply = () => {
        const rows = Array.isArray(data) ? data : [];
        const catalog = prepareCourseCatalog(rows);
        setCourseCatalog(catalog);
        setCourseCuMap(cuMapFromCatalog(catalog));
        setCourseRelations(buildCourseRelations(rows));
        perfLog("bootstrap.course_index.rows", elapsed(), { count: catalog.length });
      };
      if (typeof requestIdleCallback === "function") {
        requestIdleCallback(apply, { timeout: 1500 });
      } else {
        setTimeout(apply, 0);
      }
    });

    trackFetch("bootstrap.degree_catalog", `${API_BASE}/degree_catalog`, (data) => {
      const list = Array.isArray(data) ? data : [];
      setDegreeCatalog(list);
      if (list.length) saveCachedCatalogs({ degreeCatalog: list });
    });

    trackFetch("bootstrap.minor_catalog", `${API_BASE}/minor_catalog`, (data) => {
      const list = Array.isArray(data) ? data : [];
      setMinorCatalog(list);
      if (list.length) saveCachedCatalogs({ minorCatalog: list });
    });

    trackFetch("bootstrap.all_concentrations", `${API_BASE}/all_concentrations`, (data) => {
      const catalog = data && typeof data === "object" ? data : {};
      setConcentrationCatalog(catalog);
      if (Object.keys(catalog).length) {
        saveCachedCatalogs({ concentrationCatalog: catalog });
      }
      perfLog("bootstrap.all_concentrations.keys", elapsed(), {
        count: Object.keys(catalog).length,
      });
    });
  }, []);

  const maxScheduleYear = useMemo(
    () => Math.max(
      maxYearFromSchedule(scheduleData?.schedule),
      undergradScheduleYears(degrees),
    ),
    [scheduleData?.schedule, degrees],
  );

  const cuPolicyKey = useMemo(() => degreeCuPolicyKey(degrees), [degrees]);

  // Reset per-semester overrides when school mix changes so defaults track degree composition.
  useEffect(() => {
    if (!planReady) return;
    if (prevCuPolicyKey.current === null) {
      prevCuPolicyKey.current = cuPolicyKey;
      return;
    }
    if (prevCuPolicyKey.current !== cuPolicyKey) {
      prevCuPolicyKey.current = cuPolicyKey;
      setSemesterCuLimits({});
    }
  }, [planReady, cuPolicyKey]);

  // Auto-save on changes
  useEffect(() => {
    if (!planReady) return;
    savePlanState({
      degrees: applyCatalogNamesToDegrees(degrees, degreeCatalog, minorCatalog),
      takenCourses,
      frozenCourses,
      assignedCourses,
      allowSummer,
      semesterCuLimits,
      gapSemesters,
    });
  }, [planReady, degrees, takenCourses, frozenCourses, assignedCourses, allowSummer, semesterCuLimits, gapSemesters, degreeCatalog, minorCatalog]);

  // Generate schedule when inputs change (debounced)
  const generateSchedule = useCallback(async () => {
    if (degrees.length === 0) {
      setScheduleData(null);
      clearCachedSchedule();
      return;
    }

    const inputKey = scheduleInputKey({
      degrees,
      takenCourses,
      frozenCourses,
      assignedCourses,
      allowSummer,
      semesterCuLimits,
      gapSemesters,
    });

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
              kind: d.kind || "major",
              major: d.majorCode,
              school: d.schoolCode,
              concentrations,
              concentration: concentrations[0] || null,
            };
          }),
          frozen: allFrozen,
          allow_summer: allowSummer,
          semester_cu_limits: buildSemesterCuLimitsMap(
            degrees,
            maxScheduleYear,
            allowSummer,
            semesterCuLimits,
          ),
          gap_semesters: gapSemesterKeys(gapSemesters),
          anon_session_id: getOrCreateAnonSessionId(),
        }),
      });
      const data = await response.json();
      if (requestId !== scheduleRequestId.current) return;
      setScheduleData(data);
      if (Array.isArray(data?.schedule)) {
        saveCachedSchedule(inputKey, data);
      }
    } catch (err) {
      if (requestId !== scheduleRequestId.current) return;
      console.error("Schedule generation failed:", err);
    }
    if (requestId === scheduleRequestId.current) {
      setLoading(false);
    }
  }, [degrees, takenCourses, frozenCourses, assignedCourses, allowSummer, semesterCuLimits, gapSemesters, maxScheduleYear]);

  useEffect(() => {
    if (!planReady) return;
    if (debounceRef.current) clearTimeout(debounceRef.current);
    const delay = firstGenerateRef.current ? 0 : 500;
    firstGenerateRef.current = false;
    debounceRef.current = setTimeout(generateSchedule, delay);
    return () => clearTimeout(debounceRef.current);
  }, [planReady, generateSchedule]);

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
            if (isSchedulableRequirementSlotId(id)) openSlotIds.add(id);
          });
        });
      });
      const filtered = filterFrozenPlacements(prev).filter(
        (f) => !isRequirementSlotId(f.courseId)
            || (isSchedulableRequirementSlotId(f.courseId) && openSlotIds.has(f.courseId))
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
    setTakenCourses((prev) => {
      if (prev.includes(courseCode)) return prev;
      return [...prev, courseCode];
    });
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
    // Credits Received counts as taken — keep in My Courses and off the generated schedule
    if (year === 0 && semester != null) {
      setTakenCourses((prev) => (prev.includes(courseId) ? prev : [...prev, courseId]));
    }
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
    setTakenCourses((prev) => (prev.includes(courseId) ? prev : [...prev, courseId]));
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
    clearPlanPersistence();
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

    if (dragData.source === "cart" || dragData.source === "search") {
      if (!isValidCourseCode(courseId)) return;
      // Cart/Search → semester: mark taken (green). Credits Received uses year 0.
      setFrozenCourses((prev) => prev.filter((f) => f.courseId !== courseId));
      if (targetYear !== 0) {
        setTakenCourses((prev) => (prev.includes(courseId) ? prev : [...prev, courseId]));
      }
      assignCourse(courseId, targetYear, targetSemester);
    } else if (dragData.source === "schedule") {
      if (!isSchedulePlacementId(courseId)) return;
      // Schedule → Credits Received: mark taken (off future semesters)
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
    scheduleData?.overlap_schedule_groups?.forEach((group) => {
      group.members?.forEach((member) => {
        const slotId = member.schedule_slot_id;
        if (!slotId || labels[slotId]) return;
        const result = scheduleData.degree_results?.[member.degree_index];
        const hintMatch = slotId.match(/^(\d+):(.+)$/);
        const mapped = result?.suggested_for_unfulfilled?.find(
          (m) =>
            m.course_ids?.includes(slotId)
            || (m.instance_id && slotId.startsWith(`req:${m.instance_id}:`))
            || (hintMatch && m.instance_id === hintMatch[2]),
        );
        if (mapped?.requirement) {
          const schedulableId =
            mapped.course_ids?.find((id) => isSchedulableRequirementSlotId(id))
            ?? mapped.course_ids?.find((id) => isRequirementSlotId(id))
            ?? slotId;
          labels[slotId] = getSlotLabel(mapped.requirement, schedulableId, apiLabels);
        }
      });
    });
    return labels;
  }, [scheduleData]);

  const crossDegreeSummary = scheduleData?.cross_degree_summary ?? null;

  // ─── Build course → degree map (authoritative allocations from backend) ───
  const courseDegreesMap = useMemo(
    () =>
      buildCourseDegreesMapFromAllocations(
        crossDegreeSummary,
        scheduleData?.degree_results,
        scheduleData,
      ),
    [scheduleData, crossDegreeSummary]
  );

  const crossDegreeViolationsByCourse = useMemo(() => {
    const fromApi = courseViolationMap(crossDegreeSummary);
    if (!courseRelations) return fromApi;

    const scheduleCodes = (scheduleData?.schedule || []).flatMap((p) => p.courses || []);
    const creditCodes = (assignedCourses || [])
      .filter((a) => a.year === 0)
      .map((a) => a.courseId);
    const frozenCodes = (frozenCourses || []).map((f) => f.courseId);
    const gridCodes = [...scheduleCodes, ...creditCodes];
    const planCodes = [...takenCourses, ...frozenCodes, ...gridCodes];
    const fromMutex = courseRelations.mutexViolationsOnGrid(gridCodes);
    const fromAlsoOffered = courseRelations.alsoOfferedDuplicatesInPlan(planCodes);
    return { ...fromAlsoOffered, ...fromMutex, ...fromApi };
  }, [
    crossDegreeSummary,
    courseRelations,
    scheduleData,
    assignedCourses,
    frozenCourses,
    takenCourses,
  ]);

  const courseRequirementLinks = useMemo(() => {
    const links = {};
    if (!scheduleData?.degree_results) return links;

    const onSchedule = new Set(
      (scheduleData.schedule || []).flatMap((p) => p.courses || []),
    );

    const addLink = (mapped, courseId, degreeIndex) => {
      const result = scheduleData.degree_results[degreeIndex];
      if (!result) return;
      const degreeLabel = `${result.school}-${result.major}`;
      if (
        !isValidCourseCode(courseId)
        && !isRequirementSlotId(courseId)
        && !isOverlapScheduleGroupId(courseId)
      ) {
        return;
      }
      // Trust courses listed on this degree's own mapped requirement. Gating on
      // courseDegreesMap caused missing arrows when allocations / dual-CAS filtering
      // diverged from navigable rows (e.g. FNCE 1010 → Unrestricted Electives).
      const category = requirementCategoryForNav(mapped.requirement);
      const instanceId = getRequirementInstanceId(mapped);
      const entry = {
        degreeIndex,
        instanceId,
        category,
        label: `${degreeLabel}: ${category}`,
        href: `#${reqRowDomId(degreeIndex, instanceId)}`,
      };
      if (!links[courseId]) links[courseId] = [];
      const key = `${degreeIndex}::${instanceId}`;
      if (!links[courseId].some((l) => `${l.degreeIndex}::${l.instanceId}` === key)) {
        links[courseId].push(entry);
      }
    };

    const addSlotNavLink = (targetId, degreeIndex, nav) => {
      if (!nav) return;
      const result = scheduleData.degree_results[degreeIndex];
      if (!result) return;
      const degreeLabel = `${result.school}-${result.major}`;
      if (
        !isValidCourseCode(targetId)
        && !isRequirementSlotId(targetId)
        && !isOverlapScheduleGroupId(targetId)
      ) {
        return;
      }
      const category = normalizeCategory(nav.category);
      const rowLabel = nav.rowLabel || category;
      const entry = {
        degreeIndex,
        instanceId: nav.instanceId,
        category,
        label: `${degreeLabel}: ${rowLabel}`,
        href: `#${reqRowDomId(degreeIndex, nav.instanceId)}`,
      };
      if (!links[targetId]) links[targetId] = [];
      const key = `${degreeIndex}::${nav.instanceId}`;
      if (!links[targetId].some((l) => `${l.degreeIndex}::${l.instanceId}` === key)) {
        links[targetId].push(entry);
      }
    };

    scheduleData.degree_results.forEach((result, degreeIndex) => {
      result.fulfilled_requirements?.forEach((mapped) => {
        mapped.course_ids?.forEach((c) => addLink(mapped, c, degreeIndex));
      });
      result.unfulfilled_requirements?.forEach((mapped) => {
        if (mapped.partial && mapped.course_ids?.length) {
          mapped.course_ids.forEach((c) => addLink(mapped, c, degreeIndex));
        }
      });
      result.suggested_for_unfulfilled?.forEach((mapped) => {
        mapped.course_ids?.forEach((c) => addLink(mapped, c, degreeIndex));
      });

      (result.pool_coverage_info || []).forEach((pool) => {
        (pool.constraints || []).forEach((constraint, ci) => {
          const nav = {
            instanceId: poolConstraintInstanceId(pool.pool_index, ci),
            category: pool.category,
            rowLabel: constraint.label || constraint.description,
          };
          (constraint.matched_courses || []).forEach((courseId) => {
            addSlotNavLink(courseId, degreeIndex, nav);
          });
        });
      });
    });

    // Overlap schedule blocks (dashed dual-requirement cards).
    scheduleData.overlap_schedule_groups?.forEach((group) => {
      const parsed = parseOverlapGroupSlots(group.group_id);
      if (parsed.length) {
        parsed.forEach(({ degreeIndex, slotKey }) => {
          const result = scheduleData.degree_results[degreeIndex];
          const nav = resolveOverlapSlotNav(result, slotKey, null);
          if (nav) addSlotNavLink(group.group_id, degreeIndex, nav);
        });
        return;
      }
      group.members?.forEach((member) => {
        const result = scheduleData.degree_results[member.degree_index];
        if (!result) return;
        const slotKey = requirementSlotScope(member.schedule_slot_id);
        const nav = resolveOverlapSlotNav(
          result,
          slotKey,
          member.schedule_slot_id,
        );
        if (nav) addSlotNavLink(group.group_id, member.degree_index, nav);
      });
    });

    // Shared named courses scheduled once for an overlap pair (dual degree stripes).
    const plan = scheduleData.overlap_plan;
    if (plan?.pairs?.length) {
      plan.pairs.forEach((pair) => {
        const opp = plan.opportunities?.find((o) => overlapSlotsEqual(o.slots, pair.slots));
        if (!opp?.suggested_courses?.length) return;

        const sharedCourses = opp.suggested_courses.filter((id) => {
          if (!isValidCourseCode(id)) return false;
          if (onSchedule.has(id)) return true;
          return (courseDegreesMap[id]?.length ?? 0) >= 2;
        });
        if (!sharedCourses.length) return;

        pair.slots.forEach((slotRef) => {
          const result = scheduleData.degree_results[slotRef.degree_index];
          const nav = resolveOverlapSlotNav(result, slotRef.slot_key, null);
          if (!nav) return;
          sharedCourses.forEach((courseId) => {
            addSlotNavLink(courseId, slotRef.degree_index, nav);
          });
        });
      });
    }

    return links;
  }, [scheduleData, courseDegreesMap]);

  // ─── Build concentration tracker data ───
  const { concentrationData, courseConcentrationMap } = useMemo(() => {
    const concList = [];
    const concCourseMap = {};
    if (scheduleData?.degree_results) {
      scheduleData.degree_results.forEach((result) => {
        const degreeLabel = `${result.school}-${result.major}`;
        if (result.concentration_info) {
          result.concentration_info.forEach((ci) => {
            if (ci.is_core) return;
            const filtered = filterConcentrationInfoForDegree(ci, degreeLabel, courseDegreesMap);
            concList.push({ ...filtered, degreeLabel });

            (filtered.matched_courses || []).flat().forEach((courseId) => {
              if (!concCourseMap[courseId]) concCourseMap[courseId] = [];
              if (!concCourseMap[courseId].some(e => e.name === ci.name && e.degreeLabel === degreeLabel)) {
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
  }, [scheduleData, courseDegreesMap]);

  const canExport = Boolean(scheduleData?.schedule?.length);

  useEffect(() => {
    if (!exportOpen) return undefined;
    const onPointerDown = (e) => {
      if (exportMenuRef.current && !exportMenuRef.current.contains(e.target)) {
        setExportOpen(false);
      }
    };
    const onKeyDown = (e) => {
      if (e.key === "Escape") setExportOpen(false);
    };
    document.addEventListener("pointerdown", onPointerDown);
    document.addEventListener("keydown", onKeyDown);
    return () => {
      document.removeEventListener("pointerdown", onPointerDown);
      document.removeEventListener("keydown", onKeyDown);
    };
  }, [exportOpen]);

  const handleExportJpeg = async () => {
    if (!canExport || exporting) return;
    setExportOpen(false);
    setExporting(true);
    try {
      await exportScheduleJpeg({
        scheduleData,
        frozenCourses,
        assignedCourses,
        allowSummer,
        degrees,
        semesterCuLimits,
        gapSemesters,
        courseCuMap,
        requirementSlotLabels,
        degreeCatalog,
        minorCatalog,
        concentrationData,
      });
    } catch (err) {
      console.error("JPEG export failed", err);
      window.alert("Could not export schedule image. Try again.");
    } finally {
      setExporting(false);
    }
  };

  const handleExportExcel = async () => {
    if (!canExport || exporting) return;
    setExportOpen(false);
    setExporting(true);
    try {
      await exportScheduleExcel({
        scheduleData,
        frozenCourses,
        assignedCourses,
        allowSummer,
        degrees,
        semesterCuLimits,
        gapSemesters,
        courseCuMap,
        requirementSlotLabels,
        degreeCatalog,
        minorCatalog,
        courseDegreesMap,
        concentrationData,
      });
    } catch (err) {
      console.error("Excel export failed", err);
      window.alert("Could not export Excel workbook. Try again.");
    } finally {
      setExporting(false);
    }
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
          <h1>
            <img src={appIcon.src} alt="" className="header-icon" width={28} height={28} />
            <span>Penn Degree Planner</span>
          </h1>
          <div className="header-actions">
            {loading && <div className="loading-spinner" />}
            <a
              href="https://forms.gle/tFzvnx3iNFVWk8PX8"
              target="_blank"
              rel="noopener noreferrer"
              className="btn btn-ghost btn-sm"
              style={{ textDecoration: "none" }}
            >
              <span className="header-feedback-label-full">📝 Feedback / Bug Report</span>
              <span className="header-feedback-label-short">📝 Feedback</span>
            </a>
            <button className="btn btn-ghost btn-sm" onClick={clearAll}>
              Clear All
            </button>
          </div>
        </header>

        <DegreeSelector
          degreeCatalog={degreeCatalog}
          minorCatalog={minorCatalog}
          concentrationCatalog={concentrationCatalog}
          degrees={degrees}
          setDegrees={setDegrees}
        />

        <div
          className={`main-layout ${requirementsOpen ? "" : "requirements-collapsed"}`}
        >
          <div className="panel panel-courses">
            <div className="panel-header">
              <h2>📚 Courses</h2>
            </div>
            <div className="panel-body">
              <CourseSearch
                courseCatalog={courseCatalog}
                takenCourses={takenCourses}
                assignedCourses={assignedCourses}
                frozenCourses={frozenCourses}
                onAdd={addCourse}
                onRemove={removeCourse}
                onAssign={assignCourse}
                maxScheduleYear={maxScheduleYear}
                allowSummer={allowSummer}
              />
            </div>
          </div>

          <div className="panel panel-schedule">
            <div className="panel-header">
              <h2>📅 Schedule</h2>
              <div className="panel-header-actions">
                <label className="summer-toggle">
                  <input
                    type="checkbox"
                    checked={allowSummer}
                    onChange={e => setAllowSummer(e.target.checked)}
                  />
                  ☀️ Summer courses
                </label>
                {degrees.length > 0 && (
                  <span className="panel-toolbar-meta">
                    {assignedCourses.length} taken · {frozenCourses.length} frozen
                  </span>
                )}
                <div className="export-menu" ref={exportMenuRef}>
                  <button
                    type="button"
                    className="btn btn-ghost btn-sm"
                    disabled={!canExport || exporting}
                    aria-expanded={exportOpen}
                    aria-haspopup="menu"
                    onClick={() => setExportOpen((open) => !open)}
                  >
                    {exporting ? "Exporting…" : "Export ▾"}
                  </button>
                  {exportOpen && (
                    <div className="export-menu-dropdown" role="menu">
                      <button
                        type="button"
                        role="menuitem"
                        className="export-menu-item"
                        onClick={handleExportJpeg}
                      >
                        Export as JPEG
                      </button>
                      <button
                        type="button"
                        role="menuitem"
                        className="export-menu-item"
                        onClick={handleExportExcel}
                      >
                        Export as Excel
                      </button>
                    </div>
                  )}
                </div>
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
                courseRequirementLinks={courseRequirementLinks}
                crossDegreeViolationsByCourse={crossDegreeViolationsByCourse}
                onNavigateToRequirement={(target) => {
                  setRequirementsOpen(true);
                  setReqNavTarget(target);
                }}
                allowSummer={allowSummer}
                concentrationData={concentrationData}
                courseConcentrationMap={courseConcentrationMap}
                courseCuMap={courseCuMap}
                degreeCatalog={degreeCatalog}
                minorCatalog={minorCatalog}
                semesterCuLimits={semesterCuLimits}
                onSemesterCuLimitChange={(key, value) => {
                  setSemesterCuLimits(prev => ({ ...prev, [key]: value }));
                }}
                gapSemesters={gapSemesters}
                onToggleGapSemester={(key) => {
                  setGapSemesters((prev) => {
                    const next = { ...prev };
                    if (next[key]) {
                      delete next[key];
                    } else {
                      next[key] = true;
                    }
                    return next;
                  });
                }}
              />
            </div>
          </div>

          {requirementsOpen ? (
            <div className="panel panel-requirements">
              <div className="panel-header">
                <h2>✅ Requirements</h2>
                <div className="panel-header-actions">
                  <button
                    type="button"
                    className="btn-icon requirements-collapse-btn"
                    onClick={() => setRequirementsOpen(false)}
                    title="Collapse requirements"
                    aria-label="Collapse requirements panel"
                  >
                    ›
                  </button>
                </div>
              </div>
              <div className="panel-body panel-body-requirements">
                <RequirementsPanel
                  scheduleData={scheduleData}
                  degrees={degrees}
                  degreeCatalog={degreeCatalog}
                  minorCatalog={minorCatalog}
                  frozenCourses={frozenCourses}
                  assignedCourses={assignedCourses}
                  takenCourses={takenCourses}
                  courseDegreesMap={courseDegreesMap}
                  crossDegreeViolationsByCourse={crossDegreeViolationsByCourse}
                  navTarget={reqNavTarget}
                  onNavTargetConsumed={() => setReqNavTarget(null)}
                />
              </div>
            </div>
          ) : (
            <button
              type="button"
              className="requirements-collapsed-tab"
              onClick={() => setRequirementsOpen(true)}
              title="Show requirements"
              aria-label="Show requirements panel"
            >
              <span className="requirements-collapsed-tab-icon">✅</span>
              <span className="requirements-collapsed-tab-label">Requirements</span>
            </button>
          )}
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

function normalizeCategory(cat) {
  if (!cat || typeof cat !== "string" || !cat.trim()) return "Other";
  return cat.trim();
}

/** Category label aligned with RequirementsPanel grouping. */
function requirementCategoryForNav(req) {
  if (!req) return "Other";
  if (req.category) return normalizeCategory(req.category);
  const variants = ["SingleCourse", "CourseGroup", "AnyOf", "AllOf", "Concentration", "Restriction", "CoursePool"];
  for (const v of variants) {
    if (req[v]?.category) return normalizeCategory(req[v].category);
  }
  return "Other";
}
