import { WebviewWindow } from '@tauri-apps/api/webviewWindow'
import { getCurrentWindow } from '@tauri-apps/api/window'
import i18n from '../i18n'
import { isDesktopRuntime } from './runtime'
import { UPDATE_PROGRESS_WINDOW_LABEL, UPDATE_PROGRESS_WINDOW_QUERY_KEY } from './updaterEvents'

function buildUpdateProgressWindowUrl(): string {
  if (typeof window === 'undefined') {
    return `/?${UPDATE_PROGRESS_WINDOW_QUERY_KEY}=${UPDATE_PROGRESS_WINDOW_LABEL}`
  }

  const url = new URL(window.location.href)
  url.searchParams.set(UPDATE_PROGRESS_WINDOW_QUERY_KEY, UPDATE_PROGRESS_WINDOW_LABEL)
  return url.toString()
}

async function focusWindow(windowRef: WebviewWindow): Promise<void> {
  await windowRef.show()
  await windowRef.setFocus()
}

export async function showUpdateProgressWindow(): Promise<void> {
  if (!isDesktopRuntime()) {
    return
  }

  const existing = await WebviewWindow.getByLabel(UPDATE_PROGRESS_WINDOW_LABEL)
  if (existing) {
    await focusWindow(existing)
    return
  }

  const childWindow = new WebviewWindow(UPDATE_PROGRESS_WINDOW_LABEL, {
    center: true,
    focus: true,
    height: 360,
    maximizable: false,
    minimizable: false,
    parent: getCurrentWindow(),
    resizable: false,
    title: i18n.t('updateWindowTitle'),
    url: buildUpdateProgressWindowUrl(),
    width: 440,
  })

  await new Promise<void>((resolve, reject) => {
    void childWindow.once('tauri://created', () => resolve())
    void childWindow.once('tauri://error', (event) => reject(event.payload))
  })
}

export async function isUpdateProgressWindowVisible(): Promise<boolean> {
  if (!isDesktopRuntime()) {
    return false
  }

  const existing = await WebviewWindow.getByLabel(UPDATE_PROGRESS_WINDOW_LABEL)
  if (!existing) {
    return false
  }

  return await existing.isVisible()
}
