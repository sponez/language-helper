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

export class MockLanguageHelperClient implements LanguageHelperClient {
  private readonly usernames: string[] = []
  private readonly profiles = new Map<string, LanguageProfile[]>()
  private readonly settings = new Map<string, AiSettings>()
  private readonly cards = new Map<string, Card[]>()
  private readonly sessions = new Map<string, MockSessionState>()
  private readonly testHistory = new Map<string, string[]>()

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
    return username
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
    this.settings.set(profile.id, {
      version: 0,
      provider: null,
      apiKey: null,
      modelName: null,
    })
    this.cards.set(profile.id, [])
    return profile
  }

  async getAiSettings(
    _username: string,
    profileId: string,
  ): Promise<AiSettings> {
    const settings = this.settings.get(profileId)
    if (!settings) {
      throw new Error('Profile settings were not found.')
    }
    return { ...settings }
  }

  async saveAiSettings(input: SaveAiSettingsInput): Promise<AiSettings> {
    const current = this.settings.get(input.profileId)
    if (!current) {
      throw new Error('Profile settings were not found.')
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
    this.settings.set(input.profileId, saved)
    return { ...saved }
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
    const settings = this.settings.get(input.profileId)
    if (!settings?.provider || !settings.apiKey || !settings.modelName) {
      throw new Error('AI provider is not configured. Open Settings to configure it.')
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
        if (
          index < 0 ||
          next[index].version !== pending.expectedVersion
        ) {
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
    state.view.version += 1
    return structuredClone(state.view)
  }

  async createStudySession(
    input: CreateStudySessionInput,
  ): Promise<StudySession> {
    const eligible = this.matchingCards(input).sort(() => Math.random() - 0.5)
    if (eligible.length === 0) throw new Error('No matching cards are available.')
    let selected = eligible
    if (input.mode === 'test') {
      const history = this.testHistory.get(input.profileId) ?? []
      const banned = new Set(history.slice(-Math.floor(eligible.length / 2)))
      selected = [
        eligible.find((card) => !banned.has(card.id)) ?? eligible[0],
      ]
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
      pronunciationCheckEnabled: input.pronunciationCheckEnabled,
      pronunciationAccuracyThreshold: input.pronunciationAccuracyThreshold,
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
    return this.refreshView(state)
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
      const start =
        (state.view.currentSet - 1) * (state.input.cardsPerSet ?? 5)
      const length = Math.min(
        state.input.cardsPerSet ?? 5,
        state.cardIds.length - start,
      )
      state.currentIndex = Math.min(length - 1, state.currentIndex + 1)
    } else if (input.action === 'startMiniTest') {
      const start =
        (state.view.currentSet - 1) * (state.input.cardsPerSet ?? 5)
      state.testOrder = state.cardIds
        .slice(start, start + (state.input.cardsPerSet ?? 5))
        .sort(() => Math.random() - 0.5)
      state.currentIndex = 0
      state.view.phase = 'test'
      state.setFailed = false
    } else if (input.action === 'submitWrittenAnswer') {
      const card = this.sessionCard(state)
      const answer = input.answer?.trim().toLocaleLowerCase() ?? ''
      if (!card || !answer) throw new Error('Enter an answer.')
      let matchedIndex = -1
      let matched: string | null = null
      card.meanings.forEach((meaning, index) => {
        if (
          matchedIndex < 0 &&
          !state.completedMeanings.includes(index)
        ) {
          const candidate = meaning.wordTranslations.find(
            (translation) => translation.trim().toLocaleLowerCase() === answer,
          )
          if (candidate) {
            matchedIndex = index
            matched = candidate
          }
        }
      })
      state.answers.push(input.answer ?? '')
      if (matchedIndex >= 0) state.completedMeanings.push(matchedIndex)
      const isCorrect = matchedIndex >= 0
      const completed =
        !isCorrect || state.completedMeanings.length === card.meanings.length
      const delta =
        completed && state.view.mode === 'test'
          ? isCorrect
            ? 1
            : -2
          : 0
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
        expectedAnswers: isCorrect
          ? []
          : card.meanings.flatMap((meaning) => meaning.wordTranslations),
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
      setOutcome,
    }
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
