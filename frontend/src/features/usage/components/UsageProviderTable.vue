<template>
  <Card class="overflow-hidden flex flex-col">
    <div class="px-3 py-2 border-b flex-shrink-0">
      <h3 class="text-sm font-medium">
        {{ t('usageBreakdown.byProvider') }}
      </h3>
    </div>
    <div class="overflow-auto max-h-[320px]">
      <Table class="text-sm">
        <TableHeader>
          <TableRow>
            <TableHead class="h-8 px-2">
              {{ t('usageBreakdown.provider') }}
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
              {{ t('usageBreakdown.successRate') }}
            </TableHead>
            <TableHead class="h-8 px-2 text-right">
              {{ t('usageBreakdown.avgResponse') }}
            </TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          <TableRow v-if="data.length === 0">
            <TableCell
              :colspan="7"
              class="text-center py-6 text-muted-foreground px-2"
            >
              {{ t('usageBreakdown.noProviderData') }}
            </TableCell>
          </TableRow>
          <TableRow
            v-for="provider in data"
            :key="provider.provider"
          >
            <TableCell class="font-medium py-2 px-2">
              {{ provider.provider }}
            </TableCell>
            <TableCell class="text-right py-2 px-2">
              {{ provider.requests }}
            </TableCell>
            <TableCell class="text-right py-2 px-2">
              <div class="flex flex-col items-end text-xs gap-0.5 whitespace-nowrap">
                <span>{{ formatTokens(provider.effectiveInputTokens ?? provider.totalInputContext ?? 0) }} / {{ formatTokens(provider.outputTokens || 0) }}</span>
                <span class="text-muted-foreground">{{ formatTokens((provider.cacheReadTokens || 0) + (provider.cacheCreationTokens || 0)) }}</span>
              </div>
            </TableCell>
            <TableCell class="text-right py-2 px-2">
              <div class="flex flex-col items-end text-xs gap-0.5">
                <span class="text-primary font-medium">{{ formatCurrency(provider.totalCost) }}</span>
                <span
                  v-if="isAdmin && provider.actualCost !== undefined"
                  class="text-muted-foreground text-[10px]"
                >
                  {{ t('usageBreakdown.platformValue', { value: formatCurrency(provider.actualCost) }) }}
                </span>
              </div>
            </TableCell>
            <TableCell class="text-right py-2 px-2">
              <span>{{ formatHitRate(provider.cacheHitRate) }}</span>
            </TableCell>
            <TableCell class="text-right py-2 px-2">
              <span :class="getSuccessRateClass(provider.successRate)">{{ provider.successRate }}%</span>
            </TableCell>
            <TableCell class="text-right text-muted-foreground py-2 px-2">
              {{ provider.avgResponseTime }}
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
import type { ProviderStatsItem } from '../types'

const { t } = useI18n()

defineProps<{
  data: ProviderStatsItem[]
  isAdmin: boolean
}>()

// 成功率样式 - 简化为两种状态
function getSuccessRateClass(rate: number): string {
  if (rate < 90) return 'text-destructive'
  return ''  // 默认颜色
}
</script>
