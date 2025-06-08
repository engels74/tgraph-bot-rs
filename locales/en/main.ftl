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

# Bot information messages
bot-title = TGraph Discord Bot
bot-description = A powerful bot for generating and sharing Tautulli statistics graphs
bot-version = Version: {$version}
bot-built-with = Built with Rust and Poise
bot-features = Features: Graph generation, statistics tracking, and more!

# Uptime messages
uptime-title = Bot Uptime & Statistics
uptime-duration = Uptime: {$hours}h {$minutes}m {$seconds}s
uptime-commands-executed = Commands executed: {$total} (✅ {$success} succeeded, ❌ {$failed} failed)
uptime-status-ready = Status: Online and ready!

# User statistics messages
stats-title = Your {$period} Statistics
stats-period = Period: {$range}
stats-command-usage = Command Usage
stats-total-commands = Total Commands: {$count}
stats-successful = Successful: {$count} ({$percentage}%)
stats-failed = Failed: {$count}
stats-avg-response-time = Avg Response Time: {$time}ms
stats-most-used = Most Used
stats-most-used-command = Command: {$command}
stats-most-active-hour = Active Hour: {$hour}
stats-most-active-day = Active Day: {$day}
stats-top-commands = Top Commands
stats-activity-scope = Activity Scope
stats-unique-channels = Unique Channels: {$count}
stats-unique-servers = Unique Servers: {$count}
stats-timeline = Timeline
stats-first-command = First Command: {$time}
stats-latest-command = Latest Command: {$time}
stats-no-data = N/A
stats-none = None
stats-all-time = All Time

# Admin messages
admin-update-graphs-title = Graph Update Initiated
admin-update-graphs-starting = Starting graph regeneration process...
admin-update-graphs-wait = This may take a few moments to complete.
admin-update-graphs-updating = All graphs will be updated with the latest data from Tautulli.

admin-metrics-title = Bot Metrics Report
admin-metrics-uptime = Uptime: {$hours}h {$minutes}m
admin-metrics-total-commands = Total Commands: {$total} (Success Rate: {$rate}%)
admin-metrics-avg-response = Avg Response: {$time}ms
admin-metrics-command-usage = Command Usage
admin-metrics-last-24h = Last 24h: {$count} commands

admin-scheduler-title = Scheduling System Status
admin-scheduler-core = Core Scheduler: Integrated and ready
admin-scheduler-task-manager = Task Manager: Background task management enabled
admin-scheduler-task-queue = Task Queue: Priority queue with retry logic active
admin-scheduler-monitoring = Monitoring: Metrics collection and alerting configured
admin-scheduler-persistence = Persistence: Schedule recovery and database storage ready
admin-scheduler-description = The scheduling system is fully integrated and ready to handle automated tasks.
admin-scheduler-usage = Use this system for automated graph generation, cleanup tasks, and more.

# Data export messages
export-title = Your Complete Data Export
export-details = Export Details
export-generated = Generated: {$time}
export-command-executions = Command Executions: {$count}
export-account-created = Account Created: {$time}
export-last-updated = Last Updated: {$time}
export-privacy-notice = Privacy Notice
export-contains-all-data = This export contains ALL data we have stored about your account.
export-sent-privately = This data is sent privately and confidentially.
export-deletion-info = You can request data deletion using `/delete_my_data`.
export-gdpr-compliance = This export complies with GDPR and data protection regulations.
export-data-format = Your data (JSON format):

# Data deletion messages
delete-confirmation-required = Type 'CONFIRM' to permanently delete all your data
delete-success = Your data has been permanently deleted from our systems.
delete-no-data = No data found for your account.

# Time period names
period-daily = Daily
period-weekly = Weekly
period-monthly = Monthly
period-all-time = All Time

# Graph generation messages
graph-success-title = Graph Generated Successfully
graph-success-description = Your {$type} graph has been generated and is ready for viewing.
graph-error-title = Graph Generation Failed
graph-error-description = There was an error generating your {$type} graph: {$error}
graph-processing = Processing your {$command} command...

# Permission messages
permission-error-title = Permission Denied
permission-error-description = I don't have the required permissions to {$action} in this channel.
permission-required = Required: {$permissions}
permission-action = Action: {$action}
