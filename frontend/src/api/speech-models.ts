import type { AiSettings } from './language-helper-client'

export function speechModelName(provider: AiSettings['provider']) {
  if (provider === 'openai') return 'gpt-4o-mini-tts'
  if (provider === 'gemini') return 'gemini-3.1-flash-tts-preview'
  return null
}
