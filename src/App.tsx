import { AskApp } from "./ask/AskApp";
import { HubApp } from "./hub/HubApp";
import { TranscriptApp } from "./listen/TranscriptApp";
import { NotesApp } from "./notes/NotesApp";
import { SettingsApp } from "./settings/SettingsApp";

export function App(props: { windowKind: string }) {
  if (props.windowKind === "hub") return <HubApp />;
  if (props.windowKind === "ask") return <AskApp />;
  if (props.windowKind === "listen") return <TranscriptApp />;
  if (props.windowKind === "notes") return <NotesApp />;
  if (props.windowKind === "settings") return <SettingsApp />;
  return <div data-testid="placeholder">{props.windowKind}</div>;
}
