# cmd-buttons

Aplicación TUI en Rust para ejecutar comandos desde archivos de botones en formato TOML.

## Descripción

`cmd-buttons` es una herramienta de terminal que descubre archivos de botones TOML, los muestra en una interfaz visual y ejecuta los comandos asociados con salida en vivo.

## Instalación

```bash
cargo build --release
```

El binario se genera en `target/release/cmd-buttons`.

## Uso

```bash
cmd-buttons [-s|--save-session]
```

### Opciones

- `-s`, `--save-session`: Guarda la salida de cada ejecución en un archivo de log.

### Atajos de teclado

- `↑`/`↓` o `j`/`k`: Navegar entre botones
- `Enter`: Ejecutar comando seleccionado
- `Ctrl+C`: Cancelar ejecución en curso
- `d`: Alternar panel de diagnósticos
- `q`: Salir

## Arquitectura

### Estructura del proyecto

```
cmd-buttons/
├── Cargo.toml          # Configuración del proyecto Rust
├── src/
│   ├── main.rs         # Punto de entrada y loop principal de la TUI
│   ├── app_state.rs    # Estado de la aplicación (foco, scroll, ejecución)
│   ├── button_def.rs   # Definición del esquema TOML y validación
│   ├── config.rs       # Carga de configuración XDG y resolución de rutas
│   ├── discovery.rs    # Escaneo, ordenamiento y reconciliación de botones
│   ├── logging.rs      # Persistencia opcional de logs de sesión
│   ├── runner.rs       # Ejecución de procesos con shell configurable
│   └── ui.rs           # Layout, estilos y renderizado de la interfaz
└── cmd-buttons/        # Directorio de botones de ejemplo
    ├── build.toml
    ├── test.toml
    └── readme-complete.toml
```

### Módulos principales

#### `main.rs`
Punto de entrada que inicializa la aplicación, configura la terminal y ejecuta el loop principal de eventos. Maneja:
- Parsing de argumentos CLI
- Inicialización de rutas XDG
- Configuración de la terminal (raw mode, alternate screen)
- Loop de eventos con polling de teclado y estado de procesos

#### `app_state.rs`
Define el estado central de la aplicación:
- `ExecutionState`: enum con estados `Idle`, `Running`, `Completed`
- `AppState`: estructura con botones, foco, scroll, estado de ejecución
- Métodos para navegación y consulta de estado

#### `button_def.rs`
Define el esquema de los archivos TOML:
- `ButtonFile`: estructura serializable con `label`, `command`, `order`
- Validación de campos obligatorios
- Parsing y validación de archivos individuales

#### `config.rs`
Maneja la configuración y resolución de rutas:
- `AppConfig`: configuración de la aplicación (`buttons_dir`, `shell`)
- `Paths`: rutas XDG (config, state, sessions)
- Resolución con precedencia: `./cmd-buttons` > XDG config > XDG default

#### `discovery.rs`
Descubrimiento y reconciliación de botones:
- Escaneo de archivos `*.toml` en el directorio activo
- Ordenamiento por `order` ascendente, luego alfabético por `label`
- Asignación de índices compactos 1..N
- Generación del archivo índice persistido

#### `runner.rs`
Ejecución de comandos:
- Lanzamiento de procesos con shell configurable
- Captura concurrente de stdout/stderr
- Soporte para cancelación con señales Unix
- Monitoreo de estado del proceso

#### `ui.rs`
Renderizado de la interfaz:
- Layout adaptable (panel de botones + panel de salida)
- Estilos para estados: normal, enfocado, ejecutando, error
- Pantalla guiada cuando no hay botones
- Footer con atajos de teclado contextuales

#### `logging.rs`
Persistencia de logs de sesión:
- Generación de nombres sanitizados
- Formato de log con timestamp, metadata y salida
- Almacenamiento en `XDG_STATE_HOME/cmd-buttons/sessions/`

### Flujo de ejecución

1. **Inicialización**: Se cargan rutas XDG y configuración
2. **Descubrimiento**: Se escanea el directorio de botones activo
3. **Reconciliación**: Se parsean, validan y ordenan los botones
4. **Renderizado**: Se dibuja la interfaz con lista de botones
5. **Loop de eventos**: Se procesan teclas y se monitorean procesos
6. **Ejecución**: Al presionar Enter, se lanza el comando seleccionado
7. **Salida**: Se muestra en vivo la salida del proceso
8. **Finalización**: Al salir, se restaura la terminal

### Formato de archivo de botón

Cada botón es un archivo `.toml` con la siguiente estructura:

```toml
label = " Mi Botón"
command = "echo 'Hola Mundo'"
order = 10  # opcional
```

- `label`: nombre mostrado en la interfaz (admite Unicode/Nerd Fonts)
- `command`: comando shell a ejecutar
- `order`: prioridad de ordenamiento (opcional, menor = primero)

### Configuración

Archivo de configuración en `~/.config/cmd-buttons/config.toml`:

```toml
buttons_dir = "/ruta/a/botones"  # opcional
shell = "bash"                   # opcional, default: bash
```

### Índice persistido

Se genera automáticamente en `~/.local/state/cmd-buttons/index.toml` con información de los botones descubiertos para depuración.

## Dependencias

- `ratatui`: framework TUI
- `crossterm`: manejo de terminal
- `serde` + `toml`: serialización de configuración
- `clap`: parsing de argumentos CLI
- `directories`: resolución de rutas XDG
- `chrono`: manejo de timestamps
- `nix`: envío de señales Unix
- `unicode-segmentation`: manejo de texto Unicode

## Licencia

MIT
