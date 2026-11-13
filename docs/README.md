# Development notes

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
