import { render } from "solid-js/web";

import "./styles.css";

import { ErrorBoundary, Match, Show, Switch, createResource, createSignal, onCleanup, onMount } from "solid-js";
import { invoke } from "@tauri-apps/api/tauri";
import { DrinkPoint } from "./types/DrinkHistory.ts";
import { Heatmap } from "./components/heatmap.tsx";
import { formatDistance } from 'date-fns'
import { UnlistenFn, listen } from "@tauri-apps/api/event";
import { NotificationWarning } from "./components/NotificationWarning.tsx";

const App = () => {
  const [unlistenDrink, setUnlistenDrink] = createSignal<UnlistenFn>();

  const [latestDrink, { refetch: refetchLatestDrink }] = createResource<DrinkPoint | undefined>(() => invoke('get_latest_drink'))
  
  const notif = async () => {
    console.log("notif")
    await invoke('create_drink_notification')
  }

  const oauth = async () => {
    await invoke('start_oauth_authentication')
  }

  onMount(async () => {
    const unlisten = await listen('drink', refetchLatestDrink)
    setUnlistenDrink(() => unlisten)
  })

  onCleanup(() => {
    unlistenDrink()?.()
  })

  const lastDrinkTime = () => formatDistance(latestDrink()!.timestamp * 1000, Date.now(), { addSuffix: true })
  const isLastDrinkOld = () => {
    if (latestDrink() == null) return false
    return latestDrink()!.timestamp + 3600 < Math.floor(Date.now() / 1000)
  }

  return (
    <main class="m-4">
      <h1 class="font-light text-3xl text-blue-100">🥛 Hydrate</h1>

      <NotificationWarning />

      <ErrorBoundary fallback={(err) => <div>Error: {err.message}</div>}>
        <Show when={!latestDrink.loading}>
          <div class="bg-neutral-900 my-2 text-xs p-2">
            <Show when={latestDrink() != null} fallback={<p>You haven&apos;t drinked. Go take a sip of water!</p>}>
              <p>Your last drink was {lastDrinkTime()}.</p>
              <Switch>
                <Match when={isLastDrinkOld()}>
                  <p>It&apos;s been a while since your last drink. Go take a sip of water!</p>
                </Match>
                <Match when={!isLastDrinkOld()}>
                  <p>You will be notified {formatDistance((latestDrink()!.timestamp + 3600) * 1000, Date.now(), { addSuffix: true, includeSeconds: false })}. </p>
                </Match>
              </Switch>
            </Show>
          </div>
        </Show>
      </ErrorBoundary>

      <div class="flex items-center w-full justify-center p-4 rounded bg-neutral-900">
        <Heatmap />
      </div>

      <div class="flex gap-2 text-xs my-2">
        <button onClick={notif} class="bg-blue-800 px-2 py-1 rounded">Show notif</button>
        <button onClick={oauth} class="bg-blue-800 px-2 py-1 rounded">Google OAuth</button>
      </div>

      {/* <pre class="max-h-48 overflow-y-auto overflow-hidden my-4 text-xs bg-neutral-900 p-2 rounded">
        {JSON.stringify(drinkHistoryData(), null, 2)}
      </pre> */}
    </main>
  );
}

render(() => <App />, document.getElementById("root") as HTMLElement);
