import { invoke } from '@tauri-apps/api/core'

import type {
  BackendStatus,
  Card,
  CardPage,
  CreateLanguageProfileInput,
  CreateCardsInput,
  DeleteCardsInput,
  LanguageProfile,
  LanguageHelperClient,
  ProfileSettings,
  SaveProfileSettingsInput,
  ListCardsInput,
  PendingInverseCard,
  NormalizeCardInput,
  NewCardInput,
  PrepareInverseCardsInput,
  SaveInverseCardsInput,
  UpdateCardInput,
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

  listCards(input: ListCardsInput): Promise<CardPage> {
    return invoke<CardPage>('list_cards', { query: input })
  }

  getCard(
    username: string,
    profileId: string,
    cardId: string,
  ): Promise<Card> {
    return invoke<Card>('get_card', { username, profileId, cardId })
  }

  createCards(input: CreateCardsInput): Promise<Card[]> {
    return invoke<Card[]>('create_cards', { command: input })
  }

  normalizeCard(input: NormalizeCardInput): Promise<NewCardInput> {
    return invoke<NewCardInput>('normalize_card', { command: input })
  }

  updateCard(input: UpdateCardInput): Promise<Card> {
    return invoke<Card>('update_card', { command: input })
  }

  deleteCards(input: DeleteCardsInput): Promise<number> {
    return invoke<number>('delete_cards', { command: input })
  }

  prepareInverseCards(
    input: PrepareInverseCardsInput,
  ): Promise<PendingInverseCard[]> {
    return invoke<PendingInverseCard[]>('prepare_inverse_cards', {
      query: input,
    })
  }

  saveInverseCards(input: SaveInverseCardsInput): Promise<Card[]> {
    return invoke<Card[]>('save_inverse_cards', { command: input })
  }
}
