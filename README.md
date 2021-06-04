# telegram-capture-bot

The [Emacs Meetup FFM](https://www.meetup.com/de-DE/emacs-ffm/) plans to build a [Telegram](https://telegram.org/) bot and [GNU Emacs](https://www.gnu.org/software/emacs/) plugin, that lets one capture ideas or TODOs on the go to [Org Mode](https://orgmode.org/).
This repository contains [spikes](https://en.wikipedia.org/wiki/Spike_(software_development)) using diffetent libraries.

There's a branch for each library:
- [frankenstein](https://github.com/ayrat555/frankenstein) ([spike/frankenstein](https://github.com/zoranzaric/telegram-capture-bot/tree/spike/frankenstein))
- [telegram-bot](https://github.com/telegram-rs/telegram-bot) ([spike/telegram-bot](https://github.com/zoranzaric/telegram-capture-bot/tree/spike/telegram-bot))
- [teloxide](https://github.com/teloxide/teloxide) ([spike/teloxide](https://github.com/zoranzaric/telegram-capture-bot/tree/spike/teloxide))

There will also be a HTTP API probably being implemented with [Rocket](https://rocket.rs/).

## Building

``` sh
cargo build --release
```

## Running
### Bot
First you have to create a Telegram bot (see [How do I create a bot?](https://core.telegram.org/bots#3-how-do-i-create-a-bot)).

``` sh
export TELEGRAM_BOT_TOKEN=<YOUR_SECRET_BOT_TOKEN>
cargo run --release --bin bot
```

### API server

``` sh
cargo run --release --bin api-server
```
