# Development notes

## Type at cursor (⌃⌥T)

`typeAtCursor` (⌃⌥T by default, configurable like the other shortcuts, CloakView Pro) reads the
screen, inspects the focused field through the Accessibility API, asks the model for text that fits
that field, and inserts it at the cursor: prose is typed (line breaks as Shift+Return, so a chat box
never sends), code is pasted and the clipboard restored. Press the shortcut again, or Esc, to stop;
it also stops if you switch apps. It refuses password fields and elements that are not text fields.
It needs the Accessibility permission (System Settings → Privacy & Security → Accessibility), which
is tied to the app's code signature, so a development build asks again. Design:
`docs/superpowers/specs/2026-11-19-type-at-cursor-design.md`.

## Known limitations

### Reopening from the Dock leaves another app's full-screen Space (macOS)

With another app in full screen, hiding the overlay and then clicking cloakview's Dock
icon takes you out of that app's full-screen Space, back to the desktop.

The Dock click itself activates cloakview, before `RunEvent::Reopen` reaches the app and
before the hub is shown, and macOS switches Spaces to follow the activation. Calling
`NSApplication::deactivate` afterwards, right away or 400 ms later, does not undo it: the
debug probe (`[overlay-probe] … active=`) shows the app still active after every call.

**To reopen without leaving full screen, use the quick-hide shortcut** (`quickHide`,
⌃⌥H by default). It never activates the app, so you stay in the full-screen Space.

The Dock handler stays, so the hub can always be brought back after `Close`.
