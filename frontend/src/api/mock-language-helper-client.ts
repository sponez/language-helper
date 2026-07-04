import type {
  BackendStatus,
  Card,
  CardMeaning,
  CardPage,
  CreateCardsInput,
  CreateLanguageProfileInput,
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

interface MockSessionState {
  view: StudySession
  input: CreateStudySessionInput
  cardIds: string[]
  testOrder: string[]
  currentIndex: number
  completedMeanings: number[]
  answers: string[]
  setFailed: boolean
}

function damerauLevenshtein(left: string, right: string): number {
  const source = Array.from(left)
  const target = Array.from(right)
  const maxDistance = source.length + target.length
  const distance = Array.from({ length: source.length + 2 }, () =>
    Array<number>(target.length + 2).fill(0),
  )
  distance[0][0] = maxDistance
  for (let sourceIndex = 0; sourceIndex <= source.length; sourceIndex += 1) {
    distance[sourceIndex + 1][0] = maxDistance
    distance[sourceIndex + 1][1] = sourceIndex
  }
  for (let targetIndex = 0; targetIndex <= target.length; targetIndex += 1) {
    distance[0][targetIndex + 1] = maxDistance
    distance[1][targetIndex + 1] = targetIndex
  }

  const lastSeen = new Map<string, number>()
  for (let sourceIndex = 1; sourceIndex <= source.length; sourceIndex += 1) {
    let lastMatch = 0
    for (let targetIndex = 1; targetIndex <= target.length; targetIndex += 1) {
      const previousSourceIndex = lastSeen.get(target[targetIndex - 1]) ?? 0
      const previousTargetIndex = lastMatch
      let substitutionCost = 1
      if (source[sourceIndex - 1] === target[targetIndex - 1]) {
        substitutionCost = 0
        lastMatch = targetIndex
      }
      distance[sourceIndex + 1][targetIndex + 1] = Math.min(
        distance[sourceIndex][targetIndex] + substitutionCost,
        distance[sourceIndex + 1][targetIndex] + 1,
        distance[sourceIndex][targetIndex + 1] + 1,
        distance[previousSourceIndex][previousTargetIndex] +
          (sourceIndex - previousSourceIndex - 1) +
          1 +
          (targetIndex - previousTargetIndex - 1),
      )
    }
    lastSeen.set(source[sourceIndex - 1], sourceIndex)
  }
  return distance[source.length + 1][target.length + 1]
}

function answerSimilarity(expected: string, actual: string): number {
  const normalizedExpected = expected.trim().toLowerCase()
  const normalizedActual = actual.trim().toLowerCase()
  if (normalizedExpected === normalizedActual) return 1
  const length = Math.max(
    Array.from(normalizedExpected).length,
    Array.from(normalizedActual).length,
  )
  return length === 0
    ? 0
    : 1 - damerauLevenshtein(normalizedExpected, normalizedActual) / length
}

export class MockLanguageHelperClient implements LanguageHelperClient {
  private readonly usernames: string[] = []
  private readonly profiles = new Map<string, LanguageProfile[]>()
  private readonly settings = new Map<string, AiSettings>()
  private readonly cards = new Map<string, Card[]>()
  private readonly sessions = new Map<string, MockSessionState>()
  private readonly testHistory = new Map<string, string[]>()
  private readonly sessionPreferences = new Map<
    string,
    StudySessionPreferences
  >()

  async getBackendStatus(): Promise<BackendStatus> {
    return {
      transport: 'mock',
      ready: true,
      message: 'Frontend development mode',
    }
  }

  async getUsernames(): Promise<string[]> {
    return [...this.usernames]
  }

  async createUser(username: string): Promise<string> {
    if (this.usernames.includes(username)) {
      throw new Error(`User "${username}" already exists.`)
    }

    this.usernames.push(username)
    this.profiles.set(username, [])
    this.settings.set(username, {
      version: 0,
      provider: null,
      apiKey: null,
      modelName: null,
    })
    return username
  }

  async deleteUser(username: string): Promise<boolean> {
    const index = this.usernames.indexOf(username)
    if (index < 0) return false
    this.usernames.splice(index, 1)
    for (const profile of this.profiles.get(username) ?? []) {
      this.cards.delete(profile.id)
      this.testHistory.delete(profile.id)
      for (const mode of ['learning', 'test'] as const) {
        this.sessionPreferences.delete(`${profile.id}:${mode}`)
      }
    }
    this.profiles.delete(username)
    this.settings.delete(username)
    return true
  }

  async getLanguageProfiles(username: string): Promise<LanguageProfile[]> {
    return [...(this.profiles.get(username) ?? [])]
  }

  async createLanguageProfile(
    input: CreateLanguageProfileInput,
  ): Promise<LanguageProfile> {
    const profiles = this.profiles.get(input.username)
    if (!profiles) {
      throw new Error(`User "${input.username}" does not exist.`)
    }
    if (profiles.some((profile) => profile.name === input.name)) {
      throw new Error(`Profile "${input.name}" already exists.`)
    }

    const profile: LanguageProfile = {
      id: crypto.randomUUID(),
      name: input.name,
      sourceLanguage: input.sourceLanguage,
      targetLanguage: input.targetLanguage,
    }
    profiles.push(profile)
    this.cards.set(profile.id, [])
    return profile
  }

  async deleteLanguageProfile(
    username: string,
    profileId: string,
  ): Promise<boolean> {
    const profiles = this.profiles.get(username)
    const index =
      profiles?.findIndex((profile) => profile.id === profileId) ?? -1
    if (!profiles || index < 0) return false
    profiles.splice(index, 1)
    this.cards.delete(profileId)
    this.testHistory.delete(profileId)
    this.sessionPreferences.delete(`${profileId}:learning`)
    this.sessionPreferences.delete(`${profileId}:test`)
    return true
  }

  async getAiSettings(username: string): Promise<AiSettings> {
    const settings = this.settings.get(username)
    if (!settings) {
      throw new Error('User settings were not found.')
    }
    return { ...settings }
  }

  async saveAiSettings(input: SaveAiSettingsInput): Promise<AiSettings> {
    const current = this.settings.get(input.username)
    if (!current) {
      throw new Error('User settings were not found.')
    }
    if (current.version !== input.version) {
      throw new Error('Profile settings were changed concurrently.')
    }

    const saved: AiSettings = {
      version: current.version + 1,
      provider: input.provider,
      apiKey: input.apiKey,
      modelName: input.modelName,
    }
    this.settings.set(input.username, saved)
    return { ...saved }
  }

  async getPronunciationSettings(
    _username: string,
  ): Promise<PronunciationSettings> {
    return {
      version: 0,
      endpoint: null,
      subscriptionKey: null,
      configured: false,
    }
  }

  async savePronunciationSettings(
    _input: SavePronunciationSettingsInput,
  ): Promise<PronunciationSettings> {
    throw new Error(
      'Pronunciation assessment is available only in the desktop application.',
    )
  }

  async listCards(input: ListCardsInput): Promise<CardPage> {
    const query = input.search?.trim().toLocaleLowerCase() ?? ''
    const sorted = [...(this.cards.get(input.profileId) ?? [])]
      .filter(
        (card) =>
          !query ||
          card.word.toLocaleLowerCase().includes(query) ||
          card.readings.some((reading) =>
            reading.toLocaleLowerCase().includes(query),
          ),
      )
      .filter((card) => !input.direction || card.direction === input.direction)
      .filter(
        (card) =>
          (input.minScore === null || card.score >= input.minScore) &&
          (input.maxScore === null || card.score <= input.maxScore),
      )
      .sort((left, right) => {
        const compared =
          input.sortField === 'word'
            ? left.word.localeCompare(right.word)
            : input.sortField === 'score'
              ? left.score - right.score
              : left.createdAt - right.createdAt
        const stable = compared || left.id.localeCompare(right.id)
        return input.sortDirection === 'ascending' ? stable : -stable
      })
    const offset = Number(input.cursor ?? 0)
    const items = sorted.slice(offset, offset + input.limit)
    const nextOffset = offset + items.length
    return {
      items: items.map(({ id, word, direction, score, createdAt }) => ({
        id,
        word,
        direction,
        score,
        createdAt,
      })),
      nextCursor: nextOffset < sorted.length ? String(nextOffset) : null,
    }
  }

  async getCard(
    _username: string,
    profileId: string,
    cardId: string,
  ): Promise<Card> {
    const card = this.cards
      .get(profileId)
      ?.find((candidate) => candidate.id === cardId)
    if (!card) {
      throw new Error('Card was not found.')
    }
    return structuredClone(card)
  }

  async normalizeCard(input: NormalizeCardInput): Promise<NewCardInput> {
    const settings = this.settings.get(input.username)
    if (!settings?.provider || !settings.apiKey || !settings.modelName) {
      throw new Error(
        'AI provider is not configured. Open Settings to configure it.',
      )
    }
    return structuredClone(input.card)
  }

  async getCardSpeech(_input: GetCardSpeechInput): Promise<Blob> {
    throw new Error(
      'AI speech generation is available only through the desktop backend.',
    )
  }

  async createCards(input: CreateCardsInput): Promise<Card[]> {
    const cards = this.cards.get(input.profileId)
    if (!cards) {
      throw new Error('Language profile was not found.')
    }
    const batchWords = new Set<string>()
    const created = input.cards.map((card) => {
      const word = card.word.trim()
      if (
        batchWords.has(word) ||
        cards.some((existing) => existing.word === word)
      ) {
        throw new Error('A card with this word already exists.')
      }
      batchWords.add(word)
      return {
        ...structuredClone(card),
        id: crypto.randomUUID(),
        profileId: input.profileId,
        word,
        score: 0,
        createdAt: Date.now(),
        version: 0,
      }
    })
    cards.push(...created)
    return structuredClone(created)
  }

  async updateCard(input: UpdateCardInput): Promise<Card> {
    const cards = this.cards.get(input.profileId)
    const index = cards?.findIndex((card) => card.id === input.cardId) ?? -1
    if (!cards || index < 0) {
      throw new Error('Card was not found.')
    }
    if (cards[index].version !== input.expectedVersion) {
      throw new Error('Card was modified concurrently.')
    }
    if (
      cards.some(
        (card, candidateIndex) =>
          candidateIndex !== index && card.word === input.word.trim(),
      )
    ) {
      throw new Error('A card with this word already exists.')
    }
    cards[index] = {
      ...cards[index],
      word: input.word.trim(),
      readings: structuredClone(input.readings),
      meanings: structuredClone(input.meanings),
      version: input.expectedVersion + 1,
    }
    return structuredClone(cards[index])
  }

  async deleteCards(input: DeleteCardsInput): Promise<number> {
    const cards = this.cards.get(input.profileId)
    if (!cards) {
      throw new Error('Language profile was not found.')
    }
    const ids = new Set(input.cardIds)
    const retained = cards.filter((card) => !ids.has(card.id))
    const deleted = cards.length - retained.length
    this.cards.set(input.profileId, retained)
    return deleted
  }

  async prepareInverseCards(
    input: PrepareInverseCardsInput,
  ): Promise<PendingInverseCard[]> {
    const cards = this.cards.get(input.profileId)
    const sources = input.sourceCardIds.map((sourceCardId) =>
      cards?.find((card) => card.id === sourceCardId),
    )
    if (!cards || sources.length === 0 || sources.some((source) => !source)) {
      throw new Error('Card was not found.')
    }

    const grouped = new Map<
      string,
      { direction: Card['direction']; meanings: CardMeaning[] }
    >()
    for (const source of sources) {
      if (!source) continue
      for (const meaning of source.meanings) {
        for (const rawTranslation of meaning.wordTranslations) {
          const translation = rawTranslation.trim()
          const inverse: CardMeaning = {
            definition: meaning.translatedDefinition.trim() || translation,
            translatedDefinition: meaning.definition,
            wordTranslations: [source.word],
            examples: meaning.examples.map((example) => ({
              sentence: example.translation,
              translation: example.sentence,
            })),
          }
          const current = grouped.get(translation)
          grouped.set(translation, {
            direction:
              current?.direction ??
              (source.direction === 'straight' ? 'reverse' : 'straight'),
            meanings: [...(current?.meanings ?? []), inverse],
          })
        }
      }
    }

    return [...grouped].map(([word, generated]) => {
      const existing = cards.find((card) => card.word === word)
      if (existing) {
        return {
          card: {
            ...structuredClone(existing),
            meanings: [
              ...structuredClone(existing.meanings),
              ...generated.meanings,
            ],
          },
          expectedVersion: existing.version,
        }
      }
      return {
        card: {
          id: crypto.randomUUID(),
          profileId: input.profileId,
          direction: generated.direction,
          word,
          readings: [],
          meanings: generated.meanings,
          score: 0,
          createdAt: Date.now(),
          version: 0,
        },
        expectedVersion: null,
      }
    })
  }

  async saveInverseCards(input: SaveInverseCardsInput): Promise<Card[]> {
    const cards = this.cards.get(input.profileId)
    if (!cards || input.cards.length === 0) {
      throw new Error('No inverse cards to save.')
    }
    const next = structuredClone(cards)
    const saved: Card[] = []
    for (const pending of input.cards) {
      const index = next.findIndex((card) => card.id === pending.card.id)
      if (pending.expectedVersion === null) {
        if (next.some((card) => card.word === pending.card.word.trim())) {
          throw new Error('A card with this word already exists.')
        }
        const card = structuredClone(pending.card)
        card.word = card.word.trim()
        next.push(card)
        saved.push(card)
      } else {
        if (index < 0 || next[index].version !== pending.expectedVersion) {
          throw new Error('Card was modified concurrently.')
        }
        const card = structuredClone(pending.card)
        card.word = card.word.trim()
        card.version = pending.expectedVersion + 1
        next[index] = card
        saved.push(card)
      }
    }
    this.cards.set(input.profileId, next)
    return structuredClone(saved)
  }

  private matchingCards(input: CreateStudySessionInput): Card[] {
    return (this.cards.get(input.profileId) ?? []).filter(
      (card) =>
        (!input.direction || card.direction === input.direction) &&
        (input.minScore === null || card.score >= input.minScore) &&
        (input.maxScore === null || card.score <= input.maxScore),
    )
  }

  private sessionCard(state: MockSessionState): Card | null {
    const cards = this.cards.get(state.input.profileId) ?? []
    const id =
      state.view.mode === 'test'
        ? state.cardIds[0]
        : state.view.phase === 'study'
          ? state.cardIds[
              (state.view.currentSet - 1) * (state.input.cardsPerSet ?? 5) +
                state.currentIndex
            ]
          : state.testOrder[state.currentIndex]
    return cards.find((card) => card.id === id) ?? null
  }

  private refreshView(state: MockSessionState): StudySession {
    const card = state.view.status === 'active' ? this.sessionCard(state) : null
    state.view.currentCard = card
      ? state.view.phase === 'study'
        ? {
            kind: 'study',
            card: structuredClone(card),
            id: null,
            direction: null,
            prompt: null,
            readings: [],
            remainingMeanings: null,
            totalMeanings: null,
          }
        : {
            kind: 'test',
            card: null,
            id: card.id,
            direction: card.direction,
            prompt: card.word,
            readings: [...card.readings],
            remainingMeanings:
              card.meanings.length - state.completedMeanings.length,
            totalMeanings: card.meanings.length,
          }
      : null
    state.view.pronunciationRequired =
      state.view.status === 'active' &&
      state.view.phase === 'test' &&
      state.view.pronunciationCheckEnabled &&
      card?.direction === 'straight' &&
      !state.view.awaitingContinue
    state.view.version += 1
    return structuredClone(state.view)
  }

  async createStudySession(
    input: CreateStudySessionInput,
  ): Promise<StudySession> {
    const pronunciationCheckEnabled =
      input.pronunciationCheckEnabled && input.direction !== 'reverse'
    input = { ...input, pronunciationCheckEnabled }
    const eligible = this.matchingCards(input).sort(() => Math.random() - 0.5)
    if (eligible.length === 0)
      throw new Error('No matching cards are available.')
    let selected = eligible
    if (input.mode === 'test') {
      const history = this.testHistory.get(input.profileId) ?? []
      const banned = new Set(history.slice(-Math.floor(eligible.length / 2)))
      selected = [eligible.find((card) => !banned.has(card.id)) ?? eligible[0]]
      history.push(selected[0].id)
      this.testHistory.set(input.profileId, history)
    }
    const id = crypto.randomUUID()
    const setSize = input.cardsPerSet ?? 1
    const view: StudySession = {
      id,
      profileId: input.profileId,
      mode: input.mode,
      phase: input.mode === 'learning' ? 'study' : 'test',
      status: 'active',
      pronunciationCheckEnabled,
      pronunciationScoreThreshold: input.pronunciationScoreThreshold,
      pronunciationRequired: false,
      pronunciationAttemptsUsed: 0,
      pronunciationTechnicalFailures: 0,
      pronunciationDisableRequired: false,
      awaitingContinue: false,
      currentCard: null,
      currentCardNumber: 1,
      totalCards: input.mode === 'learning' ? selected.length : 0,
      currentSet: input.mode === 'learning' ? 1 : 0,
      totalSets:
        input.mode === 'learning' ? Math.ceil(selected.length / setSize) : 0,
      summary: { correct: 0, incorrect: 0, scoreDelta: 0 },
      version: -1,
    }
    const state: MockSessionState = {
      view,
      input,
      cardIds: selected.map((card) => card.id),
      testOrder: [],
      currentIndex: 0,
      completedMeanings: [],
      answers: [],
      setFailed: false,
    }
    this.sessions.set(id, state)
    this.sessionPreferences.set(`${input.profileId}:${input.mode}`, {
      direction: input.direction,
      minScore: input.minScore,
      maxScore: input.maxScore,
      cardsPerSet: input.mode === 'learning' ? input.cardsPerSet : null,
      pronunciationCheckEnabled,
      pronunciationScoreThreshold: input.pronunciationScoreThreshold,
    })
    return this.refreshView(state)
  }

  async getStudySessionPreferences(
    _username: string,
    profileId: string,
    mode: StudySessionMode,
  ): Promise<StudySessionPreferences> {
    return structuredClone(
      this.sessionPreferences.get(`${profileId}:${mode}`) ?? {
        direction: null,
        minScore: null,
        maxScore: null,
        cardsPerSet: mode === 'learning' ? 5 : null,
        pronunciationCheckEnabled: false,
        pronunciationScoreThreshold: 75,
      },
    )
  }

  async applyStudySessionAction(
    input: ApplyStudySessionActionInput,
  ): Promise<StudySessionTransition> {
    const state = this.sessions.get(input.sessionId)
    if (!state) throw new Error('Study session was not found.')
    if (state.view.version !== input.expectedVersion) {
      throw new Error('Study session was changed concurrently.')
    }
    let answerFeedback: StudySessionTransition['answerFeedback'] = null
    let setOutcome: StudySessionTransition['setOutcome'] = null
    if (input.action === 'previousStudyCard') {
      state.currentIndex = Math.max(0, state.currentIndex - 1)
    } else if (input.action === 'nextStudyCard') {
      const start = (state.view.currentSet - 1) * (state.input.cardsPerSet ?? 5)
      const length = Math.min(
        state.input.cardsPerSet ?? 5,
        state.cardIds.length - start,
      )
      state.currentIndex = Math.min(length - 1, state.currentIndex + 1)
    } else if (input.action === 'startMiniTest') {
      const start = (state.view.currentSet - 1) * (state.input.cardsPerSet ?? 5)
      state.testOrder = state.cardIds
        .slice(start, start + (state.input.cardsPerSet ?? 5))
        .sort(() => Math.random() - 0.5)
      state.currentIndex = 0
      state.view.phase = 'test'
      state.setFailed = false
    } else if (input.action === 'submitWrittenAnswer') {
      const card = this.sessionCard(state)
      const answer = input.answer?.trim() ?? ''
      if (!card || !answer) throw new Error('Enter an answer.')
      let matchedIndex = -1
      let matched: string | null = null
      let matchedScore = -1
      card.meanings.forEach((meaning, index) => {
        if (!state.completedMeanings.includes(index)) {
          meaning.wordTranslations.forEach((translation) => {
            const score = answerSimilarity(translation, answer)
            if (score >= 0.8 && score > matchedScore) {
              matchedScore = score
              matchedIndex = index
              matched = translation
            }
          })
        }
      })
      state.answers.push(input.answer ?? '')
      if (matchedIndex >= 0) state.completedMeanings.push(matchedIndex)
      const isCorrect = matchedIndex >= 0
      const completed =
        !isCorrect || state.completedMeanings.length === card.meanings.length
      const delta =
        completed && state.view.mode === 'test' ? (isCorrect ? 1 : -2) : 0
      if (completed) {
        state.view.awaitingContinue = true
        state.setFailed ||= !isCorrect
        state.view.summary[isCorrect ? 'correct' : 'incorrect'] += 1
        state.view.summary.scoreDelta += delta
        card.score += delta
        card.version += delta === 0 ? 0 : 1
      }
      answerFeedback = {
        isCorrect,
        matchedAnswer: matched,
        card: structuredClone(card),
        matchedMeaningIndex: matchedIndex >= 0 ? matchedIndex : null,
        completedMeaningIndices: [...state.completedMeanings],
        cardCompleted: completed,
        remainingMeanings:
          card.meanings.length - state.completedMeanings.length,
        scoreDelta: delta,
      }
    } else if (input.action === 'continueAfterFeedback') {
      state.view.awaitingContinue = false
      state.completedMeanings = []
      state.answers = []
      if (state.view.mode === 'test') {
        const eligible = this.matchingCards(state.input).sort(
          () => Math.random() - 0.5,
        )
        if (eligible.length === 0) {
          state.view.status = 'completed'
          state.cardIds = []
        } else {
          const history = this.testHistory.get(state.input.profileId) ?? []
          const banned = new Set(
            history.slice(-Math.floor(eligible.length / 2)),
          )
          const next =
            eligible.find((card) => !banned.has(card.id)) ?? eligible[0]
          state.cardIds = [next.id]
          history.push(next.id)
          this.testHistory.set(state.input.profileId, history)
        }
      } else {
        state.currentIndex += 1
        if (state.currentIndex >= state.testOrder.length) {
          if (state.setFailed) {
            setOutcome = 'retry'
            state.view.phase = 'study'
            state.currentIndex = 0
            state.setFailed = false
          } else {
            setOutcome = 'passed'
            state.view.currentSet += 1
            state.currentIndex = 0
            if (state.view.currentSet > state.view.totalSets) {
              state.view.status = 'completed'
            } else {
              state.view.phase = 'study'
            }
          }
        }
      }
    }
    return {
      session: this.refreshView(state),
      answerFeedback,
      pronunciationFeedback: null,
      setOutcome,
    }
  }

  async assessPronunciation(
    _input: AssessPronunciationInput,
  ): Promise<StudySessionTransition> {
    throw new Error(
      'Pronunciation assessment is available only in the desktop application.',
    )
  }

  async finishStudySession(input: EndStudySessionInput): Promise<StudySession> {
    const state = this.sessions.get(input.sessionId)
    if (!state) throw new Error('Study session was not found.')
    state.view.status = 'completed'
    return this.refreshView(state)
  }

  async cancelStudySession(input: EndStudySessionInput): Promise<StudySession> {
    const state = this.sessions.get(input.sessionId)
    if (!state) throw new Error('Study session was not found.')
    state.view.status = 'cancelled'
    return this.refreshView(state)
  }
}
