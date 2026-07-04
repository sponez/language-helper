import {
  ActionIcon,
  Alert,
  Badge,
  Button,
  Group,
  Loader,
  Modal,
  NumberInput,
  Paper,
  ScrollArea,
  Select,
  Stack,
  Tabs,
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
import {
  ReadOnlyCard,
  ReadOnlyMeanings,
} from '../components/ReadOnlyCard'
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

function AiNormalizeButton({
  username,
  profileId,
  card,
  onApply,
}: {
  username: string
  profileId: string
  card: NewCardInput
  onApply(card: NewCardInput): void
}) {
  const client = useLanguageHelperClient()
  const { t } = useTranslations()
  const [proposed, setProposed] = useState<NewCardInput | null>(null)
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
    onSuccess: setProposed,
  })

  return (
    <>
      <Tooltip disabled={!disabled} label={disabledHint} multiline w={280}>
        <span>
          <Button
            disabled={disabled}
            loading={normalize.isPending}
            size="xs"
            variant="light"
            onClick={() => normalize.mutate()}
          >
            ✦ {t('cards.normalizeWithAi')}
          </Button>
        </span>
      </Tooltip>
      {normalize.isError && (
        <Alert color="red" title={t('cards.aiError')}>
          {normalize.error.message}
        </Alert>
      )}
      <Modal
        centered
        opened={proposed !== null}
        size="xl"
        title={t('cards.aiPreviewTitle')}
        onClose={() => setProposed(null)}
      >
        <Stack>
          <Text c="dimmed">{t('cards.aiPreviewDescription')}</Text>
          <Tabs defaultValue="after">
            <Tabs.List grow>
              <Tabs.Tab value="before">{t('cards.aiBefore')}</Tabs.Tab>
              <Tabs.Tab value="after">{t('cards.aiAfter')}</Tabs.Tab>
            </Tabs.List>
            <Tabs.Panel value="before" pt="md">
              <ReadOnlyCard card={card} />
            </Tabs.Panel>
            <Tabs.Panel value="after" pt="md">
              {proposed && <ReadOnlyCard card={proposed} />}
            </Tabs.Panel>
          </Tabs>
          <Group justify="flex-end">
            <Button
              onClick={() => {
                if (proposed) onApply(proposed)
                setProposed(null)
              }}
            >
              {t('cards.aiApply')}
            </Button>
            <Button variant="default" onClick={() => setProposed(null)}>
              {t('cards.cancel')}
            </Button>
          </Group>
        </Stack>
      </Modal>
    </>
  )
}

function StringListEditor({
  label,
  values,
  addLabel,
  onChange,
}: {
  label: string
  values: string[]
  addLabel: string
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
              onChange(values.filter((_, currentIndex) => currentIndex !== index))
            }
          >
            −
          </ActionIcon>
        </Group>
      ))}
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
  onChange,
}: {
  meanings: CardMeaning[]
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
      {meanings.map((meaning, meaningIndex) => (
        <Paper key={meaningIndex} className={classes.meaning} p="md" withBorder>
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
      ))}
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

export function CardsPage({
  username,
  profileId,
  onBack,
}: CardsPageProps) {
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
      await queryClient.invalidateQueries({ queryKey: ['cards', username, profileId] })
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
      if (event.key === 'Enter' && event.ctrlKey) {
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
            : Math.min(
                summaries.length - 1,
                Math.max(0, currentIndex + offset),
              )
        setCursorCardId(summaries[nextIndex].id)
        return
      }
      if (
        event.key === 'Enter' &&
        currentIndex >= 0
      ) {
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
                  <Badge variant="light">
                    {t(`cards.${card.direction}`)}
                  </Badge>
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
        <Button tabIndex={-1} w={140} onClick={() => setScreen('add')}>
          {t('cards.addNew')}
        </Button>
        <Button tabIndex={-1} variant="default" w={140} onClick={onBack}>
          {t('cards.back')}
        </Button>
      </Group>
      <Text c="dimmed" size="xs" ta="center">
        {t('cards.catalogKeyboardHint')}
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
        word: editSection === 'word' || editSection === 'all' ? word : current.word,
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
      await queryClient.invalidateQueries({
        queryKey: ['cards', username, profileId],
      })
    },
  })

  function beginEdit(section: Exclude<EditSection, null>, current: Card) {
    setWord(current.word)
    setReadings([...current.readings])
    setMeanings(structuredClone(current.meanings))
    update.reset()
    setEditSection(section)
  }

  useEffect(() => {
    const handleKeyDown = (event: KeyboardEvent) => {
      if (hasOpenDialog()) return
      if (event.key === 'Escape') {
        event.preventDefault()
        if (editSection === null) onBack()
        else setEditSection(null)
      } else if (
        event.key === 'Enter' &&
        editSection === null &&
        !(event.target instanceof HTMLElement &&
          event.target.closest('button, a, input, textarea, select'))
      ) {
        event.preventDefault()
        onBack()
      }
    }

    window.addEventListener('keydown', handleKeyDown)
    return () => window.removeEventListener('keydown', handleKeyDown)
  }, [editSection, onBack])

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
  const validationError =
    editSection === null
      ? null
      : validateCard({
          direction: current.direction,
          word:
            editSection === 'word' || editSection === 'all'
              ? word
              : current.word,
          readings:
            editSection === 'readings' || editSection === 'all'
              ? readings
              : current.readings,
          meanings:
            editSection === 'meanings' || editSection === 'all'
              ? meanings
              : current.meanings,
        })

  const editActions = (
    <Group justify="flex-end">
      <Button
        disabled={Boolean(validationError)}
        loading={update.isPending}
        size="xs"
        onClick={() => update.mutate(current)}
      >
        {t('cards.save')}
      </Button>
      <Button
        disabled={update.isPending}
        size="xs"
        variant="default"
        onClick={() => setEditSection(null)}
      >
        {t('cards.cancel')}
      </Button>
    </Group>
  )

  return (
    <Stack className={classes.details} gap="md">
      <Group justify="space-between">
        <Button variant="subtle" onClick={onBack}>
          ← {t('cards.back')}
        </Button>
        <AiNormalizeButton
          card={{
            direction: current.direction,
            word: current.word,
            readings: current.readings,
            meanings: current.meanings,
          }}
          profileId={profileId}
          username={username}
          onApply={(normalized) => {
            setWord(normalized.word)
            setReadings(normalized.readings)
            setMeanings(normalized.meanings)
            setEditSection('all')
          }}
        />
      </Group>
      <Group justify="center">
        <Badge>{t(`cards.${current.direction}`)}</Badge>
        <Badge variant="light">
          {t('cards.score')}: {current.score}
        </Badge>
      </Group>
      <Text c="dimmed" size="sm" ta="center">
        {t('cards.createdAt')}:{' '}
        {new Date(current.createdAt).toLocaleString()}
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
            <TextInput value={word} onChange={(event) => setWord(event.currentTarget.value)} />
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
                setEditSection(null)
                void card.refetch()
              }}
            >
              {t('cards.reload')}
            </Button>
          </Stack>
        </Alert>
      )}
    </Stack>
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
  const [submitted, setSubmitted] = useState(false)

  const saveOne = useMutation({
    mutationFn: (card: PendingInverseCard) =>
      client.saveInverseCards({ username, profileId, cards: [card] }),
    onSuccess: async (_, saved) => {
      const remaining = pending.filter(
        (candidate) => candidate.card.id !== saved.card.id,
      )
      setPending(remaining)
      setEditing(null)
      if (remaining.length === 0) await onDone()
    },
  })
  const saveAll = useMutation({
    mutationFn: () =>
      client.saveInverseCards({ username, profileId, cards: pending }),
    onSuccess: onDone,
  })

  const draft = editing?.card
  const validationError = draft
    ? validateCard({
        direction: draft.direction,
        word: draft.word,
        readings: draft.readings,
        meanings: draft.meanings,
      })
    : null

  useEffect(() => {
    const handleKeyDown = (event: KeyboardEvent) => {
      if (event.key !== 'Escape' || hasOpenDialog()) return
      event.preventDefault()
      void onDone()
    }

    window.addEventListener('keydown', handleKeyDown)
    return () => window.removeEventListener('keydown', handleKeyDown)
  }, [onDone])

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
            <Button
              key={candidate.card.id}
              className={classes.inverseCardButton}
              justify="space-between"
              variant="default"
              onClick={() => {
                setSubmitted(false)
                saveOne.reset()
                setEditing(structuredClone(candidate))
              }}
            >
              <span>{candidate.card.word}</span>
              <Badge variant="light">
                {candidate.expectedVersion === null
                  ? t('cards.inverseCreated')
                  : t('cards.inverseUpdated')}
              </Badge>
            </Button>
          ))}
        </Stack>
      </ScrollArea>
      {saveAll.isError && (
        <Alert color="red" title={t('cards.inverseSaveError')}>
          {saveAll.error.message}
        </Alert>
      )}
      <Group justify="center">
        <Button
          loading={saveAll.isPending}
          onClick={() => saveAll.mutate()}
        >
          {t('cards.saveWithoutReview')}
        </Button>
        <Button
          disabled={saveAll.isPending}
          variant="default"
          onClick={() => void onDone()}
        >
          {t('cards.cancel')}
        </Button>
      </Group>

      <Modal
        opened={editing !== null}
        size="lg"
        title={t('cards.editInverseTitle')}
        onClose={() => setEditing(null)}
      >
        {editing && draft && (
          <Stack>
            <Group justify="space-between">
              <Badge>{t(`cards.${draft.direction}`)}</Badge>
              <AiNormalizeButton
                card={{
                  direction: draft.direction,
                  word: draft.word,
                  readings: draft.readings,
                  meanings: draft.meanings,
                }}
                profileId={profileId}
                username={username}
                onApply={(normalized) =>
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
                }
              />
            </Group>
            <TextInput
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
                onClick={() => setEditing(null)}
              >
                {t('cards.cancel')}
              </Button>
            </Group>
          </Stack>
        )}
      </Modal>
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
  const [activeIndex, setActiveIndex] = useState(0)
  const [submitted, setSubmitted] = useState(false)
  const [createdCards, setCreatedCards] = useState<Card[] | null>(null)

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

  function draftError(index: number) {
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
  }

  function changeDraft(changes: Partial<NewCardInput>) {
    setDrafts((current) =>
      current.map((candidate, index) =>
        index === activeIndex ? { ...candidate, ...changes } : candidate,
      ),
    )
    create.reset()
  }

  function addDraft() {
    if (draftError(activeIndex)) {
      setSubmitted(true)
      return
    }
    setDrafts((current) => [...current, emptyCard()])
    setActiveIndex(drafts.length)
    setSubmitted(false)
  }

  function removeDraft(index: number) {
    if (drafts.length === 1) return
    setDrafts((current) =>
      current.filter((_, candidateIndex) => candidateIndex !== index),
    )
    setActiveIndex((current) => {
      if (current > index) return current - 1
      if (current === index) return Math.max(0, current - 1)
      return current
    })
    setSubmitted(false)
    create.reset()
  }

  function submit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault()
    setSubmitted(true)
    const invalidIndex = drafts.findIndex((_, index) => draftError(index))
    if (invalidIndex >= 0) {
      setActiveIndex(invalidIndex)
      return
    }
    create.mutate(drafts)
  }

  useEffect(() => {
    const handleKeyDown = (event: KeyboardEvent) => {
      if (
        event.key !== 'Escape' ||
        createdCards !== null ||
        hasOpenDialog()
      ) {
        return
      }
      event.preventDefault()
      onCancel()
    }

    window.addEventListener('keydown', handleKeyDown)
    return () => window.removeEventListener('keydown', handleKeyDown)
  }, [createdCards, onCancel])

  return (
    <form className={classes.details} onSubmit={submit}>
      <Stack gap="md">
        <div className={classes.addHeader}>
          <Title order={2} ta="center">
            {t('cards.addTitle')}
          </Title>
          <div className={classes.addHeaderAction}>
            <AiNormalizeButton
              card={draft}
              profileId={profileId}
              username={username}
              onApply={(normalized) => changeDraft(normalized)}
            />
          </div>
        </div>
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
                      setSubmitted(false)
                    }}
                  >
                    <span>
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
          label={t('cards.word')}
          value={draft.word}
          onChange={(event) => changeDraft({ word: event.currentTarget.value })}
        />
        <Paper p="md" withBorder>
          <StringListEditor
            addLabel={t('cards.addReading')}
            label={t('cards.readings')}
            values={draft.readings}
            onChange={(readings) => changeDraft({ readings })}
          />
        </Paper>
        <MeaningsEditor
          meanings={draft.meanings}
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
          <Button
            disabled={Boolean(draftError(activeIndex)) || create.isPending}
            type="button"
            variant="light"
            onClick={addDraft}
          >
            {t('cards.addAnother')}
          </Button>
          <Button loading={create.isPending} type="submit">
            {t('cards.createCards')} ({drafts.length})
          </Button>
          <Button
            disabled={create.isPending}
            type="button"
            variant="default"
            onClick={onCancel}
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
  )
}
