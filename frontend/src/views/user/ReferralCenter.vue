<template>
  <div class="space-y-6 pb-8">
    <div>
      <h1 class="text-2xl font-semibold text-foreground">
        {{ t('referral.title') }}
      </h1>
      <p class="mt-1 text-sm text-muted-foreground">
        {{ t('referral.description') }}
      </p>
    </div>

    <div
      v-if="loading"
      class="rounded-lg border border-border bg-card p-6 text-sm text-muted-foreground"
    >
      {{ t('referral.loading') }}
    </div>

    <template v-else-if="dashboard">
      <div class="grid grid-cols-1 gap-4 md:grid-cols-3">
        <Card class="p-5">
          <p class="text-xs text-muted-foreground">
            {{ t('referral.total') }}
          </p>
          <p class="mt-2 text-2xl font-semibold">
            {{ dashboard.summary.total_invites }}
          </p>
        </Card>
        <Card class="p-5">
          <p class="text-xs text-muted-foreground">
            {{ t('referral.valid') }}
          </p>
          <p class="mt-2 text-2xl font-semibold">
            {{ dashboard.summary.effective_invites }}
          </p>
        </Card>
        <Card class="p-5">
          <p class="text-xs text-muted-foreground">
            {{ t('referral.paid') }}
          </p>
          <p class="mt-2 text-2xl font-semibold">
            {{ formatUsd(dashboard.summary.paid_reward_usd) }}
          </p>
        </Card>
      </div>

      <Card class="p-5">
        <div class="grid grid-cols-1 gap-4 lg:grid-cols-[240px_1fr]">
          <div>
            <Label class="text-xs text-muted-foreground">
              {{ t('referral.code') }}
            </Label>
            <div class="mt-2 flex items-center gap-2">
              <code class="rounded-lg border border-border bg-muted px-3 py-2 font-mono text-sm">
                {{ dashboard.invite_code }}
              </code>
              <Button
                type="button"
                variant="outline"
                size="sm"
                @click="copyToClipboard(dashboard.invite_code)"
              >
                <Copy class="mr-2 h-4 w-4" />
                {{ t('referral.copy') }}
              </Button>
            </div>
          </div>

          <div>
            <Label class="text-xs text-muted-foreground">
              {{ t('referral.link') }}
            </Label>
            <div class="mt-2 flex min-w-0 items-center gap-2">
              <Input
                :model-value="dashboard.invitation_link"
                readonly
                class="min-w-0"
              />
              <Button
                type="button"
                variant="outline"
                size="sm"
                class="shrink-0"
                @click="copyToClipboard(dashboard.invitation_link)"
              >
                <Copy class="mr-2 h-4 w-4" />
                {{ t('referral.copy') }}
              </Button>
            </div>
          </div>
        </div>
      </Card>

      <div class="grid grid-cols-1 gap-4 md:grid-cols-2">
        <Card class="p-5">
          <p class="text-xs text-muted-foreground">
            {{ t('referral.pending') }}
          </p>
          <p class="mt-2 text-xl font-semibold">
            {{ formatUsd(dashboard.summary.pending_reward_usd) }}
          </p>
        </Card>
        <Card class="p-5">
          <p class="text-xs text-muted-foreground">
            {{ t('referral.reversed') }}
          </p>
          <p class="mt-2 text-xl font-semibold">
            {{ formatUsd(dashboard.summary.reversed_reward_usd) }}
          </p>
        </Card>
      </div>
    </template>

    <div
      v-else
      class="rounded-lg border border-border bg-card p-6 text-sm text-muted-foreground"
    >
      {{ t('referral.unavailable') }}
    </div>
  </div>
</template>

<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { Copy } from 'lucide-vue-next'
import { referralApi, type ReferralDashboardResponse } from '@/api/referrals'
import { Button, Card, Input, Label } from '@/components/ui'
import { useClipboard } from '@/composables/useClipboard'
import { useToast } from '@/composables/useToast'

const dashboard = ref<ReferralDashboardResponse | null>(null)
const loading = ref(false)
const { copyToClipboard } = useClipboard()
const { t } = useI18n()
const { error: showError } = useToast()

function formatUsd(value: number): string {
  return `$${Number(value || 0).toFixed(2)}`
}

async function loadReferralDashboard() {
  loading.value = true
  try {
    dashboard.value = await referralApi.getMyReferral()
  } catch {
    dashboard.value = null
    showError(t('referralCenter.loadFailed'))
  } finally {
    loading.value = false
  }
}

onMounted(() => {
  void loadReferralDashboard()
})
</script>
