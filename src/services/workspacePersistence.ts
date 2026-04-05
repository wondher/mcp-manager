import type { ImportDetectedResult, MCPConfig, SupportedApp } from '../types/config'

interface SaveAndSyncConfigOptions {
  applyConfig: (config: MCPConfig, previousConfig: MCPConfig) => Promise<{ backups: string[] }>
  nextConfig: MCPConfig
  previousConfig: MCPConfig
  saveConfig: (config: MCPConfig) => Promise<void>
}

interface ImportConfigOnLaunchOptions {
  autoImportOnLaunch: boolean
  importDetectedConfigs: () => Promise<ImportDetectedResult>
}

export async function saveAndSyncConfig({
  applyConfig,
  nextConfig,
  previousConfig,
  saveConfig,
}: SaveAndSyncConfigOptions): Promise<{ backups: string[] }> {
  await saveConfig(nextConfig)

  try {
    return await applyConfig(nextConfig, previousConfig)
  } catch (error) {
    await saveConfig(previousConfig)
    throw error
  }
}

export async function persistImportedConfig(
  nextConfig: MCPConfig,
  saveConfig: (config: MCPConfig) => Promise<void>,
): Promise<void> {
  await saveConfig(nextConfig)
}

export async function importConfigOnLaunch({
  autoImportOnLaunch,
  importDetectedConfigs,
}: ImportConfigOnLaunchOptions): Promise<ImportDetectedResult | null> {
  if (!autoImportOnLaunch) {
    return null
  }

  return await importDetectedConfigs()
}

export function deleteServerFromConfig(config: MCPConfig, serverId: string): MCPConfig {
  return {
    ...config,
    servers: config.servers.filter((server) => server.id !== serverId),
  }
}

export function toggleServerAppInConfig(config: MCPConfig, serverId: string, app: SupportedApp): MCPConfig {
  return {
    ...config,
    servers: config.servers.map((server) =>
      server.id === serverId
        ? { ...server, apps: { ...server.apps, [app]: !server.apps[app] } }
        : server,
    ),
  }
}
