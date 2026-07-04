import {
  ActionIcon,
  Alert,
  Button,
  Group,
  Loader,
  Stack,
  Text,
  Tooltip,
} from '@mantine/core'
import { useMutation, useQuery } from '@tanstack/react-query'
import { useEffect, useRef, useState } from 'react'

import { useLanguageHelperClient } from '../api/LanguageHelperClientProvider'
import {
  matchesPrimaryShortcut,
  primaryAriaShortcut,
  primaryModifierLabel,
} from '../keyboard/shortcuts'
import { useTranslations } from '../locales/TranslationProvider'

interface CardSpeechControlsProps {
  username: string
  profileId: string
  cardId: string
  hotkeysEnabled?: boolean
}

export function CardSpeechControls({
  username,
  profileId,
  cardId,
  hotkeysEnabled = true,
}: CardSpeechControlsProps) {
  const client = useLanguageHelperClient()
  const { t } = useTranslations()
  const audioRef = useRef<HTMLAudioElement | null>(null)
  const objectUrlRef = useRef<string | null>(null)
  const [playbackError, setPlaybackError] = useState<string | null>(null)
  const settings = useQuery({
    queryKey: ['ai-settings', username],
    queryFn: () => client.getAiSettings(username),
    retry: false,
  })
  const backend = useQuery({
    queryKey: ['backend-status'],
    queryFn: () => client.getBackendStatus(),
    staleTime: Number.POSITIVE_INFINITY,
    retry: false,
  })

  const releaseAudio = () => {
    audioRef.current?.pause()
    audioRef.current = null
    if (objectUrlRef.current) URL.revokeObjectURL(objectUrlRef.current)
    objectUrlRef.current = null
  }

  useEffect(() => releaseAudio, [])

  const speech = useMutation({
    mutationFn: (regenerate: boolean) =>
      client.getCardSpeech({ username, profileId, cardId, regenerate }),
    onSuccess: async (blob) => {
      releaseAudio()
      setPlaybackError(null)
      const objectUrl = URL.createObjectURL(blob)
      objectUrlRef.current = objectUrl
      const audio = new Audio(objectUrl)
      audioRef.current = audio
      audio.addEventListener('ended', releaseAudio, { once: true })
      audio.addEventListener(
        'error',
        () => {
          setPlaybackError(t('cards.speechPlaybackError'))
          releaseAudio()
        },
        { once: true },
      )
      try {
        await audio.play()
      } catch (error) {
        setPlaybackError(
          error instanceof Error
            ? error.message
            : t('cards.speechPlaybackError'),
        )
        releaseAudio()
      }
    },
  })

  const configured = Boolean(
    settings.data?.provider && settings.data.apiKey?.trim(),
  )
  const isDesktop = backend.data?.transport === 'tauri'
  const disabled =
    settings.isPending ||
    backend.isPending ||
    !configured ||
    !isDesktop ||
    speech.isPending
  const disabledHint =
    settings.isPending || backend.isPending
      ? t('cards.speechSettingsLoading')
      : speech.isPending
        ? t('cards.speechGenerating')
        : !isDesktop
          ? t('cards.speechDesktopOnly')
          : t('cards.speechNotConfigured')
  const playShortcut = `${primaryModifierLabel()}+P`

  function playSpeech() {
    if (!disabled) speech.mutate(false)
  }

  useEffect(() => {
    const handleKeyDown = (event: KeyboardEvent) => {
      if (!hotkeysEnabled || !matchesPrimaryShortcut(event, 'KeyP')) {
        return
      }

      event.preventDefault()
      event.stopPropagation()
      if (!event.repeat) playSpeech()
    }

    window.addEventListener('keydown', handleKeyDown)
    return () => window.removeEventListener('keydown', handleKeyDown)
  })

  return (
    <Stack gap={4}>
      <Group gap="xs">
        <Tooltip
          label={
            disabled
              ? disabledHint
              : `${t('cards.playSpeech')} (${playShortcut})`
          }
          multiline
          w={280}
        >
          <span>
            <ActionIcon
              aria-label={t('cards.playSpeech')}
              aria-keyshortcuts={primaryAriaShortcut('P')}
              disabled={disabled}
              size="lg"
              variant="light"
              onClick={playSpeech}
            >
              {speech.isPending ? <Loader size="xs" /> : '🔊'}
            </ActionIcon>
          </span>
        </Tooltip>
        <Tooltip
          label={disabled ? disabledHint : t('cards.regenerateSpeech')}
          multiline
          w={280}
        >
          <span>
            <Button
              aria-label={t('cards.regenerateSpeech')}
              disabled={disabled}
              size="xs"
              variant="subtle"
              onClick={() => speech.mutate(true)}
            >
              ↻ {t('cards.regenerateSpeech')}
            </Button>
          </span>
        </Tooltip>
        <Text c="dimmed" size="xs">
          {t('cards.aiGeneratedSpeech')}
        </Text>
      </Group>
      {(speech.isError || playbackError) && (
        <Alert color="red" p="xs">
          {playbackError ??
            speech.error?.message ??
            t('cards.speechPlaybackError')}
        </Alert>
      )}
    </Stack>
  )
}
