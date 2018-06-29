# conan-exiles-discord-chatbot

Commands:

`!status` Searches the server log for the latest report and returns current player count, server uptime and cpu usage.

`!listplayers` Runs the RCON command `listplayers` on the game server. Returns a list of currently online players.

`!listlastplayers` Uses the RCON SQL command to fetch character name, clan, level and last online date of the 10 players that were last active on the server.

## Usage

Set these environment variables:

`DISCORD_TOKEN` Discord app bot user token. Generate one [here](https://discordapp.com/developers/applications/me).

`SAVED_DIR` Path to Conan Exiles dedicated server saved directory. Ex `/path/to/ConanSandbox/Saved`

`RCON_PASSWORD` Password for Conan Exiles dedicated server RCON.
