import { useEffect, useRef, useState } from "react";
import { getLocale, setLocale, t, type CatalogKey, type Locale } from "../lib/i18n";
import {
  activateLicense,
  deactivateLicense,
  getAppInfo,
  getLicenseStatus,
  listProfiles,
  openExternal,
  saveKey,
  saveSettings,
  setActiveProvider,
  setRealtimeModel,
  testConnection,
} from "../lib/ipc";
import type { ConnectionTestResult, License, ProviderInfo, Settings, SettingsPatch, Shortcuts } from "../lib/models";
import { errorCopyKey } from "../hub/hubLogic";
import { WindowFrame } from "../windows/WindowFrame";
import { applyAppearance, useSettingsBoot } from "../windows/useSettingsBoot";
import {
  MIN_OPACITY,
  clampOpacity,
  draftFromSettings,
  draftIsValid,
  draftPatch,
  isDraftDirty,
  maskKey,
  themeFromSetting,
  type SettingsDraft,
  type Theme,
} from "./settingsLogic";

/** Wait for the slider to settle before writing `settings.json`, instead of on every tick. */
const OPACITY_SAVE_DELAY_MS = 250;

const SHORTCUT_FIELDS: readonly { key: keyof Shortcuts; labelKey: CatalogKey }[] = [
  { key: "quickHide", labelKey: "settings.shortcutQuickHide" },
  { key: "pushToAsk", labelKey: "settings.shortcutPushToAsk" },
  { key: "toggleListen", labelKey: "settings.shortcutToggleListen" },
  { key: "readScreen", labelKey: "settings.shortcutReadScreen" },
];

const LANGUAGES: readonly { value: Locale; labelKey: CatalogKey }[] = [
  { value: "en", labelKey: "settings.languageEn" },
  { value: "es", labelKey: "settings.languageEs" },
];

const MIC_OPTIONS: readonly { value: "off" | "on"; labelKey: CatalogKey }[] = [
  { value: "off", labelKey: "settings.micOff" },
  { value: "on", labelKey: "settings.micOn" },
];

const ANSWER_STYLES: readonly { value: string; labelKey: CatalogKey }[] = [
  { value: "balanced", labelKey: "settings.styleBalanced" },
  { value: "interview", labelKey: "settings.styleInterview" },
  { value: "sales", labelKey: "settings.styleSales" },
  { value: "technical", labelKey: "settings.styleTechnical" },
  { value: "coding", labelKey: "settings.styleCoding" },
  { value: "short", labelKey: "settings.styleShort" },
];

const TRANSLATE_OPTIONS: readonly { value: string; labelKey: CatalogKey }[] = [
  { value: "", labelKey: "settings.translateOff" },
  { value: "en", labelKey: "settings.languageEn" },
  { value: "es", labelKey: "settings.languageEs" },
];

const CALL_LANGUAGES: readonly { value: string; labelKey: CatalogKey }[] = [
  { value: "", labelKey: "settings.callLanguageAuto" },
  { value: "en", labelKey: "settings.languageEn" },
  { value: "es", labelKey: "settings.languageEs" },
];

const THEMES: readonly { value: Theme; labelKey: CatalogKey }[] = [
  { value: "dark", labelKey: "settings.themeDark" },
  { value: "light", labelKey: "settings.themeLight" },
];

/**
 * The AI provider and its key (Keychain), and the saved settings. Language,
 * theme, opacity and the provider apply and save as soon as they change; the
 * shortcuts wait for Save, because an accelerator is invalid while being
 * typed. Answer models are not editable: they come from the provider's
 * profile. Rust re-registers the shortcuts when they change.
 */
export function SettingsApp() {
  const { settings, setSettings, locale: bootLocale, bootErrorKey } = useSettingsBoot();
  const [locale, setLocaleState] = useState<Locale>(bootLocale);
  const [errorKey, setErrorKey] = useState<CatalogKey | null>(null);
  const [providers, setProviders] = useState<ProviderInfo[] | null>(null);
  const [keyInput, setKeyInput] = useState("");
  const [savingKey, setSavingKey] = useState(false);
  const [testing, setTesting] = useState(false);
  const [testResult, setTestResult] = useState<ConnectionTestResult | null>(null);
  const [license, setLicense] = useState<License | null>(null);
  const [licenseInput, setLicenseInput] = useState("");
  const [savingLicense, setSavingLicense] = useState(false);
  const [draft, setDraft] = useState<SettingsDraft | null>(null);
  const [saving, setSaving] = useState(false);
  const [justSaved, setJustSaved] = useState(false);
  const [opacity, setOpacity] = useState<number | null>(null);
  const opacityEdited = useRef(false);

  useEffect(() => setLocaleState(bootLocale), [bootLocale]);
  useEffect(() => {
    if (bootErrorKey !== null) setErrorKey(bootErrorKey);
  }, [bootErrorKey]);

  // Seed the editable state once the settings arrive.
  useEffect(() => {
    if (settings === null) return;
    setDraft((current) => current ?? draftFromSettings(settings));
    setOpacity((current) => current ?? settings.opacity);
  }, [settings]);

  const refreshProviders = async (): Promise<void> => {
    try {
      setProviders(await listProfiles());
    } catch (error: unknown) {
      setErrorKey(errorCopyKey(error));
    }
  };

  useEffect(() => {
    let cancelled = false;
    listProfiles()
      .then((list) => {
        if (!cancelled) setProviders(list);
      })
      .catch((error: unknown) => {
        if (!cancelled) setErrorKey(errorCopyKey(error));
      });
    getLicenseStatus()
      .then((stored) => {
        if (!cancelled) setLicense(stored);
      })
      .catch((error: unknown) => {
        if (!cancelled) setErrorKey(errorCopyKey(error));
      });
    return () => {
      cancelled = true;
    };
  }, []);

  // Persist the opacity once the slider settles — but not the value just loaded.
  useEffect(() => {
    if (!opacityEdited.current || opacity === null) return;
    const timer = window.setTimeout(() => void save({ opacity }), OPACITY_SAVE_DELAY_MS);
    return () => window.clearTimeout(timer);
    // `save` only touches state setters and the IPC wrapper.
  }, [opacity]);

  async function save(patch: SettingsPatch): Promise<Settings | null> {
    try {
      const saved = await saveSettings(patch);
      setSettings(saved);
      return saved;
    } catch (error) {
      setErrorKey(errorCopyKey(error));
      return null;
    }
  }

  async function storeKey(): Promise<void> {
    const key = keyInput.trim();
    const active = providers?.find((p) => p.active);
    if (key === "" || savingKey || active === undefined) return;
    setSavingKey(true);
    try {
      await saveKey(active.id, key);
      setKeyInput("");
      setTestResult(null);
      await refreshProviders();
    } catch (error) {
      setErrorKey(errorCopyKey(error));
    } finally {
      setSavingKey(false);
    }
  }

  async function chooseProvider(id: string): Promise<void> {
    try {
      const saved = await setActiveProvider(id);
      setSettings(saved);
      setTestResult(null);
      await refreshProviders();
    } catch (error) {
      setErrorKey(errorCopyKey(error));
    }
  }

  async function runConnectionTest(): Promise<void> {
    const active = providers?.find((p) => p.active);
    if (testing || active === undefined) return;
    setTesting(true);
    setTestResult(null);
    try {
      setTestResult(await testConnection(active.id));
    } catch (error) {
      setErrorKey(errorCopyKey(error));
    } finally {
      setTesting(false);
    }
  }

  async function activate(): Promise<void> {
    const key = licenseInput.trim();
    if (key === "" || savingLicense) return;
    setSavingLicense(true);
    try {
      setLicense(await activateLicense(key));
      setLicenseInput("");
    } catch (error) {
      setErrorKey(errorCopyKey(error));
    } finally {
      setSavingLicense(false);
    }
  }

  async function deactivate(): Promise<void> {
    if (savingLicense) return;
    setSavingLicense(true);
    try {
      setLicense(await deactivateLicense());
    } catch (error) {
      setErrorKey(errorCopyKey(error));
    } finally {
      setSavingLicense(false);
    }
  }

  /// Opens a pre-filled support email. The subject carries the Pro tag, so Pro
  /// mail is easy to spot and answer first (priority support).
  async function contactSupport(): Promise<void> {
    try {
      const info = await getAppInfo();
      const pro = license !== null && license.status !== "inactive";
      const subject = encodeURIComponent(`${pro ? "[Pro] " : ""}CloakView support`);
      const body = encodeURIComponent(`${pro ? "CloakView Pro" : "CloakView (free)"} ${info.version}\n\n`);
      await openExternal(`mailto:support@getcloakview.com?subject=${subject}&body=${body}`);
    } catch (error) {
      setErrorKey(errorCopyKey(error));
    }
  }

  function changeLanguage(next: Locale): void {
    setLocale(next);
    setLocaleState(getLocale());
    void save({ language: next });
  }

  function changeTheme(next: Theme): void {
    applyAppearance(next, opacity ?? 1);
    void save({ theme: next });
  }

  async function saveDraft(): Promise<void> {
    if (draft === null || saving || !draftIsValid(draft)) return;
    setSaving(true);
    const saved = await save(draftPatch(draft));
    if (saved !== null) {
      setDraft(draftFromSettings(saved));
      setJustSaved(true);
    }
    setSaving(false);
  }

  const theme = themeFromSetting(settings?.theme ?? "dark");
  const dirty = draft !== null && settings !== null && isDraftDirty(draft, settings);

  return (
    <WindowFrame titleKey="settings.title" locale={locale} errorKey={errorKey} onDismissError={() => setErrorKey(null)}>
      <section className="set-section">
        <h2 className="win-label">{t("settings.aiProvider")}</h2>
        {providers === null ? (
          <p className="win-muted">{t("settings.loading")}</p>
        ) : (
          <>
            <div className="seg" role="radiogroup" aria-label={t("settings.aiProvider")}>
              {providers.map((p) => (
                <button
                  key={p.id}
                  type="button"
                  role="radio"
                  aria-checked={p.active}
                  className="seg-btn"
                  onClick={() => void chooseProvider(p.id)}
                >
                  {p.displayName}
                </button>
              ))}
            </div>
            {(() => {
              const active = providers.find((p) => p.active);
              if (active === undefined) return null;
              return (
                <>
                  <p className="win-hint">
                    {t("settings.modelsLine")
                      .replace("{stt}", active.sttModel ?? "—")
                      .replace("{llm}", active.llmModel)}
                  </p>
                  {active.sttMode === "batch" && <p className="set-status win-muted">{t("settings.listeningNote")}</p>}
                  {active.sttModelOptions.length > 0 && settings !== null && (
                    <>
                      <label className="win-label" htmlFor="set-realtime-model">
                        {t("settings.realtimeModel")}
                      </label>
                      <Segmented
                        id="set-realtime-model"
                        options={active.sttModelOptions.map((model) => ({ value: model, label: model }))}
                        value={settings.realtimeModel}
                        onChange={(next) => void setRealtimeModel(next).then(setSettings).catch((error: unknown) => setErrorKey(errorCopyKey(error)))}
                      />
                      <p className="win-hint">{t("settings.realtimeModelHint")}</p>

                      <label className="win-label" htmlFor="set-reply-delay">
                        {t("settings.replyDelay")}
                      </label>
                      <input
                        id="set-reply-delay"
                        className="set-range"
                        type="range"
                        min={0}
                        max={5000}
                        step={250}
                        value={settings.replyDelayMs}
                        onChange={(event) => void save({ replyDelayMs: event.currentTarget.valueAsNumber })}
                      />
                      <p className="win-hint">{t("settings.replyDelayHint").replace("{ms}", String(settings.replyDelayMs))}</p>
                    </>
                  )}
                  <form
                    className="win-row"
                    onSubmit={(event) => {
                      event.preventDefault();
                      void storeKey();
                    }}
                  >
                    <input
                      className="win-input"
                      type="password"
                      autoComplete="off"
                      spellCheck={false}
                      value={keyInput}
                      placeholder={t("settings.keyPlaceholder")}
                      aria-label={t("settings.apiKey")}
                      onChange={(event) => setKeyInput(event.currentTarget.value)}
                    />
                    <button type="submit" className="win-btn win-btn-primary" disabled={savingKey || keyInput.trim() === ""}>
                      {t("settings.saveKey")}
                    </button>
                    <button type="button" className="win-btn" disabled={testing || !active.hasKey} onClick={() => void runConnectionTest()}>
                      {testing ? "…" : t("settings.testConnection")}
                    </button>
                  </form>
                  <p className={active.hasKey ? "set-status set-ok" : "set-status win-muted"}>
                    {active.hasKey ? t("settings.keySaved") : t("settings.keyNotSet")}
                  </p>
                  {testResult !== null && (
                    <p className={testResult.stt === null && testResult.llm === null ? "set-status set-ok" : "set-status win-muted"}>
                      {testResult.stt === null && testResult.llm === null
                        ? t("settings.testOk")
                        : `${t("settings.testFail")}${testResult.stt !== null ? ` — ${testResult.stt}` : ""}${
                            testResult.llm !== null ? ` — ${testResult.llm}` : ""
                          }`}
                    </p>
                  )}
                </>
              );
            })()}
        <p className="win-hint">{t("settings.apiKeyHint")}</p>
        <p className="win-hint">{t("settings.providerHint")}</p>
          </>
        )}
      </section>

      <section className="set-section">
        <h2 className="win-label">{t("settings.pro")}</h2>
        {license === null ? (
          <p className="win-muted">{t("settings.loading")}</p>
        ) : license.status === "inactive" ? (
          <>
            <p className="set-status win-muted">{t("settings.proInactive")}</p>
            <form
              className="win-row"
              onSubmit={(event) => {
                event.preventDefault();
                void activate();
              }}
            >
              <input
                className="win-input win-mono"
                type="text"
                autoComplete="off"
                spellCheck={false}
                value={licenseInput}
                placeholder={t("settings.licensePlaceholder")}
                aria-label={t("settings.licenseKey")}
                onChange={(event) => setLicenseInput(event.currentTarget.value)}
              />
              <button type="submit" className="win-btn win-btn-primary" disabled={savingLicense || licenseInput.trim() === ""}>
                {t("settings.activate")}
              </button>
            </form>
          </>
        ) : (
          <>
            <p className="set-status set-ok">{t("settings.proActive")}</p>
            <div className="win-row">
              <input className="win-input win-mono" readOnly value={maskKey(license.key)} aria-label={t("settings.licenseKey")} />
              <button type="button" className="win-btn" disabled={savingLicense} onClick={() => void deactivate()}>
                {t("settings.deactivate")}
              </button>
            </div>
          </>
        )}
        <p className="win-hint">{t("settings.proHint")}</p>
        <div className="win-row">
          <button type="button" className="win-btn" onClick={() => void contactSupport()}>
            {t("settings.contactSupport")}
          </button>
        </div>
        <p className="win-hint">{t("settings.supportHint")}</p>
      </section>

      {settings === null || draft === null || opacity === null ? (
        bootErrorKey === null && <p className="win-muted">{t("settings.loading")}</p>
      ) : (
        <>
          <section className="set-section set-grid">
            <span className="win-label" id="set-language-label">
              {t("settings.language")}
            </span>
            <Segmented
              id="set-language"
              options={LANGUAGES}
              value={locale}
              onChange={changeLanguage}
            />

            <span className="win-label" id="set-theme-label">
              {t("settings.theme")}
            </span>
            <Segmented
              id="set-theme"
              options={THEMES}
              value={theme}
              onChange={changeTheme}
            />

            <span className="win-label" id="set-mic-label">
              {t("settings.mic")}
            </span>
            <Segmented
              id="set-mic"
              options={MIC_OPTIONS}
              value={settings.includeMic ? "on" : "off"}
              onChange={(next) => void save({ includeMic: next === "on" })}
            />
            <p className="win-hint set-span">{t("settings.micHint")}</p>

            <span className="win-label" id="set-call-language-label">
              {t("settings.callLanguage")}
            </span>
            <Segmented
              id="set-call-language"
              options={CALL_LANGUAGES}
              value={settings.callLanguage}
              onChange={(next) => void save({ callLanguage: next })}
            />
            <p className="win-hint set-span">{t("settings.callLanguageHint")}</p>

            <span className="win-label" id="set-style-label">
              {t("settings.answerStyle")}
            </span>
            <Segmented
              id="set-style"
              className="seg-styles"
              options={ANSWER_STYLES}
              value={settings.answerStyle}
              onChange={(next) => void save({ answerStyle: next })}
            />
            <p className="win-hint set-span">{t("settings.answerStyleHint")}</p>

            <span className="win-label" id="set-translate-label">
              {t("settings.translate")}
            </span>
            <Segmented
              id="set-translate"
              options={TRANSLATE_OPTIONS}
              value={settings.translateTo}
              onChange={(next) => void save({ translateTo: next })}
            />
            <p className="win-hint set-span">{t("settings.translateHint")}</p>

            <label className="win-label" htmlFor="set-opacity">
              {t("settings.opacity")}
            </label>
            <input
              id="set-opacity"
              className="set-range"
              type="range"
              min={MIN_OPACITY}
              max={1}
              step={0.05}
              value={opacity}
              onChange={(event) => {
                const next = clampOpacity(event.currentTarget.valueAsNumber);
                opacityEdited.current = true;
                setOpacity(next);
                applyAppearance(theme, next);
              }}
            />
          </section>

          <form
            className="set-section set-grid"
            onSubmit={(event) => {
              event.preventDefault();
              void saveDraft();
            }}
          >
            <label className="win-label" htmlFor="set-context">
              {t("settings.personalContext")}
            </label>
            <textarea
              id="set-context"
              className="win-input win-textarea"
              rows={4}
              value={draft.personalContext}
              placeholder={t("settings.personalContextPlaceholder")}
              onChange={(event) => {
                const personalContext = event.currentTarget.value;
                setJustSaved(false);
                setDraft((current) => (current === null ? current : { ...current, personalContext }));
              }}
            />
            <p className="win-hint set-span">{t("settings.personalContextHint")}</p>

            <h2 className="win-label set-span">{t("settings.shortcuts")}</h2>
            {SHORTCUT_FIELDS.map((field) => (
              <ShortcutField
                key={field.key}
                id={`set-shortcut-${field.key}`}
                label={t(field.labelKey)}
                value={draft.shortcuts[field.key]}
                onChange={(value) => {
                  setJustSaved(false);
                  setDraft((current) =>
                    current === null ? current : { ...current, shortcuts: { ...current.shortcuts, [field.key]: value } },
                  );
                }}
              />
            ))}
            <p className="win-hint set-span">{t("settings.shortcutsHint")}</p>

            <div className="win-row win-row-end set-span">
              {justSaved && !dirty && <span className="set-status set-ok">{t("settings.saved")}</span>}
              <button type="submit" className="win-btn win-btn-primary" disabled={saving || !dirty || !draftIsValid(draft)}>
                {t("settings.save")}
              </button>
            </div>
          </form>
        </>
      )}
    </WindowFrame>
  );
}

/**
 * A row of toggle buttons in place of a native `<select>`. A `<select>`'s menu is a
 * separate popup that needs its window to be key; in a non-activating panel it opens and
 * closes at once. Plain buttons only need the click.
 */
function Segmented<V extends string>(props: {
  id: string;
  /** Extra classes on the group, e.g. `seg-styles` to wrap a long option list onto two rows. */
  className?: string;
  options: readonly { value: V; labelKey?: CatalogKey; label?: string }[];
  value: V;
  onChange: (value: V) => void;
}) {
  return (
    <div className={props.className === undefined ? "seg" : `seg ${props.className}`} role="radiogroup" aria-labelledby={`${props.id}-label`}>
      {props.options.map((option) => {
        const selected = option.value === props.value;
        return (
          <button
            key={option.value}
            type="button"
            role="radio"
            aria-checked={selected}
            className="seg-btn"
            onClick={() => {
              if (!selected) props.onChange(option.value);
            }}
          >
            {option.label ?? t(option.labelKey as CatalogKey)}
          </button>
        );
      })}
    </div>
  );
}

function ShortcutField(props: { id: string; label: string; value: string; onChange: (value: string) => void }) {
  return (
    <>
      <label className="win-label win-label-quiet" htmlFor={props.id}>
        {props.label}
      </label>
      <input
        id={props.id}
        className="win-input win-mono"
        type="text"
        spellCheck={false}
        autoCapitalize="off"
        value={props.value}
        onChange={(event) => props.onChange(event.currentTarget.value)}
      />
    </>
  );
}
