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

export interface ProfileSettings {
  version: number
  cardsPerSet: number
  answerMode: 'written' | 'self-review'
  masteryThreshold: number
  checkReadingIfPossible: boolean
  provider: 'openai' | 'gemini' | null
  apiKey: string | null
  modelName: string | null
}

export interface SaveProfileSettingsInput extends ProfileSettings {
  username: string
  profileId: string
}

export type CardDirection = 'straight' | 'reverse'
export type CardMastery = 'any' | 'learned' | 'unlearned'
export type CardSortField = 'word' | 'createdAt' | 'streak'
export type SortDirection = 'ascending' | 'descending'

export interface UsageExample {
  sentence: string
  translation: string
}

export interface CardMeaning {
  definition: string
  translatedDefinition: string
  wordTranslations: string[]
  examples: UsageExample[]
}

export interface Card {
  id: string
  profileId: string
  direction: CardDirection
  word: string
  readings: string[]
  meanings: CardMeaning[]
  streak: number
  createdAt: number
  version: number
}

export interface CardSummary {
  id: string
  word: string
  direction: CardDirection
  streak: number
  createdAt: number
}

export interface CardPage {
  items: CardSummary[]
  nextCursor: string | null
}

export interface ListCardsInput {
  username: string
  profileId: string
  search?: string
  direction: CardDirection | null
  mastery: CardMastery
  masteryThreshold: number
  maxStreak: number | null
  sortField: CardSortField
  sortDirection: SortDirection
  cursor: string | null
  limit: number
}

export interface NewCardInput {
  direction: CardDirection
  word: string
  readings: string[]
  meanings: CardMeaning[]
}

export interface CreateCardsInput {
  username: string
  profileId: string
  cards: NewCardInput[]
}

export interface NormalizeCardInput {
  username: string
  profileId: string
  card: NewCardInput
}

export interface UpdateCardInput {
  username: string
  profileId: string
  cardId: string
  expectedVersion: number
  word: string
  readings: string[]
  meanings: CardMeaning[]
}

export interface DeleteCardsInput {
  username: string
  profileId: string
  cardIds: string[]
}

export interface PendingInverseCard {
  card: Card
  expectedVersion: number | null
}

export interface PrepareInverseCardsInput {
  username: string
  profileId: string
  sourceCardIds: string[]
}

export interface SaveInverseCardsInput {
  username: string
  profileId: string
  cards: PendingInverseCard[]
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
  getProfileSettings(
    username: string,
    profileId: string,
  ): Promise<ProfileSettings>
  saveProfileSettings(
    input: SaveProfileSettingsInput,
  ): Promise<ProfileSettings>
  listCards(input: ListCardsInput): Promise<CardPage>
  getCard(username: string, profileId: string, cardId: string): Promise<Card>
  createCards(input: CreateCardsInput): Promise<Card[]>
  normalizeCard(input: NormalizeCardInput): Promise<NewCardInput>
  updateCard(input: UpdateCardInput): Promise<Card>
  deleteCards(input: DeleteCardsInput): Promise<number>
  prepareInverseCards(
    input: PrepareInverseCardsInput,
  ): Promise<PendingInverseCard[]>
  saveInverseCards(input: SaveInverseCardsInput): Promise<Card[]>
}
