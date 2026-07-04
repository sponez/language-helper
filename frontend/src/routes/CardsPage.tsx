import {
  ActionIcon,
  Alert,
  Badge,
  Box,
  Button,
  Group,
  Loader,
  LoadingOverlay,
  Modal,
  NumberInput,
  Paper,
  ScrollArea,
  Select,
  Stack,
  Text,
  TextInput,
  Title,
  Tooltip,
} from '@mantine/core'
import { useDebouncedValue, useDisclosure } from '@mantine/hooks'
import {
  useInfiniteQuery,
  useMutation,
  useQuery,
  useQueryClient,
} from '@tanstack/react-query'
import {
  type FormEvent,
  useCallback,
  useEffect,
  useMemo,
  useRef,
  useState,
} from 'react'

import { useLanguageHelperClient } from '../api/LanguageHelperClientProvider'
import type {
  Card,
  CardDirection,
  CardMeaning,
  CardSortField,
  NewCardInput,
  PendingInverseCard,
  SortDirection,
} from '../api/language-helper-client'
import { CardSpeechControls } from '../components/CardSpeechControls'
import { ReadOnlyMeanings } from '../components/ReadOnlyCard'
import {
  matchesPrimaryShortcut,
  primaryAriaShortcut,
  primaryModifierLabel,
} from '../keyboard/shortcuts'
import { useTranslations } from '../locales/TranslationProvider'
import classes from './CardsPage.module.css'

interface CardsPageProps {
  username: string
  profileId: string
  onBack(): void
}

type Screen = 'list' | 'show' | 'add' | 'inverse-review'

function isTextEntryTarget(target: EventTarget | null) {
  return (
    target instanceof HTMLInputElement ||
    target instanceof HTMLTextAreaElement ||
    target instanceof HTMLSelectElement ||
    (target instanceof HTMLElement && target.isContentEditable)
  )
}

function hasOpenDialog() {
  return document.querySelector('[role="dialog"]') !== null
}
type EditSection = 'word' | 'readings' | 'meanings' | 'all' | null

function emptyMeaning(): CardMeaning {
  return {
    definition: '',
    translatedDefinition: '',
    wordTranslations: [''],
    examples: [],
  }
}

function emptyCard(): NewCardInput {
  return {
    direction: 'straight',
    word: '',
    readings: [],
    meanings: [emptyMeaning()],
  }
}

function validateCard(card: NewCardInput): string | null {
  if (!card.word.trim()) return 'cards.wordRequired'
  if (card.readings.some((reading) => !reading.trim())) {
    return 'cards.readingRequired'
  }
  if (card.meanings.length === 0) return 'cards.meaningRequired'
  if (card.meanings.some((meaning) => !meaning.definition.trim())) {
    return 'cards.definitionRequired'
  }
  if (
    card.meanings.some(
      (meaning) =>
        meaning.wordTranslations.length === 0 ||
        meaning.wordTranslations.some((translation) => !translation.trim()),
    )
  ) {
    return 'cards.translationRequired'
  }
  if (
    card.meanings.some((meaning) =>
      meaning.examples.some(
        (example) => !example.sentence.trim() || !example.translation.trim(),
      ),
    )
  ) {
    return 'cards.exampleRequired'
  }
  return null
}

function sameCardContent(left: NewCardInput, right: NewCardInput) {
  return JSON.stringify(left) === JSON.stringify(right)
}

function hasDraftContent(card: NewCardInput) {
  return !sameCardContent(card, emptyCard())
}

function UnsavedChangesModal({
  opened,
  pending = false,
  description,
  onDiscard,
  onKeepEditing,
}: {
  opened: boolean
  pending?: boolean
  description?: string
  onDiscard(): void
  onKeepEditing(): void
}) {
  const { t } = useTranslations()

  return (
    <Modal
      centered
      closeOnClickOutside={!pending}
      closeOnEscape={!pending}
      opened={opened}
      title={t('cards.unsavedChangesTitle')}
      onClose={onKeepEditing}
    >
      <Stack>
        <Text>{description ?? t('cards.unsavedChangesDescription')}</Text>
        <Group justify="flex-end">
          <Button color="red" disabled={pending} onClick={onDiscard}>
            {t('cards.discardChanges')}
          </Button>
          <Button disabled={pending} variant="default" onClick={onKeepEditing}>
            {t('cards.keepEditing')}
          </Button>
        </Group>
      </Stack>
    </Modal>
  )
}

function NormalizationOverlay({ visible }: { visible: boolean }) {
  const { t } = useTranslations()

  return (
    <LoadingOverlay
      loaderProps={{
        children: (
          <Stack align="center" gap="xs">
            <Loader size="sm" />
            <Text fw={600} size="sm">
              {t('cards.normalizing')}
            </Text>
          </Stack>
        ),
      }}
      visible={visible}
      zIndex={200}
    />
  )
}

function AiNormalizeButton({
  username,
  profileId,
  card,
  active = true,
  revertAvailable,
  busy,
  onApply,
  onBusyChange,
  onRevert,
}: {
  username: string
  profileId: string
  card: NewCardInput
  active?: boolean
  revertAvailable: boolean
  busy: boolean
  onApply(card: NewCardInput, previous: NewCardInput): void
  onBusyChange(busy: boolean): void
  onRevert(): void
}) {
  const client = useLanguageHelperClient()
  const { t } = useTranslations()
  const settings = useQuery({
    queryKey: ['ai-settings', username],
    queryFn: () => client.getAiSettings(username),
    retry: false,
  })
  const configured = Boolean(
    settings.data?.provider &&
    settings.data.apiKey?.trim() &&
    settings.data.modelName?.trim(),
  )
  const disabled = !card.word.trim() || !configured || settings.isPending
  const disabledHint = !card.word.trim()
    ? t('cards.wordRequired')
    : settings.isPending
      ? t('cards.aiSettingsLoading')
      : t('cards.aiNotConfigured')
  const normalize = useMutation({
    mutationFn: () => client.normalizeCard({ username, profileId, card }),
    onMutate: () => onBusyChange(true),
    onSuccess: (normalized) => onApply(normalized, structuredClone(card)),
    onSettled: () => onBusyChange(false),
  })
  const normalizeShortcut = `${primaryModifierLabel()}+Shift+N`
  const revertShortcut = `${primaryModifierLabel()}+Shift+R`

  function startNormalization() {
    if (!active || disabled || busy || revertAvailable) return
    normalize.reset()
    normalize.mutate()
  }

  useEffect(() => {
    const handleKeyDown = (event: KeyboardEvent) => {
      if (!active || event.repeat) return

      if (matchesPrimaryShortcut(event, 'KeyN', true)) {
        event.preventDefault()
        event.stopPropagation()
        startNormalization()
      } else if (
        (revertAvailable || busy) &&
        matchesPrimaryShortcut(event, 'KeyR', true)
      ) {
        event.preventDefault()
        event.stopPropagation()
        if (revertAvailable && !busy) onRevert()
      }
    }

    window.addEventListener('keydown', handleKeyDown)
    return () => window.removeEventListener('keydown', handleKeyDown)
  })

  return (
    <Stack align="flex-end" gap="xs">
      {revertAvailable ? (
        <Tooltip
          label={`${t('cards.revertAi')} (${revertShortcut})`}
          multiline
          w={280}
        >
          <Button
            aria-keyshortcuts={primaryAriaShortcut('R', true)}
            disabled={busy}
            size="xs"
            variant="default"
            onClick={onRevert}
          >
            ↶ {t('cards.revertAi')}
          </Button>
        </Tooltip>
      ) : (
        <Tooltip
          label={
            disabled
              ? disabledHint
              : `${t('cards.normalizeWithAi')} (${normalizeShortcut})`
          }
          multiline
          w={280}
        >
          <span>
            <Button
              aria-keyshortcuts={primaryAriaShortcut('N', true)}
              disabled={disabled || busy}
              loading={busy}
              size="xs"
              variant="light"
              onClick={startNormalization}
            >
              ✦ {t('cards.normalizeWithAi')}
            </Button>
          </span>
        </Tooltip>
      )}
      {normalize.isError && (
        <Alert color="red" title={t('cards.aiError')}>
          {normalize.error.message}
        </Alert>
      )}
    </Stack>
  )
}

function StringListEditor({
  label,
  values,
  addLabel,
  requireAtLeastOne = false,
  showValidation = false,
  validationMessage,
  onChange,
}: {
  label: string
  values: string[]
  addLabel: string
  requireAtLeastOne?: boolean
  showValidation?: boolean
  validationMessage?: string
  onChange(values: string[]): void
}) {
  const { t } = useTranslations()
  return (
    <Stack gap="xs">
      <Text fw={500} size="sm">
        {label}
      </Text>
      {values.map((value, index) => (
        <Group key={index} gap="xs" wrap="nowrap">
          <TextInput
            className={classes.dynamicInput}
            error={
              showValidation && !value.trim() ? validationMessage : undefined
            }
            value={value}
            onChange={(event) => {
              const nextValue = event.currentTarget.value
              onChange(
                values.map((current, currentIndex) =>
                  currentIndex === index ? nextValue : current,
                ),
              )
            }}
          />
          <ActionIcon
            aria-label={t('cards.remove')}
            color="red"
            variant="subtle"
            onClick={() =>
              onChange(
                values.filter((_, currentIndex) => currentIndex !== index),
              )
            }
          >
            −
          </ActionIcon>
        </Group>
      ))}
      {showValidation && requireAtLeastOne && values.length === 0 && (
        <Text c="red" size="xs">
          {validationMessage}
        </Text>
      )}
      <Button
        size="xs"
        variant="light"
        onClick={() => onChange([...values, ''])}
      >
        {addLabel}
      </Button>
    </Stack>
  )
}

function MeaningsEditor({
  meanings,
  showValidation = false,
  onChange,
}: {
  meanings: CardMeaning[]
  showValidation?: boolean
  onChange(meanings: CardMeaning[]): void
}) {
  const { t } = useTranslations()

  function updateMeaning(index: number, meaning: CardMeaning) {
    onChange(
      meanings.map((current, currentIndex) =>
        currentIndex === index ? meaning : current,
      ),
    )
  }

  return (
    <Stack gap="md">
      {meanings.map((meaning, meaningIndex) => {
        const meaningInvalid =
          !meaning.definition.trim() ||
          meaning.wordTranslations.length === 0 ||
          meaning.wordTranslations.some((translation) => !translation.trim()) ||
          meaning.examples.some(
            (example) =>
              !example.sentence.trim() || !example.translation.trim(),
          )

        return (
          <Paper
            key={meaningIndex}
            className={`${classes.meaning} ${
              showValidation && meaningInvalid ? classes.meaningInvalid : ''
            }`}
            p="md"
            withBorder
          >
            <Stack>
              <Group justify="space-between">
                <Text fw={600}>
                  {t('cards.meanings')} {meaningIndex + 1}
                </Text>
                <Button
                  color="red"
                  size="xs"
                  variant="subtle"
                  onClick={() =>
                    onChange(
                      meanings.filter(
                        (_, currentIndex) => currentIndex !== meaningIndex,
                      ),
                    )
                  }
                >
                  {t('cards.remove')}
                </Button>
              </Group>
              <TextInput
                error={
                  showValidation && !meaning.definition.trim()
                    ? t('cards.definitionRequired')
                    : undefined
                }
                label={t('cards.definition')}
                value={meaning.definition}
                onChange={(event) => {
                  const definition = event.currentTarget.value
                  updateMeaning(meaningIndex, { ...meaning, definition })
                }}
              />
              <TextInput
                label={t('cards.translatedDefinition')}
                value={meaning.translatedDefinition}
                onChange={(event) => {
                  const translatedDefinition = event.currentTarget.value
                  updateMeaning(meaningIndex, {
                    ...meaning,
                    translatedDefinition,
                  })
                }}
              />
              <StringListEditor
                addLabel={t('cards.addTranslation')}
                label={t('cards.translations')}
                requireAtLeastOne
                showValidation={showValidation}
                validationMessage={t('cards.translationRequired')}
                values={meaning.wordTranslations}
                onChange={(wordTranslations) =>
                  updateMeaning(meaningIndex, {
                    ...meaning,
                    wordTranslations,
                  })
                }
              />
              <Stack gap="xs">
                <Text fw={500} size="sm">
                  {t('cards.examples')}
                </Text>
                {meaning.examples.map((example, exampleIndex) => (
                  <Paper key={exampleIndex} p="sm" withBorder>
                    <Stack gap="xs">
                      <TextInput
                        error={
                          showValidation && !example.sentence.trim()
                            ? t('cards.exampleSentenceRequired')
                            : undefined
                        }
                        label={t('cards.sentence')}
                        value={example.sentence}
                        onChange={(event) => {
                          const sentence = event.currentTarget.value
                          updateMeaning(meaningIndex, {
                            ...meaning,
                            examples: meaning.examples.map(
                              (current, currentIndex) =>
                                currentIndex === exampleIndex
                                  ? { ...current, sentence }
                                  : current,
                            ),
                          })
                        }}
                      />
                      <TextInput
                        error={
                          showValidation && !example.translation.trim()
                            ? t('cards.exampleTranslationRequired')
                            : undefined
                        }
                        label={t('cards.translation')}
                        value={example.translation}
                        onChange={(event) => {
                          const translation = event.currentTarget.value
                          updateMeaning(meaningIndex, {
                            ...meaning,
                            examples: meaning.examples.map(
                              (current, currentIndex) =>
                                currentIndex === exampleIndex
                                  ? { ...current, translation }
                                  : current,
                            ),
                          })
                        }}
                      />
                      <Button
                        color="red"
                        size="xs"
                        variant="subtle"
                        onClick={() =>
                          updateMeaning(meaningIndex, {
                            ...meaning,
                            examples: meaning.examples.filter(
                              (_, currentIndex) =>
                                currentIndex !== exampleIndex,
                            ),
                          })
                        }
                      >
                        {t('cards.remove')}
                      </Button>
                    </Stack>
                  </Paper>
                ))}
                <Button
                  disabled={meaning.examples.length >= 5}
                  size="xs"
                  variant="light"
                  onClick={() =>
                    updateMeaning(meaningIndex, {
                      ...meaning,
                      examples: [
                        ...meaning.examples,
                        { sentence: '', translation: '' },
                      ],
                    })
                  }
                >
                  {t('cards.addExample')}
                </Button>
              </Stack>
            </Stack>
          </Paper>
        )
      })}
      {showValidation && meanings.length === 0 && (
        <Text c="red" size="xs">
          {t('cards.meaningRequired')}
        </Text>
      )}
      <Button
        disabled={meanings.length >= 10}
        variant="light"
        onClick={() => onChange([...meanings, emptyMeaning()])}
      >
        {t('cards.addMeaning')}
      </Button>
    </Stack>
  )
}

export function CardsPage({ username, profileId, onBack }: CardsPageProps) {
  const client = useLanguageHelperClient()
  const queryClient = useQueryClient()
  const { t } = useTranslations()
  const viewport = useRef<HTMLDivElement>(null)
  const cardRows = useRef(new Map<string, HTMLDivElement>())
  const [screen, setScreen] = useState<Screen>('list')
  const [selectedCardId, setSelectedCardId] = useState<string | null>(null)
  const [cursorCardId, setCursorCardId] = useState<string | null>(null)
  const [hoveredCardId, setHoveredCardId] = useState<string | null>(null)
  const [inverseCards, setInverseCards] = useState<PendingInverseCard[]>([])
  const [search, setSearch] = useState('')
  const [debouncedSearch] = useDebouncedValue(search, 250)
  const [direction, setDirection] = useState<CardDirection | null>(null)
  const [minScore, setMinScore] = useState<number | null>(null)
  const [maxScore, setMaxScore] = useState<number | null>(null)
  const [sortField, setSortField] = useState<CardSortField>('createdAt')
  const [sortDirection, setSortDirection] =
    useState<SortDirection>('descending')
  const [deleteTarget, setDeleteTarget] = useState<{
    id: string
    word: string
  } | null>(null)
  const [deleteOpened, deleteModal] = useDisclosure(false)

  const listKey = [
    'cards',
    username,
    profileId,
    debouncedSearch,
    direction,
    minScore,
    maxScore,
    sortField,
    sortDirection,
  ]

  const cards = useInfiniteQuery({
    queryKey: listKey,
    initialPageParam: null as string | null,
    queryFn: ({ pageParam }) =>
      client.listCards({
        username,
        profileId,
        search: debouncedSearch || undefined,
        direction,
        minScore,
        maxScore,
        sortField,
        sortDirection,
        cursor: pageParam,
        limit: 50,
      }),
    getNextPageParam: (lastPage) => lastPage.nextCursor ?? undefined,
    retry: false,
  })

  const removeCard = useMutation({
    mutationFn: (cardId: string) =>
      client.deleteCards({ username, profileId, cardIds: [cardId] }),
    onSuccess: async () => {
      deleteModal.close()
      setDeleteTarget(null)
      await queryClient.invalidateQueries({
        queryKey: ['cards', username, profileId],
      })
    },
  })

  const summaries = useMemo(
    () => cards.data?.pages.flatMap((page) => page.items) ?? [],
    [cards.data],
  )

  useEffect(() => {
    if (summaries.length === 0) {
      setCursorCardId(null)
      return
    }
    setCursorCardId((current) =>
      current === null || summaries.some((card) => card.id === current)
        ? current
        : null,
    )
  }, [summaries])

  useEffect(() => {
    if (screen !== 'list' || !cursorCardId) return
    cardRows.current.get(cursorCardId)?.scrollIntoView({ block: 'nearest' })
  }, [cursorCardId, screen])

  useEffect(() => {
    if (screen !== 'list') return

    const handleKeyDown = (event: KeyboardEvent) => {
      if (deleteOpened) {
        if (event.key === 'Escape' && !removeCard.isPending) {
          event.preventDefault()
          event.stopPropagation()
          deleteModal.close()
          setDeleteTarget(null)
          removeCard.reset()
        } else if (
          event.key === 'Enter' &&
          deleteTarget &&
          !removeCard.isPending
        ) {
          event.preventDefault()
          event.stopPropagation()
          removeCard.mutate(deleteTarget.id)
        }
        return
      }
      if (hasOpenDialog()) return

      if (event.key === 'Escape') {
        event.preventDefault()
        onBack()
        return
      }
      if (!event.repeat && matchesPrimaryShortcut(event, 'Enter')) {
        event.preventDefault()
        setScreen('add')
        return
      }
      if (isTextEntryTarget(event.target)) return

      const currentIndex = summaries.findIndex(
        (card) => card.id === cursorCardId,
      )
      if (event.key === 'Delete') {
        const targetId = hoveredCardId ?? cursorCardId
        const target = summaries.find((card) => card.id === targetId)
        if (target) {
          event.preventDefault()
          setDeleteTarget({ id: target.id, word: target.word })
          deleteModal.open()
        }
        return
      }
      if (event.key === 'ArrowDown' || event.key === 'ArrowUp') {
        if (summaries.length === 0) return
        event.preventDefault()
        const offset = event.key === 'ArrowDown' ? 1 : -1
        const nextIndex =
          currentIndex < 0
            ? event.key === 'ArrowDown'
              ? 0
              : summaries.length - 1
            : Math.min(summaries.length - 1, Math.max(0, currentIndex + offset))
        setCursorCardId(summaries[nextIndex].id)
        return
      }
      if (event.key === 'Enter' && currentIndex >= 0) {
        event.preventDefault()
        setSelectedCardId(summaries[currentIndex].id)
        setScreen('show')
      }
    }

    window.addEventListener('keydown', handleKeyDown)
    return () => window.removeEventListener('keydown', handleKeyDown)
  }, [
    cursorCardId,
    deleteModal,
    deleteOpened,
    deleteTarget,
    hoveredCardId,
    onBack,
    removeCard,
    screen,
    summaries,
  ])

  function resetFilters() {
    setSearch('')
    setDirection(null)
    setMinScore(null)
    setMaxScore(null)
    setSortField('createdAt')
    setSortDirection('descending')
  }

  if (screen === 'show' && selectedCardId) {
    return (
      <CardDetails
        cardId={selectedCardId}
        profileId={profileId}
        username={username}
        onBack={() => {
          setSelectedCardId(null)
          setScreen('list')
        }}
      />
    )
  }

  if (screen === 'add') {
    return (
      <AddCard
        profileId={profileId}
        username={username}
        onCancel={() => setScreen('list')}
        onCreated={async () => {
          resetFilters()
          setScreen('list')
          await queryClient.invalidateQueries({
            queryKey: ['cards', username, profileId],
          })
        }}
        onReview={async (pending) => {
          resetFilters()
          setInverseCards(pending)
          setScreen('inverse-review')
          await queryClient.invalidateQueries({
            queryKey: ['cards', username, profileId],
          })
        }}
      />
    )
  }

  if (screen === 'inverse-review') {
    return (
      <InverseCardsReview
        initialCards={inverseCards}
        profileId={profileId}
        username={username}
        onDone={async () => {
          setInverseCards([])
          setScreen('list')
          await queryClient.invalidateQueries({
            queryKey: ['cards', username, profileId],
          })
        }}
      />
    )
  }

  return (
    <Stack className={classes.page} gap="md">
      <Title order={2} ta="center">
        {t('cards.title')}
      </Title>
      <Paper p="md" withBorder>
        <div className={classes.filters}>
          <TextInput
            className={classes.search}
            label={t('cards.search')}
            value={search}
            onChange={(event) => setSearch(event.currentTarget.value)}
          />
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
          <div className={classes.sortRow}>
            <Select
              allowDeselect={false}
              className={classes.sortSelect}
              data={[
                { value: 'word', label: t('cards.sortWord') },
                { value: 'createdAt', label: t('cards.sortCreated') },
                { value: 'score', label: t('cards.sortScore') },
              ]}
              label={t('cards.sortBy')}
              value={sortField}
              onChange={(value) =>
                setSortField((value ?? 'createdAt') as CardSortField)
              }
            />
            <ActionIcon
              aria-label={sortDirection}
              mb={2}
              size={36}
              variant="default"
              onClick={() =>
                setSortDirection((current) =>
                  current === 'ascending' ? 'descending' : 'ascending',
                )
              }
            >
              {sortDirection === 'ascending' ? '↑' : '↓'}
            </ActionIcon>
          </div>
          <Button
            style={{ alignSelf: 'end' }}
            variant="subtle"
            onClick={resetFilters}
          >
            {t('cards.reset')}
          </Button>
        </div>
      </Paper>

      {cards.isError && (
        <Alert color="red" title={t('cards.loadError')}>
          {cards.error.message}
        </Alert>
      )}

      <Paper p="sm" withBorder>
        <ScrollArea
          className={classes.list}
          viewportRef={viewport}
          onScrollPositionChange={({ y }) => {
            const element = viewport.current
            if (
              element &&
              element.scrollHeight - element.clientHeight - y < 80 &&
              cards.hasNextPage &&
              !cards.isFetchingNextPage
            ) {
              void cards.fetchNextPage()
            }
          }}
        >
          <div className={classes.listViewport}>
            {cards.isPending ? (
              <Group justify="center" p="xl">
                <Loader />
              </Group>
            ) : summaries.length === 0 ? (
              <Text c="dimmed" p="xl" ta="center">
                {t('cards.noCards')}
              </Text>
            ) : (
              summaries.map((card) => (
                <Paper
                  key={card.id}
                  className={`${classes.cardRow} ${
                    cursorCardId === card.id ? classes.cardRowSelected : ''
                  }`}
                  data-card-id={card.id}
                  p="sm"
                  ref={(element) => {
                    if (element) cardRows.current.set(card.id, element)
                    else cardRows.current.delete(card.id)
                  }}
                  withBorder
                  onClick={() => setCursorCardId(card.id)}
                  onMouseEnter={() => setHoveredCardId(card.id)}
                  onMouseLeave={() =>
                    setHoveredCardId((current) =>
                      current === card.id ? null : current,
                    )
                  }
                >
                  <Badge variant="light">{t(`cards.${card.direction}`)}</Badge>
                  <Text className={classes.word} fw={500}>
                    {card.word}
                  </Text>
                  <Text c="dimmed" size="sm">
                    {card.score}
                  </Text>
                  <Button
                    size="xs"
                    tabIndex={-1}
                    variant="default"
                    onClick={() => {
                      setCursorCardId(card.id)
                      setSelectedCardId(card.id)
                      setScreen('show')
                    }}
                  >
                    {t('cards.show')}
                  </Button>
                  <Button
                    color="red"
                    size="xs"
                    tabIndex={-1}
                    variant="subtle"
                    onClick={() => {
                      setDeleteTarget({ id: card.id, word: card.word })
                      deleteModal.open()
                    }}
                  >
                    {t('cards.delete')}
                  </Button>
                </Paper>
              ))
            )}
            {cards.isFetchingNextPage && (
              <Group justify="center" p="md">
                <Loader size="sm" />
              </Group>
            )}
          </div>
        </ScrollArea>
      </Paper>

      <Group justify="center">
        <Button
          aria-keyshortcuts={primaryAriaShortcut('Enter')}
          tabIndex={-1}
          w={140}
          onClick={() => setScreen('add')}
        >
          {t('cards.addNew')}
        </Button>
        <Button tabIndex={-1} variant="default" w={140} onClick={onBack}>
          {t('cards.back')}
        </Button>
      </Group>
      <Text c="dimmed" size="xs" ta="center">
        {t('cards.catalogKeyboardHint').replace(
          '{mod}',
          primaryModifierLabel(),
        )}
      </Text>

      <Modal
        centered
        opened={deleteOpened}
        title={t('cards.deleteTitle')}
        onClose={() => {
          if (!removeCard.isPending) {
            deleteModal.close()
            setDeleteTarget(null)
            removeCard.reset()
          }
        }}
      >
        <Stack>
          <Text>
            {deleteTarget?.word}: {t('cards.deleteDescription')}
          </Text>
          {removeCard.isError && (
            <Alert color="red">{removeCard.error.message}</Alert>
          )}
          <Group justify="flex-end">
            <Button
              color="red"
              loading={removeCard.isPending}
              onClick={() => {
                if (deleteTarget) removeCard.mutate(deleteTarget.id)
              }}
            >
              {t('cards.delete')}
            </Button>
            <Button
              disabled={removeCard.isPending}
              variant="default"
              onClick={() => {
                deleteModal.close()
                setDeleteTarget(null)
                removeCard.reset()
              }}
            >
              {t('cards.cancel')}
            </Button>
          </Group>
        </Stack>
      </Modal>
    </Stack>
  )
}

function CardDetails({
  username,
  profileId,
  cardId,
  onBack,
}: {
  username: string
  profileId: string
  cardId: string
  onBack(): void
}) {
  const client = useLanguageHelperClient()
  const queryClient = useQueryClient()
  const { t } = useTranslations()
  const [editSection, setEditSection] = useState<EditSection>(null)
  const [word, setWord] = useState('')
  const [readings, setReadings] = useState<string[]>([])
  const [meanings, setMeanings] = useState<CardMeaning[]>([])
  const [editBaseline, setEditBaseline] = useState<NewCardInput | null>(null)
  const [discardAction, setDiscardAction] = useState<
    'cancel-edit' | 'back' | null
  >(null)
  const [normalizationBusy, setNormalizationBusy] = useState(false)
  const [normalizationSnapshot, setNormalizationSnapshot] = useState<{
    card: NewCardInput
    editSection: EditSection
  } | null>(null)

  const card = useQuery({
    queryKey: ['card', username, profileId, cardId],
    queryFn: () => client.getCard(username, profileId, cardId),
    retry: false,
  })

  const update = useMutation({
    mutationFn: (current: Card) =>
      client.updateCard({
        username,
        profileId,
        cardId,
        expectedVersion: current.version,
        word:
          editSection === 'word' || editSection === 'all' ? word : current.word,
        readings:
          editSection === 'readings' || editSection === 'all'
            ? readings
            : current.readings,
        meanings:
          editSection === 'meanings' || editSection === 'all'
            ? meanings
            : current.meanings,
      }),
    onSuccess: async (updated) => {
      queryClient.setQueryData(['card', username, profileId, cardId], updated)
      setEditSection(null)
      setEditBaseline(null)
      setNormalizationSnapshot(null)
      await queryClient.invalidateQueries({
        queryKey: ['cards', username, profileId],
      })
    },
  })

  function beginEdit(section: Exclude<EditSection, null>, current: Card) {
    setWord(current.word)
    setReadings([...current.readings])
    setMeanings(structuredClone(current.meanings))
    setEditBaseline({
      direction: current.direction,
      word: current.word,
      readings: current.readings,
      meanings: current.meanings,
    })
    update.reset()
    setNormalizationSnapshot(null)
    setEditSection(section)
  }

  const cancelEdit = useCallback(() => {
    setEditSection(null)
    setEditBaseline(null)
    setNormalizationSnapshot(null)
    update.reset()
  }, [update])

  const loadedCard = card.data
  const effectiveCard: NewCardInput | null = loadedCard
    ? {
        direction: loadedCard.direction,
        word:
          editSection === 'word' || editSection === 'all'
            ? word
            : loadedCard.word,
        readings:
          editSection === 'readings' || editSection === 'all'
            ? readings
            : loadedCard.readings,
        meanings:
          editSection === 'meanings' || editSection === 'all'
            ? meanings
            : loadedCard.meanings,
      }
    : null
  const validationError =
    editSection === null || !effectiveCard ? null : validateCard(effectiveCard)
  const hasUnsavedChanges = Boolean(
    editSection !== null &&
    editBaseline &&
    effectiveCard &&
    !sameCardContent(editBaseline, effectiveCard),
  )

  const discardChanges = useCallback(() => {
    const action = discardAction
    setDiscardAction(null)
    cancelEdit()
    if (action === 'back') onBack()
  }, [cancelEdit, discardAction, onBack])

  useEffect(() => {
    const handleKeyDown = (event: KeyboardEvent) => {
      if (normalizationBusy) {
        event.preventDefault()
        event.stopPropagation()
        return
      }
      if (update.isPending) {
        event.preventDefault()
        event.stopPropagation()
        return
      }
      if (discardAction) {
        if (event.key === 'Escape') {
          event.preventDefault()
          setDiscardAction(null)
        } else if (event.key === 'Enter') {
          event.preventDefault()
          discardChanges()
        }
        return
      }
      if (matchesPrimaryShortcut(event, 'KeyS') && editSection !== null) {
        event.preventDefault()
        event.stopPropagation()
        if (
          !event.repeat &&
          loadedCard &&
          !validationError &&
          !update.isPending
        ) {
          update.mutate(loadedCard)
        }
        return
      }
      if (hasOpenDialog()) return
      if (event.key === 'Escape') {
        event.preventDefault()
        if (editSection === null) {
          onBack()
        } else if (hasUnsavedChanges) {
          setDiscardAction('cancel-edit')
        } else {
          cancelEdit()
        }
      } else if (
        event.key === 'Enter' &&
        editSection === null &&
        !(
          event.target instanceof HTMLElement &&
          event.target.closest('button, a, input, textarea, select')
        )
      ) {
        event.preventDefault()
        onBack()
      }
    }

    window.addEventListener('keydown', handleKeyDown)
    return () => window.removeEventListener('keydown', handleKeyDown)
  }, [
    cancelEdit,
    discardAction,
    discardChanges,
    editSection,
    hasUnsavedChanges,
    loadedCard,
    normalizationBusy,
    onBack,
    update,
    validationError,
  ])

  if (card.isPending) {
    return (
      <Group justify="center" p="xl">
        <Loader />
      </Group>
    )
  }
  if (card.isError) {
    return (
      <Stack>
        <Alert color="red" title={t('cards.loadError')}>
          {card.error.message}
        </Alert>
        <Button variant="default" onClick={onBack}>
          {t('cards.back')}
        </Button>
      </Stack>
    )
  }

  const current = card.data
  const editActions = (
    <Group justify="flex-end">
      <Button
        aria-keyshortcuts={primaryAriaShortcut('S')}
        disabled={Boolean(validationError)}
        loading={update.isPending}
        size="xs"
        onClick={() => update.mutate(current)}
      >
        {t('cards.save')}
      </Button>
      <Button
        disabled={update.isPending || normalizationBusy}
        size="xs"
        variant="default"
        onClick={cancelEdit}
      >
        {t('cards.cancel')}
      </Button>
    </Group>
  )

  return (
    <Box
      aria-busy={normalizationBusy}
      className={`${classes.details} ${classes.blockingRoot}`}
    >
      <NormalizationOverlay visible={normalizationBusy} />
      <Stack
        gap="md"
        inert={normalizationBusy || update.isPending ? true : undefined}
      >
        <Group justify="space-between">
          <Button
            disabled={update.isPending}
            variant="subtle"
            onClick={() => {
              if (hasUnsavedChanges) setDiscardAction('back')
              else onBack()
            }}
          >
            ← {t('cards.back')}
          </Button>
          <AiNormalizeButton
            active={!update.isPending}
            busy={normalizationBusy}
            card={effectiveCard!}
            profileId={profileId}
            revertAvailable={normalizationSnapshot !== null}
            username={username}
            onApply={(normalized, previous) => {
              if (editSection === null) setEditBaseline(previous)
              setNormalizationSnapshot({
                card: previous,
                editSection,
              })
              setWord(normalized.word)
              setReadings(normalized.readings)
              setMeanings(normalized.meanings)
              setEditSection('all')
            }}
            onBusyChange={setNormalizationBusy}
            onRevert={() => {
              if (!normalizationSnapshot) return
              setWord(normalizationSnapshot.card.word)
              setReadings(normalizationSnapshot.card.readings)
              setMeanings(normalizationSnapshot.card.meanings)
              setEditSection(normalizationSnapshot.editSection)
              if (normalizationSnapshot.editSection === null) {
                setEditBaseline(null)
              }
              setNormalizationSnapshot(null)
              update.reset()
            }}
          />
        </Group>
        <Text c="dimmed" size="xs" ta="center">
          {t('cards.detailsKeyboardHint').replace(
            '{mod}',
            primaryModifierLabel(),
          )}
        </Text>
        <Group justify="center">
          <Badge>{t(`cards.${current.direction}`)}</Badge>
          <Badge variant="light">
            {t('cards.score')}: {current.score}
          </Badge>
        </Group>
        <Text c="dimmed" size="sm" ta="center">
          {t('cards.createdAt')}: {new Date(current.createdAt).toLocaleString()}
        </Text>

        <Paper p="lg" withBorder>
          <div className={classes.sectionHeader}>
            <Title order={3}>{t('cards.word')}</Title>
            {editSection === null && (
              <ActionIcon
                aria-label={t('cards.edit')}
                variant="subtle"
                onClick={() => beginEdit('word', current)}
              >
                ✎
              </ActionIcon>
            )}
          </div>
          {editSection === 'word' || editSection === 'all' ? (
            <Stack mt="md">
              <TextInput
                value={word}
                onChange={(event) => setWord(event.currentTarget.value)}
              />
              {editSection === 'word' && editActions}
            </Stack>
          ) : (
            <>
              <Text fw={600} mt="md" size="xl" ta="center">
                {current.word}
              </Text>
              <Group justify="center" mt="md">
                <CardSpeechControls
                  cardId={current.id}
                  hotkeysEnabled={!normalizationBusy}
                  profileId={profileId}
                  username={username}
                />
              </Group>
            </>
          )}
        </Paper>

        <Paper p="lg" withBorder>
          <div className={classes.sectionHeader}>
            <Title order={3}>{t('cards.readings')}</Title>
            {editSection === null && (
              <ActionIcon
                aria-label={t('cards.edit')}
                variant="subtle"
                onClick={() => beginEdit('readings', current)}
              >
                ✎
              </ActionIcon>
            )}
          </div>
          {editSection === 'readings' || editSection === 'all' ? (
            <Stack mt="md">
              <StringListEditor
                addLabel={t('cards.addReading')}
                label={t('cards.readings')}
                values={readings}
                onChange={setReadings}
              />
              {editSection === 'readings' && editActions}
            </Stack>
          ) : (
            <Text c={current.readings.length ? undefined : 'dimmed'} mt="md">
              {current.readings.length
                ? current.readings.join(', ')
                : t('cards.noReadings')}
            </Text>
          )}
        </Paper>

        <Paper p="lg" withBorder>
          <div className={classes.sectionHeader}>
            <Title order={3}>{t('cards.meanings')}</Title>
            {editSection === null && (
              <ActionIcon
                aria-label={t('cards.edit')}
                variant="subtle"
                onClick={() => beginEdit('meanings', current)}
              >
                ✎
              </ActionIcon>
            )}
          </div>
          {editSection === 'meanings' || editSection === 'all' ? (
            <Stack mt="md">
              <MeaningsEditor meanings={meanings} onChange={setMeanings} />
              {editSection === 'meanings' && editActions}
            </Stack>
          ) : (
            <Stack mt="md">
              <ReadOnlyMeanings meanings={current.meanings} />
            </Stack>
          )}
        </Paper>

        {editSection === 'all' && editActions}

        {validationError && <Alert color="red">{t(validationError)}</Alert>}
        {update.isError && (
          <Alert color="red" title={t('cards.updateError')}>
            <Stack gap="xs">
              <Text>{update.error.message}</Text>
              <Text size="sm">{t('cards.conflictHint')}</Text>
              <Button
                size="xs"
                variant="light"
                onClick={() => {
                  cancelEdit()
                  void card.refetch()
                }}
              >
                {t('cards.reload')}
              </Button>
            </Stack>
          </Alert>
        )}
      </Stack>
      <UnsavedChangesModal
        opened={discardAction !== null}
        onDiscard={discardChanges}
        onKeepEditing={() => setDiscardAction(null)}
      />
    </Box>
  )
}

function InverseCardsReview({
  username,
  profileId,
  initialCards,
  onDone,
}: {
  username: string
  profileId: string
  initialCards: PendingInverseCard[]
  onDone(): Promise<void>
}) {
  const client = useLanguageHelperClient()
  const { t } = useTranslations()
  const [pending, setPending] = useState(() => structuredClone(initialCards))
  const [editing, setEditing] = useState<PendingInverseCard | null>(null)
  const inverseRows = useRef(new Map<string, HTMLDivElement>())
  const [cursorCardId, setCursorCardId] = useState<string | null>(null)
  const [hoveredCardId, setHoveredCardId] = useState<string | null>(null)
  const [rejectTarget, setRejectTarget] = useState<PendingInverseCard | null>(
    null,
  )
  const [discardAllOpened, setDiscardAllOpened] = useState(false)
  const [discardEditOpened, setDiscardEditOpened] = useState(false)
  const [submitted, setSubmitted] = useState(false)
  const [normalizationBusy, setNormalizationBusy] = useState(false)
  const [normalizationSnapshot, setNormalizationSnapshot] =
    useState<NewCardInput | null>(null)

  const saveOne = useMutation({
    mutationFn: (card: PendingInverseCard) =>
      client.saveInverseCards({ username, profileId, cards: [card] }),
    onSuccess: async (_, saved) => {
      const savedIndex = pending.findIndex(
        (candidate) => candidate.card.id === saved.card.id,
      )
      const remaining = pending.filter(
        (candidate) => candidate.card.id !== saved.card.id,
      )
      setPending(remaining)
      setCursorCardId(
        remaining[Math.min(savedIndex, remaining.length - 1)]?.card.id ?? null,
      )
      setEditing(null)
      setNormalizationSnapshot(null)
      if (remaining.length === 0) await onDone()
    },
  })
  const saveAll = useMutation({
    mutationFn: () =>
      client.saveInverseCards({ username, profileId, cards: pending }),
    onSuccess: onDone,
  })

  const draft = editing?.card
  const originalEditing = editing
    ? pending.find((candidate) => candidate.card.id === editing.card.id)
    : null
  const editingDirty = Boolean(
    editing &&
    originalEditing &&
    (!sameCardContent(editing.card, originalEditing.card) ||
      editing.expectedVersion !== originalEditing.expectedVersion),
  )
  const validationError = draft
    ? validateCard({
        direction: draft.direction,
        word: draft.word,
        readings: draft.readings,
        meanings: draft.meanings,
      })
    : null

  const closeEditor = useCallback(() => {
    setDiscardEditOpened(false)
    setSubmitted(false)
    setNormalizationSnapshot(null)
    saveOne.reset()
    setEditing(null)
  }, [saveOne])

  const requestCloseEditor = useCallback(() => {
    if (normalizationBusy || saveOne.isPending) return
    if (editingDirty) setDiscardEditOpened(true)
    else closeEditor()
  }, [closeEditor, editingDirty, normalizationBusy, saveOne.isPending])

  const openEditor = useCallback(
    (candidate: PendingInverseCard) => {
      setSubmitted(false)
      saveOne.reset()
      setNormalizationSnapshot(null)
      setEditing(structuredClone(candidate))
    },
    [saveOne],
  )

  const rejectCard = useCallback(() => {
    if (!rejectTarget) return
    const rejectedIndex = pending.findIndex(
      (candidate) => candidate.card.id === rejectTarget.card.id,
    )
    const remaining = pending.filter(
      (candidate) => candidate.card.id !== rejectTarget.card.id,
    )
    setPending(remaining)
    setCursorCardId(
      remaining[Math.min(rejectedIndex, remaining.length - 1)]?.card.id ?? null,
    )
    setRejectTarget(null)
    if (remaining.length === 0) void onDone()
  }, [onDone, pending, rejectTarget])

  useEffect(() => {
    if (!cursorCardId) return
    inverseRows.current.get(cursorCardId)?.scrollIntoView({ block: 'nearest' })
  }, [cursorCardId])

  useEffect(() => {
    const handleKeyDown = (event: KeyboardEvent) => {
      if (normalizationBusy) {
        event.preventDefault()
        event.stopPropagation()
        return
      }
      if (saveAll.isPending || saveOne.isPending) {
        event.preventDefault()
        event.stopPropagation()
        return
      }
      if (rejectTarget) {
        if (event.key === 'Escape') {
          event.preventDefault()
          setRejectTarget(null)
        } else if (event.key === 'Enter') {
          event.preventDefault()
          rejectCard()
        }
        return
      }
      if (discardAllOpened) {
        if (event.key === 'Escape') {
          event.preventDefault()
          setDiscardAllOpened(false)
        } else if (event.key === 'Enter') {
          event.preventDefault()
          void onDone()
        }
        return
      }
      if (discardEditOpened) {
        if (event.key === 'Escape') {
          event.preventDefault()
          setDiscardEditOpened(false)
        } else if (event.key === 'Enter') {
          event.preventDefault()
          closeEditor()
        }
        return
      }
      if (editing) {
        if (matchesPrimaryShortcut(event, 'KeyS')) {
          event.preventDefault()
          event.stopPropagation()
          setSubmitted(true)
          if (!event.repeat && !validationError && !saveOne.isPending) {
            saveOne.mutate(editing)
          }
        } else if (event.key === 'Escape') {
          event.preventDefault()
          requestCloseEditor()
        }
        return
      }
      if (hasOpenDialog()) return
      if (event.key === 'Escape') {
        event.preventDefault()
        setDiscardAllOpened(true)
        return
      }
      if (matchesPrimaryShortcut(event, 'KeyS')) {
        event.preventDefault()
        event.stopPropagation()
        if (!event.repeat && !saveAll.isPending) saveAll.mutate()
        return
      }
      if (
        event.key === 'Enter' &&
        event.target instanceof HTMLElement &&
        event.target.closest('button, a, input, textarea, select')
      ) {
        return
      }

      const currentIndex = pending.findIndex(
        (candidate) => candidate.card.id === cursorCardId,
      )
      if (event.key === 'ArrowDown' || event.key === 'ArrowUp') {
        if (pending.length === 0) return
        event.preventDefault()
        const offset = event.key === 'ArrowDown' ? 1 : -1
        const nextIndex =
          currentIndex < 0
            ? event.key === 'ArrowDown'
              ? 0
              : pending.length - 1
            : Math.min(pending.length - 1, Math.max(0, currentIndex + offset))
        setCursorCardId(pending[nextIndex].card.id)
      } else if (event.key === 'Enter' && currentIndex >= 0) {
        event.preventDefault()
        openEditor(pending[currentIndex])
      } else if (event.key === 'Delete') {
        const targetId = hoveredCardId ?? cursorCardId
        const target = pending.find(
          (candidate) => candidate.card.id === targetId,
        )
        if (target) {
          event.preventDefault()
          setRejectTarget(target)
        }
      }
    }

    window.addEventListener('keydown', handleKeyDown)
    return () => window.removeEventListener('keydown', handleKeyDown)
  }, [
    closeEditor,
    cursorCardId,
    discardAllOpened,
    discardEditOpened,
    editing,
    hoveredCardId,
    normalizationBusy,
    onDone,
    openEditor,
    pending,
    rejectCard,
    rejectTarget,
    requestCloseEditor,
    saveAll,
    saveOne,
    validationError,
  ])

  return (
    <Stack className={classes.details} gap="md">
      <Title order={2} ta="center">
        {t('cards.inverseReviewTitle')}
      </Title>
      <Text c="dimmed" ta="center">
        {t('cards.inverseReviewDescription')}
      </Text>
      <ScrollArea className={classes.inverseList} type="auto">
        <Stack gap="xs" p="xs">
          {pending.map((candidate) => (
            <div
              key={candidate.card.id}
              className={classes.inverseCardRow}
              ref={(element) => {
                if (element) {
                  inverseRows.current.set(candidate.card.id, element)
                } else {
                  inverseRows.current.delete(candidate.card.id)
                }
              }}
              onMouseEnter={() => setHoveredCardId(candidate.card.id)}
              onMouseLeave={() =>
                setHoveredCardId((current) =>
                  current === candidate.card.id ? null : current,
                )
              }
            >
              <Button
                className={`${classes.inverseCardButton} ${
                  cursorCardId === candidate.card.id
                    ? classes.inverseCardSelected
                    : ''
                }`}
                disabled={saveAll.isPending}
                justify="space-between"
                variant="default"
                onClick={() => {
                  setCursorCardId(candidate.card.id)
                  openEditor(candidate)
                }}
              >
                <span>{candidate.card.word}</span>
                <Badge variant="light">
                  {candidate.expectedVersion === null
                    ? t('cards.inverseCreated')
                    : t('cards.inverseUpdated')}
                </Badge>
              </Button>
              <ActionIcon
                aria-label={t('cards.rejectInverse')}
                color="red"
                disabled={saveAll.isPending}
                variant="subtle"
                onClick={() => {
                  setCursorCardId(candidate.card.id)
                  setRejectTarget(candidate)
                }}
              >
                ×
              </ActionIcon>
            </div>
          ))}
        </Stack>
      </ScrollArea>
      <Text c="dimmed" size="xs" ta="center">
        {t('cards.inverseKeyboardHint').replace(
          '{mod}',
          primaryModifierLabel(),
        )}
      </Text>
      {saveAll.isError && (
        <Alert color="red" title={t('cards.inverseSaveError')}>
          {saveAll.error.message}
        </Alert>
      )}
      <Group justify="center">
        <Button
          aria-keyshortcuts={primaryAriaShortcut('S')}
          loading={saveAll.isPending}
          onClick={() => saveAll.mutate()}
        >
          {t('cards.saveWithoutReview')}
        </Button>
        <Button
          disabled={saveAll.isPending}
          variant="default"
          onClick={() => setDiscardAllOpened(true)}
        >
          {t('cards.cancel')}
        </Button>
      </Group>

      <Modal
        closeOnClickOutside={!normalizationBusy}
        closeOnEscape={false}
        opened={editing !== null}
        size="lg"
        title={t('cards.editInverseTitle')}
        onClose={requestCloseEditor}
      >
        {editing && draft && (
          <Box aria-busy={normalizationBusy} className={classes.blockingRoot}>
            <NormalizationOverlay visible={normalizationBusy} />
            <Stack
              inert={normalizationBusy || saveOne.isPending ? true : undefined}
            >
              <Group justify="space-between">
                <Badge>{t(`cards.${draft.direction}`)}</Badge>
                <AiNormalizeButton
                  active={!saveOne.isPending}
                  busy={normalizationBusy}
                  card={{
                    direction: draft.direction,
                    word: draft.word,
                    readings: draft.readings,
                    meanings: draft.meanings,
                  }}
                  profileId={profileId}
                  revertAvailable={normalizationSnapshot !== null}
                  username={username}
                  onApply={(normalized, previous) => {
                    setNormalizationSnapshot(previous)
                    setEditing({
                      ...editing,
                      card: {
                        ...draft,
                        direction: normalized.direction,
                        word: normalized.word,
                        readings: normalized.readings,
                        meanings: normalized.meanings,
                      },
                    })
                  }}
                  onBusyChange={setNormalizationBusy}
                  onRevert={() => {
                    if (!normalizationSnapshot) return
                    setEditing({
                      ...editing,
                      card: {
                        ...draft,
                        ...structuredClone(normalizationSnapshot),
                      },
                    })
                    setNormalizationSnapshot(null)
                    saveOne.reset()
                  }}
                />
              </Group>
              <Text c="dimmed" size="xs" ta="right">
                {t('cards.inverseEditKeyboardHint').replaceAll(
                  '{mod}',
                  primaryModifierLabel(),
                )}
              </Text>
              <TextInput
                error={
                  submitted && !draft.word.trim()
                    ? t('cards.wordRequired')
                    : undefined
                }
                label={t('cards.word')}
                value={draft.word}
                onChange={(event) =>
                  setEditing({
                    ...editing,
                    card: { ...draft, word: event.currentTarget.value },
                  })
                }
              />
              <Paper p="md" withBorder>
                <StringListEditor
                  addLabel={t('cards.addReading')}
                  label={t('cards.readings')}
                  showValidation={submitted}
                  validationMessage={t('cards.readingRequired')}
                  values={draft.readings}
                  onChange={(readings) =>
                    setEditing({
                      ...editing,
                      card: { ...draft, readings },
                    })
                  }
                />
              </Paper>
              <MeaningsEditor
                meanings={draft.meanings}
                showValidation={submitted}
                onChange={(meanings) =>
                  setEditing({
                    ...editing,
                    card: { ...draft, meanings },
                  })
                }
              />
              {submitted && validationError && (
                <Alert color="red">{t(validationError)}</Alert>
              )}
              {saveOne.isError && (
                <Alert color="red" title={t('cards.inverseSaveError')}>
                  {saveOne.error.message}
                </Alert>
              )}
              <Group justify="flex-end">
                <Button
                  aria-keyshortcuts={primaryAriaShortcut('S')}
                  loading={saveOne.isPending}
                  onClick={() => {
                    setSubmitted(true)
                    if (!validationError) saveOne.mutate(editing)
                  }}
                >
                  {t('cards.save')}
                </Button>
                <Button
                  disabled={saveOne.isPending}
                  variant="default"
                  onClick={requestCloseEditor}
                >
                  {t('cards.cancel')}
                </Button>
              </Group>
            </Stack>
          </Box>
        )}
      </Modal>
      <Modal
        centered
        opened={rejectTarget !== null}
        title={t('cards.rejectInverseTitle')}
        onClose={() => setRejectTarget(null)}
      >
        <Stack>
          <Text>
            {rejectTarget?.card.word}: {t('cards.rejectInverseDescription')}
          </Text>
          <Group justify="flex-end">
            <Button color="red" onClick={rejectCard}>
              {t('cards.rejectInverse')}
            </Button>
            <Button variant="default" onClick={() => setRejectTarget(null)}>
              {t('cards.cancel')}
            </Button>
          </Group>
        </Stack>
      </Modal>
      <UnsavedChangesModal
        description={t('cards.discardInverseDescription')}
        opened={discardAllOpened}
        onDiscard={() => void onDone()}
        onKeepEditing={() => setDiscardAllOpened(false)}
      />
      <UnsavedChangesModal
        opened={discardEditOpened}
        onDiscard={closeEditor}
        onKeepEditing={() => setDiscardEditOpened(false)}
      />
    </Stack>
  )
}

function AddCard({
  username,
  profileId,
  onCancel,
  onCreated,
  onReview,
}: {
  username: string
  profileId: string
  onCancel(): void
  onCreated(): Promise<void>
  onReview(cards: PendingInverseCard[]): Promise<void>
}) {
  const client = useLanguageHelperClient()
  const { t } = useTranslations()
  const [drafts, setDrafts] = useState<NewCardInput[]>([emptyCard()])
  const [normalizationSnapshots, setNormalizationSnapshots] = useState<
    Array<NewCardInput | null>
  >([null])
  const [normalizationBusy, setNormalizationBusy] = useState(false)
  const [activeIndex, setActiveIndex] = useState(0)
  const [submitted, setSubmitted] = useState(false)
  const [createdCards, setCreatedCards] = useState<Card[] | null>(null)
  const [discardOpened, setDiscardOpened] = useState(false)

  const create = useMutation({
    mutationFn: (cards: NewCardInput[]) =>
      client.createCards({ username, profileId, cards }),
    onSuccess: setCreatedCards,
  })
  const prepareInverse = useMutation({
    mutationFn: (sourceCardIds: string[]) =>
      client.prepareInverseCards({ username, profileId, sourceCardIds }),
    onSuccess: onReview,
  })

  const draft = drafts[activeIndex]
  const hasUnsavedDrafts =
    drafts.length > 1 || drafts.some((candidate) => hasDraftContent(candidate))

  const draftError = useCallback(
    (index: number) => {
      const validationError = validateCard(drafts[index])
      if (validationError) return validationError
      const word = drafts[index].word.trim()
      if (
        drafts.some(
          (candidate, candidateIndex) =>
            candidateIndex !== index && candidate.word.trim() === word,
        )
      ) {
        return 'cards.duplicateDraft'
      }
      return null
    },
    [drafts],
  )
  const activeWordError =
    submitted && !draft.word.trim()
      ? t('cards.wordRequired')
      : submitted &&
          drafts.some(
            (candidate, index) =>
              index !== activeIndex &&
              candidate.word.trim() === draft.word.trim(),
          )
        ? t('cards.duplicateDraft')
        : undefined

  function changeDraft(changes: Partial<NewCardInput>) {
    setDrafts((current) =>
      current.map((candidate, index) =>
        index === activeIndex ? { ...candidate, ...changes } : candidate,
      ),
    )
    create.reset()
  }

  const addDraft = useCallback(() => {
    setDrafts((current) => [...current, emptyCard()])
    setNormalizationSnapshots((current) => [...current, null])
    setActiveIndex(drafts.length)
  }, [drafts.length])

  function removeDraft(index: number) {
    if (drafts.length === 1) return
    setDrafts((current) =>
      current.filter((_, candidateIndex) => candidateIndex !== index),
    )
    setNormalizationSnapshots((current) =>
      current.filter((_, candidateIndex) => candidateIndex !== index),
    )
    setActiveIndex((current) => {
      if (current > index) return current - 1
      if (current === index) return Math.max(0, current - 1)
      return current
    })
    create.reset()
  }

  const saveDrafts = useCallback(() => {
    setSubmitted(true)
    const invalidIndex = drafts.findIndex((_, index) => draftError(index))
    if (invalidIndex >= 0) {
      setActiveIndex(invalidIndex)
      return
    }
    create.mutate(drafts)
  }, [create, draftError, drafts])

  function submit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault()
    saveDrafts()
  }

  useEffect(() => {
    const handleKeyDown = (event: KeyboardEvent) => {
      if (normalizationBusy) {
        event.preventDefault()
        event.stopPropagation()
        return
      }
      if (create.isPending) {
        event.preventDefault()
        event.stopPropagation()
        return
      }
      if (discardOpened) {
        if (event.key === 'Escape') {
          event.preventDefault()
          setDiscardOpened(false)
        } else if (event.key === 'Enter') {
          event.preventDefault()
          onCancel()
        }
        return
      }
      if (matchesPrimaryShortcut(event, 'KeyS')) {
        event.preventDefault()
        event.stopPropagation()
        if (
          !event.repeat &&
          createdCards === null &&
          !hasOpenDialog() &&
          !create.isPending
        ) {
          saveDrafts()
        }
        return
      }
      if (
        !event.repeat &&
        createdCards === null &&
        !hasOpenDialog() &&
        matchesPrimaryShortcut(event, 'Enter')
      ) {
        event.preventDefault()
        event.stopPropagation()
        if (!create.isPending) addDraft()
        return
      }
      if (event.key !== 'Escape' || createdCards !== null || hasOpenDialog()) {
        return
      }
      event.preventDefault()
      if (hasUnsavedDrafts) setDiscardOpened(true)
      else onCancel()
    }

    window.addEventListener('keydown', handleKeyDown)
    return () => window.removeEventListener('keydown', handleKeyDown)
  }, [
    activeIndex,
    addDraft,
    create.isPending,
    createdCards,
    drafts,
    discardOpened,
    hasUnsavedDrafts,
    normalizationBusy,
    onCancel,
    saveDrafts,
  ])

  return (
    <Box
      aria-busy={normalizationBusy}
      className={`${classes.details} ${classes.blockingRoot}`}
    >
      <NormalizationOverlay visible={normalizationBusy} />
      <form
        inert={normalizationBusy || create.isPending ? true : undefined}
        onSubmit={submit}
      >
        <Stack gap="md">
          <div className={classes.addHeader}>
            <Title order={2} ta="center">
              {t('cards.addTitle')}
            </Title>
            <div className={classes.addHeaderAction}>
              <AiNormalizeButton
                active={createdCards === null && !create.isPending}
                busy={normalizationBusy}
                card={draft}
                key={activeIndex}
                profileId={profileId}
                revertAvailable={normalizationSnapshots[activeIndex] !== null}
                username={username}
                onApply={(normalized, previous) => {
                  setNormalizationSnapshots((current) =>
                    current.map((snapshot, index) =>
                      index === activeIndex ? previous : snapshot,
                    ),
                  )
                  changeDraft(normalized)
                }}
                onBusyChange={setNormalizationBusy}
                onRevert={() => {
                  const snapshot = normalizationSnapshots[activeIndex]
                  if (!snapshot) return
                  setDrafts((current) =>
                    current.map((candidate, index) =>
                      index === activeIndex
                        ? structuredClone(snapshot)
                        : candidate,
                    ),
                  )
                  setNormalizationSnapshots((current) =>
                    current.map((candidate, index) =>
                      index === activeIndex ? null : candidate,
                    ),
                  )
                  create.reset()
                }}
              />
            </div>
          </div>
          <Text c="dimmed" size="xs" ta="center">
            {t('cards.addKeyboardHint').replaceAll(
              '{mod}',
              primaryModifierLabel(),
            )}
          </Text>
          <Paper p="sm" withBorder>
            <ScrollArea.Autosize mah={220} type="auto">
              <Stack gap="xs">
                {drafts.map((candidate, index) => (
                  <div key={index} className={classes.cardDraftRow}>
                    <Button
                      className={classes.cardDraftButton}
                      color={draftError(index) && submitted ? 'red' : undefined}
                      justify="space-between"
                      type="button"
                      variant={index === activeIndex ? 'light' : 'subtle'}
                      onClick={() => {
                        setActiveIndex(index)
                      }}
                    >
                      <span
                        className={
                          submitted && draftError(index)
                            ? classes.draftWordInvalid
                            : undefined
                        }
                      >
                        {index + 1}.{' '}
                        {candidate.word.trim() || t('cards.untitledDraft')}
                      </span>
                      <Badge variant="light">
                        {t(`cards.${candidate.direction}`)}
                      </Badge>
                    </Button>
                    <ActionIcon
                      aria-label={t('cards.removeDraft')}
                      color="red"
                      disabled={drafts.length === 1}
                      variant="subtle"
                      onClick={() => removeDraft(index)}
                    >
                      ×
                    </ActionIcon>
                  </div>
                ))}
              </Stack>
            </ScrollArea.Autosize>
          </Paper>
          <Select
            allowDeselect={false}
            data={[
              { value: 'straight', label: t('cards.straight') },
              { value: 'reverse', label: t('cards.reverse') },
            ]}
            label={t('cards.direction')}
            value={draft.direction}
            onChange={(value) =>
              changeDraft({
                direction: (value ?? 'straight') as CardDirection,
              })
            }
          />
          <TextInput
            error={activeWordError}
            label={t('cards.word')}
            value={draft.word}
            onChange={(event) =>
              changeDraft({ word: event.currentTarget.value })
            }
          />
          <Paper p="md" withBorder>
            <StringListEditor
              addLabel={t('cards.addReading')}
              label={t('cards.readings')}
              showValidation={submitted}
              validationMessage={t('cards.readingRequired')}
              values={draft.readings}
              onChange={(readings) => changeDraft({ readings })}
            />
          </Paper>
          <MeaningsEditor
            meanings={draft.meanings}
            showValidation={submitted}
            onChange={(meanings) => changeDraft({ meanings })}
          />
          {submitted && draftError(activeIndex) && (
            <Alert color="red">{t(draftError(activeIndex)!)}</Alert>
          )}
          {create.isError && (
            <Alert color="red" title={t('cards.createError')}>
              {create.error.message}
            </Alert>
          )}
          <Group justify="center">
            <Tooltip
              label={`${t('cards.addAnother')} (${primaryModifierLabel()}+Enter)`}
            >
              <span>
                <Button
                  aria-keyshortcuts={primaryAriaShortcut('Enter')}
                  disabled={create.isPending}
                  type="button"
                  variant="light"
                  onClick={addDraft}
                >
                  {t('cards.addAnother')}
                </Button>
              </span>
            </Tooltip>
            <Tooltip
              label={`${t('cards.createCards')} (${primaryModifierLabel()}+S)`}
            >
              <span>
                <Button
                  aria-keyshortcuts={primaryAriaShortcut('S')}
                  loading={create.isPending}
                  type="submit"
                >
                  {t('cards.createCards')} ({drafts.length})
                </Button>
              </span>
            </Tooltip>
            <Button
              disabled={create.isPending}
              type="button"
              variant="default"
              onClick={() => {
                if (hasUnsavedDrafts) setDiscardOpened(true)
                else onCancel()
              }}
            >
              {t('cards.cancel')}
            </Button>
          </Group>
        </Stack>
        <Modal
          centered
          closeOnClickOutside={false}
          closeOnEscape={false}
          opened={createdCards !== null}
          title={t('cards.createInverseTitle')}
          withCloseButton={false}
          onClose={() => undefined}
        >
          <Stack>
            <Text>{t('cards.createInverseDescription')}</Text>
            {prepareInverse.isError && (
              <Alert color="red" title={t('cards.inversePrepareError')}>
                {prepareInverse.error.message}
              </Alert>
            )}
            <Group justify="flex-end">
              <Button
                loading={prepareInverse.isPending}
                onClick={() => {
                  if (createdCards) {
                    prepareInverse.mutate(createdCards.map((card) => card.id))
                  }
                }}
              >
                {t('cards.createInverse')}
              </Button>
              <Button
                disabled={prepareInverse.isPending}
                variant="default"
                onClick={() => void onCreated()}
              >
                {t('cards.skipInverse')}
              </Button>
            </Group>
          </Stack>
        </Modal>
      </form>
      <UnsavedChangesModal
        opened={discardOpened}
        onDiscard={onCancel}
        onKeepEditing={() => setDiscardOpened(false)}
      />
    </Box>
  )
}
