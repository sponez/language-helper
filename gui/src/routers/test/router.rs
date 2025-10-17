//! Test router for testing unlearned cards.
//!
//! This router manages the test flow:
//! 1. **Loading**: Automatically load and shuffle all unlearned cards
//! 2. **Testing Phase**: Test user on all cards (no study phase)
//! 3. **Results**: Show pass/fail with streak updates applied

use std::rc::Rc;
use std::sync::Arc;

use iced::keyboard::{key::Named, Key};
use iced::widget::{column, container, row, stack, text, Column, Container};
use iced::{event, Alignment, Element, Event, Length, Subscription, Task};

use lh_api::app_api::AppApi;
use lh_api::models::learning_session::LearningSessionDto;

use crate::app_state::AppState;
use crate::components::back_button::back_button;
use crate::router::{self, RouterEvent, RouterNode};
use crate::routers::test::message::Message;
use crate::states::{ProfileState, UserState};

use crate::routers::learn::elements::action_button::action_button;
use crate::routers::learn::elements::answer_input::AnswerInputMessage;
use crate::routers::learn::elements::card_display::card_display;

/// State for different screens in the test flow
#[derive(Debug, Clone)]
pub enum ScreenState {
    /// Loading session
    Loading,
    /// Testing phase - testing knowledge
    Testing {
        session: LearningSessionDto,
        answer_input: String,
        feedback: Option<AnswerFeedback>,
        /// For self-review mode: tracks if answer has been shown
        answer_shown: bool,
    },
    /// Results screen after completing test
    Results { passed: bool },
}

/// Feedback after submitting an answer
#[derive(Debug, Clone)]
pub enum AnswerFeedback {
    /// Correct answer
    Correct { matched_answer: String },
    /// Incorrect answer
    Incorrect { expected_answer: String },
}

/// State for the test router
pub struct TestRouter {
    /// User context (read-only reference)
    user_state: Rc<UserState>,
    /// Profile context (read-only reference)
    profile_state: Rc<ProfileState>,
    /// API instance for backend communication
    app_api: Arc<dyn AppApi>,
    /// Global application state (theme, language, i18n)
    app_state: Rc<AppState>,
    /// Current screen state
    screen_state: ScreenState,
}

impl TestRouter {
    /// Creates a new test router and starts loading the session.
    pub fn new(
        user_state: Rc<UserState>,
        profile_state: Rc<ProfileState>,
        app_api: Arc<dyn AppApi>,
        app_state: Rc<AppState>,
    ) -> Self {
        Self {
            user_state,
            profile_state,
            app_api,
            app_state,
            screen_state: ScreenState::Loading,
        }
    }

    /// Update the router state based on messages.
    pub fn update(&mut self, message: Message) -> (Option<RouterEvent>, Task<Message>) {
        match message {
            Message::SessionStarted(result) => match result {
                Ok(session) => {
                    // Transition to testing phase
                    self.screen_state = ScreenState::Testing {
                        session,
                        answer_input: String::new(),
                        feedback: None,
                        answer_shown: false,
                    };
                    (None, Task::none())
                }
                Err(err) => {
                    eprintln!("Failed to start test session: {}", err);
                    // Go back on error
                    (Some(RouterEvent::Pop), Task::none())
                }
            },

            Message::AnswerInputChanged(value) => {
                if let ScreenState::Testing { answer_input, .. } = &mut self.screen_state {
                    *answer_input = value;
                }
                (None, Task::none())
            }

            Message::SubmitAnswer => {
                if let ScreenState::Testing {
                    session,
                    answer_input,
                    ..
                } = &self.screen_state
                {
                    let app_api = Arc::clone(&self.app_api);
                    let session_clone = session.clone();
                    let user_input = answer_input.trim().to_string();

                    let task = Task::perform(
                        async move {
                            app_api
                                .profile_api()
                                .check_answer(&session_clone, &user_input)
                                .await
                                .map_err(|e| format!("Failed to check answer: {}", e))
                        },
                        |result| Message::AnswerChecked(result),
                    );

                    return (None, task);
                }
                (None, Task::none())
            }

            Message::AnswerChecked(result) => {
                if let ScreenState::Testing {
                    session,
                    answer_input,
                    feedback,
                    ..
                } = &mut self.screen_state
                {
                    match result {
                        Ok((is_correct, matched_answer)) => {
                            if is_correct {
                                let mut updated_session = session.clone();
                                updated_session
                                    .current_card_provided_answers
                                    .push(answer_input.trim().to_string());

                                *session = updated_session;
                                *answer_input = String::new();
                                *feedback = Some(AnswerFeedback::Correct { matched_answer });
                            } else {
                                let mut updated_session = session.clone();
                                updated_session.current_card_failed = true;

                                *session = updated_session;
                                *feedback = Some(AnswerFeedback::Incorrect {
                                    expected_answer: matched_answer,
                                });
                            }
                        }
                        Err(e) => {
                            *feedback = Some(AnswerFeedback::Incorrect {
                                expected_answer: format!("Error: {}", e),
                            });
                        }
                    }
                }
                (None, Task::none())
            }

            Message::ShowAnswer => {
                if let ScreenState::Testing { answer_shown, .. } = &mut self.screen_state {
                    *answer_shown = true;
                }
                (None, Task::none())
            }

            Message::AnswerCorrect => {
                if let ScreenState::Testing { session, .. } = &self.screen_state {
                    let mut updated_session = session.clone();

                    let current_index = updated_session.current_card_in_set;
                    let card = &updated_session.all_cards[current_index];

                    let result = lh_api::models::test_result::TestResultDto {
                        word_name: card.word.name.clone(),
                        is_correct: true,
                        user_answer: None,
                        expected_answer: None,
                    };
                    updated_session.test_results.push(result.clone());

                    updated_session.current_card_in_set += 1;

                    // Update streak immediately
                    let task = self.update_card_streak(card.word.name.clone(), true);

                    if updated_session.current_card_in_set >= updated_session.all_cards.len() {
                        // Complete session
                        return self.complete_session(updated_session);
                    } else {
                        self.screen_state = ScreenState::Testing {
                            session: updated_session,
                            answer_input: String::new(),
                            feedback: None,
                            answer_shown: false,
                        };
                    }
                    return (None, task);
                }
                (None, Task::none())
            }

            Message::AnswerIncorrect => {
                if let ScreenState::Testing { session, .. } = &self.screen_state {
                    let mut updated_session = session.clone();

                    let current_index = updated_session.current_card_in_set;
                    let card = &updated_session.all_cards[current_index];

                    let result = lh_api::models::test_result::TestResultDto {
                        word_name: card.word.name.clone(),
                        is_correct: false,
                        user_answer: None,
                        expected_answer: None,
                    };
                    updated_session.test_results.push(result.clone());

                    updated_session.current_card_in_set += 1;

                    // Update streak immediately
                    let task = self.update_card_streak(card.word.name.clone(), false);

                    if updated_session.current_card_in_set >= updated_session.all_cards.len() {
                        // Complete session
                        return self.complete_session(updated_session);
                    } else {
                        self.screen_state = ScreenState::Testing {
                            session: updated_session,
                            answer_input: String::new(),
                            feedback: None,
                            answer_shown: false,
                        };
                    }
                    return (None, task);
                }
                (None, Task::none())
            }

            Message::Continue => {
                if let ScreenState::Testing { session, .. } = &self.screen_state {
                    let mut updated_session = session.clone();

                    let current_index = updated_session.current_card_in_set;
                    let card = &updated_session.all_cards[current_index];

                    let required_answers = match card.card_type {
                        lh_api::models::card::CardType::Straight => card.meanings.len(),
                        lh_api::models::card::CardType::Reverse => card
                            .meanings
                            .iter()
                            .map(|m| m.word_translations.len())
                            .sum(),
                    };

                    let is_card_complete = updated_session.current_card_provided_answers.len()
                        >= required_answers
                        || updated_session.current_card_failed;

                    if is_card_complete {
                        let is_correct = !session.current_card_failed;
                        let result = lh_api::models::test_result::TestResultDto {
                            word_name: card.word.name.clone(),
                            is_correct,
                            user_answer: Some(session.current_card_provided_answers.join(", ")),
                            expected_answer: None,
                        };
                        updated_session.test_results.push(result.clone());

                        updated_session.current_card_in_set += 1;
                        updated_session.current_card_provided_answers.clear();
                        updated_session.current_card_failed = false;

                        // Update streak immediately
                        let task = self.update_card_streak(card.word.name.clone(), is_correct);

                        if updated_session.current_card_in_set >= updated_session.all_cards.len() {
                            // Complete session
                            return self.complete_session(updated_session);
                        } else {
                            self.screen_state = ScreenState::Testing {
                                session: updated_session,
                                answer_input: String::new(),
                                feedback: None,
                                answer_shown: false,
                            };
                        }
                        return (None, task);
                    } else {
                        self.screen_state = ScreenState::Testing {
                            session: updated_session,
                            answer_input: String::new(),
                            feedback: None,
                            answer_shown: false,
                        };
                    }
                }
                (None, Task::none())
            }

            Message::CardCompleted(result) => {
                if let Err(err) = result {
                    eprintln!("Failed to update card streak: {}", err);
                }
                // Continue regardless of error
                (None, Task::none())
            }

            Message::SessionCompleted(_result) => {
                // This message is no longer used since we update streaks per-card
                (None, Task::none())
            }

            Message::RetryTest => {
                // Restart by creating new session
                self.screen_state = ScreenState::Loading;
                let app_api = Arc::clone(&self.app_api);
                let username = self.user_state.username.clone();
                let profile_name = self.profile_state.profile_name.clone();

                let task = Task::perform(
                    async move {
                        app_api
                            .profile_api()
                            .create_test_session(&username, &profile_name)
                            .await
                            .map_err(|e| format!("Failed to create test session: {}", e))
                    },
                    |result| Message::SessionStarted(result),
                );

                (None, task)
            }

            Message::Back => (Some(RouterEvent::Pop), Task::none()),

            Message::Event(event) => {
                if let Event::Keyboard(iced::keyboard::Event::KeyPressed { key, .. }) = event {
                    match key {
                        Key::Named(Named::Enter) => {
                            if let ScreenState::Testing {
                                answer_input,
                                feedback,
                                ..
                            } = &self.screen_state
                            {
                                if feedback.is_none() && !answer_input.trim().is_empty() {
                                    return self.update(Message::SubmitAnswer);
                                } else if feedback.is_some() {
                                    return self.update(Message::Continue);
                                }
                            }
                        }
                        Key::Named(Named::Escape) => {
                            return self.update(Message::Back);
                        }
                        _ => {}
                    }
                }
                (None, Task::none())
            }
        }
    }

    fn update_card_streak(&self, word_name: String, is_correct: bool) -> Task<Message> {
        let app_api = Arc::clone(&self.app_api);
        let username = self.user_state.username.clone();
        let profile_name = self.profile_state.profile_name.clone();

        let result = lh_api::models::test_result::TestResultDto {
            word_name,
            is_correct,
            user_answer: None,
            expected_answer: None,
        };

        Task::perform(
            async move {
                app_api
                    .profile_api()
                    .update_test_streaks(&username, &profile_name, vec![result])
                    .await
                    .map_err(|e| format!("Failed to update card streak: {}", e))
            },
            |result| Message::CardCompleted(result),
        )
    }

    fn complete_session(
        &mut self,
        session: LearningSessionDto,
    ) -> (Option<RouterEvent>, Task<Message>) {
        // Check if passed
        let passed = session.test_results.iter().all(|r| r.is_correct);
        self.screen_state = ScreenState::Results { passed };
        (None, Task::none())
    }

    /// Render the router's view.
    pub fn view(&self) -> Element<'_, Message> {
        let i18n = &self.app_state.i18n();

        match &self.screen_state {
            ScreenState::Loading => {
                let title = text(i18n.get("test-loading", None))
                    .size(24)
                    .shaping(iced::widget::text::Shaping::Advanced);

                Container::new(title)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .align_x(Alignment::Center)
                    .align_y(Alignment::Center)
                    .into()
            }

            ScreenState::Testing {
                session,
                answer_input,
                feedback,
                answer_shown,
            } => {
                let current_index = session.current_card_in_set;
                if let Some(card) = session.all_cards.get(current_index) {
                    let mut content_elements = Vec::new();

                    let required_answers = match card.card_type {
                        lh_api::models::card::CardType::Straight => card.meanings.len(),
                        lh_api::models::card::CardType::Reverse => card
                            .meanings
                            .iter()
                            .map(|m| m.word_translations.len())
                            .sum(),
                    };
                    let is_card_complete = session.current_card_provided_answers.len()
                        >= required_answers
                        || session.current_card_failed;

                    let is_self_review = session.test_method == "self_review";

                    let show_full_card = if is_self_review {
                        *answer_shown
                    } else {
                        is_card_complete
                    };

                    if show_full_card {
                        let card_view =
                            card_display(i18n, card, current_index + 1, session.all_cards.len());
                        content_elements.push(card_view);
                    } else {
                        let word_label = text(i18n.get("learn-foreign-word-label", None))
                            .size(16)
                            .shaping(iced::widget::text::Shaping::Advanced);

                        let word = text(&card.word.name)
                            .size(32)
                            .shaping(iced::widget::text::Shaping::Advanced);

                        let word_section = column![word_label, word]
                            .spacing(5)
                            .align_x(Alignment::Center);

                        content_elements.push(word_section.into());
                    }

                    let remaining_count = required_answers
                        .saturating_sub(session.current_card_provided_answers.len());

                    if is_self_review {
                        if !answer_shown {
                            let show_answer_btn =
                                action_button(i18n, "learn-show-answer", Some(Message::ShowAnswer));
                            content_elements.push(show_answer_btn.into());
                        } else {
                            let correct_btn = action_button(
                                i18n,
                                "learn-answer-correct",
                                Some(Message::AnswerCorrect),
                            );
                            let incorrect_btn = action_button(
                                i18n,
                                "learn-answer-incorrect",
                                Some(Message::AnswerIncorrect),
                            );

                            let button_row = row![correct_btn, incorrect_btn]
                                .spacing(20)
                                .align_y(Alignment::Center);

                            content_elements.push(button_row.into());
                        }
                    } else {
                        if feedback.is_none() {
                            let input_element =
                                crate::routers::learn::elements::answer_input::answer_input(
                                    i18n,
                                    answer_input,
                                    remaining_count,
                                )
                                .map(|msg| match msg {
                                    AnswerInputMessage::Changed(v) => {
                                        Message::AnswerInputChanged(v)
                                    }
                                });

                            content_elements.push(input_element);

                            let submit_btn = action_button(
                                i18n,
                                "learn-submit-answer",
                                if !answer_input.trim().is_empty() {
                                    Some(Message::SubmitAnswer)
                                } else {
                                    None
                                },
                            );

                            content_elements.push(submit_btn.into());
                        }

                        if let Some(fb) = feedback {
                            let feedback_text = match fb {
                                AnswerFeedback::Correct { matched_answer } => text(format!(
                                    "{}: {}",
                                    i18n.get("learn-correct", None),
                                    matched_answer
                                ))
                                .size(18)
                                .shaping(iced::widget::text::Shaping::Advanced)
                                .color([0.0, 1.0, 0.0]),
                                AnswerFeedback::Incorrect { expected_answer } => text(format!(
                                    "{}: {}",
                                    i18n.get("learn-incorrect", None),
                                    expected_answer
                                ))
                                .size(18)
                                .shaping(iced::widget::text::Shaping::Advanced)
                                .color([1.0, 0.0, 0.0]),
                            };

                            let continue_btn =
                                action_button(i18n, "learn-continue", Some(Message::Continue));

                            content_elements.push(feedback_text.into());
                            content_elements.push(continue_btn.into());
                        }
                    }

                    let mut content = Column::new().spacing(20).align_x(Alignment::Center);
                    for element in content_elements {
                        content = content.push(element);
                    }

                    let center_content = Container::new(content)
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .align_x(Alignment::Center)
                        .align_y(Alignment::Center);

                    let back_btn = back_button(i18n, "test-back", Message::Back);
                    let top_bar = Container::new(row![back_btn].spacing(10).padding(10))
                        .width(Length::Fill)
                        .align_x(Alignment::Start)
                        .align_y(Alignment::Start);

                    container(stack![center_content, top_bar])
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .into()
                } else {
                    let error_text = text(i18n.get("test-no-cards", None))
                        .size(18)
                        .shaping(iced::widget::text::Shaping::Advanced);

                    Container::new(error_text)
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .align_x(Alignment::Center)
                        .align_y(Alignment::Center)
                        .into()
                }
            }

            ScreenState::Results { passed } => {
                let title = text(if *passed {
                    i18n.get("test-test-passed", None)
                } else {
                    i18n.get("test-test-failed", None)
                })
                .size(28)
                .shaping(iced::widget::text::Shaping::Advanced);

                let message = text(if *passed {
                    i18n.get("test-passed-message", None)
                } else {
                    i18n.get("test-failed-message", None)
                })
                .size(16)
                .shaping(iced::widget::text::Shaping::Advanced);

                let retry_btn = action_button(i18n, "test-retry-test", Some(Message::RetryTest));

                let content = column![title, message, retry_btn]
                    .spacing(30)
                    .align_x(Alignment::Center);

                let center_content = Container::new(content)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .align_x(Alignment::Center)
                    .align_y(Alignment::Center);

                let back_btn = back_button(i18n, "test-back", Message::Back);
                let top_bar = Container::new(row![back_btn].spacing(10).padding(10))
                    .width(Length::Fill)
                    .align_x(Alignment::Start)
                    .align_y(Alignment::Start);

                container(stack![center_content, top_bar])
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .into()
            }
        }
    }
}

/// Implementation of RouterNode for TestRouter
impl RouterNode for TestRouter {
    fn router_name(&self) -> &'static str {
        "test"
    }

    fn update(
        &mut self,
        message: &router::Message,
    ) -> (Option<RouterEvent>, Task<router::Message>) {
        match message {
            router::Message::Test(msg) => {
                let (event, task) = TestRouter::update(self, msg.clone());
                let mapped_task = task.map(router::Message::Test);
                (event, mapped_task)
            }
            _ => (None, Task::none()),
        }
    }

    fn view(&self) -> Element<'_, router::Message> {
        TestRouter::view(self).map(router::Message::Test)
    }

    fn theme(&self) -> iced::Theme {
        self.user_state
            .theme
            .clone()
            .unwrap_or(self.app_state.theme())
    }

    fn init(&mut self, incoming_task: Task<router::Message>) -> Task<router::Message> {
        // Start loading session immediately
        let app_api = Arc::clone(&self.app_api);
        let username = self.user_state.username.clone();
        let profile_name = self.profile_state.profile_name.clone();

        let load_task = Task::perform(
            async move {
                app_api
                    .profile_api()
                    .create_test_session(&username, &profile_name)
                    .await
                    .map_err(|e| format!("Failed to create test session: {}", e))
            },
            |result| router::Message::Test(Message::SessionStarted(result)),
        );

        Task::batch([incoming_task, load_task])
    }

    fn subscription(&self) -> Subscription<router::Message> {
        event::listen().map(|e| router::Message::Test(Message::Event(e)))
    }
}
