import {
  ActionIcon,
  Alert,
  Button,
  Group,
  Loader,
  Modal,
  Select,
  Stack,
  Text,
  TextInput,
  useMantineColorScheme,
} from '@mantine/core'
import { useDisclosure } from '@mantine/hooks'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { type FormEvent, useState } from 'react'

import { useLanguageHelperClient } from '../api/LanguageHelperClientProvider'
import { useTranslations } from '../locales/TranslationProvider'
import classes from './HomePage.module.css'

const CONTROL_CHARACTER_PATTERN = /\p{Cc}/u

type UsernameError =
  | 'home.usernameRequired'
  | 'home.usernameTooLong'
  | 'home.usernameInvalid'

function validateUsername(username: string): UsernameError | null {
  const normalizedUsername = username.trim()

  if (normalizedUsername.length === 0) {
    return 'home.usernameRequired'
  }

  if ([...normalizedUsername].length > 50) {
    return 'home.usernameTooLong'
  }

  if (CONTROL_CHARACTER_PATTERN.test(normalizedUsername)) {
    return 'home.usernameInvalid'
  }

  return null
}

export function HomePage() {
  const client = useLanguageHelperClient()
  const queryClient = useQueryClient()
  const { t } = useTranslations()
  const { colorScheme, setColorScheme } = useMantineColorScheme()
  const [modalOpened, modal] = useDisclosure(false)
  const [selectedUser, setSelectedUser] = useState<string | null>(null)
  const [username, setUsername] = useState('')
  const [submitted, setSubmitted] = useState(false)

  const users = useQuery({
    queryKey: ['users'],
    queryFn: () => client.getUsernames(),
    retry: false,
  })

  const createUser = useMutation({
    mutationFn: (newUsername: string) => client.createUser(newUsername),
    onSuccess: async (createdUsername) => {
      await queryClient.invalidateQueries({ queryKey: ['users'] })
      setSelectedUser(createdUsername)
      setUsername('')
      setSubmitted(false)
      modal.close()
    },
  })

  const usernameError = validateUsername(username)

  function closeCreateUser() {
    modal.close()
    setUsername('')
    setSubmitted(false)
    createUser.reset()
  }

  function submitCreateUser(event: FormEvent<HTMLFormElement>) {
    event.preventDefault()
    setSubmitted(true)

    if (usernameError) {
      return
    }

    createUser.mutate(username.trim())
  }

  const userOptions = (users.data ?? []).map((user) => ({
    value: user,
    label: user,
  }))

  return (
    <main className={classes.page}>
      <Group className={classes.toolbar} gap={10}>
        <Select
          aria-label={t('home.theme')}
          allowDeselect={false}
          className={classes.settingSelect}
          data={[
            { value: 'dark', label: 'Dark' },
            { value: 'light', label: 'Light' },
          ]}
          value={colorScheme === 'auto' ? 'dark' : colorScheme}
          onChange={(value) => {
            if (value === 'dark' || value === 'light') {
              setColorScheme(value)
            }
          }}
        />
        <Select
          aria-label={t('home.language')}
          allowDeselect={false}
          className={classes.settingSelect}
          data={['English']}
          value="English"
          disabled
        />
      </Group>

      <Group className={classes.userPicker} gap={10} wrap="nowrap">
        <Select
          aria-label={t('home.selectUser')}
          className={classes.userSelect}
          data={userOptions}
          value={selectedUser}
          placeholder={
            users.isPending ? t('home.loadingUsers') : t('home.selectUser')
          }
          nothingFoundMessage={t('home.noUsers')}
          disabled={users.isPending || users.isError}
          rightSection={users.isPending ? <Loader size={16} /> : undefined}
          searchable
          onChange={setSelectedUser}
        />
        <ActionIcon
          aria-label={t('home.addUser')}
          className={classes.addButton}
          size={36}
          variant="default"
          onClick={modal.open}
        >
          <Text component="span" size="lg" lh={1}>
            +
          </Text>
        </ActionIcon>
      </Group>

      {users.isError && (
        <Alert
          className={classes.loadError}
          color="red"
          title={t('home.loadError')}
        >
          {users.error.message}
        </Alert>
      )}

      <Modal
        centered
        closeOnClickOutside={!createUser.isPending}
        closeOnEscape={!createUser.isPending}
        opened={modalOpened}
        overlayProps={{ backgroundOpacity: 0.5, blur: 0 }}
        radius={10}
        size={400}
        title={t('home.createTitle')}
        onClose={closeCreateUser}
      >
        <form onSubmit={submitCreateUser}>
          <Stack gap={20}>
            <TextInput
              autoFocus
              label={t('home.username')}
              description={t('home.usernameHint')}
              maxLength={50}
              placeholder={t('home.usernamePlaceholder')}
              value={username}
              error={submitted && usernameError ? t(usernameError) : undefined}
              onChange={(event) => {
                setUsername(event.currentTarget.value)
                createUser.reset()
              }}
            />

            {createUser.isError && (
              <Alert color="red" title={t('home.createError')}>
                {createUser.error.message}
              </Alert>
            )}

            <Group justify="center" gap={10}>
              <Button
                loading={createUser.isPending}
                type="submit"
                w={120}
              >
                {t('home.create')}
              </Button>
              <Button
                disabled={createUser.isPending}
                type="button"
                variant="default"
                w={120}
                onClick={closeCreateUser}
              >
                {t('home.cancel')}
              </Button>
            </Group>
          </Stack>
        </form>
      </Modal>
    </main>
  )
}
