import type {
  BackendStatus,
  Card,
  CardPage,
  CreateCardsInput,
  CreateLanguageProfileInput,
  DeleteCardsInput,
  LanguageProfile,
  LanguageHelperClient,
  ProfileSettings,
  SaveProfileSettingsInput,
  ListCardsInput,
  UpdateCardInput,
} from './language-helper-client'

export class MockLanguageHelperClient implements LanguageHelperClient {
  private readonly usernames: string[] = []
  private readonly profiles = new Map<string, LanguageProfile[]>()
  private readonly settings = new Map<string, ProfileSettings>()
  private readonly cards = new Map<string, Card[]>()

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
      cardsPerSet: 10,
      answerMode: 'written',
      masteryThreshold: 5,
      checkReadingIfPossible: false,
      provider: null,
      apiKey: null,
      modelName: null,
    })
    this.cards.set(profile.id, [])
    return profile
  }

  async getProfileSettings(
    _username: string,
    profileId: string,
  ): Promise<ProfileSettings> {
    const settings = this.settings.get(profileId)
    if (!settings) {
      throw new Error('Profile settings were not found.')
    }
    return { ...settings }
  }

  async saveProfileSettings(
    input: SaveProfileSettingsInput,
  ): Promise<ProfileSettings> {
    const current = this.settings.get(input.profileId)
    if (!current) {
      throw new Error('Profile settings were not found.')
    }
    if (current.version !== input.version) {
      throw new Error('Profile settings were changed concurrently.')
    }

    const saved: ProfileSettings = {
      version: current.version + 1,
      cardsPerSet: input.cardsPerSet,
      answerMode: input.answerMode,
      masteryThreshold: input.masteryThreshold,
      checkReadingIfPossible: input.checkReadingIfPossible,
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
      .filter((card) => {
        if (input.mastery === 'learned') {
          return card.streak >= input.masteryThreshold
        }
        if (input.mastery === 'unlearned') {
          return card.streak < input.masteryThreshold
        }
        return true
      })
      .filter(
        (card) =>
          input.maxStreak === null || card.streak <= input.maxStreak,
      )
      .sort((left, right) => {
        const compared =
          input.sortField === 'word'
            ? left.word.localeCompare(right.word)
            : input.sortField === 'streak'
              ? left.streak - right.streak
              : left.createdAt - right.createdAt
        const stable = compared || left.id.localeCompare(right.id)
        return input.sortDirection === 'ascending' ? stable : -stable
      })
    const offset = Number(input.cursor ?? 0)
    const items = sorted.slice(offset, offset + input.limit)
    const nextOffset = offset + items.length
    return {
      items: items.map(({ id, word, direction, streak, createdAt }) => ({
        id,
        word,
        direction,
        streak,
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

  async createCards(input: CreateCardsInput): Promise<Card[]> {
    const cards = this.cards.get(input.profileId)
    if (!cards) {
      throw new Error('Language profile was not found.')
    }
    const created = input.cards.map((card) => {
      if (
        cards.some(
          (existing) =>
            existing.direction === card.direction &&
            existing.word === card.word.trim(),
        )
      ) {
        throw new Error('A card with this word and direction already exists.')
      }
      return {
        ...structuredClone(card),
        id: crypto.randomUUID(),
        profileId: input.profileId,
        word: card.word.trim(),
        streak: 0,
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
    cards[index] = {
      ...cards[index],
      word: input.word,
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
}
