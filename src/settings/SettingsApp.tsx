import { useEffect, useRef, useState } from "react";
import { getLocale, setLocale, t, type CatalogKey, type Locale } from "../lib/i18n";
import {
  activateLicense,
  deactivateLicense,
  getLicenseStatus,
  hasApiKey,
  saveApiKey,
  saveSettings,
} from "../lib/ipc";
import type { License, Settings, SettingsPatch, Shortcuts } from "../lib/models";
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

const THEMES: readonly { value: Theme; labelKey: CatalogKey }[] = [
  { value: "dark", labelKey: "settings.themeDark" },
  { value: "light", labelKey: "settings.themeLight" },
];

/**
 * The OpenAI key (Keychain), and the saved settings. Language, theme and opacity apply
 * and save as soon as they change; the model and the shortcuts wait for Save, because
 * they are invalid while being typed. Rust re-registers the shortcuts when they change.
 */
export function SettingsApp() {
  const { settings, setSettings, locale: bootLocale, bootErrorKey } = useSettingsBoot();
  const [locale, setLocaleState] = useState<Locale>(bootLocale);
  const [errorKey, setErrorKey] = useState<CatalogKey | null>(null);
  const [keyStored, setKeyStored] = useState<boolean | null>(null);
  const [keyInput, setKeyInput] = useState("");
  const [savingKey, setSavingKey] = useState(false);
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

  useEffect(() => {
    let cancelled = false;
    hasApiKey()
      .then((stored) => {
        if (!cancelled) setKeyStored(stored);
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
    if (key === "" || savingKey) return;
    setSavingKey(true);
    try {
      await saveApiKey(key);
      setKeyInput("");
      setKeyStored(true);
    } catch (error) {
      setErrorKey(errorCopyKey(error));
    } finally {
      setSavingKey(false);
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
        <h2 className="win-label">{t("settings.apiKey")}</h2>
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
        </form>
        {keyStored !== null && (
          <p className={keyStored ? "set-status set-ok" : "set-status win-muted"}>
            {keyStored ? t("settings.keySaved") : t("settings.keyNotSet")}
          </p>
        )}
        <p className="win-hint">{t("settings.apiKeyHint")}</p>
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
            <label className="win-label" htmlFor="set-model">
              {t("settings.answersModel")}
            </label>
            <input
              id="set-model"
              className="win-input win-mono"
              type="text"
              spellCheck={false}
              value={draft.answersModel}
              onChange={(event) => {
                const answersModel = event.currentTarget.value;
                setJustSaved(false);
                setDraft((current) => (current === null ? current : { ...current, answersModel }));
              }}
            />

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
  options: readonly { value: V; labelKey: CatalogKey }[];
  value: V;
  onChange: (value: V) => void;
}) {
  return (
    <div className="seg" role="radiogroup" aria-labelledby={`${props.id}-label`}>
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
            {t(option.labelKey)}
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
