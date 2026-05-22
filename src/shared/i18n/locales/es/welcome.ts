export default {
  title: "Bienvenido a Hermes",
  subtitle:
    "Tu asistente de IA autoevolutivo que se ejecuta localmente en tu equipo. Privado, potente y siempre aprendiendo.",
  installIssueTitle: "Problema de instalación",
  getStarted: "Comenzar",
  retryInstall: "Reintentar la instalación",
  terminalInstallHint: "Instálalo desde la terminal y luego vuelve:",
  recheck: "Ya lo instalé — comprobar de nuevo",
  switchToLocal: "Cambiar a modo local",
  installSizeHint: "Esto instalará los componentes necesarios (~2 GB)",
  copyInstallCommand: "Copiar comando de instalación",
  dividerOr: "o",
  connectRemote: "Conectarse a Hermes remoto",
  connectRemoteTitle: "Conectarse a Hermes remoto",
  connectRemoteSubtitle:
    "Introduce la URL de un servidor de API de Hermes en ejecución.",
  remoteServerUrl: "URL del servidor",
  remoteApiKey: "API key (opcional)",
  remoteApiKeyPlaceholder: "Token Bearer (API_SERVER_KEY)",
  testingConnection: "Probando",
  connect: "Conectar",
  remoteHint:
    "Deja la clave vacía si el servidor acepta solicitudes no autenticadas (por ejemplo, mediante un túnel SSH a localhost).",
  connectViaSsh: "Conectarse vía SSH",
  sshSubtitle:
    "Conéctate a un Hermes remoto a través de SSH — sin necesidad de exponer puertos ni claves API.",
  sshHost: "Host SSH",
  sshPort: "Puerto SSH",
  username: "Nombre de usuario",
  privateKeyPath: "Ruta de la clave privada",
  privateKeyPathOptional: "(opcional — por defecto ~/.ssh/id_rsa)",
  remoteHermesPort: "Puerto de Hermes remoto",
  remoteHermesPortDefault: "(por defecto 8642)",
  testingSsh: "Probando conexión SSH…",
  sshHint:
    'Usa el SSH de tu sistema. Asegúrate de que ya puedes ejecutar <code style="font-family:monospace;font-size:12px">ssh {user}@{host}</code> sin que te pida contraseña.',
  errorUrlRequired: "Por favor, introduce una URL.",
  errorRemoteUnreachable:
    "No se pudo alcanzar Hermes en esta URL. Comprueba la URL y la clave API.\n\nDeja la clave vacía si el servidor acepta solicitudes no autenticadas (por ejemplo, mediante un túnel SSH a localhost).",
  errorConnectionFailed: "La prueba de conexión falló.",
  errorHostRequired: "El host y el nombre de usuario son obligatorios.",
  errorSshUnreachable:
    "No se pudo conectar vía SSH ni alcanzar Hermes en el remoto. Asegúrate de:\n• La clave SSH es correcta (o la predeterminada ~/.ssh/id_rsa funciona)\n• La pasarela de Hermes se está ejecutando en el remoto\n• El puerto remoto es correcto (por defecto 8642)",
  errorSshFailed: "La prueba de conexión SSH falló: {error}",
} as const;
