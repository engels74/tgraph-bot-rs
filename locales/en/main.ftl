# English (US) localization for TGraph Bot

# Common messages
hello = Hello!
welcome = Welcome to TGraph Bot!
error = An error occurred: {$message}
success = Operation completed successfully!
loading = Loading...
please-wait = Please wait...

# Commands
command-about = Shows information about the bot
command-help = Shows available commands
command-stats = Shows your statistics
command-update-graphs = Updates graph data (admin only)
command-metrics = Shows bot metrics (admin only)

# About command
about-title = About TGraph Bot
about-description = TGraph Bot is a powerful data visualization and analytics bot for Discord.
about-version = Version: {$version}
about-uptime = Uptime: {$uptime}
about-servers = Servers: {$count}
about-users = Users: {$count}

# Error messages
error-permission-denied = You don't have permission to use this command.
error-cooldown = Please wait {$seconds} seconds before using this command again.
error-invalid-input = Invalid input provided.
error-command-failed = Command execution failed.
error-not-found = The requested item was not found.

# Success messages
success-data-updated = Data has been successfully updated.
success-settings-saved = Settings have been saved.

# Time units
time-seconds = {$count ->
    [one] {$count} second
   *[other] {$count} seconds
}
time-minutes = {$count ->
    [one] {$count} minute
   *[other] {$count} minutes
}
time-hours = {$count ->
    [one] {$count} hour
   *[other] {$count} hours
}
time-days = {$count ->
    [one] {$count} day
   *[other] {$count} days
}

# Graph types
graph-line = Line Chart
graph-bar = Bar Chart
graph-pie = Pie Chart
graph-scatter = Scatter Plot

# Status messages
status-online = Online
status-offline = Offline
status-maintenance = Under Maintenance
