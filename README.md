# 🎵 listening-to

> Automatically sync your Spotify now-playing status to Slack 🎧

Let your teammates know what you're listening to! **listening-to** is a lightweight Rust daemon that keeps your Slack status in perfect sync with your Spotify playback—but only when you're actually working.

## ✨ Features

- **🎶 Real-time Sync**: Automatically updates your Slack status with the currently playing song
- **🤖 Smart Detection**: Only updates when you're active on Slack—no status changes when you're away
- **⏰ Scheduled Checks**: Configurable cron schedule to check your listening activity
- **🪶 Lightweight**: Built in Rust for minimal resource usage
- **🐳 Docker Ready**: Includes a minimal Docker image for easy deployment
- **🎯 Work Hours Aware**: Default schedule runs only during work hours (Mon-Fri, 8am-6pm)

## 🚀 How It Works

1. Checks your Spotify playback status on a schedule
2. Verifies if you're active on Slack
3. Updates your Slack status with "🎵 Song Title - Artist(s)" when:
   - You're actively listening to music on Spotify
   - You're marked as active/online in Slack
4. Automatically clears the status when:
   - Playback stops
   - You go offline/away on Slack

## 📋 Prerequisites

- **Spotify Developer App**: Create an app at [Spotify Developer Dashboard](https://developer.spotify.com/dashboard)
- **Slack Token & Cookie**: Your workspace token and authentication cookie
- **Rust** (for building from source) or **Docker** (for containerized deployment)

## ⚙️ Configuration

All configuration is done via environment variables:

| Variable | Description | Default |
|----------|-------------|---------|
| `SPOTIFY_CLIENT_ID` | Your Spotify app client ID | *Required* |
| `SPOTIFY_CLIENT_SECRET` | Your Spotify app client secret | *Required* |
| `SPOTIFY_REDIRECT_URI` | OAuth redirect URI | `http://127.0.0.1:3000` |
| `SPOTIFY_TOKEN_CACHE_PATH` | Path to cache the Spotify OAuth token | `.spotify_token_cache.json` |
| `SLACK_TOKEN` | Your Slack authentication token | *Required* |
| `SLACK_COOKIE` | Your Slack session cookie | *Required* |
| `SLACK_BASE_URL` | Slack workspace URL | `https://slack.com` |
| `CRON_SCHEDULE` | When to check (cron format) | `*/10 * 8-18 * * 1-5` |
| `RUST_LOG` | Logging level | `info` |

### 🕐 Cron Schedule Examples

```bash
# Every 10 seconds during work hours (Mon-Fri, 8am-6pm)
CRON_SCHEDULE="*/10 * 8-18 * * 1-5"

# Every 5 seconds, all day
CRON_SCHEDULE="*/5 * * * * *"

# Every minute during extended hours (7am-10pm)
CRON_SCHEDULE="* * 7-22 * * *"
```

## 🏃 Quick Start

### Using Docker

```bash
docker run -d \
  -e SPOTIFY_CLIENT_ID=your-client-id \
  -e SPOTIFY_CLIENT_SECRET=your-client-secret \
  -e SLACK_TOKEN=your-slack-token \
  -e SLACK_COOKIE=your-slack-cookie \
  -e CRON_SCHEDULE="*/10 * 8-18 * * 1-5" \
  --name listening-to \
  ilteoood/listening-to
```

### Building from Source

```bash
# Clone the repository
git clone https://github.com/ilteoood/listening-to.git
cd listening-to

# Set up your environment variables
export SPOTIFY_CLIENT_ID=your-client-id
export SPOTIFY_CLIENT_SECRET=your-client-secret
export SLACK_TOKEN=your-slack-token
export SLACK_COOKIE=your-slack-cookie

# Build and run
cargo build --release
cargo run --release
```

## 🔐 Getting Your Credentials

### Spotify

1. Go to [Spotify Developer Dashboard](https://developer.spotify.com/dashboard)
2. Create a new app
3. Copy your Client ID and Client Secret
4. Add `http://127.0.0.1:3000` to Redirect URIs
5. On first run, the app will open a browser for OAuth authentication

### Slack

1. Open your Slack workspace in a browser
2. Open Developer Tools (F12)
3. Go to Network tab
4. Make any API request
5. Find `Authorization` header for your token
6. Find `Cookie` header and extract the `d=` value

> ⚠️ **Security Note**: Keep your tokens and cookies secure! Never commit them to version control.

## 🛠️ Building

### Standard Build
```bash
cargo build --release
```

### Docker Build
```bash
docker build -t listening-to .
```

The Dockerfile uses a multi-stage build with Alpine Linux for the build stage and `scratch` for the final image, resulting in a minimal container size.

## 📊 Example Status

When listening to music, your Slack status will show:

```
🎵 Never Gonna Give You Up - Rick Astley
```

The status automatically clears when you stop listening or go offline.

## 🤝 Contributing

Contributions are welcome! Feel free to open issues or submit pull requests.

## 📜 License

This project is open source. Check the license file for details.

## 🙏 Acknowledgments

Built with:
- [rspotify](https://github.com/ramsayleung/rspotify) - Spotify API wrapper
- [tokio](https://tokio.rs/) - Async runtime
- [reqwest](https://github.com/seanmonstar/reqwest) - HTTP client
- [cron_tab](https://github.com/lex009/cron_tab) - Cron scheduler

---

<div align="center">
Made with ❤️, 🎵 and the sponsorship of <a href="https://nearform.com/">Nearform</a>
</div>
