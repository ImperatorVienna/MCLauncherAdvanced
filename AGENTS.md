# Project development constraints

## Mandatory architecture boundaries

These are persistent user requirements. Apply them to every change without
waiting for another reminder. Read `docs/platform-boundary.md` before changing
host integration or introducing a dependency across layers.

- Frontend: rendering, wording/localization, input mapping, keyboard/mouse/gamepad
  navigation, focus, page transitions and presentation state belong in the
  frontend. Never implement these in the platform layer.
- Core (`launcher-core`): portable models, validation, configuration persistence
  and other OS-independent domain rules only. No Tauri, frontend, platform
  adapter, OS commands, desktop integration or host detection dependencies.
- Domain services (including `launcher-install`): game/resource/loader rules,
  installation workflows, verification and transactions. Request OS mechanisms
  through platform APIs; do not embed OS integration to finish a feature faster.
- Platform (`launcher-platform`): narrowly scoped host mechanisms and capability
  reporting. No page IDs, focus decisions, UI text/localization, frontend events,
  product workflow decisions, account/login policy or game installation policy.
  Native input acquisition may return device/input data; navigation decisions
  remain outside platform implementations.
- Application/Tauri layer: IPC validation, lifecycle and orchestration between
  services and platform adapters. OS-specific Tauri integration, when required,
  belongs in a clearly separated host adapter; do not move UI behavior into it.

Dependencies must point from orchestration/services toward abstractions and
mechanisms, never from platform code back into frontend or application services.
The existing platform dependency on the core's generic atomic writer is allowed;
this does not authorize using core domain policy inside platform adapters.

Do not bypass these boundaries for convenience, deadlines, compatibility or
short-term fixes. If an API is missing, design the appropriate boundary first.
Keep Linux and Windows implementations separate behind a shared contract.
Unsupported capabilities must be explicit; do not claim a platform is supported
without implementing and validating its required adapters.

Before completing a change, review new imports/dependencies and responsibility
placement, check that platform-specific code has not leaked into the core or UI,
and run relevant regression checks. Preserve existing platform behavior while
adding another platform. Record material boundary changes in the architecture
documentation. Do not add a second platform's behavior by copying whole business
workflows into its adapter.

## Internal test release names and platform scope

Use `rN-<os>-<arch>` for internal revisions, with `win`, `linux`, `mac`
and standard architecture names `x86_64`, `arm64`. Keep the package identifier
`mclauncher-advanced` unchanged. Preserve earlier release files.

At this stage, requests to build Windows or Linux mean x86_64 without asking
again. Examples: `r31-win-x86_64`, `r31-linux-x86_64`.
Do not develop or package macOS x86_64. macOS is planned for arm64 only.
Defer Windows/Linux arm64 work until after macOS arm64 is implemented and the
user requests those additional targets. Naming a target does not imply support;
only deliver artifacts actually built and validated to the stated extent.
