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

export interface AiSettings {
  version: number
  provider: 'openai' | 'gemini' | null
  apiKey: string | null
  modelName: string | null
}

export interface SaveAiSettingsInput extends AiSettings {
  username: string
  profileId: string
}

export type CardDirection = 'straight' | 'reverse'
export type CardSortField = 'word' | 'createdAt' | 'score'
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
  score: number
  createdAt: number
  version: number
}

export interface CardSummary {
  id: string
  word: string
  direction: CardDirection
  score: number
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
  minScore: number | null
  maxScore: number | null
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

export interface GetCardSpeechInput {
  username: string
  profileId: string
  cardId: string
  regenerate: boolean
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

export type StudySessionMode = 'learning' | 'test'
export type StudySessionPhase = 'study' | 'test'
export type StudySessionStatus = 'active' | 'completed' | 'cancelled'
export type StudySessionAction =
  | 'previousStudyCard'
  | 'nextStudyCard'
  | 'startMiniTest'
  | 'submitWrittenAnswer'
  | 'continueAfterFeedback'

export interface CreateStudySessionInput {
  username: string
  profileId: string
  mode: StudySessionMode
  direction: CardDirection | null
  minScore: number | null
  maxScore: number | null
  cardsPerSet: number | null
  pronunciationCheckEnabled: boolean
  pronunciationAccuracyThreshold: number
}

export interface ApplyStudySessionActionInput {
  username: string
  sessionId: string
  expectedVersion: number
  action: StudySessionAction
  answer?: string
}

export interface EndStudySessionInput {
  username: string
  sessionId: string
  expectedVersion: number
}

export interface SessionCurrentCard {
  kind: 'study' | 'test'
  card: Card | null
  id: string | null
  direction: CardDirection | null
  prompt: string | null
  readings: string[]
  remainingMeanings: number | null
  totalMeanings: number | null
}

export interface StudySession {
  id: string
  profileId: string
  mode: StudySessionMode
  phase: StudySessionPhase
  status: StudySessionStatus
  pronunciationCheckEnabled: boolean
  pronunciationAccuracyThreshold: number
  awaitingContinue: boolean
  currentCard: SessionCurrentCard | null
  currentCardNumber: number
  totalCards: number
  currentSet: number
  totalSets: number
  summary: {
    correct: number
    incorrect: number
    scoreDelta: number
  }
  version: number
}

export interface StudySessionTransition {
  session: StudySession
  answerFeedback: {
    isCorrect: boolean
    matchedAnswer: string | null
    expectedAnswers: string[]
    cardCompleted: boolean
    remainingMeanings: number
    scoreDelta: number
  } | null
  setOutcome: 'passed' | 'retry' | null
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
  getAiSettings(
    username: string,
    profileId: string,
  ): Promise<AiSettings>
  saveAiSettings(input: SaveAiSettingsInput): Promise<AiSettings>
  listCards(input: ListCardsInput): Promise<CardPage>
  getCard(username: string, profileId: string, cardId: string): Promise<Card>
  createCards(input: CreateCardsInput): Promise<Card[]>
  normalizeCard(input: NormalizeCardInput): Promise<NewCardInput>
  getCardSpeech(input: GetCardSpeechInput): Promise<Blob>
  updateCard(input: UpdateCardInput): Promise<Card>
  deleteCards(input: DeleteCardsInput): Promise<number>
  prepareInverseCards(
    input: PrepareInverseCardsInput,
  ): Promise<PendingInverseCard[]>
  saveInverseCards(input: SaveInverseCardsInput): Promise<Card[]>
  createStudySession(input: CreateStudySessionInput): Promise<StudySession>
  applyStudySessionAction(
    input: ApplyStudySessionActionInput,
  ): Promise<StudySessionTransition>
  finishStudySession(input: EndStudySessionInput): Promise<StudySession>
  cancelStudySession(input: EndStudySessionInput): Promise<StudySession>
}
