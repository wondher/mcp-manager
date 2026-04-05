import { UPDATE_PROGRESS_WINDOW_LABEL, UPDATE_PROGRESS_WINDOW_QUERY_KEY } from './updaterEvents'

export type WindowSurface = 'main' | 'updater-progress'

export function resolveWindowSurface(): WindowSurface {
  if (typeof window === 'undefined') {
    return 'main'
  }

  const params = new URLSearchParams(window.location.search)
  return params.get(UPDATE_PROGRESS_WINDOW_QUERY_KEY) === UPDATE_PROGRESS_WINDOW_LABEL ? 'updater-progress' : 'main'
}
