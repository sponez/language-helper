import type {
  BackendStatus,
  LanguageHelperClient,
} from './language-helper-client'

export class MockLanguageHelperClient implements LanguageHelperClient {
  async getBackendStatus(): Promise<BackendStatus> {
    return {
      transport: 'mock',
      ready: true,
      message: 'Frontend development mode',
    }
  }

  async getUsernames(): Promise<string[]> {
    return []
  }
}
