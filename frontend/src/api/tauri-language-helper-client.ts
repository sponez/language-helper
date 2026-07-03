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
  AiSettings,
  SaveAiSettingsInput,
  ListCardsInput,
  PendingInverseCard,
  NormalizeCardInput,
  GetCardSpeechInput,
  NewCardInput,
  PrepareInverseCardsInput,
  SaveInverseCardsInput,
  UpdateCardInput,
  ApplyStudySessionActionInput,
  CreateStudySessionInput,
  EndStudySessionInput,
  StudySession,
  StudySessionTransition,
  StudySessionPreferences,
  StudySessionMode,
  PronunciationSettings,
  SavePronunciationSettingsInput,
  AssessPronunciationInput,
} from './language-helper-client'

export class TauriLanguageHelperClient implements LanguageHelperClient {
  async getBackendStatus(): Promise<BackendStatus> {
    return {
      transport: 'tauri',
      ready: true,
      message: 'Desktop backend',
    }
  }

  getUsernames(): Promise<string[]> {
    return invoke<string[]>('get_usernames')
  }

  createUser(username: string): Promise<string> {
    return invoke<string>('create_user', { username })
  }

  deleteUser(username: string): Promise<boolean> {
    return invoke<boolean>('delete_user', { username })
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

  deleteLanguageProfile(
    username: string,
    profileId: string,
  ): Promise<boolean> {
    return invoke<boolean>('delete_language_profile', {
      username,
      profileId,
    })
  }

  getAiSettings(username: string): Promise<AiSettings> {
    return invoke<AiSettings>('get_ai_settings', { username })
  }

  saveAiSettings(input: SaveAiSettingsInput): Promise<AiSettings> {
    return invoke<AiSettings>('save_ai_settings', {
      settings: input,
    })
  }

  getPronunciationSettings(username: string): Promise<PronunciationSettings> {
    return invoke<PronunciationSettings>('get_pronunciation_settings', {
      username,
    })
  }

  savePronunciationSettings(
    input: SavePronunciationSettingsInput,
  ): Promise<PronunciationSettings> {
    return invoke<PronunciationSettings>('save_pronunciation_settings', {
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

  async getCardSpeech(input: GetCardSpeechInput): Promise<Blob> {
    const response = await invoke<ArrayBuffer | Uint8Array | number[]>(
      'get_card_speech',
      { request: input },
    )
    const bytes =
      response instanceof ArrayBuffer
        ? new Uint8Array(response)
        : new Uint8Array(response)
    return new Blob([bytes], { type: 'audio/wav' })
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

  getStudySessionPreferences(
    username: string,
    profileId: string,
    mode: StudySessionMode,
  ): Promise<StudySessionPreferences> {
    return invoke<StudySessionPreferences>('get_study_session_preferences', {
      username,
      profileId,
      mode,
    })
  }

  createStudySession(input: CreateStudySessionInput): Promise<StudySession> {
    return invoke<StudySession>('create_study_session', { command: input })
  }

  applyStudySessionAction(
    input: ApplyStudySessionActionInput,
  ): Promise<StudySessionTransition> {
    return invoke<StudySessionTransition>('apply_study_session_action', {
      command: input,
    })
  }

  assessPronunciation(
    input: AssessPronunciationInput,
  ): Promise<StudySessionTransition> {
    return invoke<StudySessionTransition>('assess_pronunciation', {
      command: input,
    })
  }

  finishStudySession(input: EndStudySessionInput): Promise<StudySession> {
    return invoke<StudySession>('finish_study_session', { command: input })
  }

  cancelStudySession(input: EndStudySessionInput): Promise<StudySession> {
    return invoke<StudySession>('cancel_study_session', { command: input })
  }
}
