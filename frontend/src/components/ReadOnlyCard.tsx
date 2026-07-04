import {
  Badge,
  Button,
  Collapse,
  Group,
  Paper,
  Stack,
  Text,
  Title,
} from '@mantine/core'
import { type ReactNode, useMemo, useState } from 'react'

import type {
  CardMeaning,
  NewCardInput,
} from '../api/language-helper-client'
import { useTranslations } from '../locales/TranslationProvider'
import classes from './ReadOnlyCard.module.css'

interface ReadOnlyMeaningsProps {
  meanings: CardMeaning[]
  meaningIndices?: number[]
  highlightedMeaningIndices?: number[]
}

export function ReadOnlyMeanings({
  meanings,
  meaningIndices,
  highlightedMeaningIndices = [],
}: ReadOnlyMeaningsProps) {
  const { t } = useTranslations()
  const [openedExamples, setOpenedExamples] = useState<Set<number>>(
    () => new Set(),
  )
  const highlighted = useMemo(
    () => new Set(highlightedMeaningIndices),
    [highlightedMeaningIndices],
  )

  return (
    <Stack>
      {meanings.map((meaning, localIndex) => {
        const originalIndex = meaningIndices?.[localIndex] ?? localIndex
        const examplesOpened = openedExamples.has(originalIndex)
        const isHighlighted = highlighted.has(originalIndex)

        return (
          <Paper
            key={originalIndex}
            className={`${classes.meaning} ${
              isHighlighted ? classes.meaningHighlighted : ''
            }`}
            p="md"
            withBorder
          >
            <div className={classes.meaningLayout}>
              <div
                className={`${classes.meaningIndex} ${
                  isHighlighted ? classes.meaningIndexHighlighted : ''
                }`}
              >
                {originalIndex + 1}
              </div>
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
                        examplesOpened ? classes.examplesToggleOpened : ''
                      }`}
                      fullWidth
                      variant="light"
                      onClick={() =>
                        setOpenedExamples((opened) => {
                          const next = new Set(opened)
                          if (next.has(originalIndex)) {
                            next.delete(originalIndex)
                          } else {
                            next.add(originalIndex)
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
                      <Stack className={classes.examplesList} gap="xs">
                        {meaning.examples.map((example, exampleIndex) => (
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
                        ))}
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
  )
}

export function ReadOnlyCard({
  card,
  wordActions,
  highlightedMeaningIndices,
}: {
  card: NewCardInput
  wordActions?: ReactNode
  highlightedMeaningIndices?: number[]
}) {
  const { t } = useTranslations()

  return (
    <Stack className={classes.previewCard} gap="md">
      <Group justify="center">
        <Badge>{t(`cards.${card.direction}`)}</Badge>
      </Group>
      <Paper p="lg" withBorder>
        <Title order={3}>{t('cards.word')}</Title>
        <Text fw={600} mt="md" size="xl" ta="center">
          {card.word}
        </Text>
        {wordActions && (
          <Group justify="center" mt="md">
            {wordActions}
          </Group>
        )}
      </Paper>
      <Paper p="lg" withBorder>
        <Title order={3}>{t('cards.readings')}</Title>
        <Text c={card.readings.length ? undefined : 'dimmed'} mt="md">
          {card.readings.length
            ? card.readings.join(', ')
            : t('cards.noReadings')}
        </Text>
      </Paper>
      <Paper p="lg" withBorder>
        <Title order={3}>{t('cards.meanings')}</Title>
        <ReadOnlyMeanings
          highlightedMeaningIndices={highlightedMeaningIndices}
          meanings={card.meanings}
        />
      </Paper>
    </Stack>
  )
}
