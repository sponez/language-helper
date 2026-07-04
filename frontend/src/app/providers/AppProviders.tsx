import { MantineProvider, createTheme } from '@mantine/core'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { useState, type PropsWithChildren } from 'react'

import { LanguageHelperClientProvider } from '../../api/LanguageHelperClientProvider'
import { createLanguageHelperClient } from '../../api/createLanguageHelperClient'
import { TranslationProvider } from '../../locales/TranslationProvider'

const theme = createTheme({
  primaryColor: 'indigo',
  defaultRadius: 'md',
})

export function AppProviders({ children }: PropsWithChildren) {
  const [queryClient] = useState(() => new QueryClient())
  const [client] = useState(createLanguageHelperClient)

  return (
    <MantineProvider defaultColorScheme="dark" theme={theme}>
      <QueryClientProvider client={queryClient}>
        <LanguageHelperClientProvider client={client}>
          <TranslationProvider>{children}</TranslationProvider>
        </LanguageHelperClientProvider>
      </QueryClientProvider>
    </MantineProvider>
  )
}
