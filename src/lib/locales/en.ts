export const en = {
  "hub.listen": "Listen",
  "hub.ask": "Ask",
  "hub.notes": "Notes",
  "hub.settings": "Settings",
  "hub.close": "Close",
  "listen.title": "Live transcript",
  "listen.empty": "Start listening to see the transcript here.",
  "listen.start": "Start Listen",
  "listen.stop": "Stop",
  "listen.listening": "Listening…",
  "ask.title": "Ask",
  "ask.placeholder": "Ask a question about the conversation…",
  "ask.send": "Send",
  "ask.stop": "Stop",
  "ask.thinking": "Thinking…",
  "ask.empty": "Suggestions will appear here.",
  "notes.title": "Notes",
  "notes.new": "New note",
  "notes.empty": "No notes yet.",
  "notes.delete": "Delete",
  "settings.title": "Settings",
  "settings.apiKey": "OpenAI API key",
  "settings.apiKeyHint": "Stored in the macOS Keychain. Never leaves this device except to api.openai.com.",
  "settings.saveKey": "Save key",
  "settings.keySaved": "Key saved",
  "settings.testConnection": "Test connection",
  "settings.testOk": "Connection OK",
  "settings.testFail": "Connection failed",
  "settings.language": "Language",
  "settings.theme": "Theme",
  "settings.themeDark": "Dark",
  "settings.themeLight": "Light",
  "settings.opacity": "Opacity",
  "settings.answersModel": "Answers model",
  "settings.shortcuts": "Shortcuts",
  "perm.title": "Permissions needed",
  "perm.screenCapture": "Screen & system audio capture",
  "perm.microphone": "Microphone",
  "perm.grant": "Open System Settings",
  "perm.statusGranted": "Granted",
  "perm.statusMissing": "Missing",
  "errors.missingApiKey": "Add your OpenAI API key in Settings first.",
  "errors.generic": "Something went wrong",
};

/**
 * The shape every locale must have: exactly these keys, string values.
 *
 * `en` is deliberately NOT `as const`. With it, `typeof en` would pin each value
 * to its English literal (`"hub.listen": "Listen"`), and `es: Catalog` could never
 * hold "Escuchar" — the plan's version of this file does not compile. Without it,
 * `typeof en` still fixes the exact key set, which is what the parity rule needs.
 */
export type Catalog = typeof en;
