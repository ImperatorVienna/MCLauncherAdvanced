# Microsoft accounts

Since 1.1.0 the desktop launcher signs in with the official Minecraft Launcher's Xbox Sisu flow (legacy MSA client `00000000402b5328`, scope `service::user.auth.xboxlive.com::MBI_SSL`). No client secret is used and the launcher never sees the password: the user signs in on Microsoft's own page in an incognito launcher window.

## Flow

1. A P-256 proof-of-possession key is generated once and a device token is requested from `device.auth.xboxlive.com` (renewed when it expires). Xbox requests are signed with that key.
2. `sisu.xboxlive.com/authenticate` returns the Microsoft page (PKCE S256, random `state`) and a Sisu session ID.
3. The host window intercepts navigation to `ms-xal-00000000402b5328://auth` before it loads. Only a redirect with the matching `state` for the live attempt is accepted. Closing the window cancels the attempt.
4. The code is exchanged at `login.live.com/oauth20_token.srf`, then `sisu.xboxlive.com/authorize` returns user and title tokens, followed by XSTS for `rp://api.minecraftservices.com/`, Minecraft `login_with_xbox`, the store check and the profile check.

`CLIENT_ID` and `REDIRECT_URI` in `apps/desktop/src-tauri/src/microsoft_auth.rs` are the single configuration point; login, refresh and launch arguments use the same value.

## Boundaries

- `microsoft_auth.rs` owns the device key, signing, HTTP exchanges and the cancellable sign-in session. It has no Tauri imports. The only renderer-visible authentication data is progress and sanitized errors. Responses and credentials are never logged.
- `microsoft_auth_commands.rs` opens and closes the sign-in window and forwards the intercepted redirect. The window has no IPC capability and only allows HTTPS navigation.
- `accounts.rs` owns the portable `accounts.json` file. Authenticated profiles are identified by `@<Minecraft UUID>`. Renderer writes cannot create or rename verified profiles. Only authenticated identities can launch the game.
- `microsoftLogin.ts` polls public status and reloads the shared account list after completion. Late replies from cancelled sessions are discarded. Navigation and account UI use existing controls; no additional modal is introduced.
- `launch_commands.rs` snapshots the selected identity at launch request time and resolves credentials immediately before building the launch plan, after any repair. `launch_plan.rs` supplies the real UUID, game token, Microsoft user type and client ID. Authentication failure never silently falls back to offline identity.

## Persistence and refresh

Credentials and the Xbox device key/token remain native-only, in an atomically replaced file created with Linux mode `0600`. This is a private local file, **not encrypted storage**; do not distribute it in test packages or shared launcher archives. Removing a Microsoft account deletes its local credentials but does not revoke the application's consent on Microsoft's website.

A Minecraft token with more than two minutes remaining is reused. Otherwise the Microsoft refresh token is exchanged, the rotated token is saved immediately, and Xbox/XSTS/Minecraft/store/profile verification is repeated. A downstream failure retains the rotated token for retry. Refresh operations are serialized; conditional writes prevent resurrection of removed accounts or overwriting a new login. Only a completed Sisu sign-in adds a Microsoft account. The Minecraft store endpoint is checked before profile access, without hard-coding entitlement product names that can differ for subscriptions.

The sign-in window expires after 15 minutes. Cancellation invalidates the session before any late credentials can be stored. Known consent, expiry, Xbox profile/family and Minecraft application rejection errors are localized. Native HTTP requests have timeouts, response-size limits and redirect rejection.

Skin preview and upload use the same authenticated identity; see `skins.md`.

## Validation

Unit tests cover rejection of unverified identities, public-summary redaction, owner-only file mode, fabricated identities, deletion versus refresh, refresh rotation on downstream failure, cached-token reuse, signed Sisu/XSTS/Minecraft requests with a local mock server, redirect `state` validation, signature layout, and authenticated modern/legacy launch arguments. UI tests cover verified-account selection and removal of unverified account controls, deletion, sign-in focus/back handling and late status replies. Actual account consent and Xbox/Minecraft service acceptance require a real user login; 1.1.0 sign-in and skin upload were confirmed with a real account on Linux.

