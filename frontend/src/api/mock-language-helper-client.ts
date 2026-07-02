import type {
  BackendStatus,
  LanguageHelperClient,
} from './language-helper-client'

export class MockLanguageHelperClient implements LanguageHelperClient {
  private readonly usernames: string[] = []

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
    return username
  }
}
