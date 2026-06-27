import {
  Alert,
  AppShell,
  Badge,
  Center,
  Container,
  Group,
  Loader,
  Paper,
  Stack,
  Text,
  Title,
} from '@mantine/core'
import { useQuery } from '@tanstack/react-query'

import { useLanguageHelperClient } from '../api/LanguageHelperClientProvider'
import { useTranslations } from '../locales/TranslationProvider'

export function HomePage() {
  const client = useLanguageHelperClient()
  const { t } = useTranslations()
  const status = useQuery({
    queryKey: ['backend-status'],
    queryFn: () => client.getBackendStatus(),
    retry: false,
  })

  return (
    <AppShell header={{ height: 64 }} padding="xl">
      <AppShell.Header>
        <Group h="100%" px="xl" justify="space-between">
          <Text fw={700}>{t('app.name')}</Text>
          <Badge variant="light">English</Badge>
        </Group>
      </AppShell.Header>

      <AppShell.Main>
        <Container size="sm" py="xl">
          <Stack gap="xl">
            <div>
              <Title order={1}>{t('app.name')}</Title>
              <Text c="dimmed" mt="xs">
                {t('app.subtitle')}
              </Text>
            </div>

            <Paper withBorder p="xl" radius="lg">
              {status.isPending && (
                <Center>
                  <Group>
                    <Loader size="sm" />
                    <Text>{t('backend.loading')}</Text>
                  </Group>
                </Center>
              )}

              {status.isError && (
                <Alert color="red" title={t('backend.unavailable')}>
                  {status.error.message}
                </Alert>
              )}

              {status.data && (
                <Group justify="space-between">
                  <div>
                    <Text fw={600}>{t('backend.connected')}</Text>
                    <Text c="dimmed" size="sm">
                      {status.data.message}
                    </Text>
                  </div>
                  <Badge color={status.data.ready ? 'green' : 'yellow'}>
                    {status.data.transport}
                  </Badge>
                </Group>
              )}
            </Paper>
          </Stack>
        </Container>
      </AppShell.Main>
    </AppShell>
  )
}
