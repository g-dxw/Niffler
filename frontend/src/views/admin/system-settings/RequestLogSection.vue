<template>
  <CardSection
    :title="t('requestLog.title')"
    :description="t('requestLog.description')"
  >
    <template #actions>
      <Button
        size="sm"
        :disabled="loading || !hasChanges"
        @click="$emit('save')"
      >
        {{ loading ? t('requestLog.saving') : t('requestLog.save') }}
      </Button>
    </template>
    <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
      <div>
        <Label
          for="request-log-level"
          class="block text-sm font-medium mb-2"
        >
          {{ t('requestLog.level') }}
        </Label>
        <Select
          :model-value="requestRecordLevel"
          @update:model-value="$emit('update:requestRecordLevel', $event)"
        >
          <SelectTrigger
            id="request-log-level"
            class="mt-1"
          >
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="basic">
              {{ t('requestLog.basic') }}
            </SelectItem>
            <SelectItem value="headers">
              {{ t('requestLog.headers') }}
            </SelectItem>
            <SelectItem value="full">
              {{ t('requestLog.full') }}
            </SelectItem>
          </SelectContent>
        </Select>
        <p class="mt-1 text-xs text-muted-foreground">
          {{ t('requestLog.hint') }}
        </p>
      </div>

      <div>
        <Label
          for="max-request-body-size"
          class="block text-sm font-medium"
        >
          {{ t('requestLog.requestSize') }}
        </Label>
        <Input
          id="max-request-body-size"
          :model-value="maxRequestBodySizeKB"
          type="number"
          min="1"
          placeholder="256"
          class="mt-1"
          @update:model-value="$emit('update:maxRequestBodySizeKB', Number($event))"
        />
        <p class="mt-1 text-xs text-muted-foreground">
          {{ t('requestLog.requestSizeHint') }}
        </p>
      </div>

      <div>
        <Label
          for="max-response-body-size"
          class="block text-sm font-medium"
        >
          {{ t('requestLog.responseSize') }}
        </Label>
        <Input
          id="max-response-body-size"
          :model-value="maxResponseBodySizeKB"
          type="number"
          min="1"
          placeholder="256"
          class="mt-1"
          @update:model-value="$emit('update:maxResponseBodySizeKB', Number($event))"
        />
        <p class="mt-1 text-xs text-muted-foreground">
          {{ t('requestLog.responseSizeHint') }}
        </p>
      </div>

      <div>
        <Label
          for="sensitive-headers"
          class="block text-sm font-medium"
        >
          {{ t('requestLog.sensitive') }}
        </Label>
        <Input
          id="sensitive-headers"
          :model-value="sensitiveHeadersStr"
          placeholder="authorization, x-api-key, cookie"
          class="mt-1"
          @update:model-value="$emit('update:sensitiveHeadersStr', $event)"
        />
        <p class="mt-1 text-xs text-muted-foreground">
          {{ t('requestLog.sensitiveHint') }}
        </p>
      </div>
    </div>
  </CardSection>
</template>

<script setup lang="ts">
import { useI18n } from 'vue-i18n'

const { t } = useI18n()
import Button from '@/components/ui/button.vue'
import Input from '@/components/ui/input.vue'
import Label from '@/components/ui/label.vue'
import Select from '@/components/ui/select.vue'
import SelectTrigger from '@/components/ui/select-trigger.vue'
import SelectValue from '@/components/ui/select-value.vue'
import SelectContent from '@/components/ui/select-content.vue'
import SelectItem from '@/components/ui/select-item.vue'
import { CardSection } from '@/components/layout'

defineProps<{
  requestRecordLevel: string
  maxRequestBodySizeKB: number
  maxResponseBodySizeKB: number
  sensitiveHeadersStr: string
  loading: boolean
  hasChanges: boolean
}>()

defineEmits<{
  save: []
  'update:requestRecordLevel': [value: string]
  'update:maxRequestBodySizeKB': [value: number]
  'update:maxResponseBodySizeKB': [value: number]
  'update:sensitiveHeadersStr': [value: string]
}>()
</script>
