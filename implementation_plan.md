# Plan de implementación: `cmd-buttons`

## Resumen
Construir una aplicación TUI en Rust para Linux desktop, basada en `ratatui + crossterm`, que descubra archivos de botones en formato TOML, renderice un grid/lista visualmente pulido con soporte de texto e iconos Unicode/Nerd Fonts, y ejecute un único comando por vez dentro de la propia TUI mostrando salida en vivo, comando activo y contador de tiempo.

La app tendrá dos fuentes posibles de botones, con precedencia estricta:
1. Si existe `./cmd-buttons` en el directorio actual, usar solo esa carpeta.
2. Si no existe, usar la carpeta configurada en rutas estándar Linux/XDG.

El índice será un archivo TOML legible, persistido en `XDG_STATE_HOME`, pero derivado del escaneo real: en cada arranque se reconstruye, elimina entradas faltantes y renumera de forma compacta.

## Interfaces y contratos públicos
### CLI
- Ejecutable principal: `cmd-buttons`
- Flag soportado en v1:
  - `-ss` / `--save-session`: guarda la salida completa de cada ejecución en un log separado por botón y por corrida.
- Sin otras opciones CLI en v1 salvo ayuda/versionado estándar.

### Configuración de app
- Archivo de config en ruta XDG estándar, por ejemplo `~/.config/cmd-buttons/config.toml`.
- Campos mínimos:
  - `buttons_dir`: directorio por defecto de archivos TOML cuando no existe `./cmd-buttons`.
  - `shell`: shell para ejecutar comandos string; default `bash`.
- Si `./cmd-buttons` existe, ignora `buttons_dir` para esa ejecución.

### Archivo TOML por botón
Extensión asumida: `*.toml`.

Campos de v1:
```toml
label = " Build"
command = "cargo build --release"
order = 10 # opcional
```

Reglas:
- `label`: obligatorio, admite texto e iconos Unicode/Nerd Fonts.
- `command`: obligatorio, string de shell.
- `order`: opcional, entero.
- No se incluyen `cwd`, `env`, confirmaciones ni metadata adicional en v1.

### Índice persistido
- Archivo TOML legible en `XDG_STATE_HOME`, por ejemplo `~/.local/state/cmd-buttons/index.toml`.
- Contendrá solo estado derivado útil para inspección/depuración:
  - índice asignado
  - path del archivo
  - label resuelto
  - order efectivo
- No es fuente de verdad; siempre se reescribe al arrancar tras reconciliar el filesystem.

## Cambios de implementación
### Descubrimiento y reconciliación
- Resolver rutas de trabajo al inicio.
- Escanear solo archivos `*.toml` del directorio activo.
- Parsear cada archivo con `serde + toml`.
- Si un archivo es inválido o incompleto:
  - no genera botón
  - queda registrado en diagnóstico visible
  - no entra al índice
- Orden final:
  - primero por `order` ascendente cuando exista
  - luego alfabéticamente por `label`
  - desempate final por nombre de archivo/path
- Asignar índices compactos 1..N en cada arranque.
- Reescribir el índice reconciliado, eliminando faltantes.

### Arquitectura interna sugerida
- `config`: carga de config XDG y resolución de rutas.
- `button_def`: schema TOML y validación.
- `discovery/indexer`: escaneo, ordenamiento, reconciliación e indexado.
- `app_state`: estado TUI, foco, scroll, selección, diagnósticos y sesión actual.
- `runner`: ejecución de procesos con shell configurable, captura de stdout/stderr y cancelación.
- `ui`: layout, estilos, grid/lista de botones, panel de ejecución y pantalla vacía.
- `logging`: persistencia opcional de logs con `-ss`.

### Diseño TUI
- Tema único, sobrio y profesional, pero con jerarquía fuerte y buen contraste.
- Vista principal adaptable:
  - terminal ancho: grid de “cards” de botones + panel lateral/inferior de detalle/salida
  - terminal angosto: lista de una columna con panel de salida apilado
- Soporte de:
  - teclado primero: flechas, `Tab`, `Shift+Tab`, `Enter`, teclas de salida/ayuda
  - mouse: click para foco/ejecución, scroll en grid o panel de salida
- Estados visuales claros:
  - normal
  - enfocado
  - ejecutando
  - inválido/omitido en panel de diagnóstico
- Si no hay botones válidos:
  - mostrar TUI vacía guiada con rutas revisadas, motivo y cómo agregar archivos.

### Ejecución de comandos
- Un solo comando a la vez.
- Durante ejecución:
  - bloquear nuevos lanzamientos
  - mantener la UI activa para scroll, lectura y cancelación
  - mostrar label, comando exacto, estado y contador de tiempo
  - transmitir salida en vivo al panel integrado
- Cancelación:
  - atajo/botón explícito
  - enviar señal de interrupción al proceso; si no termina en una ventana corta de gracia, escalar a terminación forzada
- Salida en memoria:
  - se conserva mientras la app siga abierta
  - no se persiste salvo `-ss`
- Con `-ss`:
  - guardar un archivo por botón y por ejecución, con timestamp, en `XDG_STATE_HOME/cmd-buttons/sessions/`
  - usar nombre sanitizado del botón o del archivo origen para evitar colisiones

## Plan de pruebas
- Unit tests para:
  - resolución de rutas y precedencia `./cmd-buttons` sobre XDG
  - parseo válido/inválido de TOML
  - orden mixto `order` + alfabético
  - reconciliación y renumeración compacta del índice
  - sanitización de nombres de log
- Integration tests para:
  - arranque con carpeta local
  - arranque con carpeta XDG
  - omisión de archivos inválidos con diagnóstico
  - escritura del índice persistido
  - ejecución de comando simple y captura de salida
  - bloqueo de segunda ejecución mientras una sigue activa
  - cancelación de proceso largo
  - creación de logs con `-ss`
- Snapshot/render tests con buffer de `ratatui` para:
  - grid normal
  - estado enfocado
  - estado ejecutando
  - pantalla vacía guiada
  - layout reducido en terminal angosto

## Supuestos y huecos explicitados
- Se asume terminal Linux con soporte de mouse y una fuente que renderice bien Nerd Fonts; sin eso, los iconos pueden degradarse visualmente.
- Se asume disponibilidad de `bash` como default; si no existe, el usuario deberá configurar otro shell.
- En v1 no habrá recarga en caliente, solo reindexado al arranque.
- En v1 no habrá edición de botones desde la TUI.
- En v1 no habrá múltiples temas ni configuración visual avanzada.
- En v1 no habrá metadata ampliada por botón (`cwd`, `env`, confirmación, categorías, atajos dedicados); si luego la querés, conviene tratarla como una fase 2.
- Para evitar ambigüedad de implementación, el índice persistido será informativo/depurable, no una base de datos con identidad estable propia.
