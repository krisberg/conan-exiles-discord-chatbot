# conan-exiles-discord-chatbot

Commands:

`!status` Searches the server log for the latest report and returns current player count, server uptime and cpu usage.

`!listplayers` Runs the RCON command `listplayers` on the game server. Returns a list of currently online players.

## Usage

Requires https://sourceforge.net/projects/mcrcon/ in `/usr/bin/`

Set these environment variables:

`DISCORD_TOKEN` Discord app bot user token

`CONAN_DIR` Path to Conan Exiles dedicated server root directory.

`RCON_PASSWORD` Password for server Conan Exiles dedicated server RCON.
