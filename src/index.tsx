import { render } from "solid-js/web";

import "./styles.css";

import { Show, createResource } from "solid-js";
import { invoke } from "@tauri-apps/api/tauri";
import { DrinkHistory, DrinkPoint } from "./types/DrinkHistory.ts";
import { Heatmap } from "./components/heatmap.tsx";
import { formatDistance } from 'date-fns'

const App = () => {
  const [drinkHistoryData] = createResource<DrinkHistory>(() => invoke('list_drinks'))
  const [latestDrink, { refetch }] = createResource<DrinkPoint | undefined>(() => invoke('get_latest_drink'))

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

      <Show when={!latestDrink.loading}>
        <div class="bg-neutral-900 my-2 text-xs p-2">
          <Show when={latestDrink() != null} fallback={<p>You haven'n drinked. Maybe drink now?</p>}>
            <p>Your last hydration was {formatDistance(latestDrink()!.timestamp * 1000, new Date(), { addSuffix: true, includeSeconds: true })}.</p>
            <p>You will be notified {formatDistance(Date.now(), (latestDrink()!.timestamp + 3600) * 1000, { addSuffix: true, includeSeconds: true })}. </p>
          </Show>
        </div>
      </Show>

      <div class="flex items-center w-full justify-center p-4 rounded bg-neutral-900">
        <Heatmap />
      </div>

      <div class="flex gap-2 text-xs my-2">
        <button onClick={notif} class="bg-blue-800 px-2 py-1 rounded">Show notif</button>
        <button onClick={oauth} class="bg-blue-800 px-2 py-1 rounded">Google OAuth</button>
      </div>
    
      <pre class="max-h-48 overflow-y-auto overflow-hidden my-4 text-xs bg-neutral-900 p-2 rounded">
        {JSON.stringify(drinkHistoryData(), null, 2)}
      </pre>

    </main>
  );
}

render(() => <App />, document.getElementById("root") as HTMLElement);
