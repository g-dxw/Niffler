import type { ChartData, ChartOptions } from 'chart.js'

export interface TimeScatterPoint {
  x: string
  y: number
  _originalX?: string
  _originalY?: number
}

export type TimeScatterData = ChartData<'scatter', TimeScatterPoint[], unknown>
export type TimeScatterOptions = ChartOptions<'scatter'>
