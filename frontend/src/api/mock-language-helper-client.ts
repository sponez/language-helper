import type {
  BackendStatus,
  CreateLanguageProfileInput,
  LanguageProfile,
  LanguageHelperClient,
  ProfileSettings,
  SaveProfileSettingsInput,
} from './language-helper-client'

export class MockLanguageHelperClient implements LanguageHelperClient {
  private readonly usernames: string[] = []
  private readonly profiles = new Map<string, LanguageProfile[]>()
  private readonly settings = new Map<string, ProfileSettings>()

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
}
