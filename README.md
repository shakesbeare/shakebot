# For Your Use

While this bot is primarily intended for use in my own servers, you can
certainly use it in yours. Fair warning, this bot requires permission to
read chat messages, so it may not be suitable to have it join >100 servers.
That all said, if you self host the bot, you almost certainly run into this
issue.

## Setup

In the future, there might be a more robust release mechanism, but at the moment
the best way to get this bot is by simply cloning the repository and hosting it
yourself.

- Create a [Discord App](https://discord.com/developers/docs/getting-started)
  - Make sure to copy the token for later
  - Discord Developers -> Applications -> Your App -> Bot
- Clone the git repository to the folder of your choice
- Create a file named `.env` in the project root
  - Add the following line: `DISCORD_TOKEN=<your token here>`
- Start the bot with `cargo run --release`

## Requirements

- [Rust](https://rustup.rs)

# Dota Responses

The primary feature of this bot is the Dota 2 Hero Response mechanism based on 
the Reddit bot by u/Jonarz and u/MePsyDuck. This feature is opt-in per user.

<img width="388" alt="image" src="https://github.com/shakesbeare/shakebot/assets/75107188/c6b02c3c-3ccc-48d3-9482-adaa0cd0c0d0">

The bot automatically creates a thread in response to itself to hold the
attachment with the audio file in it. The thread also holds a direct link to
audio for mobile users. Shakebot automatically closes the thread to avoid
unnecessary clutter.

# Copypastas

The second main feature of Shakebot is to host whatever copypastas the host may
want to have on hand. They are stored in a json file in the project root called
`copypastas.json` which has the following structure:

```json
{
  "badhabits": {
    "content": "do u guys have bad habits? i walk to gas station every day and buyed one diet pepis or one diet rebdull. this beverage does not make it the way back ... it become consume.",
    "guild": "the cave",
  }
}
```

At the moment, the `guild` field is non-functional. But it will eventually allow
you to store copypastas on a per-server basis. `copypastas.json` is not version
controlled to keep the repository clean and friendly.

<img width="921" alt="image" src="https://github.com/shakesbeare/shakebot/assets/75107188/a12caf5c-d8ef-411b-93a6-ea7ce748f677">

# Commands

Shakebot exposes the following commands to users:
- `/copypasta <copypasta_name>`
  - Send the contents of the copypasta to the chat
- `/disable`
  - Tell Shakebot not to send you dota responses anymore
- `/enable`
  - Re-enable dota responses, for cool people
- `/help` / `/help <command>`
  - Learn about the commands
- `/dota <phrase>`
    - Fuzzy find a Dota response

# Why not a database?

The objective was to be able to version control the data stored inside the database
so the bot could be maximally portable. My server is quite weak and the original code
form the Dota Response Reddit Bot took a very long time to finish.

While I have parallelized much of that process, increasing the speed greatly, I also
felt it would be quite convenient to simply be able to run a git command to update the
entire bot memory, if needed. This method also minimizes the number of times you'd need
to run all the api calls to MediaWiki, as having a separate database server seemed like
overkill for this project and hurt the portability greatly. 

# Planned Features

- Play dota voicelines into a voice channel
- Custom responses
- Implement guild-specific responses

# Acknowledgements

- [Dota Responses Reddit Bot](https://github.com/Jonarzz/DotaResponsesRedditBot)
  - For the idea and being the foundation for much of the code.
