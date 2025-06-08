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

# Bot information messages
bot-title = TGraph Discord Bot
bot-description = Un bot puissant pour générer et partager des graphiques de statistiques Tautulli
bot-version = Version : {$version}
bot-built-with = Construit avec Rust et Poise
bot-features = Fonctionnalités : Génération de graphiques, suivi des statistiques, et plus encore !

# Uptime messages
uptime-title = Temps de Fonctionnement et Statistiques du Bot
uptime-duration = Temps de fonctionnement : {$hours}h {$minutes}m {$seconds}s
uptime-commands-executed = Commandes exécutées : {$total} (✅ {$success} réussies, ❌ {$failed} échouées)
uptime-status-ready = Statut : En ligne et prêt !

# User statistics messages
stats-title = Vos Statistiques {$period}
stats-period = Période : {$range}
stats-command-usage = Utilisation des Commandes
stats-total-commands = Total des Commandes : {$count}
stats-successful = Réussies : {$count} ({$percentage}%)
stats-failed = Échouées : {$count}
stats-avg-response-time = Temps de Réponse Moyen : {$time}ms
stats-most-used = Plus Utilisées
stats-most-used-command = Commande : {$command}
stats-most-active-hour = Heure la Plus Active : {$hour}
stats-most-active-day = Jour le Plus Actif : {$day}
stats-top-commands = Commandes Principales
stats-activity-scope = Portée d'Activité
stats-unique-channels = Canaux Uniques : {$count}
stats-unique-servers = Serveurs Uniques : {$count}
stats-timeline = Chronologie
stats-first-command = Première Commande : {$time}
stats-latest-command = Dernière Commande : {$time}
stats-no-data = N/A
stats-none = Aucune
stats-all-time = Tout le Temps

# Admin messages
admin-update-graphs-title = Mise à Jour des Graphiques Initiée
admin-update-graphs-starting = Démarrage du processus de régénération des graphiques...
admin-update-graphs-wait = Cela peut prendre quelques instants pour se terminer.
admin-update-graphs-updating = Tous les graphiques seront mis à jour avec les dernières données de Tautulli.

admin-metrics-title = Rapport des Métriques du Bot
admin-metrics-uptime = Temps de fonctionnement : {$hours}h {$minutes}m
admin-metrics-total-commands = Total des Commandes : {$total} (Taux de Réussite : {$rate}%)
admin-metrics-avg-response = Réponse Moyenne : {$time}ms
admin-metrics-command-usage = Utilisation des Commandes
admin-metrics-last-24h = Dernières 24h : {$count} commandes

admin-scheduler-title = Statut du Système de Planification
admin-scheduler-core = Planificateur Principal : Intégré et prêt
admin-scheduler-task-manager = Gestionnaire de Tâches : Gestion des tâches en arrière-plan activée
admin-scheduler-task-queue = File d'Attente des Tâches : File de priorité avec logique de nouvelle tentative active
admin-scheduler-monitoring = Surveillance : Collecte de métriques et alertes configurées
admin-scheduler-persistence = Persistance : Récupération de planification et stockage en base de données prêt
admin-scheduler-description = Le système de planification est entièrement intégré et prêt à gérer les tâches automatisées.
admin-scheduler-usage = Utilisez ce système pour la génération automatique de graphiques, les tâches de nettoyage, et plus encore.

# Data export messages
export-title = Votre Exportation Complète de Données
export-details = Détails de l'Exportation
export-generated = Généré : {$time}
export-command-executions = Exécutions de Commandes : {$count}
export-account-created = Compte Créé : {$time}
export-last-updated = Dernière Mise à Jour : {$time}
export-privacy-notice = Avis de Confidentialité
export-contains-all-data = Cette exportation contient TOUTES les données que nous avons stockées sur votre compte.
export-sent-privately = Ces données sont envoyées de manière privée et confidentielle.
export-deletion-info = Vous pouvez demander la suppression des données en utilisant `/delete_my_data`.
export-gdpr-compliance = Cette exportation est conforme au RGPD et aux réglementations de protection des données.
export-data-format = Vos données (format JSON) :

# Data deletion messages
delete-confirmation-required = Tapez 'CONFIRM' pour supprimer définitivement toutes vos données
delete-success = Vos données ont été définitivement supprimées de nos systèmes.
delete-no-data = Aucune donnée trouvée pour votre compte.

# Time period names
period-daily = Quotidien
period-weekly = Hebdomadaire
period-monthly = Mensuel
period-all-time = Tout le Temps

# Graph generation messages
graph-success-title = Graphique Généré avec Succès
graph-success-description = Votre graphique {$type} a été généré et est prêt à être visualisé.
graph-error-title = Échec de la Génération du Graphique
graph-error-description = Il y a eu une erreur lors de la génération de votre graphique {$type} : {$error}
graph-processing = Traitement de votre commande {$command}...

# Permission messages
permission-error-title = Permission Refusée
permission-error-description = Je n'ai pas les permissions requises pour {$action} dans ce canal.
permission-required = Requis : {$permissions}
permission-action = Action : {$action}
