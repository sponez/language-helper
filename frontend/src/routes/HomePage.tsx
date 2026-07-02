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
import type { CreateLanguageProfileInput } from '../api/language-helper-client'
import { useTranslations } from '../locales/TranslationProvider'
import classes from './HomePage.module.css'

const CONTROL_CHARACTER_PATTERN = /\p{Cc}/u
const LANGUAGES = [
  { value: 'en-US', label: 'English' },
  { value: 'ru-RU', label: 'Russian' },
  { value: 'ja-JP', label: 'Japanese' },
]

const LANGUAGE_LABELS = Object.fromEntries(
  LANGUAGES.map((language) => [language.value, language.label]),
)

type UsernameError =
  | 'home.usernameRequired'
  | 'home.usernameTooLong'
  | 'home.usernameInvalid'

type ProfileNameError =
  | 'home.profileNameRequired'
  | 'home.profileNameTooLong'
  | 'home.profileNameInvalid'

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

function validateProfileName(name: string): ProfileNameError | null {
  const normalizedName = name.trim()
  if (normalizedName.length === 0) {
    return 'home.profileNameRequired'
  }
  if ([...normalizedName].length > 50) {
    return 'home.profileNameTooLong'
  }
  if (CONTROL_CHARACTER_PATTERN.test(normalizedName)) {
    return 'home.profileNameInvalid'
  }
  return null
}

export function HomePage() {
  const client = useLanguageHelperClient()
  const queryClient = useQueryClient()
  const { t } = useTranslations()
  const { colorScheme, setColorScheme } = useMantineColorScheme()
  const [userModalOpened, userModal] = useDisclosure(false)
  const [profileModalOpened, profileModal] = useDisclosure(false)
  const [selectedUser, setSelectedUser] = useState<string | null>(null)
  const [selectedProfile, setSelectedProfile] = useState<string | null>(null)
  const [username, setUsername] = useState('')
  const [submitted, setSubmitted] = useState(false)
  const [profileName, setProfileName] = useState('')
  const [sourceLanguage, setSourceLanguage] = useState<string | null>(null)
  const [targetLanguage, setTargetLanguage] = useState<string | null>(null)
  const [profileSubmitted, setProfileSubmitted] = useState(false)

  const users = useQuery({
    queryKey: ['users'],
    queryFn: () => client.getUsernames(),
    retry: false,
  })

  const profiles = useQuery({
    queryKey: ['language-profiles', selectedUser],
    queryFn: () => client.getLanguageProfiles(selectedUser!),
    enabled: selectedUser !== null,
    retry: false,
  })

  const createUser = useMutation({
    mutationFn: (newUsername: string) => client.createUser(newUsername),
    onSuccess: async (createdUsername) => {
      await queryClient.invalidateQueries({ queryKey: ['users'] })
      setSelectedUser(createdUsername)
      setUsername('')
      setSubmitted(false)
      userModal.close()
    },
  })

  const createProfile = useMutation({
    mutationFn: (input: CreateLanguageProfileInput) =>
      client.createLanguageProfile(input),
    onSuccess: async (createdProfile) => {
      await queryClient.invalidateQueries({
        queryKey: ['language-profiles', selectedUser],
      })
      setSelectedProfile(createdProfile.id)
      setProfileName('')
      setSourceLanguage(null)
      setTargetLanguage(null)
      setProfileSubmitted(false)
      profileModal.close()
    },
  })

  const usernameError = validateUsername(username)
  const profileNameError = validateProfileName(profileName)
  const languagesMatch =
    sourceLanguage !== null && sourceLanguage === targetLanguage

  function closeCreateUser() {
    userModal.close()
    setUsername('')
    setSubmitted(false)
    createUser.reset()
  }

  function selectUser(user: string | null) {
    setSelectedUser(user)
    setSelectedProfile(null)
    createProfile.reset()
  }

  function closeCreateProfile() {
    profileModal.close()
    setProfileName('')
    setSourceLanguage(null)
    setTargetLanguage(null)
    setProfileSubmitted(false)
    createProfile.reset()
  }

  function submitCreateProfile(event: FormEvent<HTMLFormElement>) {
    event.preventDefault()
    setProfileSubmitted(true)

    if (
      !selectedUser ||
      profileNameError ||
      !sourceLanguage ||
      !targetLanguage ||
      languagesMatch
    ) {
      return
    }

    createProfile.mutate({
      username: selectedUser,
      name: profileName.trim(),
      sourceLanguage,
      targetLanguage,
    })
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

  const profileOptions = (profiles.data ?? []).map((profile) => ({
    value: profile.id,
    label: `${profile.name} · ${LANGUAGE_LABELS[profile.sourceLanguage]} → ${
      LANGUAGE_LABELS[profile.targetLanguage]
    }`,
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

      <Stack className={classes.pickers} gap={10}>
        <Group className={classes.pickerRow} gap={10} wrap="nowrap">
          <Select
            aria-label={t('home.selectUser')}
            className={classes.pickerSelect}
            data={userOptions}
            value={selectedUser}
            placeholder={
              users.isPending ? t('home.loadingUsers') : t('home.selectUser')
            }
            nothingFoundMessage={t('home.noUsers')}
            disabled={users.isPending || users.isError}
            rightSection={users.isPending ? <Loader size={16} /> : undefined}
            searchable
            onChange={selectUser}
          />
          <ActionIcon
            aria-label={t('home.addUser')}
            className={classes.addButton}
            size={36}
            variant="default"
            onClick={userModal.open}
          >
            <Text component="span" size="lg" lh={1}>
              +
            </Text>
          </ActionIcon>
        </Group>

        <Group className={classes.pickerRow} gap={10} wrap="nowrap">
          <Select
            aria-label={t('home.selectProfile')}
            className={classes.pickerSelect}
            data={profileOptions}
            value={selectedProfile}
            placeholder={
              profiles.isFetching
                ? t('home.loadingProfiles')
                : t('home.selectProfile')
            }
            nothingFoundMessage={t('home.noProfiles')}
            disabled={!selectedUser || profiles.isFetching || profiles.isError}
            rightSection={
              profiles.isFetching ? <Loader size={16} /> : undefined
            }
            searchable
            onChange={setSelectedProfile}
          />
          <ActionIcon
            aria-label={t('home.addProfile')}
            className={classes.addButton}
            size={36}
            variant="default"
            disabled={!selectedUser || profiles.isFetching || profiles.isError}
            onClick={profileModal.open}
          >
            <Text component="span" size="lg" lh={1}>
              +
            </Text>
          </ActionIcon>
        </Group>
      </Stack>

      {users.isError && (
        <Alert
          className={classes.loadError}
          color="red"
          title={t('home.loadError')}
        >
          {users.error.message}
        </Alert>
      )}

      {profiles.isError && selectedUser && (
        <Alert
          className={classes.loadError}
          color="red"
          title={t('home.profileLoadError')}
        >
          {profiles.error.message}
        </Alert>
      )}

      <Modal
        centered
        closeOnClickOutside={!createUser.isPending}
        closeOnEscape={!createUser.isPending}
        opened={userModalOpened}
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

      <Modal
        centered
        closeOnClickOutside={!createProfile.isPending}
        closeOnEscape={!createProfile.isPending}
        opened={profileModalOpened}
        overlayProps={{ backgroundOpacity: 0.5, blur: 0 }}
        radius={10}
        size={400}
        title={t('home.createProfileTitle')}
        onClose={closeCreateProfile}
      >
        <form onSubmit={submitCreateProfile}>
          <Stack gap={20}>
            <TextInput
              autoFocus
              label={t('home.profileName')}
              maxLength={50}
              placeholder={t('home.profileNamePlaceholder')}
              value={profileName}
              error={
                profileSubmitted && profileNameError
                  ? t(profileNameError)
                  : undefined
              }
              onChange={(event) => {
                setProfileName(event.currentTarget.value)
                createProfile.reset()
              }}
            />
            <Select
              label={t('home.sourceLanguage')}
              placeholder={t('home.selectLanguage')}
              data={LANGUAGES}
              value={sourceLanguage}
              error={
                profileSubmitted && !sourceLanguage
                  ? t('home.selectLanguage')
                  : undefined
              }
              searchable
              onChange={(value) => {
                setSourceLanguage(value)
                createProfile.reset()
              }}
            />
            <Select
              label={t('home.targetLanguage')}
              placeholder={t('home.selectLanguage')}
              data={LANGUAGES}
              value={targetLanguage}
              error={
                profileSubmitted && !targetLanguage
                  ? t('home.selectLanguage')
                  : profileSubmitted && languagesMatch
                    ? t('home.languagesMustDiffer')
                    : undefined
              }
              searchable
              onChange={(value) => {
                setTargetLanguage(value)
                createProfile.reset()
              }}
            />

            {createProfile.isError && (
              <Alert color="red" title={t('home.createProfileError')}>
                {createProfile.error.message}
              </Alert>
            )}

            <Group justify="center" gap={10}>
              <Button loading={createProfile.isPending} type="submit" w={120}>
                {t('home.create')}
              </Button>
              <Button
                disabled={createProfile.isPending}
                type="button"
                variant="default"
                w={120}
                onClick={closeCreateProfile}
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
