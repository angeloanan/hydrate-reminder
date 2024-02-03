import { createResource, createSignal } from "solid-js";
import logo from "./assets/logo.svg";
import { invoke } from "@tauri-apps/api/tauri";
import "./App.css";
import { DrinkHistory } from "./types/DrinkHistory.ts";

function App() {
  const [greetMsg, setGreetMsg] = createSignal("");
  const [name, setName] = createSignal("");

  const [drinkHistoryData] = createResource<DrinkHistory>(() => invoke('list_drinks'))

  async function greet() {
    // Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
    setGreetMsg(await invoke("greet", { name: name() }));
  }

  const notif = async () => {
    console.log("notif")
    await invoke('create_drink_notification')
  }

  const oauth = async () => {
    await invoke('start_oauth_authentication')
  }

  return (
    <div class="container">
      <h1>Welcome to Tauri!</h1>

      <div class="row">
        <a href="https://vitejs.dev" target="_blank">
          <img src="/vite.svg" class="logo vite" alt="Vite logo" />
        </a>
        <a href="https://tauri.app" target="_blank">
          <img src="/tauri.svg" class="logo tauri" alt="Tauri logo" />
        </a>
        <a href="https://solidjs.com" target="_blank">
          <img src={logo} class="logo solid" alt="Solid logo" />
        </a>
      </div>

      <p>Click on the Tauri, Vite, and Solid logos to learn more.</p>

      <form
        class="row"
        onSubmit={(e) => {
          e.preventDefault();
          greet();
        }}
      >
        <input
          id="greet-input"
          onChange={(e) => setName(e.currentTarget.value)}
          placeholder="Enter a name..."
        />
        <button type="submit">Greet</button>
      </form>

      <p>{greetMsg()}</p>

      <pre style={{ "width": "100vw", "text-align": "left" }}>
        {JSON.stringify(drinkHistoryData(), null, 2)}
      </pre>

      <button onClick={notif}>Notification!</button>
      <button onClick={oauth}>Open Google OAuth</button>
    </div>
  );
}

export default App;
