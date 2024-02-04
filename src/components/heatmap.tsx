import 'cal-heatmap/cal-heatmap.css';
import CalHeatmap from 'cal-heatmap';
import Tooltip from 'cal-heatmap/plugins/Tooltip';
import { createEffect, createResource } from 'solid-js';
import { invoke } from '@tauri-apps/api';

const cal: CalHeatmap = new CalHeatmap();

type ListDrinksGroupDayReturnType = Record<string, number>

export const Heatmap = () => {
  const [data] = createResource<ListDrinksGroupDayReturnType>(() => invoke("list_drinks_group_day"))

  createEffect(() => {
    const ret = data()
    if (ret == null) return

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
        color: {
          scheme: "viridis",
          type: "linear",
        }
      },
    }, [[Tooltip, {}]])
  })

  return (
    <div id='cal-heatmap' class='overflow-auto h-full'>
    </div>
  )
}
