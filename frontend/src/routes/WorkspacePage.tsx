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
import { speechModelName } from '../api/speech-models'
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
type MenuItem = 'cards' | 'learning' | 'test' | 'settings' | 'back'

const MENU_ITEMS: MenuItem[] = [
  'cards',
  'learning',
  'test',
  'settings',
  'back',
]

export function WorkspacePage() {
  const client = useLanguageHelperClient()
  const { t } = useTranslations()
  const navigate = useNavigate()
  const location = useLocation()
  const context = location.state as WorkspaceLocationState | null
  const [view, setView] = useState<View>('menu')
  const [menuCursor, setMenuCursor] = useState<number | null>(null)
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

  useEffect(() => {
    if (view === 'menu') setMenuCursor(null)
  }, [view])

  useEffect(() => {
    const handleKeyDown = (event: KeyboardEvent) => {
      if (view === 'menu') {
        if (event.key === 'Escape') {
          event.preventDefault()
          void navigate('/')
        } else if (event.key === 'ArrowDown' || event.key === 'ArrowUp') {
          event.preventDefault()
          setMenuCursor((current) => {
            if (current === null) {
              return event.key === 'ArrowDown' ? 0 : MENU_ITEMS.length - 1
            }
            const offset = event.key === 'ArrowDown' ? 1 : -1
            return Math.min(
              MENU_ITEMS.length - 1,
              Math.max(0, current + offset),
            )
          })
        } else if (event.key === 'Enter' && menuCursor !== null) {
          event.preventDefault()
          const item = MENU_ITEMS[menuCursor]
          if (item === 'back') void navigate('/')
          else setView(item)
        }
      } else if (view === 'settings') {
        if (event.key === 'Escape') {
          event.preventDefault()
          setSaved(false)
          save.reset()
          setView('menu')
        }
      }
    }

    window.addEventListener('keydown', handleKeyDown)
    return () => window.removeEventListener('keydown', handleKeyDown)
  }, [menuCursor, navigate, save, view])

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
              className={menuCursor === 0 ? classes.menuItemSelected : undefined}
              size="md"
              tabIndex={-1}
              variant="default"
              onClick={() => setView('cards')}
            >
              {t('workspace.cards')}
            </Button>
            <Button
              className={menuCursor === 1 ? classes.menuItemSelected : undefined}
              size="md"
              tabIndex={-1}
              variant="default"
              onClick={() => setView('learning')}
            >
              {t('workspace.learn')}
            </Button>
            <Button
              className={menuCursor === 2 ? classes.menuItemSelected : undefined}
              size="md"
              tabIndex={-1}
              variant="default"
              onClick={() => setView('test')}
            >
              {t('workspace.test')}
            </Button>
            <Button
              className={menuCursor === 3 ? classes.menuItemSelected : undefined}
              size="md"
              tabIndex={-1}
              onClick={() => setView('settings')}
            >
              {t('workspace.settings')}
            </Button>
            <Button
              className={menuCursor === 4 ? classes.menuItemSelected : undefined}
              color="gray"
              size="md"
              tabIndex={-1}
              variant="subtle"
              onClick={() => void navigate('/')}
            >
              {t('workspace.back')}
            </Button>
            <Text c="dimmed" size="xs" ta="center">
              {t('workspace.menuKeyboardHint')}
            </Text>
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
                <Text c="dimmed" size="sm">
                  {t('workspace.speechModel')}:{' '}
                  {speechModelName(settings.provider) ??
                    t('workspace.notConfigured')}
                </Text>
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
