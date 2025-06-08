# German (Germany) localization for TGraph Bot

# Common messages
hello = Hallo!
welcome = Willkommen bei TGraph Bot!
error = Ein Fehler ist aufgetreten: {$message}
success = Operation erfolgreich abgeschlossen!
loading = Lädt...
please-wait = Bitte warten...

# Commands
command-about = Zeigt Informationen über den Bot
command-help = Zeigt verfügbare Befehle
command-stats = Zeigt Ihre Statistiken
command-update-graphs = Aktualisiert Diagrammdaten (nur Admin)
command-metrics = Zeigt Bot-Metriken (nur Admin)

# About command
about-title = Über TGraph Bot
about-description = TGraph Bot ist ein leistungsstarker Datenvisualisierungs- und Analyse-Bot für Discord.
about-version = Version: {$version}
about-uptime = Betriebszeit: {$uptime}
about-servers = Server: {$count}
about-users = Benutzer: {$count}

# Error messages
error-permission-denied = Sie haben keine Berechtigung, diesen Befehl zu verwenden.
error-cooldown = Bitte warten Sie {$seconds} Sekunden, bevor Sie diesen Befehl erneut verwenden.
error-invalid-input = Ungültige Eingabe bereitgestellt.
error-command-failed = Befehlsausführung fehlgeschlagen.
error-not-found = Das angeforderte Element wurde nicht gefunden.

# Success messages
success-data-updated = Daten wurden erfolgreich aktualisiert.
success-settings-saved = Einstellungen wurden gespeichert.

# Time units
time-seconds = {$count ->
    [one] {$count} Sekunde
   *[other] {$count} Sekunden
}
time-minutes = {$count ->
    [one] {$count} Minute
   *[other] {$count} Minuten
}
time-hours = {$count ->
    [one] {$count} Stunde
   *[other] {$count} Stunden
}
time-days = {$count ->
    [one] {$count} Tag
   *[other] {$count} Tage
}

# Graph types
graph-line = Liniendiagramm
graph-bar = Balkendiagramm
graph-pie = Kreisdiagramm
graph-scatter = Streudiagramm

# Status messages
status-online = Online
status-offline = Offline
status-maintenance = In Wartung

# Bot information messages
bot-title = TGraph Discord Bot
bot-description = Ein leistungsstarker Bot zum Generieren und Teilen von Tautulli-Statistikdiagrammen
bot-version = Version: {$version}
bot-built-with = Erstellt mit Rust und Poise
bot-features = Funktionen: Diagrammerstellung, Statistikverfolgung und mehr!

# Uptime messages
uptime-title = Bot-Betriebszeit und Statistiken
uptime-duration = Betriebszeit: {$hours}h {$minutes}m {$seconds}s
uptime-commands-executed = Befehle ausgeführt: {$total} (✅ {$success} erfolgreich, ❌ {$failed} fehlgeschlagen)
uptime-status-ready = Status: Online und bereit!

# User statistics messages
stats-title = Ihre {$period} Statistiken
stats-period = Zeitraum: {$range}
stats-command-usage = Befehlsverwendung
stats-total-commands = Gesamte Befehle: {$count}
stats-successful = Erfolgreich: {$count} ({$percentage}%)
stats-failed = Fehlgeschlagen: {$count}
stats-avg-response-time = Durchschnittliche Antwortzeit: {$time}ms
stats-most-used = Am meisten verwendet
stats-most-used-command = Befehl: {$command}
stats-most-active-hour = Aktivste Stunde: {$hour}
stats-most-active-day = Aktivster Tag: {$day}
stats-top-commands = Top-Befehle
stats-activity-scope = Aktivitätsbereich
stats-unique-channels = Eindeutige Kanäle: {$count}
stats-unique-servers = Eindeutige Server: {$count}
stats-timeline = Zeitlinie
stats-first-command = Erster Befehl: {$time}
stats-latest-command = Letzter Befehl: {$time}
stats-no-data = N/A
stats-none = Keine
stats-all-time = Gesamte Zeit

# Admin messages
admin-update-graphs-title = Diagramm-Update Eingeleitet
admin-update-graphs-starting = Starte Diagramm-Regenerierungsprozess...
admin-update-graphs-wait = Dies kann einige Momente dauern.
admin-update-graphs-updating = Alle Diagramme werden mit den neuesten Daten von Tautulli aktualisiert.

admin-metrics-title = Bot-Metriken-Bericht
admin-metrics-uptime = Betriebszeit: {$hours}h {$minutes}m
admin-metrics-total-commands = Gesamte Befehle: {$total} (Erfolgsrate: {$rate}%)
admin-metrics-avg-response = Durchschnittliche Antwort: {$time}ms
admin-metrics-command-usage = Befehlsverwendung
admin-metrics-last-24h = Letzte 24h: {$count} Befehle

admin-scheduler-title = Planungssystem-Status
admin-scheduler-core = Kern-Planer: Integriert und bereit
admin-scheduler-task-manager = Task-Manager: Hintergrund-Task-Verwaltung aktiviert
admin-scheduler-task-queue = Task-Warteschlange: Prioritätswarteschlange mit Wiederholungslogik aktiv
admin-scheduler-monitoring = Überwachung: Metriken-Sammlung und Benachrichtigungen konfiguriert
admin-scheduler-persistence = Persistenz: Planungswiederherstellung und Datenbankspeicherung bereit
admin-scheduler-description = Das Planungssystem ist vollständig integriert und bereit, automatisierte Aufgaben zu verwalten.
admin-scheduler-usage = Verwenden Sie dieses System für automatische Diagrammerstellung, Bereinigungsaufgaben und mehr.

# Data export messages
export-title = Ihr Vollständiger Datenexport
export-details = Export-Details
export-generated = Generiert: {$time}
export-command-executions = Befehlsausführungen: {$count}
export-account-created = Konto Erstellt: {$time}
export-last-updated = Zuletzt Aktualisiert: {$time}
export-privacy-notice = Datenschutzhinweis
export-contains-all-data = Dieser Export enthält ALLE Daten, die wir über Ihr Konto gespeichert haben.
export-sent-privately = Diese Daten werden privat und vertraulich gesendet.
export-deletion-info = Sie können die Löschung von Daten mit `/delete_my_data` anfordern.
export-gdpr-compliance = Dieser Export entspricht der DSGVO und den Datenschutzbestimmungen.
export-data-format = Ihre Daten (JSON-Format):

# Data deletion messages
delete-confirmation-required = Geben Sie 'CONFIRM' ein, um alle Ihre Daten dauerhaft zu löschen
delete-success = Ihre Daten wurden dauerhaft aus unseren Systemen gelöscht.
delete-no-data = Keine Daten für Ihr Konto gefunden.

# Time period names
period-daily = Täglich
period-weekly = Wöchentlich
period-monthly = Monatlich
period-all-time = Gesamte Zeit

# Graph generation messages
graph-success-title = Diagramm Erfolgreich Generiert
graph-success-description = Ihr {$type}-Diagramm wurde generiert und ist bereit zur Anzeige.
graph-error-title = Diagramm-Generierung Fehlgeschlagen
graph-error-description = Es gab einen Fehler beim Generieren Ihres {$type}-Diagramms: {$error}
graph-processing = Verarbeite Ihren {$command}-Befehl...

# Permission messages
permission-error-title = Berechtigung Verweigert
permission-error-description = Ich habe nicht die erforderlichen Berechtigungen, um {$action} in diesem Kanal auszuführen.
permission-required = Erforderlich: {$permissions}
permission-action = Aktion: {$action}
