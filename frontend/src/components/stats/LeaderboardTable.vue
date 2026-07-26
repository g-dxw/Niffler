<template>
  <TableCard :title="title">
    <template #actions>
      <slot name="actions">
        <Select
          v-if="showMetricSelect"
          :model-value="metric"
          @update:model-value="emitMetric"
        >
          <SelectTrigger class="h-8 text-xs w-28">
            <SelectValue :placeholder="t('leaderboard.metric')" />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="requests">
              {{ t('leaderboard.requests') }}
            </SelectItem>
            <SelectItem value="tokens">
              Tokens
            </SelectItem>
            <SelectItem value="cost">
              {{ t('leaderboard.cost') }}
            </SelectItem>
          </SelectContent>
        </Select>
      </slot>
    </template>

    <div
      v-if="loading"
      class="p-6"
    >
      <LoadingState />
    </div>
    <div
      v-else-if="items.length === 0"
      class="p-6"
    >
      <EmptyState
        :title="t('leaderboard.empty')"
        :description="t('leaderboard.emptyDescription')"
      />
    </div>
    <Table v-else>
      <TableHeader>
        <TableRow>
          <TableHead class="w-16">
            {{ t('leaderboard.rank') }}
          </TableHead>
          <TableHead>{{ t('leaderboard.name') }}</TableHead>
          <TableHead class="text-right">
            {{ t('leaderboard.requests') }}
          </TableHead>
          <TableHead class="text-right">
            Tokens
          </TableHead>
          <TableHead class="text-right">
            {{ t('leaderboard.cost') }}
          </TableHead>
        </TableRow>
      </TableHeader>
      <TableBody>
        <TableRow
          v-for="item in items"
          :key="item.id"
        >
          <TableCell class="font-medium">
            {{ item.rank }}
          </TableCell>
          <TableCell>{{ item.name }}</TableCell>
          <TableCell class="text-right">
            {{ item.requests }}
          </TableCell>
          <TableCell class="text-right">
            {{ formatTokens(item.tokens) }}
          </TableCell>
          <TableCell class="text-right">
            {{ formatCurrency(item.cost) }}
          </TableCell>
        </TableRow>
      </TableBody>
    </Table>

    <slot name="pagination" />
  </TableCard>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { EmptyState, LoadingState } from '@/components/common'
import { TableCard } from '@/components/ui'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow
} from '@/components/ui'
import { formatCurrency, formatTokens } from '@/utils/format'
import type { LeaderboardItem } from '@/api/admin'

const { t } = useI18n()

interface Props {
  title: string
  items: LeaderboardItem[]
  metric: 'requests' | 'tokens' | 'cost'
  loading?: boolean
  showMetricSelect?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  loading: false,
  showMetricSelect: true
})

const emit = defineEmits<{
  (e: 'update:metric', value: 'requests' | 'tokens' | 'cost'): void
}>()

const metric = computed(() => props.metric)

function emitMetric(value: string) {
  if (value === 'requests' || value === 'tokens' || value === 'cost') {
    emit('update:metric', value)
  }
}
</script>
