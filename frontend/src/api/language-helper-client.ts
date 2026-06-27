export interface BackendStatus {
  transport: 'tauri' | 'mock'
  ready: boolean
  message: string
}

/**
 * Transport-independent boundary between React and the application backend.
 *
 * Keep Tauri imports out of components. A future browser build can provide an
 * HTTP implementation without changing routes or feature components.
 */
export interface LanguageHelperClient {
  getBackendStatus(): Promise<BackendStatus>
  getUsernames(): Promise<string[]>
}
