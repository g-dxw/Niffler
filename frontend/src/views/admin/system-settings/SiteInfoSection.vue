<template>
  <CardSection
    :title="t('siteInfo.title')"
    :description="t('siteInfo.description')"
  >
    <template #actions>
      <Button
        size="sm"
        :disabled="loading || !hasChanges"
        @click="$emit('save')"
      >
        {{ loading ? t('siteInfo.saving') : t('siteInfo.save') }}
      </Button>
    </template>
    <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
      <div>
        <Label
          for="site-name"
          class="block text-sm font-medium"
        >
          {{ t('siteInfo.name') }}
        </Label>
        <Input
          id="site-name"
          :model-value="siteName"
          type="text"
          placeholder="Niffler"
          class="mt-1"
          @update:model-value="$emit('update:siteName', $event)"
        />
        <p class="mt-1 text-xs text-muted-foreground">
          {{ t('siteInfo.nameHint') }}
        </p>
      </div>
      <div>
        <Label
          for="site-subtitle"
          class="block text-sm font-medium"
        >
          {{ t('siteInfo.subtitle') }}
        </Label>
        <Input
          id="site-subtitle"
          :model-value="siteSubtitle"
          type="text"
          placeholder="AI Gateway"
          class="mt-1"
          @update:model-value="$emit('update:siteSubtitle', $event)"
        />
        <p class="mt-1 text-xs text-muted-foreground">
          {{ t('siteInfo.subtitleHint') }}
        </p>
      </div>
    </div>

    <div class="mt-6 space-y-4 border-t border-border/60 pt-5">
      <p class="text-xs text-muted-foreground">{{ t('siteInfo.contactHint') }}</p>
      <div>
        <div>
          <Label for="contact-us-format" class="block text-sm font-medium">{{ t('siteInfo.format') }}</Label>
          <Select
            :model-value="contactUsFormat"
            @update:model-value="$emit('update:contactUsFormat', $event as 'markdown' | 'html')"
          >
            <SelectTrigger id="contact-us-format" class="mt-1">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="markdown">Markdown</SelectItem>
              <SelectItem value="html">HTML</SelectItem>
            </SelectContent>
          </Select>
        </div>
      </div>
      <div>
        <Label for="contact-us-content" class="block text-sm font-medium">{{ t('siteInfo.contact') }}</Label>
        <Textarea
          id="contact-us-content"
          :model-value="contactUsContent"
          rows="10"
          class="mt-1 font-mono text-xs"
          :placeholder="t('siteInfo.contactPlaceholder')"
          @update:model-value="$emit('update:contactUsContent', $event)"
        />
      </div>
    </div>
  </CardSection>
</template>

<script setup lang="ts">
import { useI18n } from 'vue-i18n'
import Button from '@/components/ui/button.vue'
import Input from '@/components/ui/input.vue'
import Label from '@/components/ui/label.vue'
import Textarea from '@/components/ui/textarea.vue'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui'
import { CardSection } from '@/components/layout'

const { t } = useI18n()

withDefaults(defineProps<{
  siteName: string
  siteSubtitle: string
  contactUsFormat?: 'markdown' | 'html'
  contactUsContent?: string
  loading: boolean
  hasChanges: boolean
}>(), {
  contactUsFormat: 'markdown',
  contactUsContent: '',
})

defineEmits<{
  save: []
  'update:siteName': [value: string]
  'update:siteSubtitle': [value: string]
  'update:contactUsFormat': [value: 'markdown' | 'html']
  'update:contactUsContent': [value: string]
}>()
</script>
