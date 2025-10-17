# Main Screen - Create New User Modal
create-new-user-title = Створити нового користувача
username-placeholder = Ім'я користувача
choose-language-placeholder = Виберіть мову
ok-button = OK
cancel-button = Скасувати

# Main Screen - User Selection
user-list-select-placeholder = Виберіть користувача

# Main Screen - Validation Errors
error-username-too-short = Ім'я користувача має містити принаймні 5 символів
error-username-too-long = Ім'я користувача не може перевищувати 50 символів
error-language-not-selected = Будь ласка, виберіть мову

# Main Screen - API Errors
error-create-user = Не вдалося створити користувача
error-update-theme = Не вдалося оновити тему
error-update-language = Не вдалося оновити мову
error-load-user-settings = Не вдалося завантажити налаштування користувача

# User Screen - Navigation
user-back-button = Назад
user-profiles-button = Профілі
user-settings-button = Налаштування

# User Screen - Title
user-account-title = Обліковий запис: {$username} | Мова: {$language}

# User Settings Screen
user-settings-language-label = Мова:
user-settings-theme-label = Тема:
user-settings-delete-button = Видалити користувача
user-settings-back-button = Назад
user-settings-delete-warning = Ви впевнені, що хочете видалити цього користувача? Це видалить усі профілі та дані.
user-settings-delete-yes = Так, видалити
user-settings-delete-no = Скасувати
user-settings-api-error-theme = Не вдалося оновити тему
user-settings-api-error-delete = Не вдалося видалити користувача

# Error Modal
error-modal-close-button = Закрити

# Loading State
loading = Завантаження...

# Profile List Screen
profile-list-title = Виберіть профіль
profile-list-back-button = Назад

# Create New Profile Modal
create-new-profile-title = Створити новий профіль
profile-name-placeholder = Назва профілю
profile-language-placeholder = Виберіть цільову мову
profile-ok-button = Створити
profile-cancel-button = Скасувати

# Profile List - Validation Errors
error-profile-name-too-short = Назва профілю має містити принаймні 5 символів
error-profile-name-too-long = Назва профілю не може перевищувати 50 символів
error-profile-language-not-selected = Будь ласка, виберіть цільову мову

# Profile List - API Errors
error-create-profile = Не вдалося створити профіль
error-load-profiles = Не вдалося завантажити профілі

# Profile Screen
profile-title = Профіль: {$profile} | Вивчається: {$language}
profile-back-button = Назад
profile-cards-button = Картки
profile-explain-ai-button = Пояснення AI
profile-settings-button = Налаштування

# Profile Screen - API Errors
error-load-card-settings = Не вдалося завантажити налаштування карток

# Profile Settings Screen
profile-settings-back-button = Назад
profile-settings-card-settings-button = Налаштування карток
profile-settings-assistant-settings-button = Налаштування AI-асистента
profile-settings-delete-profile = Видалити профіль
profile-settings-delete-warning = Ви впевнені, що хочете видалити цей профіль? Це назавжди видалить усі картки та прогрес.
profile-settings-delete-yes = Так, видалити
profile-settings-delete-no = Скасувати
profile-settings-api-error-delete = Не вдалося видалити профіль

# Card Settings Screen
card-settings-title = Налаштування карток
card-settings-back-button = Назад
card-settings-cards-per-set = Карток у наборі:
card-settings-test-method = Метод тестування:
card-settings-test-method-manual = Ручне введення
card-settings-test-method-self = Самоперевірка
card-settings-streak-length = Довжина серії:
card-settings-save = Зберегти налаштування
card-settings-saved = Налаштування успішно збережено
error-cards-per-set-range = Кількість карток у наборі має бути від 1 до 100
error-streak-length-range = Довжина серії має бути від 1 до 50
error-invalid-number = Будь ласка, введіть правильне число
error-save-card-settings = Не вдалося зберегти налаштування карток

# Assistant Settings Screen
assistant-settings-title = Налаштування AI-асистента
assistant-settings-back-button = Назад
assistant-settings-model-label = Модель:
assistant-settings-tiny = Дуже маленька
assistant-settings-light = Легка
assistant-settings-weak = Слабка
assistant-settings-medium = Середня
assistant-settings-strong = Сильна
assistant-settings-api = API

# API Configuration
assistant-settings-api-endpoint = Кінцева точка API:
assistant-settings-api-key = Ключ API:
assistant-settings-api-model = Назва моделі:

# System Requirements
assistant-settings-requirements-title = Системні вимоги
assistant-settings-incompatible = Ваша система не відповідає вимогам для цієї моделі
assistant-settings-no-data = Неможливо перевірити системні вимоги
assistant-settings-ollama-not-installed = Ollama не встановлено. Для встановлення відвідайте

# Actions
assistant-settings-start-assistant = Запустити асистента
assistant-settings-stop-assistant = Зупинити асистента
assistant-settings-change-assistant = Змінити асистента
assistant-settings-save-api = Зберегти конфігурацію API
assistant-settings-download = Завантажити
assistant-settings-cancel = Скасувати
assistant-settings-close = Закрити

# Launch Modal
assistant-settings-launching = Асистент запускається, будь ласка, зачекайте...
assistant-settings-checking-server = Перевірка статусу сервера Ollama...
assistant-settings-starting-server = Запуск сервера Ollama...
assistant-settings-checking-models = Перевірка доступних моделей...
assistant-settings-pulling-model = Завантаження моделі...
assistant-settings-launching-model = Запуск моделі...
assistant-settings-launch-success = Модель успішно запущено!
assistant-settings-launch-error = Не вдалося запустити асистента

# API Errors
error-load-assistant-settings = Не вдалося завантажити налаштування асистента
error-save-assistant-settings = Не вдалося зберегти налаштування асистента
error-clear-assistant-settings = Не вдалося очистити налаштування асистента

# Explain AI Screen
explain-ai-title = Пояснення AI
explain-ai-back = Назад
explain-ai-input-label = Введіть фразу для пояснення:
explain-ai-send = Надіслати
explain-ai-response-label = Пояснення AI:
explain-ai-placeholder = Ваше пояснення з'явиться тут...
explain-ai-loading = Відповідь генерується...

# Cards Menu Screen
cards-menu-title = Меню карток
cards-menu-back = Назад
cards-menu-manage = Керувати картками
cards-menu-learn = Вивчити
cards-menu-test = Тест
cards-menu-repeat = Повторити

# Manage Cards Screen
manage-cards-title = Керування картками
manage-cards-back = Назад
manage-cards-unlearned-tab = Не вивчені
manage-cards-learned-tab = Вивчені
manage-cards-no-unlearned = Поки що немає не вивчених карток
manage-cards-no-learned = Поки що немає вивчених карток
manage-cards-edit = Редагувати
manage-cards-delete = Видалити
manage-cards-add-new = Додати нову картку

# Add Card Screen
add-card-title = Додати картку
add-card-fill-ai = Заповнити за допомогою AI
add-card-ai-filling = AI заповнює картку...
add-card-type-label = Тип картки:
add-card-type-straight = Пряма
add-card-type-reverse = Зворотна
add-card-word-label = Слово:
add-card-word-placeholder = Введіть назву слова
add-card-readings-label = Читання (необов'язково):
add-card-reading-placeholder = Введіть читання
add-card-add-reading = Додати читання
add-card-meanings-label = Значення:
add-card-definition-label = Визначення:
add-card-definition-placeholder = Введіть визначення
add-card-translated-def-label = Перекладене визначення (необов'язково):
add-card-translated-def-placeholder = Введіть перекладене визначення
add-card-translations-label = Переклади:
add-card-translation-placeholder = Введіть переклад
add-card-add-translation = Додати переклад
add-card-remove-meaning = Видалити значення
add-card-add-meaning = Додати значення
add-card-save = Зберегти
add-card-cancel = Скасувати
add-card-inverse-modal-title = Бажаєте створити зворотні картки?
add-card-inverse-manually = Вручну
add-card-inverse-with-assistant = За допомогою AI-асистента
add-card-inverse-no = Ні

# Inverse Cards Review Screen
inverse-cards-review-title = Перегляд зворотних карток
inverse-cards-back = Назад
inverse-cards-no-pending = Немає зворотних карток, що очікують
inverse-cards-edit = Редагувати
inverse-cards-delete = Видалити
inverse-cards-save-all = Зберегти всі
inverse-cards-skip-all = Пропустити всі
inverse-cards-saving = Збереження карток...

# Learn Router - Start Screen
learn-title = Режим навчання
learn-start-instruction = Введіть номер початкової картки:
learn-card-number-placeholder = Номер картки
learn-start-button = Почати
learn-back = Назад

# Learn Router - Loading
learn-loading = Завантаження сесії...

# Learn Router - Study Phase
learn-foreign-word-label = Слово:
learn-readings-label = Читання:
learn-meanings-label = Значення:
learn-next-card = Наступна картка
learn-start-test = Почати тест
learn-no-cards = Немає доступних карток

# Learn Router - Test Phase
learn-answer-label = Ваша відповідь:
learn-remaining-answers = Залишилось відповідей
learn-answer-placeholder = Введіть вашу відповідь
learn-submit-answer = Надіслати
learn-correct = Правильно
learn-incorrect = Неправильно
learn-continue = Продовжити

# Learn Router - Self-Review Mode
learn-show-answer = Показати відповідь
learn-answer-correct = Правильно
learn-answer-incorrect = Неправильно

# Learn Router - Results
learn-test-passed = Тест пройдено!
learn-test-failed = Тест не пройдено
learn-passed-message = Вітаємо! Ви опанували цей набір карток.
learn-failed-message = Продовжуйте практикуватися! Ви можете спробувати цей набір знову.
learn-next-set = Наступний набір
learn-retry-set = Спробувати набір знову

# Test Router - Loading
test-loading = Завантаження тесту...
test-back = Назад
test-no-cards = Немає карток для тестування

# Test Router - Results
test-test-passed = Тест пройдено!
test-test-failed = Тест не пройдено
test-passed-message = Чудова робота! Усі не вивчені слова успішно протестовано.
test-failed-message = Деякі відповіді були неправильними. Продовжуйте практикуватися!
test-retry-test = Спробувати тест знову

# Repeat Router - Loading
repeat-loading = Завантаження сесії повторення...
repeat-back = Назад
repeat-no-cards = Немає вивчених карток для повторення

# Repeat Router - Results
repeat-test-passed = Повторення пройдено!
repeat-test-failed = Повторення не пройдено
repeat-passed-message = Чудово! Ви запам'ятали всі свої вивчені слова.
repeat-failed-message = Деякі слова потребують більше практики. Їх повернуто до не вивчених.
repeat-retry-repeat = Спробувати повторення знову
