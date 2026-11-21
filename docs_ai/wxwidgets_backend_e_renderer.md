# Backend e renderer di wxWidgets

Verifica eseguita il 17 novembre 2026 sui sorgenti locali di wxWidgets 3.3.4.

## Ambito della verifica

Nella directory di questo repository wxDragon non è stata trovata una copia dei
sorgenti di wxWidgets. La copia esaminata si trova nel progetto adiacente:

```text
../dual-rust/target/debug/wxWidgets
```

La versione è dichiarata in `include/wx/version.h`. Il file
`rust/wxdragon-sys/build.rs` di questo repository prevede a sua volta wxWidgets
3.3.4 e consente di specificare percorsi personalizzati tramite `WXWIDGETS_DIR`
e `WXWIDGETS_BUILD_DIR`.

I risultati sui valori predefiniti derivano dai sorgenti; quelli sulla build
effettiva riguardano soltanto la build macOS locale indicata più avanti.

## Catena degli strati

Su macOS la catena generale è:

```text
Applicazione Rust
  → wxDragon
  → interfaccia C / wrapper C++
  → wxWidgets
  → Cocoa / AppKit
  → servizi di macOS
```

wxWidgets offre un'API comune implementata da diversi backend, chiamati anche
*port*. Alcuni esempi sono wxMSW per Windows, wxOSX/Cocoa per macOS, wxGTK e wxQt.
GTK e Qt sono toolkit; GNOME e KDE sono ambienti desktop. Il toolkit usato da
un'applicazione non deve necessariamente corrispondere a quello dell'ambiente
desktop in cui viene eseguita.

Il port viene normalmente scelto quando si compila wxWidgets. La presenza dei
sorgenti di GTK e Qt non comporta una selezione automatica fra loro all'avvio.

## Port dell'interfaccia e renderer di disegno

Sono due scelte diverse:

- Il **port** realizza finestre, controlli, gestione dell'input, eventi e
  integrazione con la piattaforma.
- Il **renderer di `wxGraphicsContext`** realizza le operazioni di disegno 2D
  richieste tramite questa API.

Scegliere un renderer per un contesto di disegno non sostituisce automaticamente
l'implementazione di tutti i pulsanti, menu e altri controlli nativi dell'app.
Inoltre `wxDC` e `wxGraphicsContext` sono API distinte: questa tabella descrive
specificamente il renderer predefinito di `wxGraphicsContext`. Su macOS il codice
di configurazione precisa che anche `wxDC` è implementato tramite
`wxGraphicsContext`.

## Renderer predefiniti e alternative

| Port | Renderer predefinito di `wxGraphicsContext` | Alternative disponibili se incluse nella build |
|---|---|---|
| Windows / wxMSW | GDI+ | Direct2D; Cairo |
| macOS / Cocoa | Core Graphics | Cairo |
| Linux / wxGTK | Cairo | Nessun secondo renderer specifico di GTK individuato in questa API |
| wxQt | Renderer Qt | Cairo |

Le alternative non sono necessariamente compilate, né equivalenti per tutte le
destinazioni e operazioni di disegno. Il chiamante deve gestirne la disponibilità.
La tabella non comprende percorsi separati come un canvas OpenGL.

La scelta predefinita è esplicita nel codice, non è una ricerca automatica del
renderer più veloce per ogni operazione. In particolare, su wxMSW il valore
predefinito resta GDI+ anche quando Direct2D è disponibile.

Riferimenti nella copia locale, con numeri di riga al momento della verifica:

| File relativo alla radice di wxWidgets | Riga | Evidenza |
|---|---:|---|
| `src/msw/graphics.cpp` | 2698 | `GetDefaultRenderer()` restituisce `GetGDIPlusRenderer()` |
| `src/msw/graphicsd2d.cpp` | — | Implementazione del renderer Direct2D |
| `src/osx/carbon/graphics.cpp` | 2888 | `GetDefaultRenderer()` restituisce il renderer Core Graphics |
| `src/generic/graphicc.cpp` | 3720 | Accesso al renderer Cairo; restituisce `nullptr` quando disabilitato |
| `src/generic/graphicc.cpp` | 3734 | I port diversi da MSW, OSX e Qt usano Cairo come predefinito |
| `src/qt/graphics.cpp` | 1606 | `GetDefaultRenderer()` restituisce il renderer Qt |
| `src/common/graphcmn.cpp` | 996 | Le funzioni di creazione standard di `wxGraphicsContext` usano il renderer predefinito |

Il nome storico della directory `src/osx/carbon` non indica che la build
esaminata usi Carbon come backend delle finestre: il port selezionato è Cocoa.

## Opzioni di compilazione

In `build/cmake/options.cmake`, dalle righe 305–324:

- Su macOS `wxUSE_GRAPHICS_CONTEXT` è imposto a `ON`.
- Su Windows viene offerta l'opzione `wxUSE_GRAPHICS_DIRECT2D`.
- `wxUSE_CAIRO` è predefinito a `ON` per wxGTK e a `OFF` negli altri casi.

In `build/cmake/init.cmake`, dalle righe 428–436, sono presenti controlli per
Direct2D e una regola per MSVC che ne allinea l'abilitazione a
`wxUSE_GRAPHICS_CONTEXT`. L'inclusione del renderer nella build resta distinta
dalla scelta di usarlo per creare un contesto.

## Build macOS locale verificata

Cache CMake esaminata:

```text
../dual-rust/target/debug/wxdragon_sys_cmake_build/build/CMakeCache.txt
```

Valori rilevati:

```text
wxBUILD_TOOLKIT:STRING=osx_cocoa
wxUSE_CAIRO:BOOL=OFF
```

Header di configurazione generato esaminato:

```text
../dual-rust/target/debug/wxwidgets_cmake_build/lib/wx/include/osx_cocoa-unicode-static-3.3/wx/setup.h
```

Valori rilevati:

```c
#define wxUSE_GRAPHICS_CONTEXT 1
#define wxUSE_CAIRO 0
```

Per questa build il port è dunque **Cocoa**, il renderer predefinito di
`wxGraphicsContext` è **Core Graphics** e l'alternativa Cairo è disabilitata.
Non sono state verificate build Windows o Linux effettive.

## Accelerazione GPU

Il nome del port o del renderer non garantisce che ogni operazione venga
eseguita dalla GPU. Occorre distinguere il disegno del contenuto dalla
composizione delle finestre sul desktop: il sistema può comporre tramite GPU
una finestra il cui contenuto è stato rasterizzato dalla CPU.

Per stabilire l'accelerazione effettiva servono anche la destinazione del
disegno, il percorso usato dal toolkit, la configurazione e i driver. La verifica
dei sorgenti qui documentata identifica le scelte predefinite, non misura le
prestazioni né dimostra l'uso della GPU.
