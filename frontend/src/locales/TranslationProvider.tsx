import {
  createContext,
  useContext,
  type PropsWithChildren,
} from 'react'

import { en } from './en'

interface Messages {
  [key: string]: string | Messages
}

interface TranslationContextValue {
  t(key: string): string
}

const TranslationContext = createContext<TranslationContextValue | null>(null)

function translate(key: string): string {
  let value: string | Messages = en

  for (const segment of key.split('.')) {
    if (typeof value === 'string' || !(segment in value)) {
      return key
    }

    value = value[segment]
  }

  return typeof value === 'string' ? value : key
}

export function TranslationProvider({ children }: PropsWithChildren) {
  return (
    <TranslationContext value={{ t: translate }}>
      {children}
    </TranslationContext>
  )
}

// oxlint-disable-next-line react/only-export-components
export function useTranslations(): TranslationContextValue {
  const translations = useContext(TranslationContext)

  if (!translations) {
    throw new Error('useTranslations must be used inside TranslationProvider')
  }

  return translations
}
