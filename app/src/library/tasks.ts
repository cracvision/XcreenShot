// Background tasks of the Library (LIB-026): what the status bar shows while they run,
// with their progress and Cancel, and what a screen reader hears when they end.
import { create } from "zustand";

export interface LibraryTask {
  id: string;
  /** i18n key of what it does. */
  label: string;
  done?: number;
  total?: number;
  cancel?: () => void;
}

export const useLibraryTasks = create<{ tasks: LibraryTask[]; said: string }>(() => ({
  tasks: [],
  said: "",
}));

/** Starts or updates a task. */
export function taskProgress(task: LibraryTask): void {
  useLibraryTasks.setState((s) => {
    const others = s.tasks.filter((t) => t.id !== task.id);
    return { tasks: [...others, task] };
  });
}

/** Ends a task; `said` is what the live region reads (the end, or an error). */
export function taskDone(id: string, said?: string): void {
  useLibraryTasks.setState((s) => ({
    tasks: s.tasks.filter((t) => t.id !== id),
    said: said ?? s.said,
  }));
}

/** The status bar is expanded (LIB-026): on by default, remembered. */
const KEY = "xs.library.status";

export function statusExpanded(): boolean {
  try {
    return localStorage.getItem(KEY) !== "collapsed";
  } catch {
    return true;
  }
}

export function keepStatusExpanded(expanded: boolean): void {
  try {
    localStorage.setItem(KEY, expanded ? "expanded" : "collapsed");
  } catch {
    // Not remembered: no harm.
  }
}

/** Bytes of the selected items. */
export function totalSize(items: { id: string; size: number }[], selection: string[]): number {
  const chosen = new Set(selection);
  return items.reduce((n, i) => (chosen.has(i.id) ? n + i.size : n), 0);
}
