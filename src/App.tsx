import { HubApp } from "./hub/HubApp";

export function App(props: { windowKind: string }) {
  if (props.windowKind === "hub") return <HubApp />;
  return <div data-testid="placeholder">{props.windowKind}</div>;
}
