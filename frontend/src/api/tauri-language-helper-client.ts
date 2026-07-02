import { invoke } from '@tauri-apps/api/core'

import type {
  BackendStatus,
  CreateLanguageProfileInput,
  LanguageProfile,
  LanguageHelperClient,
  ProfileSettings,
  SaveProfileSettingsInput,
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

  getLanguageProfiles(username: string): Promise<LanguageProfile[]> {
    return invoke<LanguageProfile[]>('list_language_profiles', { username })
  }

  createLanguageProfile(
    input: CreateLanguageProfileInput,
  ): Promise<LanguageProfile> {
    return invoke<LanguageProfile>('create_language_profile', {
      username: input.username,
      name: input.name,
      sourceLanguage: input.sourceLanguage,
      targetLanguage: input.targetLanguage,
    })
  }

  getProfileSettings(
    username: string,
    profileId: string,
  ): Promise<ProfileSettings> {
    return invoke<ProfileSettings>('get_profile_settings', {
      username,
      profileId,
    })
  }

  saveProfileSettings(
    input: SaveProfileSettingsInput,
  ): Promise<ProfileSettings> {
    return invoke<ProfileSettings>('save_profile_settings', {
      settings: input,
    })
  }
}
