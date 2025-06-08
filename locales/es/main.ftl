# Spanish (Spain) localization for TGraph Bot

# Common messages
hello = ¡Hola!
welcome = ¡Bienvenido a TGraph Bot!
error = Ocurrió un error: {$message}
success = ¡Operación completada exitosamente!
loading = Cargando...
please-wait = Por favor espera...

# Commands
command-about = Muestra información sobre el bot
command-help = Muestra comandos disponibles
command-stats = Muestra tus estadísticas
command-update-graphs = Actualiza datos de gráficos (solo admin)
command-metrics = Muestra métricas del bot (solo admin)

# About command
about-title = Acerca de TGraph Bot
about-description = TGraph Bot es un poderoso bot de visualización de datos y análisis para Discord.
about-version = Versión: {$version}
about-uptime = Tiempo activo: {$uptime}
about-servers = Servidores: {$count}
about-users = Usuarios: {$count}

# Error messages
error-permission-denied = No tienes permiso para usar este comando.
error-cooldown = Por favor espera {$seconds} segundos antes de usar este comando nuevamente.
error-invalid-input = Entrada inválida proporcionada.
error-command-failed = La ejecución del comando falló.
error-not-found = El elemento solicitado no fue encontrado.

# Success messages
success-data-updated = Los datos han sido actualizados exitosamente.
success-settings-saved = La configuración ha sido guardada.

# Time units
time-seconds = {$count ->
    [one] {$count} segundo
   *[other] {$count} segundos
}
time-minutes = {$count ->
    [one] {$count} minuto
   *[other] {$count} minutos
}
time-hours = {$count ->
    [one] {$count} hora
   *[other] {$count} horas
}
time-days = {$count ->
    [one] {$count} día
   *[other] {$count} días
}

# Graph types
graph-line = Gráfico de Líneas
graph-bar = Gráfico de Barras
graph-pie = Gráfico Circular
graph-scatter = Diagrama de Dispersión

# Status messages
status-online = En línea
status-offline = Desconectado
status-maintenance = En Mantenimiento
