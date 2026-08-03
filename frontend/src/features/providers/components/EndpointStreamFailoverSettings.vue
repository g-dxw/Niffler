<template>
  <div class="rounded-md border bg-muted/20 p-3 space-y-3">
    <div class="flex items-start justify-between gap-4">
      <div class="min-w-0 space-y-1">
        <Label class="text-sm font-medium">
          {{ t('streamFailoverUi.title') }}
        </Label>
        <p class="text-xs leading-5 text-muted-foreground">
          {{ t('streamFailoverUi.description') }}
        </p>
      </div>
      <div class="flex items-center gap-1 shrink-0">
        <template v-if="changed">
          <Button
            variant="ghost"
            size="icon"
            class="h-8 w-8"
            :title="t('endpointForm.save')"
            :disabled="saving || !!validationMessage"
            @click="emit('save')"
          >
            <Check class="w-4 h-4" />
          </Button>
          <Button
            variant="ghost"
            size="icon"
            class="h-8 w-8"
            :title="t('endpointForm.reset')"
            :disabled="saving"
            @click="emit('reset')"
          >
            <RotateCcw class="w-4 h-4" />
          </Button>
        </template>
        <Switch
          :model-value="modelValue.enabled"
          :disabled="saving"
          :aria-label="t('streamFailoverUi.title')"
          @update:model-value="updateEnabled"
        />
      </div>
    </div>

    <div
      v-if="modelValue.enabled"
      class="grid grid-cols-2 gap-3 lg:grid-cols-4"
    >
      <div
        v-for="field in fields"
        :key="field.key"
        class="space-y-1.5"
      >
        <Label class="text-xs text-muted-foreground">
          {{ t(field.label) }}
        </Label>
        <Input
          type="number"
          :min="field.min"
          :max="field.max"
          :step="field.step"
          :model-value="modelValue[field.key]"
          :disabled="saving"
          @update:model-value="(value) => updateField(field.key, value)"
        />
      </div>
    </div>

    <p
      v-if="changed && validationMessage"
      class="text-xs text-destructive"
    >
      {{ validationMessage }}
    </p>
  </div>
</template>

<script setup lang="ts">
import { useI18n } from 'vue-i18n'
import { Check, RotateCcw } from 'lucide-vue-next'

import { Button, Input, Label, Switch } from '@/components/ui'
import type { EndpointStreamFailoverState } from './endpoint-stream-failover'

type NumberField = Exclude<keyof EndpointStreamFailoverState, 'enabled'>

const { modelValue, saving, changed, validationMessage } = defineProps<{
  modelValue: EndpointStreamFailoverState
  saving: boolean
  changed: boolean
  validationMessage: string | null
}>()

const emit = defineEmits<{
  'update:modelValue': [value: EndpointStreamFailoverState]
  save: []
  reset: []
}>()

const { t } = useI18n()

const fields: Array<{
  key: NumberField
  label: string
  min: string
  max: string
  step: string
}> = [
  { key: 'maxRetries', label: 'streamFailoverUi.maxRetries', min: '0', max: '999', step: '1' },
  { key: 'maxWaitSeconds', label: 'streamFailoverUi.maxWaitSeconds', min: '0.25', max: '30', step: '0.25' },
  { key: 'maxBufferKilobytes', label: 'streamFailoverUi.maxBufferKilobytes', min: '16', max: '1024', step: '1' },
  { key: 'cooldownSeconds', label: 'streamFailoverUi.cooldownSeconds', min: '1', max: '1920', step: '1' },
]

function updateEnabled(enabled: boolean) {
  emit('update:modelValue', { ...modelValue, enabled })
}

function updateField(field: NumberField, value: unknown) {
  emit('update:modelValue', { ...modelValue, [field]: String(value ?? '') })
}
</script>
