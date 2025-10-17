# Main Screen - Create New User Modal
create-new-user-title = Create New User
username-placeholder = Username
choose-language-placeholder = Choose a language
ok-button = OK
cancel-button = Cancel

# Main Screen - User Selection
user-list-select-placeholder = Select a user

# Main Screen - Validation Errors
error-username-too-short = Username must be at least 5 characters
error-username-too-long = Username cannot exceed 50 characters
error-language-not-selected = Please select a language

# Main Screen - API Errors
error-create-user = Failed to create user
error-update-theme = Failed to update theme
error-update-language = Failed to update language
error-load-user-settings = Failed to load user settings

# User Screen - Navigation
user-back-button = Back
user-profiles-button = Profiles
user-settings-button = Settings

# User Screen - Title
user-account-title = Account: {$username} | Language: {$language}

# User Settings Screen
user-settings-language-label = Language:
user-settings-theme-label = Theme:
user-settings-delete-button = Delete User
user-settings-back-button = Back
user-settings-delete-warning = Are you sure you want to delete this user? This will delete all profiles and data.
user-settings-delete-yes = Yes, Delete
user-settings-delete-no = Cancel
user-settings-api-error-theme = Failed to update theme
user-settings-api-error-delete = Failed to delete user

# Error Modal
error-modal-close-button = Close

# Loading State
loading = Loading...

# Profile List Screen
profile-list-title = Select Profile
profile-list-back-button = Back

# Create New Profile Modal
create-new-profile-title = Create New Profile
profile-name-placeholder = Profile Name
profile-language-placeholder = Select target language
profile-ok-button = Create
profile-cancel-button = Cancel

# Profile List - Validation Errors
error-profile-name-too-short = Profile name must be at least 5 characters
error-profile-name-too-long = Profile name cannot exceed 50 characters
error-profile-language-not-selected = Please select a target language

# Profile List - API Errors
error-create-profile = Failed to create profile
error-load-profiles = Failed to load profiles

# Profile Screen
profile-title = Profile: {$profile} | Learning: {$language}
profile-back-button = Back
profile-cards-button = Cards
profile-explain-ai-button = AI Explanations
profile-settings-button = Settings

# Profile Screen - API Errors
error-load-card-settings = Failed to load card settings

# Profile Settings Screen
profile-settings-back-button = Back
profile-settings-card-settings-button = Card Settings
profile-settings-assistant-settings-button = AI Assistant Settings
profile-settings-delete-profile = Delete Profile
profile-settings-delete-warning = Are you sure you want to delete this profile? This will permanently delete all cards and progress.
profile-settings-delete-yes = Yes, Delete
profile-settings-delete-no = Cancel
profile-settings-api-error-delete = Failed to delete profile

# Card Settings Screen
card-settings-title = Card Settings
card-settings-back-button = Back
card-settings-cards-per-set = Cards per set:
card-settings-test-method = Test method:
card-settings-test-method-manual = Manual Input
card-settings-test-method-self = Self Review
card-settings-streak-length = Streak length:
card-settings-save = Save Settings
card-settings-saved = Settings saved successfully
error-cards-per-set-range = Cards per set must be between 1 and 100
error-streak-length-range = Streak length must be between 1 and 50
error-invalid-number = Please enter a valid number
error-save-card-settings = Failed to save card settings

# Assistant Settings Screen
assistant-settings-title = AI Assistant Settings
assistant-settings-back-button = Back
assistant-settings-model-label = Model:
assistant-settings-tiny = Tiny
assistant-settings-light = Light
assistant-settings-weak = Weak
assistant-settings-medium = Medium
assistant-settings-strong = Strong
assistant-settings-api = API

# API Configuration
assistant-settings-api-endpoint = API Endpoint:
assistant-settings-api-key = API Key:
assistant-settings-api-model = Model Name:

# System Requirements
assistant-settings-requirements-title = System Requirements
assistant-settings-incompatible = Your system does not meet the requirements for this model
assistant-settings-no-data = Unable to check system requirements
assistant-settings-ollama-not-installed = Ollama is not installed. To install, visit

# Actions
assistant-settings-start-assistant = Start Assistant
assistant-settings-stop-assistant = Stop Assistant
assistant-settings-change-assistant = Change Assistant
assistant-settings-save-api = Save API Config
assistant-settings-download = Download
assistant-settings-cancel = Cancel
assistant-settings-close = Close

# Launch Modal
assistant-settings-launching = Assistant is starting, please wait...
assistant-settings-checking-server = Checking Ollama server status...
assistant-settings-starting-server = Starting Ollama server...
assistant-settings-checking-models = Checking available models...
assistant-settings-pulling-model = Downloading model...
assistant-settings-launching-model = Launching model...
assistant-settings-launch-success = Model launched successfully!
assistant-settings-launch-error = Failed to launch assistant

# API Errors
error-load-assistant-settings = Failed to load assistant settings
error-save-assistant-settings = Failed to save assistant settings
error-clear-assistant-settings = Failed to clear assistant settings

# Explain AI Screen
explain-ai-title = AI Explanations
explain-ai-back = Back
explain-ai-input-label = Enter a phrase to explain:
explain-ai-send = Send
explain-ai-response-label = AI Explanation:
explain-ai-placeholder = Your explanation will appear here...
explain-ai-loading = Response is being generated...

# Cards Menu Screen
cards-menu-title = Cards Menu
cards-menu-back = Back
cards-menu-manage = Manage Cards
cards-menu-learn = Learn
cards-menu-test = Test
cards-menu-repeat = Repeat

# Manage Cards Screen
manage-cards-title = Manage Cards
manage-cards-back = Back
manage-cards-unlearned-tab = Unlearned
manage-cards-learned-tab = Learned
manage-cards-no-unlearned = No unlearned cards yet
manage-cards-no-learned = No learned cards yet
manage-cards-edit = Edit
manage-cards-delete = Delete
manage-cards-add-new = Add New Card

# Add Card Screen
add-card-title = Add Card
add-card-fill-ai = Fill with AI
add-card-ai-filling = AI is filling the card...
add-card-type-label = Card Type:
add-card-type-straight = Straight
add-card-type-reverse = Reverse
add-card-word-label = Word:
add-card-word-placeholder = Enter word name
add-card-readings-label = Readings (Optional):
add-card-reading-placeholder = Enter reading
add-card-add-reading = Add Reading
add-card-meanings-label = Meanings:
add-card-definition-label = Definition:
add-card-definition-placeholder = Enter definition
add-card-translated-def-label = Translated Definition (Optional):
add-card-translated-def-placeholder = Enter translated definition
add-card-translations-label = Translations:
add-card-translation-placeholder = Enter translation
add-card-add-translation = Add Translation
add-card-remove-meaning = Remove Meaning
add-card-add-meaning = Add Meaning
add-card-save = Save
add-card-cancel = Cancel
add-card-inverse-modal-title = Do you want to create inverse cards?
add-card-inverse-manually = Manually
add-card-inverse-with-assistant = With AI Assistant
add-card-inverse-no = No

# Inverse Cards Review Screen
inverse-cards-review-title = Review Inverse Cards
inverse-cards-back = Back
inverse-cards-no-pending = No pending inverse cards
inverse-cards-edit = Edit
inverse-cards-delete = Delete
inverse-cards-save-all = Save All
inverse-cards-skip-all = Skip All
inverse-cards-saving = Saving cards...

# Learn Router - Start Screen
learn-title = Learn Mode
learn-start-instruction = Enter the starting card number:
learn-card-number-placeholder = Card number
learn-start-button = Start
learn-back = Back

# Learn Router - Loading
learn-loading = Loading session...

# Learn Router - Study Phase
learn-foreign-word-label = Word:
learn-readings-label = Readings:
learn-meanings-label = Meanings:
learn-next-card = Next Card
learn-start-test = Start Test
learn-no-cards = No cards available

# Learn Router - Test Phase
learn-answer-label = Your Answer:
learn-remaining-answers = Remaining answers
learn-answer-placeholder = Enter your answer
learn-submit-answer = Submit
learn-correct = Correct
learn-incorrect = Incorrect
learn-continue = Continue

# Learn Router - Self-Review Mode
learn-show-answer = Show Answer
learn-answer-correct = Correct
learn-answer-incorrect = Incorrect

# Learn Router - Results
learn-test-passed = Test Passed!
learn-test-failed = Test Failed
learn-passed-message = Congratulations! You've mastered this set of cards.
learn-failed-message = Keep practicing! You can retry this set.
learn-next-set = Next Set
learn-retry-set = Retry Set
