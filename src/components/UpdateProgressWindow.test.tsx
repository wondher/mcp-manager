import { renderToStaticMarkup } from 'react-dom/server'
import { describe, expect, it, vi } from 'vitest'
import type { UpdateState } from '../services/updater'
import { UpdateProgressWindowView } from './UpdateProgressWindow'

vi.mock('react-i18next', () => ({
  useTranslation: () => ({
    t: (key: string) => key,
  }),
}))

function renderWindow(state: UpdateState) {
  return renderToStaticMarkup(
    <UpdateProgressWindowView state={state} onContinueInBackground={() => {}} onInstallNow={() => {}} />,
  )
}

describe('UpdateProgressWindow', () => {
  it('renders live download progress and a background action while downloading', () => {
    const html = renderWindow({
      currentVersion: '0.1.5',
      downloadedBytes: 620,
      error: null,
      percent: 62,
      phase: 'downloading',
      targetVersion: '0.1.6',
      totalBytes: 1000,
    })

    expect(html).toContain('update-window-shell')
    expect(html).toContain('updateWindowDownloadingTitle')
    expect(html).toContain('62%')
    expect(html).toContain('updateWindowContinueBackground')
    expect(html).toContain('update-window-progress-bar')
  })

  it('renders restart controls once the update package is ready', () => {
    const html = renderWindow({
      currentVersion: '0.1.5',
      downloadedBytes: 1000,
      error: null,
      percent: 100,
      phase: 'downloaded',
      targetVersion: '0.1.6',
      totalBytes: 1000,
    })

    expect(html).toContain('updateWindowDownloadedTitle')
    expect(html).toContain('updateWindowInstallNow')
    expect(html).toContain('updateWindowInstallLater')
    expect(html).toContain('100%')
  })
})
