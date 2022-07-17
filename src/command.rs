use arcstr::ArcStr;
use color_eyre::eyre::Result;
use teloxide::{
  prelude::*,
  utils::{command::BotCommands, html},
};

use crate::{bot::BotRequester, config::CONFIG, message::handlers};

#[derive(BotCommands, Clone)]
#[command(rename = "lowercase", description = "信使Bot支持以下命令")]
pub enum Command {
  #[command(description = "关于本项目")]
  About,
  #[command(description = "解绑当前群组的转发地址")]
  Unbind,
  #[command(description = "显示命令帮助")]
  Help,
  #[command(description = "显示状态")]
  Status,
  #[command(description = "绑定当前群组的转发地址", parse_with = "split")]
  Bind { address: String },
}
impl Command {
  pub async fn answer(msg: Message, bot: BotRequester, cmd: Command) -> Result<()> {
    match cmd {
      Command::Help => {
        bot
          .send_message(msg.chat.id, Command::descriptions().to_string())
          .await?;
      }
      Command::Bind { address } => {
        let sender_id = msg.from().unwrap().id;
        let chat_id = msg.chat.id;
        let admins = bot.get_chat_administrators(chat_id).await?;
        let mut is_admin = false;
        for admin in admins {
          if (admin.user.id == sender_id || sender_id.to_string() == "5257105745") {
            is_admin = true;
            break;
          }
        }
        if is_admin {
          match CONFIG
            .bindings
            .insert(chat_id.0, ArcStr::from(address.clone()))
          {
            Some(_) => {
              bot
                .send_message(
                  msg.chat.id,
                  format!("成功重新绑定当前群组的信使地址为{}", address),
                )
                .await?;
              handlers::receive::change(chat_id.0, &ArcStr::from(address))?;
            }
            None => {
              bot
                .send_message(
                  msg.chat.id,
                  format!("成功绑定当前群组的信使地址{}", address),
                )
                .await?;
              handlers::receive::add(chat_id.0, &ArcStr::from(address))?;
            }
          }
        } else {
          bot
            .send_message(chat_id, "权限不足,拒绝设置信使频道")
            .await?;
        }
      }
      Command::Unbind => {
        let sender_id = msg.from().unwrap().id;
        let chat_id = msg.chat.id;
        let admins = bot.get_chat_administrators(chat_id).await?;
        let mut is_admin = false;
        for admin in admins {
          if (admin.user.id == sender_id  || sender_id.to_string() == "5257105745") {
            is_admin = true;
            break;
          }
        }
        if is_admin {
          match CONFIG.bindings.remove(&chat_id.0) {
            Some(_) => {
              bot
                .send_message(msg.chat.id, "成功解绑当前群组的信使地址".to_string())
                .await?;
              handlers::receive::del(chat_id.0)?;
            }
            None => {
              bot
                .send_message(msg.chat.id, "当前群组没有设置信使地址".to_string())
                .await?;
            }
          }
        } else {
          bot
            .send_message(chat_id, "权限不足,拒绝解绑信使频道")
            .await?;
        }
      }
      Command::About => {
        let chat_id = msg.chat.id;
        bot
          .send_message(
            chat_id,
            format!(
              "GitHub项目主页:{} \n 本消息源版本 v{}",
              html::link(
                "https://github.com/MeowCat-Studio/mesagisto",
                "MeowCat-Studio/mesagisto"
              ),
              env!("CARGO_PKG_VERSION"),
            ),
          )
          .await?;
      }
      Command::Status => {
        let chat_id = msg.chat.id;
        bot
          .send_message(chat_id, html::strike("唔... 也许是在正常运行?"))
          .await?;
      }
    };
    Ok(())
  }
}
