//! Learn router for study and test sessions.
//!
//! This router manages the learning flow:
//! 1. **Start Screen**: User inputs starting card number
//! 2. **Study Phase**: Show cards one by one with full information
//! 3. **Test Phase**: Test user on the same cards with answer validation
//! 4. **Results**: Show pass/fail and allow retry or advance to next set
//!
//! # Architecture
//!
//! - **Async Loading**: Session creation and answer checking are async operations
//! - **ProfileState**: Read-only reference to profile data
//! - **UserState**: Read-only reference from parent router
//! - **LearningSession**: Tracks progress through study and test phases

use std::rc::Rc;
use std::sync::Arc;

use iced::keyboard::{key::Named, Key};
use iced::widget::{column, container, row, stack, text, text_input, Column, Container};
use iced::{event, Alignment, Element, Event, Length, Subscription, Task};

use lh_api::app_api::AppApi;
use lh_api::models::learning_session::LearningSessionDto;

use crate::app_state::AppState;
use crate::components::back_button::back_button;
use crate::router::{self, RouterEvent, RouterNode};
use crate::routers::learn::message::Message;
use crate::states::{ProfileState, UserState};

use super::elements::action_button::action_button;
use super::elements::answer_input::AnswerInputMessage;
use super::elements::card_display::card_display;

/// State for different screens in the learn flow
#[derive(Debug, Clone)]
pub enum ScreenState {
    /// Initial screen to input start card number
    Start {
        start_card_input: String,
        error_message: Option<String>,
    },
    /// Loading session
    Loading,
    /// Study phase - showing cards
    Study { session: LearningSessionDto },
    /// Test phase - testing knowledge
    Test {
        session: LearningSessionDto,
        answer_input: String,
        feedback: Option<AnswerFeedback>,
        /// For self-review mode: tracks if answer has been shown
        answer_shown: bool,
    },
    /// Results screen after completing test
    Results {
        session: LearningSessionDto,
        passed: bool,
    },
}

/// Feedback after submitting an answer
#[derive(Debug, Clone)]
pub enum AnswerFeedback {
    /// Correct answer
    Correct { matched_answer: String },
    /// Incorrect answer
    Incorrect { expected_answer: String },
}

/// State for the learn router
pub struct LearnRouter {
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

impl LearnRouter {
    /// Creates a new learn router.
    ///
    /// # Arguments
    ///
    /// * `user_state` - User context (read-only reference)
    /// * `profile_state` - Profile context (read-only reference)
    /// * `app_api` - The API instance for backend communication
    /// * `app_state` - Global application state (read-only reference)
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
            screen_state: ScreenState::Start {
                start_card_input: String::from("1"),
                error_message: None,
            },
        }
    }

    /// Update the router state based on messages.
    pub fn update(&mut self, message: Message) -> (Option<RouterEvent>, Task<Message>) {
        match message {
            Message::StartCardNumberChanged(value) => {
                if let ScreenState::Start {
                    start_card_input, ..
                } = &mut self.screen_state
                {
                    *start_card_input = value;
                }
                (None, Task::none())
            }

            Message::Start => {
                if let ScreenState::Start {
                    start_card_input, ..
                } = &self.screen_state
                {
                    // Parse start card number
                    match start_card_input.parse::<usize>() {
                        Ok(start_number) if start_number > 0 => {
                            self.screen_state = ScreenState::Loading;

                            // Create learning session
                            let app_api = Arc::clone(&self.app_api);
                            let username = self.user_state.username.clone();
                            let profile_name = self.profile_state.profile_name.clone();

                            let task = Task::perform(
                                async move {
                                    app_api
                                        .profile_api()
                                        .create_learning_session(
                                            &username,
                                            &profile_name,
                                            start_number,
                                        )
                                        .await
                                        .map_err(|e| format!("Failed to create session: {}", e))
                                },
                                |result| match result {
                                    Ok(session) => Message::SessionStarted(Ok(session)),
                                    Err(e) => Message::SessionStarted(Err(e)),
                                },
                            );

                            (None, task)
                        }
                        _ => {
                            self.screen_state = ScreenState::Start {
                                start_card_input: start_card_input.clone(),
                                error_message: Some("Please enter a valid card number".to_string()),
                            };
                            (None, Task::none())
                        }
                    }
                } else {
                    (None, Task::none())
                }
            }

            Message::SessionStarted(result) => match result {
                Ok(session) => {
                    // Transition to study phase
                    self.screen_state = ScreenState::Study { session };
                    (None, Task::none())
                }
                Err(err) => {
                    self.screen_state = ScreenState::Start {
                        start_card_input: String::from("1"),
                        error_message: Some(err),
                    };
                    (None, Task::none())
                }
            },

            Message::NextCardInStudy => {
                if let ScreenState::Study { session } = &mut self.screen_state {
                    // Move to next card in the set
                    let mut updated_session = session.clone();
                    updated_session.current_card_in_set += 1;

                    // Calculate actual set size (may be smaller than cards_per_set if few cards remaining)
                    let remaining_cards =
                        updated_session.all_cards.len() - updated_session.current_set_start_index;
                    let actual_set_size = updated_session.cards_per_set.min(remaining_cards);

                    // Check if we've reached the end of the set
                    if updated_session.current_card_in_set >= actual_set_size {
                        // Reset to first card and transition to test phase
                        updated_session.current_card_in_set = 0;
                        updated_session.phase =
                            lh_api::models::learning_session::LearningPhase::Test;

                        self.screen_state = ScreenState::Test {
                            session: updated_session,
                            answer_input: String::new(),
                            feedback: None,
                            answer_shown: false,
                        };
                    } else {
                        // Stay in study phase with next card
                        self.screen_state = ScreenState::Study {
                            session: updated_session,
                        };
                    }
                }
                (None, Task::none())
            }

            Message::StartTest => {
                if let ScreenState::Study { session } = &self.screen_state {
                    // Transition to test phase
                    let mut updated_session = session.clone();
                    updated_session.current_card_in_set = 0;
                    updated_session.phase = lh_api::models::learning_session::LearningPhase::Test;

                    self.screen_state = ScreenState::Test {
                        session: updated_session,
                        answer_input: String::new(),
                        feedback: None,
                        answer_shown: false,
                    };
                }
                (None, Task::none())
            }

            Message::AnswerInputChanged(value) => {
                if let ScreenState::Test { answer_input, .. } = &mut self.screen_state {
                    *answer_input = value;
                }
                (None, Task::none())
            }

            Message::SubmitAnswer => {
                if let ScreenState::Test {
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
                if let ScreenState::Test {
                    session,
                    answer_input,
                    feedback,
                    ..
                } = &mut self.screen_state
                {
                    match result {
                        Ok((is_correct, matched_answer)) => {
                            if is_correct {
                                // Update session with the provided answer
                                let mut updated_session = session.clone();
                                updated_session
                                    .current_card_provided_answers
                                    .push(answer_input.trim().to_string());

                                // Check if card is complete
                                let current_index = updated_session.current_set_start_index
                                    + updated_session.current_card_in_set;
                                let card = &updated_session.all_cards[current_index];

                                let required_answers = match card.card_type {
                                    lh_api::models::card::CardType::Straight => card.meanings.len(),
                                    lh_api::models::card::CardType::Reverse => card
                                        .meanings
                                        .iter()
                                        .map(|m| m.word_translations.len())
                                        .sum(),
                                };

                                *session = updated_session;
                                *answer_input = String::new();
                                *feedback = Some(AnswerFeedback::Correct { matched_answer });

                                // If card is complete, Continue button will advance
                                if session.current_card_provided_answers.len() >= required_answers {
                                    // Card complete - Continue will move to next card
                                } else {
                                    // More answers needed - Continue will clear feedback
                                }
                            } else {
                                // Incorrect answer - mark card as failed
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

            Message::Continue => {
                if let ScreenState::Test { session, .. } = &self.screen_state {
                    let mut updated_session = session.clone();

                    // Check if card is complete
                    let current_index = updated_session.current_set_start_index
                        + updated_session.current_card_in_set;
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
                        // Record test result before clearing
                        let result = lh_api::models::test_result::TestResultDto {
                            word_name: card.word.name.clone(),
                            is_correct: !session.current_card_failed,
                            user_answer: Some(session.current_card_provided_answers.join(", ")),
                            expected_answer: None, // Will be set by backend if needed
                        };
                        updated_session.test_results.push(result);

                        // Move to next card
                        updated_session.current_card_in_set += 1;
                        updated_session.current_card_provided_answers.clear();
                        updated_session.current_card_failed = false;

                        // Calculate actual set size
                        let remaining_cards = updated_session.all_cards.len()
                            - updated_session.current_set_start_index;
                        let actual_set_size = updated_session.cards_per_set.min(remaining_cards);

                        // Check if set is complete
                        if updated_session.current_card_in_set >= actual_set_size {
                            // All cards in set tested - show results
                            let passed = updated_session.test_results.iter().all(|r| r.is_correct);

                            self.screen_state = ScreenState::Results {
                                session: updated_session,
                                passed,
                            };
                        } else {
                            // Continue testing next card
                            self.screen_state = ScreenState::Test {
                                session: updated_session,
                                answer_input: String::new(),
                                feedback: None,
                                answer_shown: false,
                            };
                        }
                    } else {
                        // More answers needed - just clear feedback
                        self.screen_state = ScreenState::Test {
                            session: updated_session,
                            answer_input: String::new(),
                            feedback: None,
                            answer_shown: false,
                        };
                    }
                }
                (None, Task::none())
            }

            Message::RetrySet => {
                if let ScreenState::Results { session, .. } = &self.screen_state {
                    // Reset to study phase with same set
                    let mut updated_session = session.clone();
                    updated_session.current_card_in_set = 0;
                    updated_session.phase = lh_api::models::learning_session::LearningPhase::Study;
                    updated_session.test_results.clear();
                    updated_session.current_card_provided_answers.clear();
                    updated_session.current_card_failed = false;

                    self.screen_state = ScreenState::Study {
                        session: updated_session,
                    };
                }
                (None, Task::none())
            }

            Message::NextSet => {
                if let ScreenState::Results { session, .. } = &self.screen_state {
                    // Move to next set
                    let mut updated_session = session.clone();
                    updated_session.current_set_start_index += updated_session.cards_per_set;
                    updated_session.current_card_in_set = 0;
                    updated_session.phase = lh_api::models::learning_session::LearningPhase::Study;
                    updated_session.test_results.clear();
                    updated_session.current_card_provided_answers.clear();
                    updated_session.current_card_failed = false;

                    // Check if there are more cards available
                    if updated_session.current_set_start_index >= updated_session.all_cards.len() {
                        // No more cards - go back to start screen
                        self.screen_state = ScreenState::Start {
                            start_card_input: String::from("1"),
                            error_message: Some("All cards completed!".to_string()),
                        };
                    } else {
                        // Start next set
                        self.screen_state = ScreenState::Study {
                            session: updated_session,
                        };
                    }
                }
                (None, Task::none())
            }

            Message::ShowAnswer => {
                if let ScreenState::Test { answer_shown, .. } = &mut self.screen_state {
                    *answer_shown = true;
                }
                (None, Task::none())
            }

            Message::AnswerCorrect => {
                if let ScreenState::Test { session, .. } = &self.screen_state {
                    let mut updated_session = session.clone();

                    // Get current card
                    let current_index = updated_session.current_set_start_index
                        + updated_session.current_card_in_set;
                    let card = &updated_session.all_cards[current_index];

                    // Record correct result
                    let result = lh_api::models::test_result::TestResultDto {
                        word_name: card.word.name.clone(),
                        is_correct: true,
                        user_answer: None, // Self-review has no user input
                        expected_answer: None,
                    };
                    updated_session.test_results.push(result);

                    // Move to next card
                    updated_session.current_card_in_set += 1;

                    // Calculate actual set size
                    let remaining_cards =
                        updated_session.all_cards.len() - updated_session.current_set_start_index;
                    let actual_set_size = updated_session.cards_per_set.min(remaining_cards);

                    // Check if set is complete
                    if updated_session.current_card_in_set >= actual_set_size {
                        // All cards in set tested - show results
                        let passed = updated_session.test_results.iter().all(|r| r.is_correct);

                        self.screen_state = ScreenState::Results {
                            session: updated_session,
                            passed,
                        };
                    } else {
                        // Continue testing next card
                        self.screen_state = ScreenState::Test {
                            session: updated_session,
                            answer_input: String::new(),
                            feedback: None,
                            answer_shown: false,
                        };
                    }
                }
                (None, Task::none())
            }

            Message::AnswerIncorrect => {
                if let ScreenState::Test { session, .. } = &self.screen_state {
                    let mut updated_session = session.clone();

                    // Get current card
                    let current_index = updated_session.current_set_start_index
                        + updated_session.current_card_in_set;
                    let card = &updated_session.all_cards[current_index];

                    // Record incorrect result
                    let result = lh_api::models::test_result::TestResultDto {
                        word_name: card.word.name.clone(),
                        is_correct: false,
                        user_answer: None, // Self-review has no user input
                        expected_answer: None,
                    };
                    updated_session.test_results.push(result);

                    // Move to next card
                    updated_session.current_card_in_set += 1;

                    // Calculate actual set size
                    let remaining_cards =
                        updated_session.all_cards.len() - updated_session.current_set_start_index;
                    let actual_set_size = updated_session.cards_per_set.min(remaining_cards);

                    // Check if set is complete
                    if updated_session.current_card_in_set >= actual_set_size {
                        // All cards in set tested - show results
                        let passed = updated_session.test_results.iter().all(|r| r.is_correct);

                        self.screen_state = ScreenState::Results {
                            session: updated_session,
                            passed,
                        };
                    } else {
                        // Continue testing next card
                        self.screen_state = ScreenState::Test {
                            session: updated_session,
                            answer_input: String::new(),
                            feedback: None,
                            answer_shown: false,
                        };
                    }
                }
                (None, Task::none())
            }

            Message::Back => (Some(RouterEvent::Pop), Task::none()),

            Message::Event(event) => {
                // Handle keyboard events
                if let Event::Keyboard(iced::keyboard::Event::KeyPressed { key, .. }) = event {
                    match key {
                        // Enter key: Submit answer in Test phase, navigate in Study phase
                        Key::Named(Named::Enter) => {
                            if let ScreenState::Test {
                                answer_input,
                                feedback,
                                ..
                            } = &self.screen_state
                            {
                                // If no feedback shown and input is not empty, submit answer
                                if feedback.is_none() && !answer_input.trim().is_empty() {
                                    return self.update(Message::SubmitAnswer);
                                } else if feedback.is_some() {
                                    // If feedback is shown, continue to next card
                                    return self.update(Message::Continue);
                                }
                            } else if let ScreenState::Study { .. } = &self.screen_state {
                                // In Study screen, Enter triggers NextCardInStudy
                                return self.update(Message::NextCardInStudy);
                            } else if let ScreenState::Start {
                                start_card_input, ..
                            } = &self.screen_state
                            {
                                // In Start screen, Enter triggers Start
                                if !start_card_input.trim().is_empty() {
                                    return self.update(Message::Start);
                                }
                            }
                        }
                        // Escape key: Go back
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

    /// Render the router's view.
    pub fn view(&self) -> Element<'_, Message> {
        let i18n = &self.app_state.i18n();

        match &self.screen_state {
            ScreenState::Start {
                start_card_input,
                error_message,
            } => {
                // Start screen UI
                let title = text(i18n.get("learn-title", None))
                    .size(24)
                    .shaping(iced::widget::text::Shaping::Advanced);

                let instruction = text(i18n.get("learn-start-instruction", None))
                    .size(14)
                    .shaping(iced::widget::text::Shaping::Advanced);

                let input = text_input(
                    &i18n.get("learn-card-number-placeholder", None),
                    start_card_input,
                )
                .on_input(Message::StartCardNumberChanged)
                .padding(10)
                .width(Length::Fixed(200.0));

                let start_btn = action_button(i18n, "learn-start-button", Some(Message::Start));

                let mut content = column![title, instruction, input, start_btn]
                    .spacing(20)
                    .align_x(Alignment::Center);

                if let Some(err) = error_message {
                    let error_text = text(err)
                        .size(14)
                        .shaping(iced::widget::text::Shaping::Advanced)
                        .color([1.0, 0.0, 0.0]);
                    content = content.push(error_text);
                }

                // Center content
                let center_content = Container::new(content)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .align_x(Alignment::Center)
                    .align_y(Alignment::Center);

                // Top-left: Back button
                let back_btn = back_button(i18n, "learn-back", Message::Back);
                let top_bar = Container::new(row![back_btn].spacing(10).padding(10))
                    .width(Length::Fill)
                    .align_x(Alignment::Start)
                    .align_y(Alignment::Start);

                // Use stack to overlay back button on top of centered content
                container(stack![center_content, top_bar])
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .into()
            }

            ScreenState::Loading => {
                let title = text(i18n.get("learn-loading", None))
                    .size(24)
                    .shaping(iced::widget::text::Shaping::Advanced);

                Container::new(title)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .align_x(Alignment::Center)
                    .align_y(Alignment::Center)
                    .into()
            }

            ScreenState::Study { session } => {
                // Study phase UI
                let current_index = session.current_set_start_index + session.current_card_in_set;
                // Calculate actual set size
                let remaining_cards = session.all_cards.len() - session.current_set_start_index;
                let actual_set_size = session.cards_per_set.min(remaining_cards);

                if let Some(card) = session.all_cards.get(current_index) {
                    let card_view =
                        card_display(i18n, card, session.current_card_in_set + 1, actual_set_size);

                    let next_btn =
                        action_button(i18n, "learn-next-card", Some(Message::NextCardInStudy));

                    let start_test_btn =
                        action_button(i18n, "learn-start-test", Some(Message::StartTest));

                    let button_row = row![next_btn, start_test_btn]
                        .spacing(20)
                        .align_y(Alignment::Center);

                    let content = column![card_view, button_row]
                        .spacing(20)
                        .align_x(Alignment::Center);

                    // Center content vertically and horizontally
                    let center_content = Container::new(content)
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .align_x(Alignment::Center)
                        .align_y(Alignment::Center);

                    // Top-left: Back button
                    let back_btn = back_button(i18n, "learn-back", Message::Back);
                    let top_bar = Container::new(row![back_btn].spacing(10).padding(10))
                        .width(Length::Fill)
                        .align_x(Alignment::Start)
                        .align_y(Alignment::Start);

                    container(stack![center_content, top_bar])
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .into()
                } else {
                    // No cards available
                    let error_text = text(i18n.get("learn-no-cards", None))
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

            ScreenState::Test {
                session,
                answer_input,
                feedback,
                answer_shown,
            } => {
                // Test phase UI
                let current_index = session.current_set_start_index + session.current_card_in_set;
                if let Some(card) = session.all_cards.get(current_index) {
                    let mut content_elements = Vec::new();

                    // Check if card is complete (all answers given OR failed)
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

                    // Check test method
                    let is_self_review = session.test_method == "self_review";

                    // Determine if full card should be shown
                    // - Manual mode: show full card if card is complete
                    // - Self-review mode: show full card if answer has been shown
                    let show_full_card = if is_self_review {
                        *answer_shown
                    } else {
                        is_card_complete
                    };

                    // Show full card or just the word
                    if show_full_card {
                        // Calculate actual set size for card display
                        let remaining_cards =
                            session.all_cards.len() - session.current_set_start_index;
                        let actual_set_size = session.cards_per_set.min(remaining_cards);

                        let card_view = card_display(
                            i18n,
                            card,
                            session.current_card_in_set + 1,
                            actual_set_size,
                        );
                        content_elements.push(card_view);
                    } else {
                        // Show just the word for testing
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

                    // Calculate remaining answers needed
                    let required_answers = match card.card_type {
                        lh_api::models::card::CardType::Straight => card.meanings.len(),
                        lh_api::models::card::CardType::Reverse => card
                            .meanings
                            .iter()
                            .map(|m| m.word_translations.len())
                            .sum(),
                    };
                    let remaining_count = required_answers
                        .saturating_sub(session.current_card_provided_answers.len());

                    // Check test method to determine UI
                    let is_self_review = session.test_method == "self_review";

                    if is_self_review {
                        // Self-review mode: Show Answer button, then Correct/False buttons
                        if !answer_shown {
                            // Show "Show Answer" button
                            let show_answer_btn =
                                action_button(i18n, "learn-show-answer", Some(Message::ShowAnswer));
                            content_elements.push(show_answer_btn.into());
                        } else {
                            // Answer is shown - display Correct/False buttons
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
                        // Manual mode: Answer input and Submit button (only show if no feedback yet)
                        if feedback.is_none() {
                            let input_element = super::elements::answer_input::answer_input(
                                i18n,
                                answer_input,
                                remaining_count,
                            )
                            .map(|msg| match msg {
                                AnswerInputMessage::Changed(v) => Message::AnswerInputChanged(v),
                            });

                            content_elements.push(input_element);

                            // Submit button
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

                        // Show feedback if available
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

                    // Build column from elements - centered
                    let mut content = Column::new().spacing(20).align_x(Alignment::Center);
                    for element in content_elements {
                        content = content.push(element);
                    }

                    // Center content
                    let center_content = Container::new(content)
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .align_x(Alignment::Center)
                        .align_y(Alignment::Center);

                    // Top-left: Back button
                    let back_btn = back_button(i18n, "learn-back", Message::Back);
                    let top_bar = Container::new(row![back_btn].spacing(10).padding(10))
                        .width(Length::Fill)
                        .align_x(Alignment::Start)
                        .align_y(Alignment::Start);

                    container(stack![center_content, top_bar])
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .into()
                } else {
                    // No cards available
                    let error_text = text(i18n.get("learn-no-cards", None))
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

            ScreenState::Results { session: _, passed } => {
                // Results UI
                let title = text(if *passed {
                    i18n.get("learn-test-passed", None)
                } else {
                    i18n.get("learn-test-failed", None)
                })
                .size(28)
                .shaping(iced::widget::text::Shaping::Advanced);

                let message = text(if *passed {
                    i18n.get("learn-passed-message", None)
                } else {
                    i18n.get("learn-failed-message", None)
                })
                .size(16)
                .shaping(iced::widget::text::Shaping::Advanced);

                let buttons = if *passed {
                    row![action_button(
                        i18n,
                        "learn-next-set",
                        Some(Message::NextSet)
                    )]
                    .spacing(20)
                    .align_y(Alignment::Center)
                } else {
                    row![action_button(
                        i18n,
                        "learn-retry-set",
                        Some(Message::RetrySet)
                    )]
                    .spacing(20)
                    .align_y(Alignment::Center)
                };

                let content = column![title, message, buttons]
                    .spacing(30)
                    .align_x(Alignment::Center);

                // Center content
                let center_content = Container::new(content)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .align_x(Alignment::Center)
                    .align_y(Alignment::Center);

                // Top-left: Back button
                let back_btn = back_button(i18n, "learn-back", Message::Back);
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

/// Implementation of RouterNode for LearnRouter
impl RouterNode for LearnRouter {
    fn router_name(&self) -> &'static str {
        "learn"
    }

    fn update(
        &mut self,
        message: &router::Message,
    ) -> (Option<RouterEvent>, Task<router::Message>) {
        match message {
            router::Message::Learn(msg) => {
                let (event, task) = LearnRouter::update(self, msg.clone());
                let mapped_task = task.map(router::Message::Learn);
                (event, mapped_task)
            }
            _ => (None, Task::none()),
        }
    }

    fn view(&self) -> Element<'_, router::Message> {
        LearnRouter::view(self).map(router::Message::Learn)
    }

    fn theme(&self) -> iced::Theme {
        self.user_state
            .theme
            .clone()
            .unwrap_or(self.app_state.theme())
    }

    fn init(&mut self, incoming_task: Task<router::Message>) -> Task<router::Message> {
        // No special initialization needed
        incoming_task
    }

    fn subscription(&self) -> Subscription<router::Message> {
        event::listen().map(|e| router::Message::Learn(Message::Event(e)))
    }
}
