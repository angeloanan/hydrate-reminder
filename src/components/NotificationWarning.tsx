import { Show, createResource } from "solid-js"
import { WarningIcon } from "./icons/warning.tsx"
import { invoke } from "@tauri-apps/api"

export const NotificationWarning = () => {
  const [canSendNotification] = createResource<boolean>(() => invoke('can_send_notification'))

  return (
    <Show when={!canSendNotification.loading && canSendNotification() == false}>
      <div class="bg-yellow-100 my-4 text-xs p-2 text-yellow-800 rounded border-yellow-400">
        <div class="flex items-center">
          <WarningIcon class='mr-1' /> <h1 class="font-bold text-lg">Unable to send notifs</h1>
        </div>
        <p>Your OS settings might be interferring with the app's ability to send notifications.</p>
      </div>
    </Show>
  )
}
