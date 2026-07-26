<template>
  <Popover
    :open="open"
    @update:open="emit('update:open', $event)"
  >
    <PopoverTrigger as-child>
      <Button
        variant="ghost"
        size="icon"
        class="h-8 w-8"
        :class="nodeId ? 'text-blue-600' : ''"
        :disabled="saving"
        :title="title"
      >
        <Globe class="w-3.5 h-3.5" />
      </Button>
    </PopoverTrigger>
    <PopoverContent
      class="w-72 p-3"
      side="bottom"
      align="end"
    >
      <div class="space-y-2">
        <div class="flex items-center justify-between">
          <span class="text-xs font-medium">{{ t('providerProxyPopover.title') }}</span>
          <Button
            v-if="nodeId"
            variant="ghost"
            size="sm"
            class="h-6 px-2 text-[10px] text-muted-foreground"
            :disabled="saving"
            @click="emit('clear')"
          >
            {{ t('providerProxyPopover.clear') }}
          </Button>
        </div>
        <ProxyNodeSelect
          :model-value="nodeId || ''"
          trigger-class="h-8"
          @update:model-value="emit('select', $event)"
        />
        <p class="text-[10px] text-muted-foreground">
          {{ nodeId ? t('providerProxyPopover.usingIndependent') : t('providerProxyPopover.notSetFallback') }}
        </p>
      </div>
    </PopoverContent>
  </Popover>
</template>

<script setup lang="ts">
import { useI18n } from 'vue-i18n'
import { Globe } from 'lucide-vue-next'
import { Button, Popover, PopoverTrigger, PopoverContent } from '@/components/ui'
import ProxyNodeSelect from '@/features/providers/components/ProxyNodeSelect.vue'

const { t } = useI18n()

defineProps<{
  open: boolean
  nodeId: string | null | undefined
  saving: boolean
  title: string
}>()

const emit = defineEmits<{
  'update:open': [value: boolean]
  select: [nodeId: string]
  clear: []
}>()
</script>
