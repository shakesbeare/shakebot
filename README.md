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
  "copypasta_name": {
    "content": "This will be sent to the server!",
    "guild": "My_Server",
  }
}
```

At the moment, the `guild` field is non-functional. But it will eventually allow
you to store copypastas on a per-server basis. `copypastas.json` is not version
controlled to keep the repository clean and friendly.

<img width="921" alt="image" src="https://github.com/shakesbeare/shakebot/assets/75107188/a12caf5c-d8ef-411b-93a6-ea7ce748f677">

# Commands

Shakebot exposes four commands to users:
- `/copypasta <copypasta_name>`
  - Send the contents of the copypasta to the chat
- `/enable`
  - Tell Shakebot not to send you dota responses anymore
- `/disable`
  - Re-enable dota responses, for cool people
- `/help` / `/help <command>`
  - Learn about the commands

# Why not a database?

The objective was to be able to version control the data stored inside the database
so the bot could be maximally portable. My server is quite weak and the original code
form the Dota Response Reddit Bot took a very long time to finish.

While I have parallelized much of that process, increasing the speed greatly, I also
felt it would be quite convenient to simply be able to run a git command to update the
entire bot memory, if needed. This method also minimizes the number of times you'd need
to run all the api calls to MediaWiki, as having a separate database server seemed like
overkill for this project and hurt the portability greatly. 

# Acknowledgements

- [Dota Responses Reddit Bot](https://github.com/Jonarzz/DotaResponsesRedditBot)
  - For the idea and being the foundation for much of the code.
