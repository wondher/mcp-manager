import { emitTo } from '@tauri-apps/api/event'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { useEffect, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { isDesktopRuntime } from '../services/runtime'
import type { UpdateState } from '../services/updater'
import { UPDATE_INSTALL_REQUEST_EVENT, UPDATE_STATE_EVENT, UPDATE_SYNC_REQUEST_EVENT } from '../services/updaterEvents'

const EMPTY_UPDATE_STATE: UpdateState = {
  currentVersion: null,
  downloadedBytes: 0,
  error: null,
  percent: 0,
  phase: 'idle',
  targetVersion: null,
  totalBytes: null,
}

function formatBytes(bytes: number): string {
  if (bytes < 1024) {
    return `${bytes} B`
  }

  const units = ['KB', 'MB', 'GB', 'TB']
  let value = bytes / 1024
  let unitIndex = 0

  while (value >= 1024 && unitIndex < units.length - 1) {
    value /= 1024
    unitIndex += 1
  }

  return `${value.toFixed(value >= 100 ? 0 : value >= 10 ? 1 : 2)} ${units[unitIndex]}`
}

function resolveTitleKey(state: UpdateState): string {
  if (state.phase === 'downloaded') {
    return 'updateWindowDownloadedTitle'
  }
  if (state.phase === 'installing') {
    return 'updateWindowInstallingTitle'
  }
  if (state.phase === 'error') {
    return 'updateWindowErrorTitle'
  }
  return 'updateWindowDownloadingTitle'
}

function resolveBodyKey(state: UpdateState): string {
  if (state.phase === 'downloaded') {
    return 'updateWindowDownloadedBody'
  }
  if (state.phase === 'installing') {
    return 'updateWindowInstallingBody'
  }
  if (state.phase === 'error') {
    return 'updateWindowErrorBody'
  }
  return 'updateWindowDownloadingBody'
}

function resolveProgressDetailKey(state: UpdateState): string {
  return state.totalBytes ? 'updateWindowProgressDetail' : 'updateWindowProgressUnknown'
}

function progressPercent(state: UpdateState): number {
  if (typeof state.percent === 'number') {
    return state.percent
  }

  if (state.phase === 'downloaded' || state.phase === 'installing') {
    return 100
  }

  return 0
}

export function UpdateProgressWindowView({
  onContinueInBackground,
  onInstallNow,
  state,
}: {
  onContinueInBackground: () => void
  onInstallNow: () => void
  state: UpdateState
}) {
  const { t } = useTranslation()
  const percent = progressPercent(state)
  const progressDetail = t(resolveProgressDetailKey(state), {
    downloaded: formatBytes(state.downloadedBytes),
    total: state.totalBytes ? formatBytes(state.totalBytes) : '',
  })
  const versionSummary =
    state.currentVersion && state.targetVersion
      ? t('updateWindowVersionSummary', {
          currentVersion: state.currentVersion,
          targetVersion: state.targetVersion,
        })
      : null
  const body =
    state.phase === 'downloading'
      ? t(resolveBodyKey(state), { version: state.targetVersion ?? '' })
      : state.phase === 'error' && state.error
        ? `${t(resolveBodyKey(state))} ${state.error}`
        : t(resolveBodyKey(state))

  return (
    <div className="update-window-shell">
      <div className="update-window-card">
        <div className="update-window-header">
          <p className="update-window-eyebrow">{t('updateWindowTitle')}</p>
          <h1 className="update-window-title">{t(resolveTitleKey(state))}</h1>
          <p className="update-window-body">{body}</p>
          {versionSummary ? <p className="update-window-version">{versionSummary}</p> : null}
        </div>

        <div className="update-window-progress">
          <div className="update-window-progress-head">
            <span className="update-window-progress-label">{progressDetail}</span>
            <strong className="update-window-progress-percent">{percent}%</strong>
          </div>
          <div className="update-window-progress-track" aria-hidden="true">
            <div className="update-window-progress-bar" style={{ width: `${percent}%` }} />
          </div>
        </div>

        <div className="update-window-actions">
          {state.phase === 'downloaded' ? (
            <>
              <button type="button" className="primary-button" onClick={onInstallNow}>
                {t('updateWindowInstallNow')}
              </button>
              <button type="button" className="ghost-button compact" onClick={onContinueInBackground}>
                {t('updateWindowInstallLater')}
              </button>
            </>
          ) : (
            <button type="button" className="ghost-button compact" onClick={onContinueInBackground}>
              {t('updateWindowContinueBackground')}
            </button>
          )}
        </div>
      </div>
    </div>
  )
}

export function UpdateProgressWindow() {
  const [state, setState] = useState<UpdateState>(EMPTY_UPDATE_STATE)

  useEffect(() => {
    if (!isDesktopRuntime()) {
      return
    }

    let disposed = false
    let unlisten: (() => void) | undefined

    void getCurrentWindow()
      .listen<UpdateState>(UPDATE_STATE_EVENT, (event) => {
        if (!disposed) {
          setState(event.payload)
        }
      })
      .then((cleanup) => {
        if (disposed) {
          cleanup()
          return
        }
        unlisten = cleanup
      })

    void emitTo('main', UPDATE_SYNC_REQUEST_EVENT)

    return () => {
      disposed = true
      unlisten?.()
    }
  }, [])

  const handleContinueInBackground = () => {
    if (!isDesktopRuntime()) {
      return
    }

    void getCurrentWindow().close()
  }

  const handleInstallNow = () => {
    if (!isDesktopRuntime()) {
      return
    }

    void emitTo('main', UPDATE_INSTALL_REQUEST_EVENT)
  }

  return (
    <UpdateProgressWindowView
      state={state}
      onContinueInBackground={handleContinueInBackground}
      onInstallNow={handleInstallNow}
    />
  )
}
