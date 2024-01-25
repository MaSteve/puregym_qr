# puregym_qr
Small library in Rust containing a wrapper of PureGym's API for generating QR access codes. The repo also implements 3 binaries for making use of it in a Telegram bot:
- `bot.rs`: standard bot supporting polling and webhooks.
- `bot_lambda`: simplified version for hosting the bot in AWS Lambda.
- `bot_management`: CLI for registering and disabling webhooks.

Telegram's API token and other credentials are stored in a `secrets.json` file. `secrets.json.example` can be renamed and populated for this purpose.

More details can be found in [this post](https://raincoatmoon.com/blog/telegram-bot-in-aws-lambda/).
