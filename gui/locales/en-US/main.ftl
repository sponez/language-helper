# Language Helper - English (US) Localization

## User List Screen
user-list-title = Select User
user-list-select-placeholder = Select a username...
user-list-add-new = + Add new user
user-list-enter-username = Enter new username:
user-list-username-placeholder = Enter username...
user-list-ok-button = OK
user-list-exit-button = Exit

## User Screen
user-account-title = Account: { $username }
user-settings-button = Settings
user-profiles-button = Profiles
user-back-button = Back

## User Settings Screen
user-settings-language-label = User Language:
user-settings-theme-label = Theme:
user-settings-delete-button = Delete User
user-settings-delete-warning = Are you sure you want to delete this user? This action cannot be undone.
user-settings-delete-yes = Yes
user-settings-delete-no = No

## Profile List Screen
profile-list-title = Learning Profiles
profile-list-create-new = + Create New Profile
profile-list-select-language = Select target language for new profile:
profile-list-create-button = Create
profile-list-cancel-button = Cancel

## Profile Screen
profile-cards-button = Cards
profile-explain-ai-button = Explain with AI
profile-chat-ai-button = Chat with AI
profile-settings-button = Settings
profile-back-button = Back
profile-back-where = Where would you like to go?
profile-back-to-profiles = To Profile Selection
profile-back-to-user = To User Selection
profile-exit = Exit

## Profile Settings Screen
profile-settings-title = Profile settings
profile-settings-cards-per-set = Cards per set:
profile-settings-test-method = Test answer method:
profile-settings-test-method-manual = Manual text entry
profile-settings-test-method-self = Self review
profile-settings-streak-length = Streak length for remembering:
profile-settings-assistant-settings-button = Assistant settings
profile-settings-save = Save
profile-settings-delete-profile = Delete profile
profile-settings-delete-warning = Are you sure you want to delete this profile? All cards and progress will be permanently lost.
profile-settings-delete-confirm = Yes, delete
profile-settings-delete-cancel = No, keep it
profile-settings-back = Back
profile-settings-saved = Settings saved successfully

## Assistant Settings Screen
assistant-settings-title = Assistant Settings
assistant-settings-model-label = Model Strength:
assistant-settings-tiny = Tiny (1B)
assistant-settings-light = Light (3-4B)
assistant-settings-weak = Weak (3B)
assistant-settings-medium = Medium (7B)
assistant-settings-strong = Strong (14B)
assistant-settings-api = API
assistant-settings-requirements-title = System Requirements:
assistant-settings-incompatible = This model cannot run on your system due to missing requirements.
assistant-settings-no-data = System requirements information not available.
assistant-settings-ollama-status = Ollama Status:
assistant-settings-ollama-not-installed = Ollama is not installed. To install, go to ollama.com
assistant-settings-ollama-installed = Ollama is installed
assistant-settings-api-endpoint = API Endpoint:
assistant-settings-api-key = API Key:
assistant-settings-api-model = Model Name:
assistant-settings-start-assistant = Start Assistant
assistant-settings-stop-assistant = Stop Assistant
assistant-settings-change-assistant = Change Assistant
assistant-settings-save-api = Save API Config
assistant-settings-back = Back

## Error Messages
error-username-empty = Username cannot be empty
error-user-not-found = User not found
error-create-user = Error creating user: { $error }
error-update-theme = Failed to update theme: { $error }
error-update-language = Failed to update language: { $error }
error-user-created-not-found = User created but not found
error-invalid-number = Please enter a valid number
error-cards-per-set-range = Cards per set must be between 1 and 100
error-streak-length-range = Streak length must be between 1 and 50

## Settings
settings-theme = Theme
settings-language = Language

## Pluralization Examples
users-count =
    { $count ->
        [0] No users
        [1] 1 user
       *[other] { $count } users
    }

items-remaining =
    { $count ->
        [0] No items remaining
        [1] 1 item remaining
       *[other] { $count } items remaining
    }

## Common UI Elements
ok = OK
cancel = Cancel
save = Save
delete = Delete
edit = Edit
add = Add
close = Close
confirm = Confirm
yes = Yes
no = No

## General Messages
welcome = Welcome to Language Helper
loading = Loading...
success = Success
failed = Failed
