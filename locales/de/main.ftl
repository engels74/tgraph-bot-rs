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
