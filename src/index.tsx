import { render } from "solid-js/web";

import "./styles.css";

import { createResource } from "solid-js";
import { invoke } from "@tauri-apps/api/tauri";
import { DrinkHistory } from "./types/DrinkHistory.ts";
import { Heatmap } from "./components/heatmap.tsx";

const App = () => {
  const [drinkHistoryData] = createResource<DrinkHistory>(() => invoke('list_drinks'))

  const notif = async () => {
    console.log("notif")
    await invoke('create_drink_notification')
  }

  const oauth = async () => {
    await invoke('start_oauth_authentication')
  }

  return (
    <main class="m-4">
      <h1 class="font-light text-3xl text-blue-100">🥛 Hydrate</h1>

      <pre class="max-h-48 overflow-y-auto overflow-hidden my-4 text-xs bg-neutral-900 p-2 rounded">
        {JSON.stringify(drinkHistoryData(), null, 2)}
      </pre>

      <div class="flex items-center w-full justify-center p-4 bg-[hsl(288deg,93%,97%)] rounded text-neutral-800">
        <Heatmap />
      </div>

      <div class="flex gap-2 text-xs my-2">
        <button onClick={notif} class="bg-blue-800 px-2 py-1 rounded">Show notif</button>
        <button onClick={oauth} class="bg-blue-800 px-2 py-1 rounded">Google OAuth</button>
      </div>
    </main>
  );
}

render(() => <App />, document.getElementById("root") as HTMLElement);
