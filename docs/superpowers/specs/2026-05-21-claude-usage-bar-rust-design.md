# Claude Usage Bar (Rust, cross-platform) — Design

**Data:** 2026-05-21
**Status:** Aprovado (brainstorming)

## Objetivo

Reescrever o Claude Usage Bar em Rust como um único codebase cross-platform —
macOS, Linux e Windows — aposentando o app Swift macOS-only
(`~/dev/claude-usage-bar/`, que permanece intacto, parado).

Um app de bandeja (system tray / menu bar) que mostra o consumo do plano Claude
(Pro/Max) contra os limites da assinatura: janela de 5h e limite semanal.

## Pré-requisitos

- Toolchain Rust (`rustup` / `cargo`) instalado em cada máquina onde se vai
  compilar. Não está instalado na máquina macOS atual — instalar via `rustup`.
- Linux: bibliotecas de desenvolvimento do GTK (o backend do `tray-icon`/`tao`).

## Fonte de dados

Endpoint não-documentado, o mesmo usado pelo `/usage` do Claude Code:

```
GET https://api.anthropic.com/api/oauth/usage
Authorization: Bearer <oauth-access-token>
anthropic-beta: oauth-2025-04-20
```

Resposta (HTTP 200), formato confirmado em 2026-05-21:

```json
{
  "five_hour":        { "utilization": 17, "resets_at": "2026-05-21T20:50:00.508668+00:00" },
  "seven_day":        { "utilization": 18, "resets_at": "2026-05-24T07:00:00.508691+00:00" },
  "seven_day_opus":   null,
  "seven_day_sonnet": { "utilization": 5,  "resets_at": "2026-05-24T07:00:00.508699+00:00" },
  "extra_usage":      { "is_enabled": true, "used_credits": 82, "currency": "BRL",
                        "monthly_limit": null, "utilization": null, "disabled_reason": null }
}
```

Campos extras (`seven_day_cowork`, `tangelo`, etc.) são ignorados.

## Stack

- `tao` — event loop cross-platform. No Linux é baseado em GTK, que é o que o
  `tray-icon` precisa. É o event loop que o Tauri usa.
- `tray-icon` — ícone de bandeja + menu nativo. A crate de bandeja do Tauri.
- `ureq` — cliente HTTP bloqueante, pequeno, sem runtime async.
- `serde` + `serde_json` — JSON.
- `#[cfg]`-gated: macOS não precisa de crate extra (shell out para `security`);
  Windows pode precisar da crate `windows` (ver "Risco: token no Windows").

Compilação nativa em cada plataforma (`cargo build --release`). Sem
cross-compilation — apps de bandeja cross-compilam mal.

## Riscos conhecidos

### Token no Windows (gate de viabilidade)

Onde o Claude Code guarda o token OAuth varia por SO:

- **macOS** — Keychain, item `Claude Code-credentials` (confirmado).
- **Linux** — provavelmente `~/.claude/.credentials.json` em texto puro.
- **Windows** — incerto: pode ser `%USERPROFILE%\.claude\.credentials.json` ou
  o Credential Manager / DPAPI.

A primeira tarefa do plano de implementação verifica isso numa máquina Linux e
numa Windows reais. Se no Windows for DPAPI, `token/windows.rs` usa a crate
`windows` para descriptografar. O risco é isolado num único arquivo; o resto do
projeto não depende da resposta.

### Endpoint não-documentado

`/api/oauth/usage` pode mudar de formato sem aviso. Falha de decode →
estado de erro `⚠ fmt`, sem crash.

### Bandeja no macOS exige bundle

Um binário cru não apresenta `NSStatusItem` de forma confiável (lição do app
Swift). O build macOS empacota o binário num `.app` (`Info.plist` com
`LSUIElement`). Windows e Linux rodam o executável puro.

### Bandeja no Linux é fragmentada

GNOME moderno não tem bandeja por padrão — exige a extensão AppIndicator.
KDE/XFCE têm. Fora do escopo resolver isso; documentar no README.

## Arquitetura

Projeto: `~/dev/claude-usage-bar-rs/`, projeto Cargo.

```
src/
  main.rs        — dispatch de CLI (--once/--selftest/--install/--uninstall) ou roda a bandeja
  error.rs       — enum WidgetError (4 variantes)
  usage.rs       — structs Usage/Window/ExtraUsage + serde, decode
  client.rs      — fetch_usage(): GET no endpoint, Result<Usage, WidgetError>
  render.rs      — puro: cor por nível, formatar reset, barra ASCII, strings, gerar RGBA do ícone
  tray.rs        — TrayApp: dona do TrayIcon + menu, event loop, fio do polling
  autostart.rs   — instalar/remover auto-start por SO (#[cfg]-gated)
  token/
    mod.rs       — fetch_token() despachado por #[cfg]
    macos.rs     — Keychain via /usr/bin/security
    linux.rs     — ~/.claude/.credentials.json
    windows.rs   — %USERPROFILE%\.claude\.credentials.json (provisório — ver risco)
```

Só `token/*` e `autostart.rs` têm código específico de SO. `usage`, `client`,
`render`, `error` são totalmente portáveis. `tray` é portável (`tao`/`tray-icon`
abstraem as plataformas).

### Unidades

| Unidade | O que faz | Depende de |
|---|---|---|
| `error::WidgetError` | enum: `TokenNotFound`, `TokenMalformed`, `Auth`, `Network(String)`, `Format` | — |
| `usage` | structs + `decode_usage(&[u8]) -> Result<Usage, WidgetError>` | `serde` |
| `token::fetch_token` | obtém o token OAuth do SO atual; lido fresco a cada poll, nunca cacheado | `#[cfg]` impls |
| `client::fetch_usage` | `fetch_token()` → GET → `decode_usage` → `Usage` | `token`, `usage`, `ureq` |
| `render` | funções puras: nível de cor, formatação de tempo, geração do ícone RGBA, strings de menu/tooltip/título | `usage` |
| `tray::TrayApp` | dona do `TrayIcon` + menu; event loop `tao`; recebe resultados do polling e atualiza a bandeja | `tray-icon`, `tao`, `render` |
| `autostart` | instala/remove auto-start do SO atual | `#[cfg]` |

### Fluxo de dados

```
thread de polling: fetch_token() → fetch_usage() → Usage
   → mpsc channel → thread do event loop (tao user-event)
   → render → TrayIcon (ícone + tooltip + título + menu)
```

## Token por plataforma

`token::fetch_token() -> Result<String, WidgetError>`, despachado por
`#[cfg(target_os)]`. Lido fresco a cada poll.

- **macOS** — `/usr/bin/security find-generic-password -s "Claude Code-credentials" -w`,
  parseia o JSON, extrai `claudeAiOauth.accessToken`.
- **Linux** — lê `~/.claude/.credentials.json`, parseia, extrai
  `claudeAiOauth.accessToken`.
- **Windows** — provisório: lê `%USERPROFILE%\.claude\.credentials.json` do mesmo
  jeito. Sujeito à verificação (ver "Risco: token no Windows").

Erros: arquivo/Keychain ausente → `TokenNotFound`; JSON inválido ou sem
`accessToken` → `TokenMalformed`.

## Cliente HTTP

`client::fetch_usage() -> Result<Usage, WidgetError>`:

1. `token::fetch_token()`.
2. GET no endpoint via `ureq`, headers `Authorization: Bearer <token>` e
   `anthropic-beta: oauth-2025-04-20`, timeout de 15s.
3. Mapeamento: erro de transporte → `Network(String)`; HTTP 401 → `Auth`;
   não-200 → `Network`; falha de decode → `Format`.
4. HTTP 200 → `decode_usage` → `Usage`.

## UI da bandeja

### Ícone

Gerado em runtime como RGBA (sem arquivos de asset). Quadrado arredondado
preenchido com a **cor do nível**, definida pela janela mais cheia entre 5h e 7d:

| Utilização máxima | Cor |
|---|---|
| `< 50%` | verde |
| `50–80%` | laranja |
| `≥ 80%` | vermelho |

Estado de erro → ícone cinza. Regenerado quando o nível/estado muda.
`tray_icon::Icon::from_rgba`.

### Título (somente macOS)

`set_title("5h 17% · 7d 18%")` — texto puro. No Windows/Linux `set_title` não se
aplica; lá o ícone colorido é o sinal à primeira vista. Em estado de erro o
título mostra o aviso (`⚠ token` etc.).

### Tooltip (todas as plataformas)

`"Claude — 5h 17% · 7d 18%"` no hover. É o "à primeira vista" exato no Win/Linux.

### Menu (dropdown nativo, todas as plataformas)

```
Janela de 5h      17%   ▓▓░░░░░░░░
reseta em 3h 12m  ·  20:50
─────────────────────────────────
Semanal (7d)      18%   ▓▓░░░░░░░░
reseta em 2d 15h  ·  sáb 07:00
─────────────────────────────────
Semanal · Sonnet   5%
Semanal · Opus     —
Uso extra         82 créditos (BRL)
─────────────────────────────────
Atualizado às 16:42
Atualizar agora
Sair
```

- Linhas per-modelo (`seven_day_sonnet`/`seven_day_opus`): `—` quando o campo é
  `null`.
- `extra_usage`: linha some quando `is_enabled` é `false`.
- Itens são texto puro — menus nativos cross-platform não suportam cor por item
  de forma confiável; o sinal de cor vive no ícone.
- "Atualizar agora" dispara um poll imediato; "Sair" encerra.

## Comportamento

### Polling

- Thread de fundo: `fetch` → manda resultado pelo canal `mpsc` → dorme 300s.
- Poll imediato no launch.
- "Atualizar agora" dispara um fetch extra.
- O event loop recebe os resultados via user-event do `tao` e atualiza a bandeja.
- Token relido do SO a cada poll (nunca cacheado).

### Wake-from-sleep

Removido de propósito. Detecção de wake cross-platform é complexa e o timer de
5 min já limita a idade do dado. Simplificação consciente (YAGNI) — é uma
mudança em relação ao app Swift, que tinha refresh no wake.

### Estados de erro

Os mesmos 4 do app Swift. O widget nunca apaga o último dado bom por falha
transitória.

| Situação | Ícone | Título (macOS) / Tooltip / Menu |
|---|---|---|
| Sem rede / timeout | mantém cor, marca ⚠ | mantém últimos valores; menu: "sem conexão" |
| HTTP 401 | cinza | `⚠ auth`; "token expirado — abra o Claude Code" |
| Token ausente / inacessível | cinza | `⚠ token`; instrução por SO |
| JSON em formato inesperado | cinza | `⚠ fmt`; "endpoint mudou de formato" |

## CLI e auto-start

`main.rs` despacha por argumento:

- `--once` — busca e imprime o uso, sai. Verificação portável.
- `--selftest` — roda asserts internos, sai 0/1.
- `--install` / `--uninstall` — instala/remove o auto-start do SO atual.
- sem args — roda a bandeja.

`autostart` (`#[cfg]`-gated):

- **macOS** — escreve `~/Library/LaunchAgents/com.samdev.claude-usage-bar.plist`,
  apontando para o executável dentro do `.app`; `launchctl bootstrap`.
- **Linux** — escreve `~/.config/autostart/claude-usage-bar.desktop` (autostart XDG).
- **Windows** — adiciona um valor em
  `HKCU\Software\Microsoft\Windows\CurrentVersion\Run`.

## Build

`cargo build --release` nativo em cada SO. No macOS, um script de build empacota
o binário resultante num `ClaudeUsageBar.app` (`Info.plist` com `LSUIElement`),
necessário para a bandeja aparecer. Windows e Linux usam o executável direto.

Artefatos de build (binário, `.app`, `target/`) ficam fora do git.

## Testes / critério de sucesso

- `cargo test` — testes unitários dos módulos portáveis:
  - `usage`: `decode_usage` contra um JSON de amostra (utilizações, `opus` null,
    `extra_usage`, datas com fração de segundo).
  - `render`: cor por nível nos limites 49/50/79/80; formatação de reset
    (relativo/absoluto); seleção de cor do ícone pela janela mais cheia.
- `--once` — verificação do caminho de dados ao vivo, rodado em cada SO.
- `--selftest` — smoke test rápido no binário final.
- UI da bandeja — verificação manual por plataforma.

**Pronto quando:** `cargo test` passa; `--once` retorna dados ao vivo nas 3
plataformas; ícone colorido + tooltip + menu funcionam em cada SO; auto-start
instala e funciona em cada SO.

## Fora de escopo (YAGNI)

- Refresh no wake-from-sleep.
- Histórico / gráficos de uso.
- Notificações do sistema ao atingir um limite.
- Configuração via UI (intervalo de poll, thresholds são constantes).
- Cor por item no menu.
- Cross-compilation / distribuição de binários prontos — build nativo em cada SO.
- Resolver a ausência de bandeja no GNOME — apenas documentar.
