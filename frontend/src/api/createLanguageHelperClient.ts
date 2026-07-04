import type { LanguageHelperClient } from './language-helper-client'
import { MockLanguageHelperClient } from './mock-language-helper-client'
import { TauriLanguageHelperClient } from './tauri-language-helper-client'

function isRunningInTauri(): boolean {
  return '__TAURI_INTERNALS__' in window
}

export function createLanguageHelperClient(): LanguageHelperClient {
  return isRunningInTauri()
    ? new TauriLanguageHelperClient()
    : new MockLanguageHelperClient()
}
