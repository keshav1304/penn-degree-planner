/** Durable anonymous analytics ID for this browser (not a login). */
const SESSION_KEY = "penn_degree_planner_anon_session";

function randomId() {
  if (typeof crypto !== "undefined" && typeof crypto.randomUUID === "function") {
    return crypto.randomUUID();
  }
  return `anon-${Date.now().toString(36)}-${Math.random().toString(36).slice(2, 12)}`;
}

/** Returns a stable UUID for this browser; creates one on first use. */
export function getOrCreateAnonSessionId() {
  if (typeof window === "undefined") return null;
  try {
    const existing = localStorage.getItem(SESSION_KEY);
    if (existing && existing.length >= 8) return existing;
    const id = randomId();
    localStorage.setItem(SESSION_KEY, id);
    return id;
  } catch {
    return null;
  }
}
