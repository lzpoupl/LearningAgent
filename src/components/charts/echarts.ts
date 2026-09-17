import { BarChart, PieChart } from 'echarts/charts'
import { DataZoomComponent, GridComponent, TooltipComponent } from 'echarts/components'
import * as echarts from 'echarts/core'
import { CanvasRenderer } from 'echarts/renderers'

/** 只注册统计页用到的图表与组件，保持打包体积可控。 */
echarts.use([
  BarChart,
  PieChart,
  DataZoomComponent,
  GridComponent,
  TooltipComponent,
  CanvasRenderer,
])

/** ECharts 实例类型，避免各组件重复导入类型。 */
export type EChartsInstance = ReturnType<typeof echarts.init>

export type { EChartsOption } from 'echarts'

export { echarts }