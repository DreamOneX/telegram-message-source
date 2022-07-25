#![allow(incomplete_features)]
#![feature(backtrace, capture_disjoint_fields, let_chains)]

use bot::TG_BOT;
use color_eyre::eyre::Result;
use futures::FutureExt;
use mesagisto_client::MesagistoConfig;
use rust_i18n::t;
use teloxide::{prelude::*, types::ParseMode, Bot};
use tracing::*;

use self::message::handlers;
use crate::config::{Config, CONFIG};

#[macro_use]
extern crate educe;
#[macro_use]
extern crate automatic_config;
#[macro_use]
extern crate singleton;
#[macro_use]
extern crate rust_i18n;
i18n!("locales");

mod bot;
mod command;
mod config;
mod dispatch;
pub mod ext;
mod log;
mod message;
mod net;

const TARGET: &str = "mesagisto";

#[tokio::main]
async fn main() -> Result<()> {
  if cfg!(feature = "color") {
    color_eyre::install()?;
  } else {
    color_eyre::config::HookBuilder::new()
      .theme(color_eyre::config::Theme::new())
      .install()?;
  }

  self::log::init();
  run().await?;
  Ok(())
}

async fn run() -> Result<()> {
  Config::reload().await?;
  rust_i18n::set_locale(&CONFIG.locale);
  if !CONFIG.enable {
    warn!(target: TARGET, "{}", t!("log.not-enable"));
    warn!(target: TARGET, "{}", t!("log.not-enable-helper"));
    return Ok(());
  }
  CONFIG.migrate();

  MesagistoConfig::builder()
    .name("tg")
    .cipher_key(CONFIG.cipher.key.clone())
    .nats_address(CONFIG.nats.address.clone())
    .proxy(if CONFIG.proxy.enable {
      Some(CONFIG.proxy.address.clone())
    } else {
      None
    })
    .photo_url_resolver(|id_pair| {
      async {
        let file = String::from_utf8_lossy(&id_pair.1);
        let file_path = TG_BOT.get_file(file).await.unwrap().file_path;
        Ok(TG_BOT.get_url_by_path(file_path))
      }
      .boxed()
    })
    .build()
    .apply()
    .await?;
  info!(
    target: TARGET,
    "{}",
    t!("log.boot-start", version = env!("CARGO_PKG_VERSION"))
  );
  let bot = Bot::with_client(CONFIG.telegram.token.clone(), net::client_from_config())
    .parse_mode(ParseMode::Html)
    .auto_send();

  TG_BOT.init(bot).await?;

  handlers::receive::recover()?;
  tokio::spawn(async {
    dispatch::start(&TG_BOT).await;
  });
  tokio::signal::ctrl_c().await?;
  CONFIG.save().await.expect("保存配置文件失败");
  // info!(target: TARGET, "TG信使即将关闭");
  info!(target: TARGET,"{}",t!("log.shutdown"));
  Ok(())
}
