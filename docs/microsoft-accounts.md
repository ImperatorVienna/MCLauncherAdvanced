# Microsoft accounts

The desktop launcher uses Microsoft device authorization (`consumers`, `XboxLive.signin offline_access`) with a build-time public client ID. No client secret or embedded password form is used. The user opens Microsoft's verification page and completes consent in their browser. Registering an Entra application does not itself guarantee approval by Minecraft Services.

## Client ID configuration

The single configuration point is `CLIENT_ID` in
`apps/desktop/src-tauri/src/microsoft_auth.rs`. It currently contains the project owner’s registered Application (client) ID.
Registration does not imply Minecraft API approval.
After your application is approved, put its Entra Application (client) ID between
the quotes and rebuild the launcher. Do not enter a client secret or tenant ID.
Device authorization, token refresh and game launch arguments use the same value.
An empty value rejects sign-in and authenticated launch locally, before network
requests, including when cached credentials exist. No settings field or platform
specific configuration is required. Previously built release files are unchanged.

## Boundaries

- `microsoft_auth.rs` owns HTTP exchanges and the cancellable device-code session. The only renderer-visible authentication data is the short user code, fixed Microsoft verification URL, progress and sanitized errors. Responses and credentials are never logged.
- `accounts.rs` owns the portable `accounts.json` file. Authenticated profiles are identified by `@<Minecraft UUID>`. Renderer writes cannot create or rename verified profiles. Only authenticated identities can launch the game.
- `microsoftLogin.ts` polls public status and reloads the shared account list after completion. Late replies from cancelled sessions are discarded. Navigation and account UI use existing controls; no additional modal is introduced.
- `launch_commands.rs` snapshots the selected identity at launch request time and resolves credentials immediately before building the launch plan, after any repair. `launch_plan.rs` supplies the real UUID, game token, Microsoft user type and client ID. Authentication failure never silently falls back to offline identity.

## Persistence and refresh

Credentials remain native-only, in an atomically replaced file created with Linux mode `0600`. This is a private local file, **not encrypted storage**; do not distribute it in test packages or shared launcher archives. Removing a Microsoft account deletes its local credentials but does not revoke the application's consent on Microsoft's website.

A Minecraft token with more than two minutes remaining is reused. Otherwise the Microsoft refresh token is exchanged, the rotated token is saved immediately, and Xbox/XSTS/Minecraft/store/profile verification is repeated. A downstream failure retains the rotated token for retry. Refresh operations are serialized; conditional writes prevent resurrection of removed accounts or overwriting a new login. Only successful device authorization adds a Microsoft account. The Minecraft store endpoint is checked before profile access, without hard-coding entitlement product names that can differ for subscriptions.

Pending and slow-down device responses are handled until expiry. Cancellation invalidates the session before any late credentials can be stored. Known consent, expiry, Xbox profile/family and Minecraft application rejection errors are localized. Native HTTP requests have timeouts, response-size limits and redirect rejection.

Skin upload remains a separate feature. This change enables authenticated game sessions; the existing skin library does not yet upload selected skins to Minecraft Services.

## Validation

Unit tests cover rejection of unverified identities, public-summary redaction, owner-only file mode, fabricated identities, deletion versus refresh, refresh rotation on downstream failure, cached-token reuse, protocol requests with a local mock server, and authenticated modern/legacy launch arguments. UI tests cover verified-account selection and removal of unverified account controls, deletion, sign-in focus/back handling and late status replies. Actual account consent and Xbox/Minecraft service acceptance require a real user login; device-code issuance alone is not an end-to-end sign-in test.

