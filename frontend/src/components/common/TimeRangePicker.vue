<template>
  <div class="flex flex-wrap items-center gap-2">
    <Select
      v-model="selectedPreset"
    >
      <SelectTrigger class="h-8 w-32 text-xs border-border/60">
        <SelectValue placeholder="选择时间段" />
      </SelectTrigger>
      <SelectContent :searchable="false">
        <SelectItem
          v-for="preset in activePresetOptions"
          :key="preset"
          :value="preset"
        >
          {{ presetLabels[preset] }}
        </SelectItem>
      </SelectContent>
    </Select>

    <div
      v-if="selectedPreset === 'custom'"
      class="flex flex-wrap items-center gap-1.5"
    >
      <div class="flex h-8 max-w-full items-center gap-1.5 rounded-md border border-border/60 bg-background/60 px-2 transition-colors focus-within:border-primary/70 focus-within:bg-background focus-within:ring-1 focus-within:ring-primary/20">
        <span class="shrink-0 text-xs text-muted-foreground">开始</span>
        <Input
          v-if="showTimePicker"
          v-model="startTimeDate"
          type="date"
          aria-label="开始日期"
          class="h-6 w-32 border-0 bg-transparent px-0 text-xs shadow-none focus-visible:ring-0"
        />
        <Input
          v-else
          v-model="startDate"
          type="date"
          aria-label="开始日期"
          class="h-6 w-32 border-0 bg-transparent px-0 text-xs shadow-none focus-visible:ring-0"
        />
        <Input
          v-if="showTimePicker"
          v-model="startTimeClock"
          type="time"
          aria-label="开始时间"
          class="h-6 w-20 border-0 bg-transparent px-0 text-xs shadow-none focus-visible:ring-0"
        />
      </div>

      <span class="hidden text-xs text-muted-foreground sm:inline">至</span>

      <div class="flex h-8 max-w-full items-center gap-1.5 rounded-md border border-border/60 bg-background/60 px-2 transition-colors focus-within:border-primary/70 focus-within:bg-background focus-within:ring-1 focus-within:ring-primary/20">
        <span class="shrink-0 text-xs text-muted-foreground">结束</span>
        <Input
          v-if="showTimePicker"
          v-model="endTimeDate"
          type="date"
          aria-label="结束日期"
          class="h-6 w-32 border-0 bg-transparent px-0 text-xs shadow-none focus-visible:ring-0"
        />
        <Input
          v-else
          v-model="endDate"
          type="date"
          aria-label="结束日期"
          class="h-6 w-32 border-0 bg-transparent px-0 text-xs shadow-none focus-visible:ring-0"
        />
        <Input
          v-if="showTimePicker"
          v-model="endTimeClock"
          type="time"
          aria-label="结束时间"
          class="h-6 w-20 border-0 bg-transparent px-0 text-xs shadow-none focus-visible:ring-0"
        />
      </div>
    </div>

    <Select
      v-if="showGranularity"
      v-model="selectedGranularity"
    >
      <SelectTrigger class="h-8 w-24 text-xs border-border/60">
        <SelectValue placeholder="粒度" />
      </SelectTrigger>
      <SelectContent>
        <SelectItem
          v-if="allowHourly && canUseHourly"
          value="hour"
        >
          小时
        </SelectItem>
        <SelectItem value="day">
          天
        </SelectItem>
        <SelectItem value="week">
          周
        </SelectItem>
        <SelectItem value="month">
          月
        </SelectItem>
      </SelectContent>
    </Select>
  </div>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
  Input
} from '@/components/ui'
import type { DateRangeParams } from '@/features/usage/types'

const selectablePresets = ['today', 'yesterday', 'last7days', 'last30days', 'last90days', 'custom'] as const
type SelectablePreset = typeof selectablePresets[number]

const presetLabels: Record<SelectablePreset, string> = {
  today: '今天',
  yesterday: '昨天',
  last7days: '最近7天',
  last30days: '最近30天',
  last90days: '最近90天',
  custom: '自定义'
}

const props = withDefaults(defineProps<{
  modelValue: DateRangeParams
  showGranularity?: boolean
  allowHourly?: boolean
  showTime?: boolean
  presetOptions?: SelectablePreset[]
}>(), {
  presetOptions: () => ['today', 'yesterday', 'last7days', 'last30days', 'last90days', 'custom']
})

const emit = defineEmits<{
  'update:modelValue': [value: DateRangeParams]
}>()

const activePresetOptions = computed<SelectablePreset[]>(() => {
  const unique = new Set(props.presetOptions)
  const filtered = selectablePresets.filter((preset) => unique.has(preset))
  return filtered.length > 0 ? filtered : [...selectablePresets]
})

function defaultPreset(): SelectablePreset {
  const options = activePresetOptions.value
  if (options.includes('last7days')) return 'last7days'
  return options[0] ?? 'last7days'
}

function normalizePreset(value: DateRangeParams): SelectablePreset {
  if (value.preset && activePresetOptions.value.includes(value.preset as SelectablePreset)) {
    return value.preset as SelectablePreset
  }
  if (!value.preset && (value.start_time || value.end_time || value.start_date || value.end_date) && activePresetOptions.value.includes('custom')) {
    return 'custom'
  }
  return defaultPreset()
}

const selectedPreset = ref<SelectablePreset>(normalizePreset(props.modelValue))
const startDate = ref(props.modelValue.start_date || '')
const endDate = ref(props.modelValue.end_date || '')
const startTime = ref(props.modelValue.start_time || '')
const endTime = ref(props.modelValue.end_time || '')
const selectedGranularity = ref(props.modelValue.granularity || 'day')

const showGranularity = computed(() => props.showGranularity !== false)
const allowHourly = computed(() => props.allowHourly === true)
const showTimePicker = computed(() => props.showTime === true)

function pad2(value: number): string {
  return value.toString().padStart(2, '0')
}

function dateInputValue(date: Date): string {
  return `${date.getFullYear()}-${pad2(date.getMonth() + 1)}-${pad2(date.getDate())}`
}

function timeInputValue(date: Date): string {
  return `${pad2(date.getHours())}:${pad2(date.getMinutes())}`
}

function splitDateTime(value: string): { date: string, time: string } {
  const [date = '', rawTime = ''] = value.split('T')
  return {
    date,
    time: rawTime.slice(0, 5)
  }
}

function joinDateTime(date: string, time: string, defaultTime: string): string {
  const normalizedDate = date || dateInputValue(new Date())
  const normalizedTime = time || defaultTime
  return `${normalizedDate}T${normalizedTime}`
}

function ensureCustomRangeDefaults() {
  const now = new Date()
  const today = dateInputValue(now)

  if (showTimePicker.value) {
    if (!startTime.value) startTime.value = `${today}T00:00`
    if (!endTime.value) endTime.value = `${today}T${timeInputValue(now)}`
    return
  }

  if (!startDate.value) startDate.value = today
  if (!endDate.value) endDate.value = today
}

const startTimeDate = computed({
  get: () => splitDateTime(startTime.value).date,
  set: (value: string) => {
    const current = splitDateTime(startTime.value)
    startTime.value = joinDateTime(value, current.time, '00:00')
  }
})

const startTimeClock = computed({
  get: () => splitDateTime(startTime.value).time,
  set: (value: string) => {
    const current = splitDateTime(startTime.value)
    startTime.value = joinDateTime(current.date, value, '00:00')
  }
})

const endTimeDate = computed({
  get: () => splitDateTime(endTime.value).date,
  set: (value: string) => {
    const current = splitDateTime(endTime.value)
    endTime.value = joinDateTime(value, current.time, '23:59')
  }
})

const endTimeClock = computed({
  get: () => splitDateTime(endTime.value).time,
  set: (value: string) => {
    const current = splitDateTime(endTime.value)
    endTime.value = joinDateTime(current.date, value, '23:59')
  }
})

const canUseHourly = computed(() => {
  if (selectedPreset.value === 'today' || selectedPreset.value === 'yesterday') return true
  if (selectedPreset.value === 'custom' && showTimePicker.value && startTime.value && endTime.value) {
    return startTime.value.slice(0, 10) === endTime.value.slice(0, 10)
  }
  if (selectedPreset.value === 'custom' && startDate.value && endDate.value) {
    return startDate.value === endDate.value
  }
  return false
})

// 记录上次 emit 的值，避免重复触发
let lastEmittedValue: string | null = null

function buildEmitValue(): DateRangeParams {
  const timezone = Intl.DateTimeFormat().resolvedOptions().timeZone
  const tz_offset_minutes = -new Date().getTimezoneOffset()

  if (selectedPreset.value === 'custom') {
    if (showTimePicker.value) {
      const start = startTime.value <= endTime.value ? startTime.value : endTime.value
      const end = endTime.value >= startTime.value ? endTime.value : startTime.value
      return {
        start_date: start.slice(0, 10),
        end_date: end.slice(0, 10),
        start_time: start,
        end_time: end,
        granularity: selectedGranularity.value,
        timezone,
        tz_offset_minutes
      }
    }

    const start = startDate.value <= endDate.value ? startDate.value : endDate.value
    const end = endDate.value >= startDate.value ? endDate.value : startDate.value
    return {
      start_date: start,
      end_date: end,
      granularity: selectedGranularity.value,
      timezone,
      tz_offset_minutes
    }
  }

  return {
    preset: selectedPreset.value,
    granularity: selectedGranularity.value,
    timezone,
    tz_offset_minutes
  }
}

function getValueKey(value: DateRangeParams): string {
  // 只比较核心字段，忽略 timezone 和 tz_offset_minutes（这些每次都会重新计算）
  if (value.preset) {
    return `preset:${value.preset}:${value.granularity}`
  }
  if (value.start_time || value.end_time) {
    return `custom-time:${value.start_time}:${value.end_time}:${value.granularity}`
  }
  return `custom:${value.start_date}:${value.end_date}:${value.granularity}`
}

watch(() => props.modelValue, (value) => {
  selectedPreset.value = normalizePreset(value)
  if (value.start_date !== undefined) startDate.value = value.start_date || ''
  if (value.end_date !== undefined) endDate.value = value.end_date || ''
  if (value.start_time !== undefined) startTime.value = value.start_time || ''
  if (value.end_time !== undefined) endTime.value = value.end_time || ''
  if (value.granularity) selectedGranularity.value = value.granularity
  // 同步更新 lastEmittedValue，避免外部设置值后触发重复 emit
  lastEmittedValue = getValueKey(value)
}, { deep: true })

watch(activePresetOptions, () => {
  if (!activePresetOptions.value.includes(selectedPreset.value)) {
    selectedPreset.value = normalizePreset(props.modelValue)
  }
})

watch([selectedPreset, startDate, endDate, startTime, endTime, selectedGranularity], () => {
  if (selectedPreset.value === 'custom') {
    ensureCustomRangeDefaults()
  }

  if (!allowHourly.value || !canUseHourly.value) {
    if (selectedGranularity.value === 'hour') {
      selectedGranularity.value = 'day'
    }
  }

  if (selectedPreset.value === 'custom') {
    if (showTimePicker.value) {
      if (!startTime.value || !endTime.value) return
    } else if (!startDate.value || !endDate.value) {
      return
    }
  }

  const newValue = buildEmitValue()
  const newKey = getValueKey(newValue)

  // 只有当值真正变化时才 emit，避免初始化时的重复触发
  if (newKey !== lastEmittedValue) {
    lastEmittedValue = newKey
    emit('update:modelValue', newValue)
  }
}, { immediate: true })
</script>
