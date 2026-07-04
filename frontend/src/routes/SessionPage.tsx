import {
  Accordion,
  Alert,
  Button,
  Checkbox,
  Group,
  Modal,
  NumberInput,
  Paper,
  Select,
  Stack,
  Text,
  TextInput,
  Title,
} from '@mantine/core'
import { useDisclosure } from '@mantine/hooks'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import {
  type FormEvent,
  useCallback,
  useEffect,
  useRef,
  useState,
} from 'react'

import type {
  CardDirection,
  StudySession,
  StudySessionAction,
  StudySessionMode,
  StudySessionTransition,
} from '../api/language-helper-client'
import { useLanguageHelperClient } from '../api/LanguageHelperClientProvider'
import { recordingToWav } from '../audio/recording'
import { CardSpeechControls } from '../components/CardSpeechControls'
import {
  ReadOnlyCard,
  ReadOnlyMeanings,
} from '../components/ReadOnlyCard'
import { useTranslations } from '../locales/TranslationProvider'
import classes from './SessionPage.module.css'

interface SessionPageProps {
  username: string
  profileId: string
  mode: StudySessionMode
  onBack(): void
}

export function SessionPage({
  username,
  profileId,
  mode,
  onBack,
}: SessionPageProps) {
  const client = useLanguageHelperClient()
  const queryClient = useQueryClient()
  const { t } = useTranslations()
  const [session, setSession] = useState<StudySession | null>(null)
  const [direction, setDirection] = useState<CardDirection | null>(null)
  const [minScore, setMinScore] = useState<number | null>(null)
  const [maxScore, setMaxScore] = useState<number | null>(null)
  const [cardsPerSet, setCardsPerSet] = useState(5)
  const [pronunciation, setPronunciation] = useState(false)
  const [pronunciationScore, setPronunciationScore] = useState(75)
  const [pronunciationFeedback, setPronunciationFeedback] =
    useState<StudySessionTransition['pronunciationFeedback']>(null)
  const [recording, setRecording] = useState(false)
  const [recordingSeconds, setRecordingSeconds] = useState(0)
  const [answer, setAnswer] = useState('')
  const [feedback, setFeedback] =
    useState<StudySessionTransition['answerFeedback']>(null)
  const [setOutcome, setSetOutcome] =
    useState<StudySessionTransition['setOutcome']>(null)
  const [endOpened, endModal] = useDisclosure(false)
  const recorderRef = useRef<MediaRecorder | null>(null)
  const streamRef = useRef<MediaStream | null>(null)
  const recordingTimerRef = useRef<number | null>(null)
  const recordingTimeoutRef = useRef<number | null>(null)
  const discardRecordingRef = useRef(false)

  const sessionPreferences = useQuery({
    queryKey: ['study-session-preferences', username, profileId, mode],
    queryFn: () =>
      client.getStudySessionPreferences(username, profileId, mode),
    retry: false,
  })

  const pronunciationSettings = useQuery({
    queryKey: ['pronunciation-settings', username],
    queryFn: () => client.getPronunciationSettings(username),
    retry: false,
  })

  useEffect(() => {
    const preferences = sessionPreferences.data
    if (!preferences) return
    setDirection(preferences.direction)
    setMinScore(preferences.minScore)
    setMaxScore(preferences.maxScore)
    setCardsPerSet(preferences.cardsPerSet ?? 5)
    setPronunciation(
      preferences.pronunciationCheckEnabled &&
        preferences.direction !== 'reverse' &&
        Boolean(pronunciationSettings.data?.configured),
    )
    setPronunciationScore(preferences.pronunciationScoreThreshold)
  }, [
    pronunciationSettings.data?.configured,
    sessionPreferences.data,
  ])

  const currentCardId = session?.currentCard?.id ?? session?.currentCard?.card?.id
  useEffect(() => {
    setPronunciationFeedback(null)
    setFeedback(null)
    setAnswer('')
  }, [currentCardId])

  const start = useMutation({
    mutationFn: () =>
      client.createStudySession({
        username,
        profileId,
        mode,
        direction,
        minScore,
        maxScore,
        cardsPerSet: mode === 'learning' ? cardsPerSet : null,
        pronunciationCheckEnabled: pronunciation,
        pronunciationScoreThreshold: pronunciationScore,
      }),
    onSuccess: setSession,
  })

  const action = useMutation({
    mutationFn: ({
      action,
      answer,
      message,
    }: {
      action: StudySessionAction
      answer?: string
      message?: string
    }) =>
      client.applyStudySessionAction({
        username,
        sessionId: session!.id,
        expectedVersion: session!.version,
        action,
        answer,
        message,
      }),
    onSuccess: async (transition, variables) => {
      setSession(transition.session)
      setFeedback(transition.answerFeedback)
      setPronunciationFeedback(transition.pronunciationFeedback)
      setSetOutcome(transition.setOutcome)
      setAnswer('')
      if (
        variables.action === 'continueAfterFeedback' ||
        variables.action === 'startMiniTest'
      ) {
        setPronunciationFeedback(null)
      }
      if (transition.answerFeedback?.scoreDelta) {
        await queryClient.invalidateQueries({
          queryKey: ['cards', username, profileId],
        })
      }
    },
  })

  const assessment = useMutation({
    mutationFn: (audio: Uint8Array) =>
      client.assessPronunciation({
        username,
        sessionId: session!.id,
        expectedVersion: session!.version,
        audio: Array.from(audio),
      }),
    onSuccess: async (transition) => {
      setSession(transition.session)
      setFeedback(transition.answerFeedback)
      setPronunciationFeedback(transition.pronunciationFeedback)
      if (transition.session.summary.scoreDelta !== session?.summary.scoreDelta) {
        await queryClient.invalidateQueries({
          queryKey: ['cards', username, profileId],
        })
      }
    },
  })

  const finish = useMutation({
    mutationFn: () =>
      client.finishStudySession({
        username,
        sessionId: session!.id,
        expectedVersion: session!.version,
      }),
    onSuccess: setSession,
  })

  const cancel = useMutation({
    mutationFn: () =>
      client.cancelStudySession({
        username,
        sessionId: session!.id,
        expectedVersion: session!.version,
      }),
    onSuccess: onBack,
  })

  const invalidRange =
    minScore !== null && maxScore !== null && minScore > maxScore

  const clearRecordingTimers = useCallback(() => {
    if (recordingTimerRef.current !== null) {
      window.clearInterval(recordingTimerRef.current)
      recordingTimerRef.current = null
    }
    if (recordingTimeoutRef.current !== null) {
      window.clearTimeout(recordingTimeoutRef.current)
      recordingTimeoutRef.current = null
    }
  }, [])

  const releaseMicrophone = useCallback(() => {
    streamRef.current?.getTracks().forEach((track) => track.stop())
    streamRef.current = null
    recorderRef.current = null
    clearRecordingTimers()
    setRecording(false)
  }, [clearRecordingTimers])

  const stopRecording = useCallback(() => {
    if (recorderRef.current?.state === 'recording') {
      recorderRef.current.stop()
    }
  }, [])

  const reportCaptureFailure = useCallback(
    (error: unknown) => {
      releaseMicrophone()
      const message =
        error instanceof Error
          ? error.message
          : t('sessions.microphoneError')
      action.mutate({
        action: 'registerPronunciationCaptureFailure',
        message,
      })
    },
    [action, releaseMicrophone, t],
  )

  const startRecording = useCallback(async () => {
    if (recording || assessment.isPending || action.isPending) return
    try {
      if (!navigator.mediaDevices?.getUserMedia || !window.MediaRecorder) {
        throw new Error(t('sessions.microphoneUnsupported'))
      }
      discardRecordingRef.current = false
      const stream = await navigator.mediaDevices.getUserMedia({
        audio: {
          channelCount: 1,
          echoCancellation: true,
          noiseSuppression: true,
          autoGainControl: true,
        },
      })
      streamRef.current = stream
      const preferredType = 'audio/webm;codecs=opus'
      const recorder = MediaRecorder.isTypeSupported(preferredType)
        ? new MediaRecorder(stream, { mimeType: preferredType })
        : new MediaRecorder(stream)
      recorderRef.current = recorder
      const chunks: Blob[] = []
      recorder.ondataavailable = (event) => {
        if (event.data.size > 0) chunks.push(event.data)
      }
      recorder.onerror = () => {
        discardRecordingRef.current = true
        reportCaptureFailure(new Error(t('sessions.microphoneError')))
      }
      recorder.onstop = async () => {
        const discarded = discardRecordingRef.current
        const mimeType = recorder.mimeType || 'audio/webm'
        releaseMicrophone()
        if (discarded) return
        try {
          const wav = await recordingToWav(
            new Blob(chunks, { type: mimeType }),
          )
          assessment.mutate(wav)
        } catch (error) {
          reportCaptureFailure(error)
        }
      }
      recorder.start()
      setRecordingSeconds(0)
      setRecording(true)
      const startedAt = Date.now()
      recordingTimerRef.current = window.setInterval(() => {
        setRecordingSeconds((Date.now() - startedAt) / 1000)
      }, 100)
      recordingTimeoutRef.current = window.setTimeout(stopRecording, 10_000)
    } catch (error) {
      reportCaptureFailure(error)
    }
  }, [
    action.isPending,
    assessment,
    recording,
    releaseMicrophone,
    reportCaptureFailure,
    stopRecording,
    t,
  ])

  useEffect(
    () => () => {
      discardRecordingRef.current = true
      if (recorderRef.current?.state === 'recording') {
        recorderRef.current.stop()
      }
      releaseMicrophone()
    },
    [releaseMicrophone],
  )

  useEffect(() => {
    if (recorderRef.current?.state === 'recording') {
      discardRecordingRef.current = true
      recorderRef.current.stop()
      releaseMicrophone()
    }
  }, [currentCardId, releaseMicrophone])

  useEffect(() => {
    const handleKeyDown = (event: KeyboardEvent) => {
      if (endOpened) {
        if (event.key === 'Escape') {
          event.preventDefault()
          event.stopPropagation()
          endModal.close()
        } else if (
          event.key === 'Enter' &&
          !finish.isPending &&
          !cancel.isPending
        ) {
          event.preventDefault()
          event.stopPropagation()
          endModal.close()
          if (mode === 'test') finish.mutate()
          else cancel.mutate()
        }
        return
      }
      if (
        event.key === 'Enter' &&
        event.target instanceof HTMLElement &&
        event.target.closest('button, a, input, textarea, select')
      ) {
        return
      }
      if (event.key === 'Escape') {
        event.preventDefault()
        if (!session || session.status === 'completed') onBack()
        else endModal.open()
        return
      }
      if (
        event.key !== 'Enter' &&
        event.key !== 'ArrowLeft' &&
        event.key !== 'ArrowRight'
      ) {
        return
      }
      if (event.key === 'Enter' && session?.status === 'completed') {
        event.preventDefault()
        onBack()
        return
      }
      if (
        !session ||
        session.status !== 'active' ||
        action.isPending ||
        assessment.isPending
      ) {
        return
      }

      const current = session.currentCard
      if (current?.kind === 'study') {
        if (event.key === 'ArrowLeft') {
          event.preventDefault()
          if (session.currentCardNumber > 1) {
            action.mutate({ action: 'previousStudyCard' })
          }
        } else if (event.key === 'ArrowRight') {
          event.preventDefault()
          action.mutate({ action: 'nextStudyCard' })
        } else {
          event.preventDefault()
          action.mutate({ action: 'startMiniTest' })
        }
        return
      }

      if (event.key !== 'Enter') return
      if (current?.kind === 'test' && pronunciationFeedback && !feedback) {
        event.preventDefault()
        if (pronunciationFeedback.kind === 'failed') {
          action.mutate({ action: 'continueAfterFeedback' })
        } else if (pronunciationFeedback.kind === 'disableRequired') {
          action.mutate({ action: 'disablePronunciation' })
        } else {
          setPronunciationFeedback(null)
        }
      } else if (
        current?.kind === 'test' &&
        session.pronunciationRequired &&
        !feedback
      ) {
        event.preventDefault()
        if (recording) stopRecording()
        else void startRecording()
      } else if (feedback) {
        event.preventDefault()
        if (feedback.cardCompleted) {
          action.mutate({ action: 'continueAfterFeedback' })
        } else {
          setFeedback(null)
        }
      }
    }

    window.addEventListener('keydown', handleKeyDown)
    return () => window.removeEventListener('keydown', handleKeyDown)
  }, [
    action,
    assessment.isPending,
    cancel,
    endModal,
    endOpened,
    feedback,
    finish,
    mode,
    onBack,
    pronunciationFeedback,
    recording,
    session,
    startRecording,
    stopRecording,
  ])

  if (!session) {
    return (
      <Stack className={classes.setup} gap="md">
        <Title order={2} ta="center">
          {mode === 'test'
            ? t('sessions.testSetup')
            : t('sessions.learnSetup')}
        </Title>
        <Select
          allowDeselect={false}
          data={[
            { value: 'any', label: t('cards.anyDirection') },
            { value: 'straight', label: t('cards.straight') },
            { value: 'reverse', label: t('cards.reverse') },
          ]}
          label={t('cards.direction')}
          value={direction ?? 'any'}
          onChange={(value) => {
            const nextDirection =
              value === 'straight' || value === 'reverse' ? value : null
            setDirection(nextDirection)
            if (nextDirection === 'reverse') setPronunciation(false)
          }}
        />
        <Group grow>
          <NumberInput
            allowDecimal={false}
            label={t('cards.minScore')}
            placeholder={t('sessions.noMinimum')}
            value={minScore ?? ''}
            onChange={(value) =>
              setMinScore(typeof value === 'number' ? value : null)
            }
          />
          <NumberInput
            allowDecimal={false}
            label={t('cards.maxScore')}
            placeholder={t('sessions.noMaximum')}
            value={maxScore ?? ''}
            onChange={(value) =>
              setMaxScore(typeof value === 'number' ? value : null)
            }
          />
        </Group>
        {mode === 'learning' && (
          <NumberInput
            allowDecimal={false}
            allowNegative={false}
            label={t('sessions.cardsPerSet')}
            max={100}
            min={1}
            value={cardsPerSet}
            onChange={(value) => setCardsPerSet(Number(value) || 5)}
          />
        )}
        <Checkbox
          checked={pronunciation}
          disabled={
            direction === 'reverse' ||
            pronunciationSettings.isLoading ||
            !pronunciationSettings.data?.configured
          }
          description={
            direction === 'reverse'
              ? t('sessions.pronunciationStraightOnly')
              : !pronunciationSettings.isLoading &&
            !pronunciationSettings.data?.configured
                ? t('sessions.pronunciationNotConfigured')
                : t('sessions.pronunciationStraightOnly')
          }
          label={t('sessions.checkPronunciation')}
          onChange={(event) =>
            setPronunciation(event.currentTarget.checked)
          }
        />
        <NumberInput
          allowDecimal={false}
          allowNegative={false}
          description={t('sessions.pronunciationScoreDescription')}
          disabled={!pronunciation || direction === 'reverse'}
          label={t('sessions.pronunciationScoreThreshold')}
          max={100}
          min={1}
          suffix="%"
          value={pronunciationScore}
          onChange={(value) => setPronunciationScore(Number(value) || 75)}
        />
        {invalidRange && (
          <Alert color="red">{t('sessions.invalidScoreRange')}</Alert>
        )}
        {start.isError && (
          <Alert color="red" title={t('sessions.startError')}>
            {start.error.message}
          </Alert>
        )}
        {sessionPreferences.isError && (
          <Alert color="red">{sessionPreferences.error.message}</Alert>
        )}
        <Group justify="center">
          <Button
            disabled={invalidRange || sessionPreferences.isPending}
            loading={start.isPending}
            w={150}
            onClick={() => start.mutate()}
          >
            {t('sessions.start')}
          </Button>
          <Button variant="default" w={150} onClick={onBack}>
            {t('cards.back')}
          </Button>
        </Group>
      </Stack>
    )
  }

  if (session.status === 'completed') {
    return (
      <Stack className={classes.page} align="center">
        <Title order={2}>{t('sessions.complete')}</Title>
        <Paper p="xl" withBorder>
          <Stack>
            <Text>
              {t('sessions.correct')}: {session.summary.correct}
            </Text>
            <Text>
              {t('sessions.incorrect')}: {session.summary.incorrect}
            </Text>
            {mode === 'test' && (
              <Text>
                {t('sessions.scoreChange')}: {session.summary.scoreDelta}
              </Text>
            )}
          </Stack>
        </Paper>
        <Button onClick={onBack}>{t('sessions.backToMenu')}</Button>
        <Text c="dimmed" size="xs">
          {t('sessions.completeKeyboardHint')}
        </Text>
      </Stack>
    )
  }

  const current = session.currentCard
  const testCard = current?.kind === 'test' ? current : null
  const showPronunciation =
    testCard &&
    (session.pronunciationRequired || pronunciationFeedback !== null) &&
    !feedback

  function submitAnswer(event: FormEvent) {
    event.preventDefault()
    if (answer.trim()) {
      action.mutate({ action: 'submitWrittenAnswer', answer })
    }
  }

  return (
    <Stack className={classes.page} gap="md">
      <Group justify="space-between">
        <div>
          <Title order={2}>
            {mode === 'test' ? t('workspace.test') : t('workspace.learn')}
          </Title>
          <Text c="dimmed" size="sm">
            {mode === 'learning'
              ? `${t('sessions.set')} ${session.currentSet}/${session.totalSets}`
              : `${t('sessions.attempt')} ${session.currentCardNumber}`}
          </Text>
        </div>
        <Button color="red" variant="subtle" onClick={endModal.open}>
          {mode === 'test' ? t('sessions.stop') : t('sessions.cancel')}
        </Button>
      </Group>

      {setOutcome && (
        <Alert color={setOutcome === 'passed' ? 'green' : 'orange'}>
          {setOutcome === 'passed'
            ? t('sessions.setPassed')
            : t('sessions.setRetry')}
        </Alert>
      )}

      {current?.kind === 'study' && current.card && (
        <>
          <ReadOnlyCard
            card={current.card}
            wordActions={
              <CardSpeechControls
                cardId={current.card.id}
                profileId={profileId}
                username={username}
              />
            }
          />
          <Group justify="center">
            <Button
              disabled={session.currentCardNumber <= 1}
              variant="default"
              onClick={() =>
                action.mutate({ action: 'previousStudyCard' })
              }
            >
              ← {t('sessions.previous')}
            </Button>
            <Button
              variant="default"
              onClick={() => action.mutate({ action: 'nextStudyCard' })}
            >
              {t('sessions.next')} →
            </Button>
            <Button onClick={() => action.mutate({ action: 'startMiniTest' })}>
              {t('sessions.startMiniTest')}
            </Button>
          </Group>
        </>
      )}

      {showPronunciation && (
        <Paper className={classes.prompt} p="xl" withBorder>
          <Stack align="center">
            <Title order={2}>{t('sessions.sayThisWord')}</Title>
            <Title order={1}>{testCard.prompt}</Title>
            <Text c="dimmed">
              {t('sessions.requiredPronunciationScore')}:{' '}
              {session.pronunciationScoreThreshold}%
            </Text>
            {pronunciationFeedback ? (
              <Stack align="center" w="100%">
                <Alert
                  color={
                    pronunciationFeedback.kind === 'passed'
                      ? 'green'
                      : pronunciationFeedback.kind === 'retry'
                        ? 'orange'
                        : 'red'
                  }
                  w="100%"
                >
                  {pronunciationFeedback.report ? (
                    <Stack gap={4}>
                      <Text fw={600}>
                        {t('sessions.pronunciationScore')}:{' '}
                        {pronunciationFeedback.report.strictScore}% /{' '}
                        {pronunciationFeedback.threshold}%
                      </Text>
                      {pronunciationFeedback.report.weakestWordScore !==
                        null && (
                        <Text size="sm">
                          {t('sessions.weakestWordScore')}:{' '}
                          {pronunciationFeedback.report.weakestWordScore}%
                        </Text>
                      )}
                      {pronunciationFeedback.report.weakestPhonemeScore !==
                        null && (
                        <Text size="sm">
                          {t('sessions.weakestPhonemeScore')}:{' '}
                          {pronunciationFeedback.report.weakestPhonemeScore}%
                        </Text>
                      )}
                      {pronunciationFeedback.report.recognizedText && (
                        <Text size="sm">
                          {t('sessions.recognizedAs')}:{' '}
                          {pronunciationFeedback.report.recognizedText}
                        </Text>
                      )}
                      {pronunciationFeedback.report.issues.map(
                        (issue, index) => (
                          <Text c="red" key={`${issue.kind}-${index}`} size="sm">
                            {issue.kind === 'phonemeSubstitution'
                              ? `${t('sessions.expectedPhoneme')} /${issue.expected}/, ${t('sessions.detectedPhoneme')} /${issue.detected}/ (${issue.word}).`
                              : `${issue.word}: ${t('sessions.azureWordError')} ${issue.errorType}.`}
                          </Text>
                        ),
                      )}
                      <Accordion mt="xs" variant="contained">
                        <Accordion.Item value="azure-details">
                          <Accordion.Control>
                            {t('sessions.azureDetails')}
                          </Accordion.Control>
                          <Accordion.Panel>
                            <Stack gap={2}>
                              {pronunciationFeedback.report
                                .pronunciationScore !== null && (
                                <Text size="xs">
                                  PronScore:{' '}
                                  {
                                    pronunciationFeedback.report
                                      .pronunciationScore
                                  }
                                  %
                                </Text>
                              )}
                              {pronunciationFeedback.report.fluencyScore !==
                                null && (
                                <Text size="xs">
                                  Fluency:{' '}
                                  {
                                    pronunciationFeedback.report
                                      .fluencyScore
                                  }
                                  %
                                </Text>
                              )}
                              {pronunciationFeedback.report
                                .completenessScore !== null && (
                                <Text size="xs">
                                  Completeness:{' '}
                                  {
                                    pronunciationFeedback.report
                                      .completenessScore
                                  }
                                  %
                                </Text>
                              )}
                              {pronunciationFeedback.report.prosodyScore !==
                                null && (
                                <Text size="xs">
                                  Prosody:{' '}
                                  {pronunciationFeedback.report.prosodyScore}%
                                </Text>
                              )}
                              <Text c="dimmed" size="xs">
                                {t('sessions.scoringVersion')}:{' '}
                                {pronunciationFeedback.report.scoringVersion}
                              </Text>
                            </Stack>
                          </Accordion.Panel>
                        </Accordion.Item>
                      </Accordion>
                      <Text size="sm">
                        {pronunciationFeedback.kind === 'passed'
                          ? t('sessions.pronunciationPassed')
                          : pronunciationFeedback.kind === 'failed'
                            ? t('sessions.pronunciationFailed')
                            : t('sessions.pronunciationRetry')}
                      </Text>
                    </Stack>
                  ) : (
                    <Text>
                      {pronunciationFeedback.message ??
                        t('sessions.pronunciationTechnicalError')}
                    </Text>
                  )}
                </Alert>
                <Button
                  loading={action.isPending}
                  onClick={() => {
                    if (pronunciationFeedback.kind === 'failed') {
                      action.mutate({ action: 'continueAfterFeedback' })
                    } else if (
                      pronunciationFeedback.kind === 'disableRequired'
                    ) {
                      action.mutate({ action: 'disablePronunciation' })
                    } else {
                      setPronunciationFeedback(null)
                    }
                  }}
                >
                  {pronunciationFeedback.kind === 'retry' ||
                  pronunciationFeedback.kind === 'technicalError'
                    ? t('sessions.tryAgain')
                    : pronunciationFeedback.kind === 'disableRequired'
                      ? t('sessions.disablePronunciation')
                      : t('sessions.continue')}
                </Button>
              </Stack>
            ) : (
              <Stack align="center">
                {recording && (
                  <Text c="red" fw={600}>
                    {t('sessions.recording')} {recordingSeconds.toFixed(1)}s
                  </Text>
                )}
                <Button
                  color={recording ? 'red' : 'blue'}
                  loading={assessment.isPending}
                  onClick={() => {
                    if (recording) stopRecording()
                    else void startRecording()
                  }}
                >
                  {recording
                    ? t('sessions.stopRecording')
                    : t('sessions.startRecording')}
                </Button>
                <Text c="dimmed" size="xs">
                  {t('sessions.recordingHint')}
                </Text>
              </Stack>
            )}
          </Stack>
        </Paper>
      )}

      {testCard && !showPronunciation && (
        <Paper className={classes.prompt} p="xl" withBorder>
          <Stack className={classes.answer} align="stretch">
            <Title order={1}>{testCard.prompt}</Title>
            {testCard.readings.length > 0 && (
              <Text c="dimmed">{testCard.readings.join(', ')}</Text>
            )}
            <Text size="sm">
              {t('sessions.meaningsRemaining')}: {testCard.remainingMeanings}/
              {testCard.totalMeanings}
            </Text>
            {feedback ? (
              <Stack>
                <Alert color={feedback.isCorrect ? 'green' : 'red'}>
                  {feedback.isCorrect
                    ? t('sessions.correct')
                    : t('sessions.incorrect')}
                </Alert>
                <Accordion variant="contained">
                  <Accordion.Item value="answer-details">
                    <Accordion.Control>
                      {t('sessions.answerDetails')}
                    </Accordion.Control>
                    <Accordion.Panel>
                      {!feedback.isCorrect ? (
                        <ReadOnlyMeanings
                          highlightedMeaningIndices={
                            feedback.completedMeaningIndices
                          }
                          meanings={feedback.card.meanings}
                        />
                      ) : feedback.cardCompleted ? (
                        <ReadOnlyCard
                          card={feedback.card}
                          highlightedMeaningIndices={feedback.card.meanings.map(
                            (_, index) => index,
                          )}
                        />
                      ) : feedback.matchedMeaningIndex !== null ? (
                        <ReadOnlyMeanings
                          highlightedMeaningIndices={[
                            feedback.matchedMeaningIndex,
                          ]}
                          meaningIndices={[feedback.matchedMeaningIndex]}
                          meanings={[
                            feedback.card.meanings[
                              feedback.matchedMeaningIndex
                            ],
                          ]}
                        />
                      ) : null}
                    </Accordion.Panel>
                  </Accordion.Item>
                </Accordion>
                <Button
                  loading={action.isPending}
                  onClick={() => {
                    if (feedback.cardCompleted) {
                      action.mutate({ action: 'continueAfterFeedback' })
                    } else {
                      setFeedback(null)
                    }
                  }}
                >
                  {t('sessions.continue')}
                </Button>
              </Stack>
            ) : (
              <form onSubmit={submitAnswer}>
                <Stack>
                  <TextInput
                    autoFocus
                    label={t('sessions.answer')}
                    value={answer}
                    onChange={(event) => setAnswer(event.currentTarget.value)}
                  />
                  <Button
                    disabled={!answer.trim()}
                    loading={action.isPending}
                    type="submit"
                  >
                    {t('sessions.submit')}
                  </Button>
                </Stack>
              </form>
            )}
          </Stack>
        </Paper>
      )}

      {action.isError && (
        <Alert color="red">{action.error.message}</Alert>
      )}
      {assessment.isError && (
        <Alert color="red">{assessment.error.message}</Alert>
      )}
      {(finish.isError || cancel.isError) && (
        <Alert color="red">
          {(finish.error ?? cancel.error)?.message}
        </Alert>
      )}

      <Modal
        centered
        opened={endOpened}
        title={t('sessions.endTitle')}
        onClose={endModal.close}
      >
        <Stack>
          <Text>{t('sessions.endDescription')}</Text>
          <Group justify="flex-end">
            <Button
              color="red"
              loading={finish.isPending || cancel.isPending}
              onClick={() => {
                endModal.close()
                if (mode === 'test') finish.mutate()
                else cancel.mutate()
              }}
            >
              {mode === 'test' ? t('sessions.stop') : t('sessions.cancel')}
            </Button>
            <Button variant="default" onClick={endModal.close}>
              {t('cards.cancel')}
            </Button>
          </Group>
        </Stack>
      </Modal>
    </Stack>
  )
}
