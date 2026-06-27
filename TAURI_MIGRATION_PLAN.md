# План миграции с Iced на Tauri

## 1. Цель

Перенести desktop-интерфейс Language Helper 2 с Iced на Tauri, сохранив существующую
бизнес-логику, модели и SQLite-хранилище на Rust.

Первая версия ориентирована только на PC. Frontend проектируется так, чтобы позднее
его можно было использовать в браузере с отдельным HTTP backend.

## 2. Зафиксированные решения

- Desktop shell: Tauri 2.
- Frontend: React, TypeScript и Vite.
- Маршрутизация: React Router.
- Работа с асинхронными данными: TanStack Query.
- UI-компоненты и темы: Mantine.
- Основной и единственный язык первой версии: английский.
- Текущие Fluent-локализации из `backend/gui/locales` не переносятся.
- Архитектура frontend не должна мешать последующему подключению i18n.
- Rust crates `api`, `core` и `persistence` сохраняются.
- Iced-приложение остаётся рабочим до достижения функционального паритета.

## 3. Целевая архитектура

```text
React SPA
    |
    v
LanguageHelperClient
    |
    +-- TauriLanguageHelperClient
    |       |
    |       v
    |   Tauri commands
    |       |
    |       v
    |   AppApi -> core -> persistence -> SQLite
    |
    +-- HttpLanguageHelperClient (будущая браузерная версия)
    |
    +-- MockLanguageHelperClient (тесты и Storybook/dev mode)
```

React-компоненты не должны напрямую импортировать или вызывать Tauri `invoke`.
Взаимодействие с backend выполняется только через `LanguageHelperClient`.

Это позволит запускать один frontend:

- внутри Tauri через `TauriLanguageHelperClient`;
- в браузере через будущий `HttpLanguageHelperClient`;
- в тестах через `MockLanguageHelperClient`.

## 4. Предлагаемая структура проекта

```text
backend/
  Cargo.toml
  api/
  core/
  persistence/
  app/                  # Legacy Iced entry point during migration
  gui/                  # Legacy Iced UI during migration
  bootstrap/            # Planned shared composition root
  desktop/              # Planned Tauri Rust crate
    capabilities/
    src/
      commands/
        users.rs
        profiles.rs
        cards.rs
        learning.rs
        settings.rs
        assistant.rs
      error.rs
      state.rs
      lib.rs
      main.rs

frontend/
  src/
    app/
      providers/
      router/
      theme/
    api/
      language-helper-client.ts
      tauri-language-helper-client.ts
      http-language-helper-client.ts
      mock-language-helper-client.ts
    components/
    features/
      users/
      profiles/
      cards/
      learning/
      settings/
      assistant/
    locales/
      en.ts
    models/
    routes/
    main.tsx
```

Название нового Rust crate можно уточнить при реализации. Рабочее название:
`lh_bootstrap`.

## 5. Этапы миграции

### Этап 0. Зафиксировать исходное поведение

Перед изменениями:

1. Проверить сборку и существующие тесты workspace.
2. Зафиксировать список рабочих пользовательских сценариев.
3. Подготовить тестовую базу с пользователем, профилем, карточками и прогрессом.
4. Проверить, где сейчас хранятся production-данные.
5. Создать короткий smoke-checklist для сравнения Iced и Tauri версий.

Результат: понятна исходная функциональность и есть данные для проверки совместимости.

### Этап 1. Убрать дублирование инициализации Rust

Создать `lh_bootstrap` и перенести в него:

- `AppConfig`;
- определение путей к базе и директории данных;
- создание persistence repositories;
- создание adapters и services;
- сборку `AppApi`.

Целевой публичный интерфейс:

```rust
pub fn create_app_api(
    config: AppConfig,
) -> Result<Arc<dyn AppApi>, BootstrapError>;
```

После этого и старое Iced-приложение, и Tauri backend должны использовать один
composition root. Дублирование сборки backend в UI-слоях не допускается.

Результат: создание backend не зависит от UI-фреймворка.

### Этап 2. Создать каркас Tauri и React

1. Добавить React + TypeScript + Vite frontend.
2. Добавить Tauri crate в Rust workspace.
3. Инициализировать `AppApi` при запуске Tauri.
4. Сохранить `Arc<dyn AppApi>` в Tauri managed state.
5. Добавить обработку стартовых ошибок без panic.
6. Настроить development и production build.
7. Ограничить Tauri capabilities минимально необходимым набором.

Результат: пустое desktop-приложение запускается и может выполнить тестовую Rust-команду.

### Этап 3. Создать контракт frontend/backend

1. Определить `LanguageHelperClient`.
2. Разделить методы по областям:
   - users;
   - profiles;
   - cards;
   - learning;
   - settings;
   - AI assistant.
3. Добавить тонкие Tauri commands поверх существующего `AppApi`.
4. Использовать DTO из `lh_api` для аргументов и результатов.
5. Ввести единый сериализуемый формат ошибки:

```text
code
message
details (optional)
```

6. Настроить генерацию TypeScript-типов из Rust либо contract-тесты, которые
   обнаруживают расхождение Rust и TypeScript DTO.
7. Не передавать JSON как вложенные строки.

Результат: frontend получает типизированный API, не связанный напрямую с Tauri.

### Этап 4. Реализовать вертикальный прототип

Перенести один полный сценарий:

1. Запуск приложения и открытие существующей базы.
2. Получение списка пользователей.
3. Создание пользователя.
4. Открытие пользователя.
5. Получение списка профилей.
6. Открытие профиля.
7. Получение списка карточек.
8. Создание карточки.
9. Перезапуск приложения и проверка сохранённых данных.

На этом же этапе проверить:

- отображение loading/error/empty states;
- обновление данных после mutation;
- навигацию назад;
- работу с Unicode;
- отсутствие блокировки UI во время SQLite и AI операций.

Результат: подтверждена работоспособность всего пути React -> Tauri -> Rust -> SQLite.

После этого этапа следует уточнить оценку оставшейся миграции.

### Этап 5. Перенести основной CRUD

Порядок:

1. Users.
2. User settings.
3. Profiles.
4. Profile settings.
5. Card settings.
6. Card list and filtering.
7. Add/edit/delete card.
8. Inverse card review.

Предлагаемые маршруты:

```text
/
/users/:username
/users/:username/settings
/users/:username/profiles
/users/:username/profiles/:profileName
/users/:username/profiles/:profileName/settings
/users/:username/profiles/:profileName/cards
/users/:username/profiles/:profileName/cards/new
/users/:username/profiles/:profileName/cards/:wordName/edit
```

Выбранные пользователь и профиль определяются URL. Их не следует дублировать в
большом глобальном store.

Результат: управление пользователями, профилями и карточками доступно в Tauri.

### Этап 6. Перенести обучение

Порядок:

1. Learn session.
2. Test session.
3. Repeat session.
4. Manual answer input.
5. Self-review.
6. Обновление streak.
7. Straight/reverse card filters.

Маршруты:

```text
/users/:username/profiles/:profileName/learn
/users/:username/profiles/:profileName/test
/users/:username/profiles/:profileName/repeat
```

Состояние активной сессии должно быть локальным для соответствующего feature.
Persisted-данные загружаются через TanStack Query, временное состояние ответа
хранится в React state/reducer.

Результат: основные учебные сценарии имеют функциональный паритет с Iced.

### Этап 7. Перенести AI-функции

1. Assistant settings.
2. OpenAI/Gemini configuration.
3. Ollama configuration и system checks, если они остаются актуальными.
4. AI Explain.
5. AI Fill Card.
6. AI inverse-card merge.
7. Отмена или безопасное игнорирование результата долгих операций после ухода со страницы.
8. Понятные сообщения об ошибках сети, API key и модели.

AI-запросы продолжают выполняться в Rust. API keys не передаются дальше frontend
без необходимости.

Результат: AI-возможности работают через асинхронные Tauri commands без блокировки UI.

### Этап 8. Темы и задел под локализацию

Для первой версии:

- весь интерфейс написан на английском;
- текущие `.ftl` файлы не копируются;
- переключатель языка отсутствует либо показывает только English;
- RTL и проверка остальных языков не входят в scope.

При этом пользовательские строки не следует хаотично размещать внутри JSX.
Достаточно простого английского каталога:

```ts
export const messages = {
  common: {
    save: "Save",
    cancel: "Cancel",
    delete: "Delete",
  },
  users: {
    title: "Users",
  },
} as const;
```

Компоненты получают строки через одну функцию или hook:

```ts
const { t } = useTranslations();
t("common.save");
```

Первая реализация может просто читать английский объект без полноценной i18n
библиотеки. В будущем этот provider можно заменить на i18next без переписывания
компонентов.

Результат: английский UI не несёт стоимость переноса старых переводов, но готов к
последующему добавлению языков.

### Этап 9. Пути к данным и миграция

1. Перейти с относительного `data/` на Tauri application data directory.
2. Добавить обнаружение старого расположения базы.
3. Не перемещать данные без подтверждённой резервной копии.
4. Проверять схему и доступность базы до запуска UI.
5. Сохранить совместимость с существующими profile database files.
6. Добавить понятное сообщение и путь восстановления при ошибке миграции.

Результат: обновление приложения не приводит к потере пользовательских данных.

### Этап 10. Тестирование и CI

Rust:

- `cargo fmt --check`;
- `cargo clippy`;
- тесты `api`, `core`, `persistence` и `lh_bootstrap`;
- тесты Tauri command wrappers;
- тесты открытия существующей базы.

Frontend:

- ESLint;
- TypeScript typecheck;
- Vitest;
- React Testing Library;
- тесты `LanguageHelperClient`;
- тесты loading/error/empty states;
- browser smoke tests с `MockLanguageHelperClient`.

Build:

- production Vite build;
- Tauri build для Windows;
- smoke test установщика;
- проверка запуска на чистой пользовательской системе.

Результат: миграция защищена тестами на уровне domain, transport и UI.

### Этап 11. Выпуск PC-версии

Первый целевой релиз:

- Windows;
- локальная SQLite база;
- английский UI;
- существующие CRUD, learning и AI функции;
- импорт/использование старых пользовательских данных;
- установщик и понятная директория данных.

Linux и macOS добавляются после стабилизации Windows-сборки.

Результат: Tauri-версия пригодна для ежедневного использования.

### Этап 12. Удалить Iced

Удаление выполняется только после прохождения smoke-checklist и периода
параллельного использования.

Удалить:

- `gui`;
- старый Iced-based `app`;
- Iced dependencies;
- неиспользуемые assets и runtime utilities;
- документацию, описывающую старый router stack.

Обновить:

- workspace members;
- README;
- ARCHITECTURE;
- инструкции сборки и релиза.

## 6. Подготовка к браузерной версии

Tauri frontend можно собрать как обычную SPA, но браузер не сможет напрямую
использовать Tauri commands, native Rust и текущий `rusqlite`.

Для полноценной web-версии позднее потребуется один из вариантов:

1. Рекомендуемый: Rust HTTP backend и `HttpLanguageHelperClient`.
2. Отдельный local-first backend на WASM и IndexedDB/OPFS.

Второй вариант потребует существенной переработки persistence и части core,
поэтому он не входит в текущий план.

На текущем этапе достаточно:

- не импортировать Tauri API в React components;
- держать DTO и transport отдельно;
- иметь mock transport;
- использовать browser-compatible routing;
- не полагаться в UI на абсолютные filesystem paths.

## 7. Что не входит в текущую миграцию

- Mobile UI.
- Перенос существующих переводов.
- Добавление новых языков.
- Полноценный web backend.
- Синхронизация данных между устройствами.
- Облачная авторизация.
- Переписывание бизнес-логики только ради смены UI.
- Изменение SQLite-схемы без необходимости.
- Большой визуальный редизайн до подтверждения вертикального прототипа.

## 8. Критерии завершения

Миграция завершена, когда:

- все необходимые пользовательские сценарии работают в Tauri;
- старые данные открываются без потери;
- UI не блокируется во время долгих операций;
- ошибки backend отображаются пользователю;
- английский интерфейс покрывает все экраны;
- frontend не зависит напрямую от Tauri transport;
- production Windows build и установщик воспроизводимы;
- тесты Rust и frontend проходят;
- Iced больше не нужен для ежедневного использования;
- документация соответствует новой архитектуре.

## 9. Предварительная оценка

Ориентир для полной миграции: 3-5 недель полноценной разработки.

Оценка должна быть пересмотрена после вертикального прототипа, поскольку именно
он покажет реальную стоимость:

- преобразования DTO;
- Tauri command layer;
- переноса сложных learning screens;
- AI lifecycle;
- совместимости существующих баз данных.
