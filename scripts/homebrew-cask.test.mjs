import { describe, expect, it } from 'vitest'

async function loadHomebrewHelpers() {
  try {
    return await import('./homebrew-cask.mjs')
  } catch {
    return null
  }
}

const release = {
  tag_name: 'v0.1.6',
  html_url: 'https://github.com/xjeway/mcp-manager/releases/tag/v0.1.6',
  assets: [
    {
      name: 'MCP.Manager_aarch64.app.tar.gz',
      browser_download_url:
        'https://github.com/xjeway/mcp-manager/releases/download/v0.1.6/MCP.Manager_aarch64.app.tar.gz',
    },
    {
      name: 'MCP.Manager_x64.app.tar.gz',
      browser_download_url:
        'https://github.com/xjeway/mcp-manager/releases/download/v0.1.6/MCP.Manager_x64.app.tar.gz',
    },
    {
      name: 'MCP.Manager_x64.dmg',
      browser_download_url:
        'https://github.com/xjeway/mcp-manager/releases/download/v0.1.6/MCP.Manager_x64.dmg',
    },
  ],
}

describe('homebrew cask helpers', () => {
  it('selects macOS app tarballs for both Homebrew architectures', async () => {
    const helpers = await loadHomebrewHelpers()

    expect(helpers).not.toBeNull()

    expect(helpers.collectMacosCaskAssets(release.assets)).toEqual({
      arm: expect.objectContaining({
        name: 'MCP.Manager_aarch64.app.tar.gz',
      }),
      intel: expect.objectContaining({
        name: 'MCP.Manager_x64.app.tar.gz',
      }),
    })
  })

  it('requires both macOS app tarballs before rendering a cask', async () => {
    const helpers = await loadHomebrewHelpers()

    expect(helpers).not.toBeNull()

    expect(() => helpers.collectMacosCaskAssets([release.assets[1]])).toThrow(
      /Missing required macOS Homebrew assets: arm/,
    )
  })

  it('renders the tap cask with versioned release URLs and checksums', async () => {
    const helpers = await loadHomebrewHelpers()

    expect(helpers).not.toBeNull()

    const rendered = helpers.renderCask({
      version: '0.1.6',
      assets: {
        arm: {
          name: 'MCP.Manager_aarch64.app.tar.gz',
          sha256: 'a'.repeat(64),
        },
        intel: {
          name: 'MCP.Manager_x64.app.tar.gz',
          sha256: 'b'.repeat(64),
        },
      },
      owner: 'xjeway',
      repo: 'mcp-manager',
    })

    expect(rendered).toContain('cask "mcp-manager" do')
    expect(rendered).toContain('version "0.1.6"')
    expect(rendered).toContain(`sha256 arm: "${'a'.repeat(64)}",`)
    expect(rendered).toContain(`        intel: "${'b'.repeat(64)}"`)
    expect(rendered).toContain(
      'url "https://github.com/xjeway/mcp-manager/releases/download/v#{version}/#{asset_name}"',
    )
    expect(rendered).toContain('app "MCP Manager.app"')
  })

  it('uses GitHub asset digests when available', async () => {
    const helpers = await loadHomebrewHelpers()

    expect(helpers).not.toBeNull()
    expect(
      helpers.sha256FromAssetDigest({
        digest: `sha256:${'c'.repeat(64)}`,
      }),
    ).toBe('c'.repeat(64))
    expect(helpers.sha256FromAssetDigest({ digest: 'md5:not-supported' })).toBeNull()
  })
})
