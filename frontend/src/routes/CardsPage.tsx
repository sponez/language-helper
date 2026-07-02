import {
  ActionIcon,
  Alert,
  Badge,
  Button,
  Collapse,
  Group,
  Loader,
  Modal,
  NumberInput,
  Paper,
  ScrollArea,
  Select,
  Stack,
  Text,
  TextInput,
  Title,
} from '@mantine/core'
import { useDebouncedValue, useDisclosure } from '@mantine/hooks'
import {
  useInfiniteQuery,
  useMutation,
  useQuery,
  useQueryClient,
} from '@tanstack/react-query'
import { type FormEvent, useRef, useState } from 'react'

import { useLanguageHelperClient } from '../api/LanguageHelperClientProvider'
import type {
  Card,
  CardDirection,
  CardMastery,
  CardMeaning,
  CardSortField,
  NewCardInput,
  SortDirection,
} from '../api/language-helper-client'
import { useTranslations } from '../locales/TranslationProvider'
import classes from './CardsPage.module.css'

interface CardsPageProps {
  username: string
  profileId: string
  masteryThreshold: number
  onBack(): void
}

type Screen = 'list' | 'show' | 'add'
type EditSection = 'word' | 'readings' | 'meanings' | null

function emptyMeaning(): CardMeaning {
  return {
    definition: '',
    translatedDefinition: '',
    wordTranslations: [''],
    examples: [],
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
  masteryThreshold,
  onBack,
}: CardsPageProps) {
  const client = useLanguageHelperClient()
  const queryClient = useQueryClient()
  const { t } = useTranslations()
  const viewport = useRef<HTMLDivElement>(null)
  const [screen, setScreen] = useState<Screen>('list')
  const [selectedCardId, setSelectedCardId] = useState<string | null>(null)
  const [search, setSearch] = useState('')
  const [debouncedSearch] = useDebouncedValue(search, 250)
  const [direction, setDirection] = useState<CardDirection | null>(null)
  const [mastery, setMastery] = useState<CardMastery>('any')
  const [maxStreak, setMaxStreak] = useState<number | null>(null)
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
    mastery,
    masteryThreshold,
    maxStreak,
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
        mastery,
        masteryThreshold,
        maxStreak,
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

  const summaries = cards.data?.pages.flatMap((page) => page.items) ?? []

  function resetFilters() {
    setSearch('')
    setDirection(null)
    setMastery('any')
    setMaxStreak(null)
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
          <Select
            allowDeselect={false}
            data={[
              { value: 'any', label: t('cards.anyMastery') },
              { value: 'learned', label: t('cards.learned') },
              { value: 'unlearned', label: t('cards.unlearned') },
            ]}
            label={t('cards.mastery')}
            value={mastery}
            onChange={(value) => setMastery((value ?? 'any') as CardMastery)}
          />
          <NumberInput
            allowDecimal={false}
            allowNegative={false}
            label={t('cards.maxStreak')}
            min={0}
            value={maxStreak ?? ''}
            onChange={(value) =>
              setMaxStreak(typeof value === 'number' ? value : null)
            }
          />
          <div className={classes.sortRow}>
            <Select
              allowDeselect={false}
              className={classes.sortSelect}
              data={[
                { value: 'word', label: t('cards.sortWord') },
                { value: 'createdAt', label: t('cards.sortCreated') },
                { value: 'streak', label: t('cards.sortStreak') },
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
                <Paper key={card.id} className={classes.cardRow} p="sm" withBorder>
                  <Badge variant="light">
                    {t(`cards.${card.direction}`)}
                  </Badge>
                  <Text className={classes.word} fw={500}>
                    {card.word}
                  </Text>
                  <Text c="dimmed" size="sm">
                    {card.streak}
                  </Text>
                  <Button
                    size="xs"
                    variant="default"
                    onClick={() => {
                      setSelectedCardId(card.id)
                      setScreen('show')
                    }}
                  >
                    {t('cards.show')}
                  </Button>
                  <Button
                    color="red"
                    size="xs"
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
        <Button w={140} onClick={() => setScreen('add')}>
          {t('cards.addNew')}
        </Button>
        <Button variant="default" w={140} onClick={onBack}>
          {t('cards.back')}
        </Button>
      </Group>

      <Modal
        centered
        opened={deleteOpened}
        title={t('cards.deleteTitle')}
        onClose={deleteModal.close}
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
            <Button variant="default" onClick={deleteModal.close}>
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
  const [openedExamples, setOpenedExamples] = useState<Set<number>>(
    () => new Set(),
  )

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
        word: editSection === 'word' ? word : current.word,
        readings: editSection === 'readings' ? readings : current.readings,
        meanings: editSection === 'meanings' ? meanings : current.meanings,
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
          word: editSection === 'word' ? word : current.word,
          readings: editSection === 'readings' ? readings : current.readings,
          meanings: editSection === 'meanings' ? meanings : current.meanings,
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
        <Group>
          <Badge>{t(`cards.${current.direction}`)}</Badge>
          <Badge variant="light">
            {t('cards.streak')}: {current.streak}
          </Badge>
        </Group>
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
        {editSection === 'word' ? (
          <Stack mt="md">
            <TextInput value={word} onChange={(event) => setWord(event.currentTarget.value)} />
            {editActions}
          </Stack>
        ) : (
          <Text fw={600} mt="md" size="xl" ta="center">
            {current.word}
          </Text>
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
        {editSection === 'readings' ? (
          <Stack mt="md">
            <StringListEditor
              addLabel={t('cards.addReading')}
              label={t('cards.readings')}
              values={readings}
              onChange={setReadings}
            />
            {editActions}
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
        {editSection === 'meanings' ? (
          <Stack mt="md">
            <MeaningsEditor meanings={meanings} onChange={setMeanings} />
            {editActions}
          </Stack>
        ) : (
          <Stack mt="md">
            {current.meanings.map((meaning, index) => {
              const examplesOpened = openedExamples.has(index)

              return (
                <Paper key={index} className={classes.meaning} p="md" withBorder>
                  <div className={classes.meaningLayout}>
                    <div className={classes.meaningIndex}>{index + 1}</div>
                    <Stack className={classes.meaningContent} gap="sm">
                      <div className={classes.meaningField}>
                        <Text c="dimmed" size="xs">
                          {t('cards.definition')}
                        </Text>
                        <Text>{meaning.definition}</Text>
                      </div>
                      {meaning.translatedDefinition && (
                        <div className={classes.meaningField}>
                          <Text c="dimmed" size="xs">
                            {t('cards.translatedDefinition')}
                          </Text>
                          <Text>{meaning.translatedDefinition}</Text>
                        </div>
                      )}
                      <div className={classes.meaningField}>
                        <Text c="dimmed" size="xs">
                          {t('cards.translations')}
                        </Text>
                        <Text size="lg">
                          {meaning.wordTranslations.join(', ')}
                        </Text>
                      </div>
                      {meaning.examples.length > 0 && (
                        <div>
                          <Button
                            className={`${classes.examplesToggle} ${
                              examplesOpened
                                ? classes.examplesToggleOpened
                                : ''
                            }`}
                            fullWidth
                            variant="light"
                            onClick={() =>
                              setOpenedExamples((opened) => {
                                const next = new Set(opened)
                                if (next.has(index)) {
                                  next.delete(index)
                                } else {
                                  next.add(index)
                                }
                                return next
                              })
                            }
                          >
                            {examplesOpened
                              ? t('cards.hideExamples')
                              : t('cards.showExamples')}{' '}
                            ({meaning.examples.length})
                          </Button>
                          <Collapse expanded={examplesOpened}>
                            <Stack
                              className={classes.examplesList}
                              gap="xs"
                            >
                              {meaning.examples.map(
                                (example, exampleIndex) => (
                                  <Paper
                                    key={exampleIndex}
                                    className={classes.example}
                                    p="sm"
                                    withBorder
                                  >
                                    <Text>{example.sentence}</Text>
                                    <Text c="dimmed" size="sm">
                                      {example.translation}
                                    </Text>
                                  </Paper>
                                ),
                              )}
                            </Stack>
                          </Collapse>
                        </div>
                      )}
                    </Stack>
                  </div>
                </Paper>
              )
            })}
          </Stack>
        )}
      </Paper>

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

function AddCard({
  username,
  profileId,
  onCancel,
  onCreated,
}: {
  username: string
  profileId: string
  onCancel(): void
  onCreated(): Promise<void>
}) {
  const client = useLanguageHelperClient()
  const { t } = useTranslations()
  const [direction, setDirection] = useState<CardDirection>('straight')
  const [word, setWord] = useState('')
  const [readings, setReadings] = useState<string[]>([])
  const [meanings, setMeanings] = useState<CardMeaning[]>([emptyMeaning()])
  const [submitted, setSubmitted] = useState(false)

  const create = useMutation({
    mutationFn: (card: NewCardInput) =>
      client.createCards({ username, profileId, cards: [card] }),
    onSuccess: onCreated,
  })
  const draft: NewCardInput = { direction, word, readings, meanings }
  const validationError = validateCard(draft)

  function submit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault()
    setSubmitted(true)
    if (!validationError) create.mutate(draft)
  }

  return (
    <form className={classes.details} onSubmit={submit}>
      <Stack gap="md">
        <Title order={2} ta="center">
          {t('cards.addTitle')}
        </Title>
        <Select
          allowDeselect={false}
          data={[
            { value: 'straight', label: t('cards.straight') },
            { value: 'reverse', label: t('cards.reverse') },
          ]}
          label={t('cards.direction')}
          value={direction}
          onChange={(value) =>
            setDirection((value ?? 'straight') as CardDirection)
          }
        />
        <TextInput
          label={t('cards.word')}
          value={word}
          onChange={(event) => setWord(event.currentTarget.value)}
        />
        <Paper p="md" withBorder>
          <StringListEditor
            addLabel={t('cards.addReading')}
            label={t('cards.readings')}
            values={readings}
            onChange={setReadings}
          />
        </Paper>
        <MeaningsEditor meanings={meanings} onChange={setMeanings} />
        {submitted && validationError && (
          <Alert color="red">{t(validationError)}</Alert>
        )}
        {create.isError && (
          <Alert color="red" title={t('cards.createError')}>
            {create.error.message}
          </Alert>
        )}
        <Group justify="center">
          <Button loading={create.isPending} type="submit" w={140}>
            {t('cards.create')}
          </Button>
          <Button
            disabled={create.isPending}
            type="button"
            variant="default"
            w={140}
            onClick={onCancel}
          >
            {t('cards.cancel')}
          </Button>
        </Group>
      </Stack>
    </form>
  )
}
