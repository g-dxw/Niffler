<template>
  <div class="space-y-6 pb-8">
    <div class="flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
      <div>
        <h1 class="text-2xl font-semibold text-foreground">
          {{ t('referralAdmin.title') }}
        </h1>
        <p class="mt-1 text-sm text-muted-foreground">
          {{ t('referralAdmin.description') }}
        </p>
      </div>
      <Button
        variant="outline"
        :disabled="loading"
        @click="loadAll"
      >
        <RefreshCw
          class="mr-2 h-4 w-4"
          :class="{ 'animate-spin': loading }"
        />
        {{ t('referralAdmin.refresh') }}
      </Button>
    </div>

    <div class="grid grid-cols-1 gap-4 md:grid-cols-5">
      <Card
        v-for="item in statCards"
        :key="item.label"
        class="p-4"
      >
        <p class="text-xs text-muted-foreground">
          {{ item.label }}
        </p>
        <p class="mt-2 text-xl font-semibold">
          {{ item.value }}
        </p>
      </Card>
    </div>

    <Card class="overflow-hidden">
      <div class="border-b border-border px-5 py-4">
        <h2 class="text-base font-semibold">
              {{ t('referralAdmin.relationships') }}
        </h2>
      </div>
      <div class="grid grid-cols-1 gap-3 border-b border-border/70 p-4 md:grid-cols-5">
        <Input
          v-model="relationshipFilters.inviter"
              :placeholder="t('referralAdmin.inviter')"
        />
        <Input
          v-model="relationshipFilters.invitee"
              :placeholder="t('referralAdmin.invitee')"
        />
        <Input
          v-model="relationshipFilters.invite_code"
              :placeholder="t('referralAdmin.code')"
        />
        <Select v-model="firstPaidFilter">
          <SelectTrigger>
            <SelectValue :placeholder="t('referralAdmin.firstPaid')" />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="all">
              {{ t('referralAdmin.all') }}
            </SelectItem>
            <SelectItem value="true">
              {{ t('referralAdmin.paid') }}
            </SelectItem>
            <SelectItem value="false">
              {{ t('referralAdmin.unpaid') }}
            </SelectItem>
          </SelectContent>
        </Select>
        <Button
          type="button"
          @click="loadRelationships"
        >
              {{ t('referralAdmin.query') }}
        </Button>
      </div>

      <div class="overflow-x-auto">
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead>{{ t('referralAdmin.inviter') }}</TableHead>
              <TableHead>{{ t('referralAdmin.invitee') }}</TableHead>
              <TableHead>{{ t('referralAdmin.code') }}</TableHead>
              <TableHead>{{ t('referralAdmin.bindingTime') }}</TableHead>
              <TableHead>{{ t('referralAdmin.firstPaid') }}</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            <TableRow
              v-for="item in relationships"
              :key="item.id"
            >
              <TableCell>{{ item.inviter_username || item.inviter_user_id }}</TableCell>
              <TableCell>{{ item.invitee_username || item.invitee_user_id }}</TableCell>
              <TableCell class="font-mono text-xs">
                {{ item.invite_code_snapshot }}
              </TableCell>
              <TableCell>{{ formatUnix(item.created_at_unix_secs) }}</TableCell>
              <TableCell>
                <Badge :variant="item.first_paid_order_id ? 'success' : 'secondary'">
                  {{ item.first_paid_order_id ? t('referralAdmin.paid') : t('referralAdmin.unpaid') }}
                </Badge>
              </TableCell>
            </TableRow>
            <TableRow v-if="relationships.length === 0">
              <TableCell
                colspan="5"
                class="py-8 text-center text-sm text-muted-foreground"
              >
                {{ t('referralAdmin.emptyRelationships') }}
              </TableCell>
            </TableRow>
          </TableBody>
        </Table>
      </div>
    </Card>

    <Card class="overflow-hidden">
      <div class="border-b border-border px-5 py-4">
        <h2 class="text-base font-semibold">
          {{ t('referralAdmin.records') }}
        </h2>
      </div>
      <div class="grid grid-cols-1 gap-3 border-b border-border/70 p-4 md:grid-cols-5">
        <Input
          v-model="rewardFilters.order_id"
              :placeholder="t('referralAdmin.order')"
        />
        <Select v-model="rewardFilters.reward_type">
          <SelectTrigger>
            <SelectValue :placeholder="t('referralAdmin.type')" />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="all">
              {{ t('referralAdmin.all') }}
            </SelectItem>
            <SelectItem value="percent">
              {{ t('referralAdmin.ratio') }}
            </SelectItem>
            <SelectItem value="headcount">
              {{ t('referralAdmin.headcount') }}
            </SelectItem>
          </SelectContent>
        </Select>
        <Select v-model="rewardFilters.status">
          <SelectTrigger>
            <SelectValue :placeholder="t('referralAdmin.status')" />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="all">
              {{ t('referralAdmin.all') }}
            </SelectItem>
            <SelectItem value="pending">
              {{ t('referralAdmin.pending') }}
            </SelectItem>
            <SelectItem value="failed">
              {{ t('referralAdmin.failed') }}
            </SelectItem>
            <SelectItem value="applied">
              {{ t('referralAdmin.issued') }}
            </SelectItem>
            <SelectItem value="voided">
              {{ t('referralAdmin.voided') }}
            </SelectItem>
            <SelectItem value="reversed">
              {{ t('referralAdmin.reversed') }}
            </SelectItem>
          </SelectContent>
        </Select>
        <Button
          type="button"
          class="md:col-start-5"
          @click="loadRewards"
        >
              {{ t('referralAdmin.query') }}
        </Button>
      </div>

      <div class="overflow-x-auto">
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead>{{ t('referralAdmin.type') }}</TableHead>
              <TableHead>{{ t('referralAdmin.sourceOrder') }}</TableHead>
              <TableHead>{{ t('referralAdmin.amount') }}</TableHead>
              <TableHead>{{ t('referralAdmin.status') }}</TableHead>
              <TableHead>{{ t('referralAdmin.reversal') }}</TableHead>
              <TableHead>{{ t('referralAdmin.createdAt') }}</TableHead>
              <TableHead class="text-right">
                {{ t('referralAdmin.actions') }}
              </TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            <TableRow
              v-for="item in rewards"
              :key="item.id"
            >
              <TableCell>{{ getRewardTypeLabel(item.reward_type) }}</TableCell>
              <TableCell class="font-mono text-xs">
                {{ item.source_order_id || '-' }}
              </TableCell>
              <TableCell>{{ formatUsd(item.amount_usd) }}</TableCell>
              <TableCell>
                <Badge :variant="getRewardStatusVariant(item.status)">
                  {{ getRewardStatusLabel(item.status) }}
                </Badge>
              </TableCell>
              <TableCell>
                {{ formatUsd(item.reversed_amount_usd) }}
                <span
                  v-if="item.pending_reversal_amount_usd > 0"
                  class="text-xs text-amber-600 dark:text-amber-400"
                >
                  / {{ t('referralAdmin.pendingReversal', { amount: formatUsd(item.pending_reversal_amount_usd) }) }}
                </span>
              </TableCell>
              <TableCell>{{ formatUnix(item.created_at_unix_secs) }}</TableCell>
              <TableCell class="text-right">
                <div class="flex justify-end gap-2">
                  <Button
                    v-if="item.status === 'failed'"
                    variant="outline"
                    size="sm"
                    :disabled="mutatingRewardId === item.id"
                    @click="retryReward(item)"
                  >
                    {{ t('referralAdmin.retry') }}
                  </Button>
                  <Button
                    v-if="item.status === 'failed' || item.status === 'pending'"
                    variant="ghost"
                    size="sm"
                    :disabled="mutatingRewardId === item.id"
                    @click="voidReward(item)"
                  >
                    {{ t('referralAdmin.void') }}
                  </Button>
                </div>
              </TableCell>
            </TableRow>
            <TableRow v-if="rewards.length === 0">
              <TableCell
                colspan="7"
                class="py-8 text-center text-sm text-muted-foreground"
              >
                {{ t('referralAdmin.emptyRewards') }}
              </TableCell>
            </TableRow>
          </TableBody>
        </Table>
      </div>
    </Card>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useI18n } from 'vue-i18n'

const { t } = useI18n()
import { RefreshCw } from 'lucide-vue-next'
import {
  referralApi,
  type ReferralRelationshipRecord,
  type ReferralRewardRecord,
  type ReferralSummary
} from '@/api/referrals'
import {
  Badge,
  Button,
  Card,
  Input,
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
import { useToast } from '@/composables/useToast'

const relationships = ref<ReferralRelationshipRecord[]>([])
const rewards = ref<ReferralRewardRecord[]>([])
const stats = ref<ReferralSummary>({
  total_invites: 0,
  effective_invites: 0,
  paid_reward_usd: 0,
  pending_reward_usd: 0,
  reversed_reward_usd: 0
})
const loading = ref(false)
const mutatingRewardId = ref<string | null>(null)
const relationshipFilters = ref({
  inviter: '',
  invitee: '',
  invite_code: ''
})
const firstPaidFilter = ref('all')
const rewardFilters = ref({
  order_id: '',
  reward_type: 'all',
  status: 'all'
})
const { success, error: showError } = useToast()

const statCards = computed(() => [
  { label: t('referralAdmin.totalInvites'), value: stats.value.total_invites },
  { label: t('referralAdmin.effectiveInvites'), value: stats.value.effective_invites },
  { label: t('referralAdmin.paidRewards'), value: formatUsd(stats.value.paid_reward_usd) },
  { label: t('referralAdmin.pendingRewards'), value: formatUsd(stats.value.pending_reward_usd) },
  { label: t('referralAdmin.reversedRewards'), value: formatUsd(stats.value.reversed_reward_usd) },
])

function formatUsd(value: number): string {
  return `$${Number(value || 0).toFixed(2)}`
}

function formatUnix(value?: number | null): string {
  if (!value) return '-'
  return new Date(value * 1000).toLocaleString('zh-CN')
}

function getRewardTypeLabel(value: string): string {
  if (value === 'percent') return t('referralAdmin.ratio')
  if (value === 'headcount') return t('referralAdmin.headcount')
  return value
}

function getRewardStatusLabel(value: string): string {
  switch (value) {
    case 'applied':
      return t('referralAdmin.issued')
    case 'pending':
      return t('referralAdmin.pending')
    case 'failed':
      return t('referralAdmin.failed')
    case 'voided':
      return t('referralAdmin.voided')
    case 'reversed':
      return t('referralAdmin.reversed')
    default:
      return value
  }
}

function getRewardStatusVariant(value: string): 'default' | 'secondary' | 'destructive' | 'outline' | 'success' | 'warning' | 'dark' {
  switch (value) {
    case 'applied':
      return 'success'
    case 'failed':
      return 'destructive'
    case 'pending':
      return 'warning'
    case 'voided':
      return 'secondary'
    default:
      return 'outline'
  }
}

async function loadRelationships() {
  const firstPaid =
    firstPaidFilter.value === 'true' ? true : firstPaidFilter.value === 'false' ? false : null
  const response = await referralApi.getAdminReferrals({
    ...relationshipFilters.value,
    first_paid: firstPaid,
    limit: 100,
    offset: 0
  })
  relationships.value = response.items
  stats.value = response.stats
}

async function loadRewards() {
  const response = await referralApi.getAdminReferralRewards({
    order_id: rewardFilters.value.order_id,
    reward_type: rewardFilters.value.reward_type === 'all' ? undefined : rewardFilters.value.reward_type,
    status: rewardFilters.value.status === 'all' ? undefined : rewardFilters.value.status,
    limit: 100,
    offset: 0
  })
  rewards.value = response.items
  stats.value = response.stats
}

async function loadAll() {
  loading.value = true
  try {
    await Promise.all([loadRelationships(), loadRewards()])
  } catch {
    showError(t('referralAdmin.loadFailed'))
  } finally {
    loading.value = false
  }
}

async function retryReward(item: ReferralRewardRecord) {
  mutatingRewardId.value = item.id
  try {
    const response = await referralApi.retryReferralReward(item.id, t('referralAdmin.adminRetryReason'))
    replaceReward(response.reward)
    success(t('referralAdmin.retrySuccess'))
  } catch {
    showError(t('referralAdmin.retryFailed'))
  } finally {
    mutatingRewardId.value = null
  }
}

async function voidReward(item: ReferralRewardRecord) {
  mutatingRewardId.value = item.id
  try {
    const response = await referralApi.voidReferralReward(item.id, t('referralAdmin.adminVoidReason'))
    replaceReward(response.reward)
    success(t('referralAdmin.voidSuccess'))
  } catch {
    showError(t('referralAdmin.voidFailed'))
  } finally {
    mutatingRewardId.value = null
  }
}

function replaceReward(updated: ReferralRewardRecord) {
  rewards.value = rewards.value.map(item => item.id === updated.id ? updated : item)
}

onMounted(() => {
  void loadAll()
})
</script>
