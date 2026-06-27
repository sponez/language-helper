import {
  createContext,
  useContext,
  type PropsWithChildren,
} from 'react'

import type { LanguageHelperClient } from './language-helper-client'

const LanguageHelperClientContext =
  createContext<LanguageHelperClient | null>(null)

interface LanguageHelperClientProviderProps extends PropsWithChildren {
  client: LanguageHelperClient
}

export function LanguageHelperClientProvider({
  children,
  client,
}: LanguageHelperClientProviderProps) {
  return (
    <LanguageHelperClientContext value={client}>
      {children}
    </LanguageHelperClientContext>
  )
}

// oxlint-disable-next-line react/only-export-components
export function useLanguageHelperClient(): LanguageHelperClient {
  const client = useContext(LanguageHelperClientContext)

  if (!client) {
    throw new Error(
      'useLanguageHelperClient must be used inside LanguageHelperClientProvider',
    )
  }

  return client
}
