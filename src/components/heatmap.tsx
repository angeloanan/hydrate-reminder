import 'cal-heatmap/cal-heatmap.css';
import CalHeatmap from 'cal-heatmap';
import Tooltip from 'cal-heatmap/plugins/Tooltip';
import { createEffect, createResource, createSignal, onCleanup, onMount } from 'solid-js';
import { invoke } from '@tauri-apps/api';
import { WaterVolumeFormatter } from '../util/formatter';
import { MONTHS } from '../util/datetime';
import { UnlistenFn, listen } from '@tauri-apps/api/event';

const cal: CalHeatmap = new CalHeatmap();

type ListDrinksGroupDayReturnType = Record<string, number>

export const Heatmap = () => {
  const [unlistenDrink, setUnlistenDrink] = createSignal<UnlistenFn>()
  const [data, { refetch }] = createResource<ListDrinksGroupDayReturnType>(() => invoke("list_drinks_group_day"))

  onMount(async () => {
    const unlisten = await listen('drink', refetch)
    setUnlistenDrink(() => unlisten)
  })

  onCleanup(() => {
    unlistenDrink()?.()
  })

  createEffect(() => {
    const ret = data()
    if (data.loading) return

    const processed_data = Object.entries(ret)
    console.log(processed_data)

    cal.paint({
      theme: "light",
      data: {
        source: processed_data, 
        x: p => p[0],
        y : p => p[1],
        defaultValue: 0
      },
      range: 3,
      date: {
        start: new Date(processed_data[0][0])
      },
      domain: {
        type: "month",
      },
      subDomain: {
        type: "day"
      },
      scale: {
        color: { type: 'diverging', scheme: 'PRGn', domain: [0, 2500] } 
      },
    }, [[Tooltip, {
      text: (timestamp: number, value: number) => {
        const date = new Date(timestamp)
        return `${WaterVolumeFormatter.format(value)} on ${MONTHS[date.getMonth()]} ${date.getDate()}, ${date.getFullYear()}`
      }
    }]])
  })

  return (
    <div id='cal-heatmap' class='overflow-auto h-full'>
    </div>
  )
}
