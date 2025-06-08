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

# Bot information messages
bot-title = TGraph Discord Bot
bot-description = Un poderoso bot para generar y compartir gráficos de estadísticas de Tautulli
bot-version = Versión: {$version}
bot-built-with = Construido con Rust y Poise
bot-features = Características: Generación de gráficos, seguimiento de estadísticas, ¡y más!

# Uptime messages
uptime-title = Tiempo Activo y Estadísticas del Bot
uptime-duration = Tiempo activo: {$hours}h {$minutes}m {$seconds}s
uptime-commands-executed = Comandos ejecutados: {$total} (✅ {$success} exitosos, ❌ {$failed} fallidos)
uptime-status-ready = Estado: En línea y listo!

# User statistics messages
stats-title = Tus Estadísticas {$period}
stats-period = Período: {$range}
stats-command-usage = Uso de Comandos
stats-total-commands = Total de Comandos: {$count}
stats-successful = Exitosos: {$count} ({$percentage}%)
stats-failed = Fallidos: {$count}
stats-avg-response-time = Tiempo de Respuesta Promedio: {$time}ms
stats-most-used = Más Utilizados
stats-most-used-command = Comando: {$command}
stats-most-active-hour = Hora Más Activa: {$hour}
stats-most-active-day = Día Más Activo: {$day}
stats-top-commands = Comandos Principales
stats-activity-scope = Alcance de Actividad
stats-unique-channels = Canales Únicos: {$count}
stats-unique-servers = Servidores Únicos: {$count}
stats-timeline = Línea de Tiempo
stats-first-command = Primer Comando: {$time}
stats-latest-command = Último Comando: {$time}
stats-no-data = N/A
stats-none = Ninguno
stats-all-time = Todo el Tiempo

# Admin messages
admin-update-graphs-title = Actualización de Gráficos Iniciada
admin-update-graphs-starting = Iniciando proceso de regeneración de gráficos...
admin-update-graphs-wait = Esto puede tomar unos momentos para completarse.
admin-update-graphs-updating = Todos los gráficos se actualizarán con los datos más recientes de Tautulli.

admin-metrics-title = Reporte de Métricas del Bot
admin-metrics-uptime = Tiempo activo: {$hours}h {$minutes}m
admin-metrics-total-commands = Total de Comandos: {$total} (Tasa de Éxito: {$rate}%)
admin-metrics-avg-response = Respuesta Promedio: {$time}ms
admin-metrics-command-usage = Uso de Comandos
admin-metrics-last-24h = Últimas 24h: {$count} comandos

admin-scheduler-title = Estado del Sistema de Programación
admin-scheduler-core = Programador Principal: Integrado y listo
admin-scheduler-task-manager = Gestor de Tareas: Gestión de tareas en segundo plano habilitada
admin-scheduler-task-queue = Cola de Tareas: Cola de prioridad con lógica de reintento activa
admin-scheduler-monitoring = Monitoreo: Recolección de métricas y alertas configuradas
admin-scheduler-persistence = Persistencia: Recuperación de programación y almacenamiento en base de datos listo
admin-scheduler-description = El sistema de programación está completamente integrado y listo para manejar tareas automatizadas.
admin-scheduler-usage = Usa este sistema para generación automática de gráficos, tareas de limpieza, y más.

# Data export messages
export-title = Tu Exportación Completa de Datos
export-details = Detalles de Exportación
export-generated = Generado: {$time}
export-command-executions = Ejecuciones de Comandos: {$count}
export-account-created = Cuenta Creada: {$time}
export-last-updated = Última Actualización: {$time}
export-privacy-notice = Aviso de Privacidad
export-contains-all-data = Esta exportación contiene TODOS los datos que tenemos almacenados sobre tu cuenta.
export-sent-privately = Estos datos se envían de forma privada y confidencial.
export-deletion-info = Puedes solicitar la eliminación de datos usando `/delete_my_data`.
export-gdpr-compliance = Esta exportación cumple con GDPR y regulaciones de protección de datos.
export-data-format = Tus datos (formato JSON):

# Data deletion messages
delete-confirmation-required = Escribe 'CONFIRM' para eliminar permanentemente todos tus datos
delete-success = Tus datos han sido eliminados permanentemente de nuestros sistemas.
delete-no-data = No se encontraron datos para tu cuenta.

# Time period names
period-daily = Diario
period-weekly = Semanal
period-monthly = Mensual
period-all-time = Todo el Tiempo

# Graph generation messages
graph-success-title = Gráfico Generado Exitosamente
graph-success-description = Tu gráfico {$type} ha sido generado y está listo para visualizar.
graph-error-title = Falló la Generación del Gráfico
graph-error-description = Hubo un error generando tu gráfico {$type}: {$error}
graph-processing = Procesando tu comando {$command}...

# Permission messages
permission-error-title = Permiso Denegado
permission-error-description = No tengo los permisos requeridos para {$action} en este canal.
permission-required = Requerido: {$permissions}
permission-action = Acción: {$action}
