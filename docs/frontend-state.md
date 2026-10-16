# Frontend state ownership and interaction checks

App owns navigation, selected-instance identity, modal lifetime and shared
library/account/skin inventories. File and game tasks belong to application-wide
hooks; changing pages does not cancel a running download or game.

InstanceSettings is the single selected-instance lifecycle boundary. App keys it
by instance ID. It owns the subtree containing loader, Java and launch settings.
Children do not repeat that key: sibling duplicate keys previously caused old
loader panels and focus targets to survive instance switches. Mounting a different
instance resets temporary selections, input drafts and expanded menus together.
Settings callbacks patch only Java or launch settings; they do not close editors
or change instance identity. Explicit name/icon edits remain in InstanceEditor.

Global busy state guards loader/Java writes and game-related settings. Launch
settings serialize immediate saves while allowing a subsequent draft to queue.
Shared inventories survive switching, but only the newest inventory request may
publish its result. Version and Java recommendation effects ignore responses
after cleanup. Never infer an instance from a mutable label in asynchronous work;
write operations carry its stable ID.

Control registers/unregisters focus targets. SelectMenu owns only its transient
menu, and removes its action interceptor on close/unmount. Field cancels manual
text drafts before Back reaches an editor or page; immediate fields retain their
normal save semantics. Keep these mechanisms shared across all pages.

Regression checks include repeated Fabric/Quilt/vanilla switches, exactly one
settings panel per type, globally unique DOM IDs, selection focus restoration,
Java download draft reset, out-of-order inventory responses, modal cleanup,
text-edit Back, dropdown keyboard/controller navigation and serialized autosave.
Browser controller simulation does not substitute for physical hardware testing.

## Module boundaries

- platform/desktop.ts owns desktop detection and IPC. Account, skin, controller,
  installer and Java modules import it directly, not the instance library.
- pages/InstancesPage.tsx renders instance management. App owns shared hooks and
  navigation; InstanceSettings remains the keyed per-instance lifecycle boundary.
- useLibrary routes metadata saves, create/delete and loader changes through one
  mutation lifecycle for locking, busy/error state and final library publication.
- Java recommendations initialize the download choice only while the user has
  not chosen a version. Late recommendations must not overwrite explicit input.

Do not retain unreachable placeholder branches after adding a real page, or
CSS selectors for child elements a component no longer renders. Keep English
source keys and Chinese presentation separate when moving interface text.

## Focus when a control becomes unavailable

Control retains native focus when an already-focused button becomes disabled.
It exposes aria-disabled, blocks pointer/keyboard activation and leaves the Tab
order. On blur it becomes natively disabled. FocusManager distinguishes the
current visible control from enabled navigation targets: direction input can
leave an unavailable button, but cannot return to it until it is enabled again.
Preserve this shared rule instead of adding per-page focus flags or fake outlines.
