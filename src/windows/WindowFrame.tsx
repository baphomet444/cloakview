import { getCurrentWindow } from "@tauri-apps/api/window";
import type { ReactNode } from "react";
import { t, type CatalogKey, type Locale } from "../lib/i18n";

/**
 * The chrome every feature window shares: a title bar that drags the window, a Close
 * button that hides this window only (the hub's Close hides everything), and an error
 * line that dismisses on click.
 *
 * Nothing in here, or in the windows, takes focus on mount: these are non-activating
 * panels over another app, and an autofocused field would take that app's keyboard.
 * The panel becomes key only when the user clicks into a field.
 */
export function WindowFrame(props: {
  titleKey: CatalogKey;
  locale: Locale;
  errorKey: CatalogKey | null;
  onDismissError: () => void;
  children: ReactNode;
  footer?: ReactNode;
}) {
  return (
    <div className="win" lang={props.locale}>
      <header className="win-bar" data-tauri-drag-region>
        <h1 className="win-title" data-tauri-drag-region>
          {t(props.titleKey)}
        </h1>
        <button type="button" className="win-btn win-close" onClick={() => void getCurrentWindow().hide()}>
          {t("window.close")}
        </button>
      </header>
      {props.errorKey !== null && (
        <button type="button" className="win-error" role="alert" onClick={props.onDismissError}>
          {t(props.errorKey)}
        </button>
      )}
      <main className="win-body">{props.children}</main>
      {props.footer !== undefined && <footer className="win-foot">{props.footer}</footer>}
    </div>
  );
}
