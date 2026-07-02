import {
  Alert,
  Button,
  Group,
  NumberInput,
  Paper,
  PasswordInput,
  Checkbox,
  Select,
  SimpleGrid,
  Stack,
  Text,
  TextInput,
  Title,
} from '@mantine/core'
import { useMutation, useQuery } from '@tanstack/react-query'
import { useEffect, useState } from 'react'
import { Navigate, useLocation, useNavigate } from 'react-router'

import type { LanguageProfile } from '../api/language-helper-client'
import { useLanguageHelperClient } from '../api/LanguageHelperClientProvider'
import { useTranslations } from '../locales/TranslationProvider'
import classes from './WorkspacePage.module.css'

interface WorkspaceLocationState {
  username: string
  profile: LanguageProfile
}

interface ProfileSettings {
  version: number
  cardsPerSet: number
  answerMode: 'written' | 'self-review'
  masteryThreshold: number
  checkReadingIfPossible: boolean
  provider: 'openai' | 'gemini' | null
  apiKey: string
  modelName: string
}

const DEFAULT_SETTINGS: ProfileSettings = {
  version: 0,
  cardsPerSet: 10,
  answerMode: 'written',
  masteryThreshold: 5,
  checkReadingIfPossible: false,
  provider: null,
  apiKey: '',
  modelName: '',
}

export function WorkspacePage() {
  const client = useLanguageHelperClient()
  const { t } = useTranslations()
  const navigate = useNavigate()
  const location = useLocation()
  const context = location.state as WorkspaceLocationState | null
  const [view, setView] = useState<'menu' | 'settings'>('menu')
  const [placeholder, setPlaceholder] = useState<string | null>(null)
  const [settings, setSettings] = useState(DEFAULT_SETTINGS)
  const [saved, setSaved] = useState(false)

  const loadedSettings = useQuery({
    queryKey: ['profile-settings', context?.username, context?.profile.id],
    queryFn: () =>
      client.getProfileSettings(context!.username, context!.profile.id),
    enabled: Boolean(context?.username && context?.profile.id),
    retry: false,
  })

  const save = useMutation({
    mutationFn: () =>
      client.saveProfileSettings({
        username: context!.username,
        profileId: context!.profile.id,
        ...settings,
        provider: settings.provider,
        apiKey: settings.apiKey || null,
        modelName: settings.modelName || null,
      }),
    onSuccess: (result) => {
      setSettings({
        ...result,
        apiKey: result.apiKey ?? '',
        modelName: result.modelName ?? '',
      })
      setSaved(true)
    },
  })

  useEffect(() => {
    if (loadedSettings.data) {
      setSettings({
        ...loadedSettings.data,
        apiKey: loadedSettings.data.apiKey ?? '',
        modelName: loadedSettings.data.modelName ?? '',
      })
    }
  }, [loadedSettings.data])

  if (!context?.username || !context.profile) {
    return <Navigate replace to="/" />
  }

  const profileCaption = `${context.profile.name} · ${context.profile.sourceLanguage} → ${context.profile.targetLanguage}`

  function saveSettings() {
    save.mutate()
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

        {view === 'menu' ? (
          <Stack className={classes.menu} gap="sm">
            {(['cards', 'learn', 'test'] as const).map((action) => (
              <Button
                key={action}
                size="md"
                variant="default"
                onClick={() => setPlaceholder(t(`workspace.${action}`))}
              >
                {t(`workspace.${action}`)}
              </Button>
            ))}
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
            {placeholder && (
              <Alert
                color="indigo"
                title={placeholder}
                onClose={() => setPlaceholder(null)}
                withCloseButton
              >
                {t('workspace.comingSoon')}
              </Alert>
            )}
          </Stack>
        ) : (
          <Stack gap="md">
            <SimpleGrid className={classes.settingsGrid} cols={{ base: 1, md: 2 }}>
              <Paper className={classes.settingsBlock} p="xl" withBorder>
                <Stack>
                  <Title order={3}>{t('workspace.testSettings')}</Title>
                  <NumberInput
                    label={t('workspace.cardsPerSet')}
                    min={1}
                    max={100}
                    value={settings.cardsPerSet}
                    onChange={(value) => {
                      setSaved(false)
                      setSettings((current) => ({
                        ...current,
                        cardsPerSet: Number(value) || 1,
                      }))
                    }}
                  />
                  <Select
                    allowDeselect={false}
                    data={[
                      { value: 'written', label: t('workspace.writtenAnswer') },
                      { value: 'self-review', label: t('workspace.selfReview') },
                    ]}
                    label={t('workspace.answerMode')}
                    value={settings.answerMode}
                    onChange={(value) => {
                      setSaved(false)
                      setSettings((current) => ({
                        ...current,
                        answerMode: (value ?? 'written') as ProfileSettings['answerMode'],
                      }))
                    }}
                  />
                  <NumberInput
                    label={t('workspace.masteryThreshold')}
                    min={1}
                    max={50}
                    value={settings.masteryThreshold}
                    onChange={(value) => {
                      setSaved(false)
                      setSettings((current) => ({
                        ...current,
                        masteryThreshold: Number(value) || 1,
                      }))
                    }}
                  />
                  <Checkbox
                    checked={settings.checkReadingIfPossible}
                    label={t('workspace.checkReadingIfPossible')}
                    onChange={(event) => {
                      const checked = event.currentTarget.checked
                      setSaved(false)
                      setSettings((current) => ({
                        ...current,
                        checkReadingIfPossible: checked,
                      }))
                    }}
                  />
                </Stack>
              </Paper>

              <Paper className={classes.settingsBlock} p="xl" withBorder>
                <Stack>
                  <Title order={3}>{t('workspace.aiSettings')}</Title>
                  <Select
                    allowDeselect={false}
                    data={[
                      {
                        value: 'none',
                        label: t('workspace.notConfigured'),
                      },
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
                          value === 'none'
                            ? null
                            : (value as ProfileSettings['provider']),
                      }))
                    }}
                  />
                  <PasswordInput
                    description={t('workspace.apiKeyHint')}
                    label={t('workspace.apiKey')}
                    value={settings.apiKey}
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
                    value={settings.modelName}
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
            </SimpleGrid>

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
              <Button loading={save.isPending} w={140} onClick={saveSettings}>
                {t('workspace.save')}
              </Button>
              <Button
                variant="default"
                w={140}
                onClick={() => {
                  setSaved(false)
                  setView('menu')
                }}
              >
                {t('workspace.back')}
              </Button>
            </Group>
          </Stack>
        )}
      </Stack>
    </main>
  )
}
