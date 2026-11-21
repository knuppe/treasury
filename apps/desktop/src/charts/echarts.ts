/**
 * ECharts com só os módulos usados (tree-shaking): o bundle fica bem menor que
 * `import * as echarts from 'echarts'`.
 */
import {
  BarChart,
  HeatmapChart,
  LineChart,
  PieChart,
  SankeyChart,
  ScatterChart,
  SunburstChart,
  TreemapChart,
} from 'echarts/charts';
import {
  CalendarComponent,
  DataZoomComponent,
  GridComponent,
  LegendComponent,
  MarkAreaComponent,
  MarkLineComponent,
  MarkPointComponent,
  TitleComponent,
  TooltipComponent,
  VisualMapComponent,
  AriaComponent,
} from 'echarts/components';
import * as echarts from 'echarts/core';
import { CanvasRenderer } from 'echarts/renderers';

echarts.use([
  BarChart,
  HeatmapChart,
  LineChart,
  PieChart,
  SankeyChart,
  ScatterChart,
  SunburstChart,
  TreemapChart,
  CalendarComponent,
  DataZoomComponent,
  GridComponent,
  LegendComponent,
  MarkAreaComponent,
  MarkLineComponent,
  MarkPointComponent,
  TitleComponent,
  TooltipComponent,
  VisualMapComponent,
  AriaComponent,
  CanvasRenderer,
]);

export { echarts };
export type { EChartsCoreOption as EChartsOption, ECharts } from 'echarts/core';
