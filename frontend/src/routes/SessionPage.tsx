import {
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
import { useMutation, useQueryClient } from '@tanstack/react-query'
import { type FormEvent, useEffect, useState } from 'react'

import type {
  CardDirection,
  StudySession,
  StudySessionAction,
  StudySessionMode,
  StudySessionTransition,
} from '../api/language-helper-client'
import { useLanguageHelperClient } from '../api/LanguageHelperClientProvider'
import { useTranslations } from '../locales/TranslationProvider'
import { ReadOnlyCard } from './CardsPage'
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
  const [pronunciationAccuracy, setPronunciationAccuracy] = useState(75)
  const [pronunciationDone, setPronunciationDone] = useState(false)
  const [answer, setAnswer] = useState('')
  const [feedback, setFeedback] =
    useState<StudySessionTransition['answerFeedback']>(null)
  const [setOutcome, setSetOutcome] =
    useState<StudySessionTransition['setOutcome']>(null)
  const [endOpened, endModal] = useDisclosure(false)

  const currentCardId = session?.currentCard?.id ?? session?.currentCard?.card?.id
  useEffect(() => {
    setPronunciationDone(false)
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
        pronunciationAccuracyThreshold: pronunciationAccuracy,
      }),
    onSuccess: setSession,
  })

  const action = useMutation({
    mutationFn: ({
      action,
      answer,
    }: {
      action: StudySessionAction
      answer?: string
    }) =>
      client.applyStudySessionAction({
        username,
        sessionId: session!.id,
        expectedVersion: session!.version,
        action,
        answer,
      }),
    onSuccess: async (transition, variables) => {
      setSession(transition.session)
      setFeedback(transition.answerFeedback)
      setSetOutcome(transition.setOutcome)
      setAnswer('')
      if (
        variables.action === 'continueAfterFeedback' ||
        variables.action === 'startMiniTest'
      ) {
        setPronunciationDone(false)
      }
      if (transition.answerFeedback?.scoreDelta) {
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
          onChange={(value) =>
            setDirection(
              value === 'straight' || value === 'reverse' ? value : null,
            )
          }
        />
        <Group grow>
          <NumberInput
            allowDecimal={false}
            label={t('cards.minScore')}
            value={minScore ?? ''}
            onChange={(value) =>
              setMinScore(typeof value === 'number' ? value : null)
            }
          />
          <NumberInput
            allowDecimal={false}
            label={t('cards.maxScore')}
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
          label={t('sessions.checkPronunciation')}
          onChange={(event) =>
            setPronunciation(event.currentTarget.checked)
          }
        />
        <NumberInput
          allowDecimal={false}
          allowNegative={false}
          disabled={!pronunciation}
          label={t('sessions.pronunciationAccuracy')}
          max={100}
          min={1}
          suffix="%"
          value={pronunciationAccuracy}
          onChange={(value) => setPronunciationAccuracy(Number(value) || 75)}
        />
        {invalidRange && (
          <Alert color="red">{t('sessions.invalidScoreRange')}</Alert>
        )}
        {start.isError && (
          <Alert color="red" title={t('sessions.startError')}>
            {start.error.message}
          </Alert>
        )}
        <Group justify="center">
          <Button
            disabled={invalidRange}
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
      </Stack>
    )
  }

  const current = session.currentCard
  const testCard = current?.kind === 'test' ? current : null
  const showPronunciation =
    testCard &&
    session.pronunciationCheckEnabled &&
    !pronunciationDone &&
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
          <ReadOnlyCard card={current.card} />
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
            <Text>{testCard.readings.join(', ')}</Text>
            <Text c="dimmed">
              {t('sessions.requiredAccuracy')}:{' '}
              {session.pronunciationAccuracyThreshold}%
            </Text>
            <Alert color="indigo">{t('sessions.pronunciationStub')}</Alert>
            <Button onClick={() => setPronunciationDone(true)}>
              {t('sessions.continue')}
            </Button>
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
                  {!feedback.isCorrect && (
                    <Text mt="xs">
                      {t('sessions.acceptedAnswers')}:{' '}
                      {feedback.expectedAnswers.join(', ')}
                    </Text>
                  )}
                </Alert>
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
