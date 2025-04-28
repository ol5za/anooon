use teloxide::prelude::*;
use teloxide::types::{InputFile, MessageKind, MediaKind, ReplyMarkup, ReplyKeyboardMarkup, KeyboardButton};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

type UserId = i64;
type PartnerMap = Arc<Mutex<HashMap<UserId, UserId>>>;
type FeedbackMap = Arc<Mutex<HashMap<UserId, bool>>>;

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    log::info!("Запуск Анонимного Чата...");

    let bot = Bot::new("7814168987:AAHJp2M-kd9k63WApKbv-h7fThUoCUCKDBU");
    let partners: PartnerMap = Arc::new(Mutex::new(HashMap::new()));
    let feedback: FeedbackMap = Arc::new(Mutex::new(HashMap::new()));

    teloxide::repl_with_listener(
        bot.clone(),
        move |bot: Bot, msg: Message| {
            let partners = partners.clone();
            let feedback = feedback.clone();
            async move {
                let chat_id = msg.chat.id;
                if let Some(text) = msg.text() {
                    match text {
                        "/start" => {
                            bot.send_message(chat_id, "Привет! Сейчас найду тебе собеседника...")
                                .await?;

                            let mut partners_lock = partners.lock().unwrap();
                            if let Some((&other_id, _)) = partners_lock.iter().find(|&(_, v)| *v == 0) {
                                // Подключаем пользователей
                                partners_lock.insert(chat_id, other_id);
                                partners_lock.insert(other_id, chat_id);

                                bot.send_message(chat_id, "Собеседник найден! Можешь начинать чат.")
                                    .await?;
                                bot.send_message(ChatId(other_id), "Собеседник найден! Можешь начинать чат.")
                                    .await?;
                            } else {
                                partners_lock.insert(chat_id, 0);
                                bot.send_message(chat_id, "Ожидание собеседника...")
                                    .await?;
                            }
                        }
                        "/stop" => {
                            let mut partners_lock = partners.lock().unwrap();
                            if let Some(&partner_id) = partners_lock.get(&chat_id) {
                                if partner_id != 0 {
                                    bot.send_message(ChatId(partner_id), "Собеседник покинул чат.")
                                        .await?;
                                    partners_lock.remove(&partner_id);
                                }
                                partners_lock.remove(&chat_id);
                            }
                            bot.send_message(chat_id, "Вы покинули чат. Оцените собеседника:")
                                .reply_markup(ReplyMarkup::Keyboard(ReplyKeyboardMarkup::new(vec![
                                    vec![
                                        KeyboardButton::new("👍"),
                                        KeyboardButton::new("👎"),
                                    ]
                                ]).resize_keyboard(true)))
                                .await?;

                            // Відмічаємо, що чекаємо на відгук
                            feedback.lock().unwrap().insert(chat_id, true);
                        }
                        "👍" | "👎" => {
                            let mut feedback_lock = feedback.lock().unwrap();
                            if feedback_lock.remove(&chat_id).is_some() {
                                bot.send_message(chat_id, "Спасибо за отзыв!")
                                    .await?;
                            } else {
                                bot.send_message(chat_id, "Вы ещё не завершили чат.")
                                    .await?;
                            }
                        }
                        _ => {
                            let partners_lock = partners.lock().unwrap();
                            if let Some(&partner_id) = partners_lock.get(&chat_id) {
                                if partner_id != 0 {
                                    bot.send_message(ChatId(partner_id), text)
                                        .await?;
                                } else {
                                    bot.send_message(chat_id, "Ожидаем собеседника...")
                                        .await?;
                                }
                            } else {
                                bot.send_message(chat_id, "Отправьте /start, чтобы начать чат.")
                                    .await?;
                            }
                        }
                    }
                } else {
                    // Пересылка других типов сообщений
                    let partners_lock = partners.lock().unwrap();
                    if let Some(&partner_id) = partners_lock.get(&chat_id) {
                        if partner_id != 0 {
                            match &msg.kind {
                                MessageKind::Common(common) => {
                                    match &common.media_kind {
                                        MediaKind::Photo { photo, caption, .. } => {
                                            bot.send_photo(ChatId(partner_id), InputFile::file_id(&photo.last().unwrap().file_id))
                                                .caption(caption.clone().unwrap_or_default())
                                                .await?;
                                        }
                                        MediaKind::Sticker(sticker) => {
                                            bot.send_sticker(ChatId(partner_id), InputFile::file_id(&sticker.file_id))
                                                .await?;
                                        }
                                        MediaKind::Video(video) => {
                                            bot.send_video(ChatId(partner_id), InputFile::file_id(&video.file.id))
                                                .await?;
                                        }
                                        MediaKind::Voice(voice) => {
                                            bot.send_voice(ChatId(partner_id), InputFile::file_id(&voice.file.id))
                                                .await?;
                                        }
                                        MediaKind::Audio(audio) => {
                                            bot.send_audio(ChatId(partner_id), InputFile::file_id(&audio.file.id))
                                                .await?;
                                        }
                                        _ => {}
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                }
                respond(())
            }
        },
        Update::filter_message(),
    )
    .await;
                      }
