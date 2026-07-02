import { invoke } from '@tauri-apps/api/core'

import type {
  BackendStatus,
  LanguageHelperClient,
} from './language-helper-client'

export class TauriLanguageHelperClient implements LanguageHelperClient {
  getBackendStatus(): Promise<BackendStatus> {
    return invoke<BackendStatus>('get_backend_status')
  }

  getUsernames(): Promise<string[]> {
    return invoke<string[]>('get_usernames')
  }

  createUser(username: string): Promise<string> {
    return invoke<string>('create_user', { username })
  }
}
