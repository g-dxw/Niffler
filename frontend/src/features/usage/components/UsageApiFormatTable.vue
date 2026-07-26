<template>
  <Card class="overflow-hidden flex flex-col">
    <div class="px-3 py-2 border-b flex-shrink-0">
      <h3 class="text-sm font-medium">
        {{ t('usageBreakdown.byApiFormat') }}
      </h3>
    </div>
    <div class="overflow-auto max-h-[320px]">
      <Table class="text-sm">
        <TableHeader>
          <TableRow>
            <TableHead class="h-8 px-2">
              {{ t('usageBreakdown.apiFormat') }}
            </TableHead>
            <TableHead class="h-8 px-2 text-right">
              {{ t('usageBreakdown.requests') }}
            </TableHead>
            <TableHead class="h-8 px-2 text-right">
              <div class="flex flex-col text-xs gap-0.5 whitespace-nowrap">
                <span>{{ t('usageBreakdown.inputOutput') }}</span>
                <span class="text-muted-foreground font-normal">{{ t('usageBreakdown.cache') }}</span>
              </div>
            </TableHead>
            <TableHead class="h-8 px-2 text-right">
              {{ isAdmin ? t('usageBreakdown.chargePlatformCost') : t('usageBreakdown.userCharge') }}
            </TableHead>
            <TableHead class="h-8 px-2 text-right">
              {{ t('usageBreakdown.cacheHitRate') }}
            </TableHead>
            <TableHead class="h-8 px-2 text-right">
              {{ t('usageBreakdown.avgResponse') }}
            </TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          <TableRow v-if="data.length === 0">
            <TableCell
              :colspan="6"
              class="text-center py-6 text-muted-foreground px-2"
            >
              {{ t('usageBreakdown.noApiFormatData') }}
            </TableCell>
          </TableRow>
          <TableRow
            v-for="item in data"
            :key="item.api_format"
          >
            <TableCell class="font-medium py-2 px-2">
              {{ formatApiFormat(item.api_format) }}
            </TableCell>
            <TableCell class="text-right py-2 px-2">
              {{ item.request_count }}
            </TableCell>
            <TableCell class="text-right py-2 px-2">
              <div class="flex flex-col items-end text-xs gap-0.5 whitespace-nowrap">
                <span>{{ formatTokens(item.effective_input_tokens ?? item.total_input_context ?? 0) }} / {{ formatTokens(item.output_tokens || 0) }}</span>
                <span class="text-muted-foreground">{{ formatTokens((item.cache_read_tokens || 0) + (item.cache_creation_tokens || 0)) }}</span>
              </div>
            </TableCell>
            <TableCell class="text-right py-2 px-2">
              <div class="flex flex-col items-end text-xs gap-0.5">
                <span class="text-primary font-medium">{{ formatCurrency(item.total_cost) }}</span>
                <span
                  v-if="isAdmin && item.actual_cost !== undefined"
                  class="text-muted-foreground text-[10px]"
                >
                  {{ t('usageBreakdown.platformValue', { value: formatCurrency(item.actual_cost) }) }}
                </span>
              </div>
            </TableCell>
            <TableCell class="text-right py-2 px-2">
              <span>{{ formatHitRate(item.cache_hit_rate) }}</span>
            </TableCell>
            <TableCell class="text-right text-muted-foreground py-2 px-2">
              {{ item.avgResponseTime }}
            </TableCell>
          </TableRow>
        </TableBody>
      </Table>
    </div>
  </Card>
</template>

<script setup lang="ts">
import { useI18n } from 'vue-i18n'
import Card from '@/components/ui/card.vue'
import Table from '@/components/ui/table.vue'
import TableHeader from '@/components/ui/table-header.vue'
import TableBody from '@/components/ui/table-body.vue'
import TableRow from '@/components/ui/table-row.vue'
import TableHead from '@/components/ui/table-head.vue'
import TableCell from '@/components/ui/table-cell.vue'
import { formatTokens, formatCurrency, formatHitRate } from '@/utils/format'
import { formatApiFormat } from '@/api/endpoints/types/api-format'
import type { ApiFormatStatsItem } from '../types'

const { t } = useI18n()

defineProps<{
  data: ApiFormatStatsItem[]
  isAdmin: boolean
}>()
</script>
