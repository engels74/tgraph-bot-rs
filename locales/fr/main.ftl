# French (France) localization for TGraph Bot

# Common messages
hello = Bonjour !
welcome = Bienvenue sur TGraph Bot !
error = Une erreur s'est produite : {$message}
success = Opération terminée avec succès !
loading = Chargement...
please-wait = Veuillez patienter...

# Commands
command-about = Affiche des informations sur le bot
command-help = Affiche les commandes disponibles
command-stats = Affiche vos statistiques
command-update-graphs = Met à jour les données graphiques (admin uniquement)
command-metrics = Affiche les métriques du bot (admin uniquement)

# About command
about-title = À propos de TGraph Bot
about-description = TGraph Bot est un puissant bot de visualisation de données et d'analyse pour Discord.
about-version = Version : {$version}
about-uptime = Temps de fonctionnement : {$uptime}
about-servers = Serveurs : {$count}
about-users = Utilisateurs : {$count}

# Error messages
error-permission-denied = Vous n'avez pas la permission d'utiliser cette commande.
error-cooldown = Veuillez attendre {$seconds} secondes avant d'utiliser cette commande à nouveau.
error-invalid-input = Entrée invalide fournie.
error-command-failed = L'exécution de la commande a échoué.
error-not-found = L'élément demandé n'a pas été trouvé.

# Success messages
success-data-updated = Les données ont été mises à jour avec succès.
success-settings-saved = Les paramètres ont été sauvegardés.

# Time units
time-seconds = {$count ->
    [one] {$count} seconde
   *[other] {$count} secondes
}
time-minutes = {$count ->
    [one] {$count} minute
   *[other] {$count} minutes
}
time-hours = {$count ->
    [one] {$count} heure
   *[other] {$count} heures
}
time-days = {$count ->
    [one] {$count} jour
   *[other] {$count} jours
}

# Graph types
graph-line = Graphique Linéaire
graph-bar = Graphique en Barres
graph-pie = Graphique Circulaire
graph-scatter = Nuage de Points

# Status messages
status-online = En ligne
status-offline = Hors ligne
status-maintenance = En Maintenance
