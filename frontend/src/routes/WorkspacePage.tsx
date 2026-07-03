import {
  Alert,
  Button,
  Group,
  Paper,
  PasswordInput,
  Select,
  Stack,
  Text,
  TextInput,
  Title,
} from '@mantine/core'
import { useMutation, useQuery } from '@tanstack/react-query'
import { useEffect, useState } from 'react'
import { Navigate, useLocation, useNavigate } from 'react-router'

import type {
  AiSettings,
  LanguageProfile,
  StudySessionMode,
} from '../api/language-helper-client'
import { useLanguageHelperClient } from '../api/LanguageHelperClientProvider'
import { useTranslations } from '../locales/TranslationProvider'
import { CardsPage } from './CardsPage'
import { SessionPage } from './SessionPage'
import classes from './WorkspacePage.module.css'

interface WorkspaceLocationState {
  username: string
  profile: LanguageProfile
}

const DEFAULT_SETTINGS: AiSettings = {
  version: 0,
  provider: null,
  apiKey: null,
  modelName: null,
}

type View = 'menu' | 'settings' | 'cards' | StudySessionMode

export function WorkspacePage() {
  const client = useLanguageHelperClient()
  const { t } = useTranslations()
  const navigate = useNavigate()
  const location = useLocation()
  const context = location.state as WorkspaceLocationState | null
  const [view, setView] = useState<View>('menu')
  const [settings, setSettings] = useState(DEFAULT_SETTINGS)
  const [saved, setSaved] = useState(false)

  const loadedSettings = useQuery({
    queryKey: ['ai-settings', context?.username, context?.profile.id],
    queryFn: () => client.getAiSettings(context!.username, context!.profile.id),
    enabled: Boolean(context?.username && context?.profile.id),
    retry: false,
  })

  const save = useMutation({
    mutationFn: () =>
      client.saveAiSettings({
        username: context!.username,
        profileId: context!.profile.id,
        ...settings,
        apiKey: settings.apiKey?.trim() || null,
        modelName: settings.modelName?.trim() || null,
      }),
    onSuccess: (result) => {
      setSettings(result)
      setSaved(true)
    },
  })

  useEffect(() => {
    if (loadedSettings.data) setSettings(loadedSettings.data)
  }, [loadedSettings.data])

  if (!context?.username || !context.profile) {
    return <Navigate replace to="/" />
  }

  const profileCaption = `${context.profile.name} · ${context.profile.sourceLanguage} → ${context.profile.targetLanguage}`
  const closeSettings = () => {
    setSaved(false)
    save.reset()
    setView('menu')
  }

  return (
    <main className={classes.page}>
      <Stack className={classes.content} gap="xl">
        <Stack align="center" gap={4}>
          <Title order={2}>{context.profile.name}</Title>
          <Text c="dimmed" size="sm">
            {t('workspace.profileFor')} {context.username} · {profileCaption}
          </Text>
        </Stack>

        {view === 'menu' && (
          <Stack className={classes.menu} gap="sm">
            <Button
              size="md"
              variant="default"
              onClick={() => setView('cards')}
            >
              {t('workspace.cards')}
            </Button>
            <Button
              size="md"
              variant="default"
              onClick={() => setView('learning')}
            >
              {t('workspace.learn')}
            </Button>
            <Button
              size="md"
              variant="default"
              onClick={() => setView('test')}
            >
              {t('workspace.test')}
            </Button>
            <Button size="md" onClick={() => setView('settings')}>
              {t('workspace.settings')}
            </Button>
            <Button
              color="gray"
              size="md"
              variant="subtle"
              onClick={() => void navigate('/')}
            >
              {t('workspace.back')}
            </Button>
          </Stack>
        )}

        {view === 'cards' && (
          <CardsPage
            profileId={context.profile.id}
            username={context.username}
            onBack={() => setView('menu')}
          />
        )}

        {(view === 'learning' || view === 'test') && (
          <SessionPage
            mode={view}
            profileId={context.profile.id}
            username={context.username}
            onBack={() => setView('menu')}
          />
        )}

        {view === 'settings' && (
          <Stack className={classes.menu} gap="md">
            <Paper className={classes.settingsBlock} p="xl" withBorder>
              <Stack>
                <Title order={3}>{t('workspace.aiSettings')}</Title>
                <Select
                  allowDeselect={false}
                  data={[
                    { value: 'none', label: t('workspace.notConfigured') },
                    { value: 'openai', label: t('workspace.openAi') },
                    { value: 'gemini', label: t('workspace.gemini') },
                  ]}
                  label={t('workspace.provider')}
                  value={settings.provider ?? 'none'}
                  onChange={(value) => {
                    setSaved(false)
                    setSettings((current) => ({
                      ...current,
                      provider:
                        value === 'openai' || value === 'gemini' ? value : null,
                    }))
                  }}
                />
                <PasswordInput
                  description={t('workspace.apiKeyHint')}
                  label={t('workspace.apiKey')}
                  value={settings.apiKey ?? ''}
                  onChange={(event) => {
                    const apiKey = event.currentTarget.value
                    setSaved(false)
                    setSettings((current) => ({
                      ...current,
                      apiKey,
                    }))
                  }}
                />
                <TextInput
                  label={t('workspace.modelName')}
                  value={settings.modelName ?? ''}
                  onChange={(event) => {
                    const modelName = event.currentTarget.value
                    setSaved(false)
                    setSettings((current) => ({
                      ...current,
                      modelName,
                    }))
                  }}
                />
              </Stack>
            </Paper>
            {loadedSettings.isError && (
              <Alert color="red" title={t('workspace.loadSettingsError')}>
                {loadedSettings.error.message}
              </Alert>
            )}
            {save.isError && (
              <Alert color="red" title={t('workspace.saveSettingsError')}>
                {save.error.message}
              </Alert>
            )}
            {saved && <Alert color="green">{t('workspace.saved')}</Alert>}
            <Group justify="center">
              <Button
                loading={save.isPending}
                w={140}
                onClick={() => save.mutate()}
              >
                {t('workspace.save')}
              </Button>
              <Button variant="default" w={140} onClick={closeSettings}>
                {t('workspace.back')}
              </Button>
            </Group>
          </Stack>
        )}
      </Stack>
    </main>
  )
}
