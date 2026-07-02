export interface BackendStatus {
  transport: 'tauri' | 'mock'
  ready: boolean
  message: string
}

export interface LanguageProfile {
  id: string
  name: string
  sourceLanguage: string
  targetLanguage: string
}

export interface CreateLanguageProfileInput {
  username: string
  name: string
  sourceLanguage: string
  targetLanguage: string
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
  createUser(username: string): Promise<string>
  getLanguageProfiles(username: string): Promise<LanguageProfile[]>
  createLanguageProfile(
    input: CreateLanguageProfileInput,
  ): Promise<LanguageProfile>
}
