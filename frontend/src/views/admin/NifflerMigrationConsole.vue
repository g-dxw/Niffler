<template>
  <PageContainer>
    <PageHeader
      :title="t('migrationConsole.title')"
      :description="t('migrationConsole.description')"
      :icon="Gauge"
    >
      <template #actions>
        <Button
          variant="outline"
          class="admin-filter-action"
          :disabled="pageLoading"
          @click="refreshAll"
        >
          <RefreshCw
            class="mr-2 h-4 w-4"
            :class="{ 'animate-spin': pageLoading }"
          />
          {{ t('migrationConsole.refresh') }}
        </Button>
      </template>
    </PageHeader>

    <div class="mt-6 space-y-5">
      <Card class="overflow-hidden">
        <div class="flex flex-col gap-4 border-b border-border/70 p-5 lg:flex-row lg:items-start lg:justify-between">
          <div>
            <h2 class="text-lg font-semibold">
              {{ t('migrationConsole.rollout') }}
            </h2>
            <p class="mt-1 text-sm text-muted-foreground">
              {{ t('migrationConsole.rolloutHint') }}
            </p>
          </div>
          <Badge variant="secondary">
            {{ runtimeRolloutSettings.length }} {{ t('migrationConsole.records') }}
          </Badge>
        </div>

        <div class="grid gap-5 p-5 xl:grid-cols-[minmax(0,1fr)_380px]">
          <section class="space-y-4">
            <div class="grid gap-3 sm:grid-cols-2 xl:grid-cols-3">
              <label
                v-for="item in runtimeRolloutFlagOptions"
                :key="item.key"
                class="flex items-start gap-3 rounded-lg border border-border/70 p-3"
              >
                <Switch v-model="runtimeRolloutForm[item.key]" />
                <span>
                  <span class="block text-sm font-medium">{{ item.label }}</span>
                  <span class="block text-xs text-muted-foreground">{{ item.description }}</span>
                </span>
              </label>
              <label class="flex items-start gap-3 rounded-lg border border-border/70 p-3">
                <Switch v-model="runtimeRolloutForm.is_active" />
                <span>
                  <span class="block text-sm font-medium">{{ t('migrationConsole.enabled') }}</span>
                  <span class="block text-xs text-muted-foreground">{{ t('migrationConsole.enabledHint') }}</span>
                </span>
              </label>
            </div>

            <p
              v-if="runtimeRolloutError"
              class="rounded-md border border-destructive/20 bg-destructive/5 px-4 py-3 text-sm text-destructive"
            >
              {{ runtimeRolloutError }}
            </p>

            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>{{ t('migrationConsole.target') }}</TableHead>
                  <TableHead>{{ t('migrationConsole.switch') }}</TableHead>
                  <TableHead>{{ t('migrationConsole.status') }}</TableHead>
                  <TableHead>{{ t('migrationConsole.updatedAt') }}</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                <TableRow
                  v-if="runtimeRolloutSettings.length === 0"
                >
                  <TableCell
                    colspan="4"
                    class="py-10 text-center text-sm text-muted-foreground"
                  >
                    {{ t('migrationConsole.empty') }}
                  </TableCell>
                </TableRow>
                <TableRow
                  v-for="setting in runtimeRolloutSettings"
                  :key="setting.id"
                >
                  <TableCell>
                    <div class="font-medium">
                      {{ runtimeRolloutTargetLabel(setting) }}
                    </div>
                    <div class="mt-1 text-xs text-muted-foreground">
                      {{ runtimeRolloutTargetScopeLabel(setting.target_scope) }}
                    </div>
                  </TableCell>
                  <TableCell>
                    <div
                      v-if="runtimeRolloutEnabledLabels(setting).length > 0"
                      class="flex flex-wrap gap-2"
                    >
                      <Badge
                        v-for="label in runtimeRolloutEnabledLabels(setting)"
                        :key="label"
                        variant="outline"
                      >
                        {{ label }}
                      </Badge>
                    </div>
                    <span
                      v-else
                      class="text-sm text-muted-foreground"
                    >
                      {{ t('migrationConsole.disabled') }}
                    </span>
                  </TableCell>
                  <TableCell>
                    <Badge :variant="setting.is_active ? 'outline' : 'secondary'">
                      {{ setting.is_active ? t('migrationConsole.enabled') : t('migrationConsole.disabled') }}
                    </Badge>
                  </TableCell>
                  <TableCell>
                    {{ formatNifflerUnixMs(setting.updated_at_unix_ms) }}
                  </TableCell>
                </TableRow>
              </TableBody>
            </Table>
          </section>

          <aside class="space-y-4 rounded-lg border border-border/70 bg-muted/20 p-4">
            <div class="space-y-2">
              <Label for="rollout-product-plan">{{ t('migrationConsole.productPlan') }}</Label>
              <Select v-model="selectedProductPlanId">
                <SelectTrigger id="rollout-product-plan">
                  <SelectValue :placeholder="t('migrationConsole.selectPlan')" />
                </SelectTrigger>
                <SelectContent :search-placeholder="t('migrationConsole.searchPlan')">
                  <SelectItem
                    v-for="plan in productPlans"
                    :key="plan.id"
                    :value="plan.id"
                  >
                    {{ plan.display_name }}
                  </SelectItem>
                </SelectContent>
              </Select>
              <Button
                class="admin-entry-action w-full"
                :disabled="!selectedProductPlan || !selectedProductPlan.is_active || savingRuntimeRolloutTargetKey === runtimeRolloutTargetKey('product_plan', selectedProductPlanId)"
                @click="saveSelectedProductPlanRuntimeRollout"
              >
                {{ t('migrationConsole.registerPlan') }}
              </Button>
            </div>

            <div class="space-y-2">
              <Label for="rollout-api-key">{{ t('migrationConsole.apiKey') }}</Label>
              <Select v-model="selectedRuntimeRolloutApiKeyId">
                <SelectTrigger id="rollout-api-key">
                  <SelectValue :placeholder="t('migrationConsole.selectKey')" />
                </SelectTrigger>
                <SelectContent :search-placeholder="t('migrationConsole.searchKey')">
                  <SelectItem
                    v-for="apiKey in standaloneApiKeys"
                    :key="apiKey.id"
                    :value="apiKey.id"
                    :text-value="`${formatApiKeyName(apiKey)} ${formatApiKeyOwner(apiKey)}`"
                  >
                    {{ formatApiKeyName(apiKey) }} · {{ formatApiKeyOwner(apiKey) }}
                  </SelectItem>
                </SelectContent>
              </Select>
              <div class="grid grid-cols-2 gap-2">
                <Button
                  variant="outline"
                  class="admin-entry-action"
                  :disabled="!selectedRuntimeRolloutApiKey || !selectedRuntimeRolloutApiKey.is_active || savingRuntimeRolloutTargetKey === runtimeRolloutTargetKey('api_key', selectedRuntimeRolloutApiKeyId)"
                  @click="saveSelectedApiKeyRuntimeRollout"
                >
                  {{ t('migrationConsole.registerKey') }}
                </Button>
                <Button
                  variant="outline"
                  class="admin-entry-action"
                  :disabled="!selectedRuntimeRolloutApiKeyId || runtimeRolloutPreviewLoading"
                  @click="loadRuntimeRolloutPreview"
                >
                  {{ t('migrationConsole.preview') }}
                </Button>
              </div>
            </div>

            <p
              v-if="runtimeRolloutPreviewError"
              class="rounded-md border border-destructive/20 bg-destructive/5 px-3 py-2 text-xs text-destructive"
            >
              {{ runtimeRolloutPreviewError }}
            </p>

            <div
              v-if="runtimeRolloutPreview"
              class="space-y-3 rounded-lg border border-border/70 bg-background p-3"
            >
              <Badge :variant="runtimeRolloutPreview.decision.is_active ? 'outline' : 'secondary'">
                {{ runtimeRolloutPreview.decision.is_active ? t('migrationConsole.previewEnabled') : t('migrationConsole.previewDisabled') }}
              </Badge>
              <p class="text-xs text-muted-foreground">
                {{ runtimeRolloutPreview.decision.reason }}
              </p>
              <div class="space-y-1 text-xs text-muted-foreground">
                <p>{{ t('migrationConsole.keyValue', { value: runtimeRolloutPreview.api_key.name || runtimeRolloutPreview.api_key.id }) }}</p>
                <p>{{ t('migrationConsole.policyValue', { value: runtimeRolloutPreview.product_plan?.display_name || runtimeRolloutPreview.product_plan?.id || t('migrationConsole.unbound') }) }}</p>
              </div>
            </div>
          </aside>
        </div>
      </Card>

      <Card class="overflow-hidden">
        <Tabs
          default-value="routing"
          class="p-5"
        >
          <TabsList class="tabs-button-list grid w-full max-w-4xl grid-cols-5">
            <TabsTrigger value="routing">
              {{ t('migrationConsole.route') }}
            </TabsTrigger>
            <TabsTrigger value="settlement">
              {{ t('migrationConsole.settlement') }}
            </TabsTrigger>
            <TabsTrigger value="reservation">
              {{ t('migrationConsole.reservation') }}
            </TabsTrigger>
            <TabsTrigger value="referral">
              {{ t('migrationConsole.referral') }}
            </TabsTrigger>
            <TabsTrigger value="consistency">
              {{ t('migrationConsole.consistency') }}
            </TabsTrigger>
          </TabsList>

          <TabsContent
            value="routing"
            class="mt-5"
          >
            <ObservedTable
              :title="t('migrationConsole.routeRecords')"
              :loading="routeAttemptLoading"
              :error="routeAttemptError"
              :empty="routeAttempts.length === 0"
              :empty-text="t('migrationConsole.noRouteRecords')"
              @refresh="loadRouteAttempts"
            >
              <Table v-if="routeAttempts.length > 0">
                <TableHeader>
                  <TableRow>
                    <TableHead>{{ t('migrationConsole.request') }}</TableHead>
                    <TableHead>{{ t('migrationConsole.serviceAccount') }}</TableHead>
                    <TableHead>{{ t('migrationConsole.result') }}</TableHead>
                    <TableHead>{{ t('migrationConsole.upstream') }}</TableHead>
                    <TableHead>{{ t('migrationConsole.time') }}</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  <TableRow
                    v-for="attempt in routeAttempts"
                    :key="attempt.id"
                  >
                    <TableCell>
                      <div class="font-mono text-xs">
                        {{ attempt.request_id }}
                      </div>
                      <div class="mt-1 text-xs text-muted-foreground">
                        {{ attempt.model_name }}
                      </div>
                    </TableCell>
                    <TableCell>
                      <div class="text-sm">
                        {{ routeAttemptServiceLabel(attempt) }}
                      </div>
                      <div class="mt-1 text-xs text-muted-foreground">
                        {{ routeAttemptAccountLabel(attempt) }}
                      </div>
                    </TableCell>
                    <TableCell>
                      <Badge :variant="routeAttemptStatusVariant(attempt.status)">
                        {{ routeAttemptStatusLabel(attempt.status) }}
                      </Badge>
                      <div
                        v-if="attempt.skip_reason"
                        class="mt-1 text-xs text-muted-foreground"
                      >
                        {{ attempt.skip_reason }}
                      </div>
                    </TableCell>
                    <TableCell>
                      <div class="text-xs">
                        {{ t('migrationConsole.statusCodeValue', { value: attempt.upstream_status_code ?? t('migrationConsole.none') }) }}
                      </div>
                      <div class="mt-1 text-xs text-muted-foreground">
                        {{ t('migrationConsole.latencyValue', { value: formatLatencyMs(attempt.latency_ms) }) }}
                      </div>
                    </TableCell>
                    <TableCell>
                      {{ formatNifflerUnixMs(attempt.created_at_unix_ms) }}
                    </TableCell>
                  </TableRow>
                </TableBody>
              </Table>
            </ObservedTable>
          </TabsContent>

          <TabsContent
            value="settlement"
            class="mt-5"
          >
            <ObservedTable
              :title="t('migrationConsole.settlementSnapshot')"
              :loading="settlementSnapshotLoading"
              :error="settlementSnapshotError"
              :empty="settlementSnapshots.length === 0"
              :empty-text="t('migrationConsole.noSettlementSnapshots')"
              @refresh="loadSettlementSnapshots"
            >
              <Table v-if="settlementSnapshots.length > 0">
                <TableHeader>
                  <TableRow>
                    <TableHead>{{ t('migrationConsole.request') }}</TableHead>
                    <TableHead>{{ t('migrationConsole.serviceAccount') }}</TableHead>
                    <TableHead>{{ t('migrationConsole.amount') }}</TableHead>
                    <TableHead>{{ t('migrationConsole.time') }}</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  <TableRow
                    v-for="snapshot in settlementSnapshots"
                    :key="snapshot.id"
                  >
                    <TableCell>
                      <div class="font-mono text-xs">
                        {{ snapshot.request_id }}
                      </div>
                      <div class="mt-1 text-xs text-muted-foreground">
                        {{ snapshot.requested_model_name }}
                      </div>
                    </TableCell>
                    <TableCell>
                      <div class="text-sm">
                        {{ settlementSnapshotServiceLabel(snapshot) }}
                      </div>
                      <div class="mt-1 text-xs text-muted-foreground">
                        {{ settlementSnapshotAccountLabel(snapshot) }}
                      </div>
                    </TableCell>
                    <TableCell>
                      <div class="text-sm font-medium">
                        {{ t('migrationConsole.walletValue', { value: formatUsdAmount(snapshot.wallet_charge_usd) }) }}
                      </div>
                      <div class="mt-1 text-xs text-muted-foreground">
                        {{ t('migrationConsole.planValue', { value: formatUsdAmount(snapshot.entitlement_charge_usd) }) }}
                      </div>
                      <div class="text-xs text-muted-foreground">
                        {{ t('migrationConsole.costValue', { value: formatUsdAmount(snapshot.upstream_cost_usd) }) }}
                      </div>
                    </TableCell>
                    <TableCell>
                      {{ formatNifflerUnixMs(snapshot.created_at_unix_ms) }}
                    </TableCell>
                  </TableRow>
                </TableBody>
              </Table>
            </ObservedTable>
          </TabsContent>

          <TabsContent
            value="reservation"
            class="mt-5"
          >
            <ObservedTable
              :title="t('migrationConsole.billingReservation')"
              :loading="billingReservationLoading"
              :error="billingReservationError"
              :empty="billingReservations.length === 0"
              :empty-text="t('migrationConsole.noReservations')"
              @refresh="loadBillingReservations"
            >
              <Table v-if="billingReservations.length > 0">
                <TableHeader>
                  <TableRow>
                    <TableHead>{{ t('migrationConsole.request') }}</TableHead>
                    <TableHead>{{ t('migrationConsole.status') }}</TableHead>
                    <TableHead>{{ t('migrationConsole.amount') }}</TableHead>
                    <TableHead>{{ t('migrationConsole.time') }}</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  <TableRow
                    v-for="reservation in billingReservations"
                    :key="reservation.id"
                  >
                    <TableCell>
                      <div class="font-mono text-xs">
                        {{ reservation.request_id }}
                      </div>
                      <div class="mt-1 text-xs text-muted-foreground">
                        {{ t('migrationConsole.userValue', { value: reservation.user_id || t('migrationConsole.unknown') }) }}
                      </div>
                    </TableCell>
                    <TableCell>
                      <Badge :variant="reconciliationStatusVariant(reservation.status)">
                        {{ billingReservationStatusLabel(reservation.status) }}
                      </Badge>
                      <div
                        v-if="reservation.release_reason"
                        class="mt-1 text-xs text-muted-foreground"
                      >
                        {{ reservation.release_reason }}
                      </div>
                    </TableCell>
                    <TableCell>
                      <div class="text-sm font-medium">
                        {{ formatUsdAmount(reservation.reserved_total_usd) }}
                      </div>
                      <div class="mt-1 text-xs text-muted-foreground">
                        {{ t('migrationConsole.walletValue', { value: formatUsdAmount(reservation.wallet_reserved_usd) }) }}
                      </div>
                      <div class="text-xs text-muted-foreground">
                        {{ t('migrationConsole.planValue', { value: formatUsdAmount(reservation.entitlement_reserved_usd) }) }}
                      </div>
                    </TableCell>
                    <TableCell>
                      {{ formatNifflerUnixMs(reservation.reserved_at_unix_ms) }}
                    </TableCell>
                  </TableRow>
                </TableBody>
              </Table>
            </ObservedTable>
          </TabsContent>

          <TabsContent
            value="referral"
            class="mt-5"
          >
            <ObservedTable
              :title="t('migrationConsole.referralLedger')"
              :loading="referralRewardLedgerLoading"
              :error="referralRewardLedgerError"
              :empty="referralRewardLedger.length === 0"
              :empty-text="t('migrationConsole.noReferralLedger')"
              @refresh="loadReferralRewardLedger"
            >
              <Table v-if="referralRewardLedger.length > 0">
                <TableHeader>
                  <TableRow>
                    <TableHead>{{ t('migrationConsole.order') }}</TableHead>
                    <TableHead>{{ t('migrationConsole.status') }}</TableHead>
                    <TableHead>{{ t('migrationConsole.amount') }}</TableHead>
                    <TableHead class="text-right">
                      {{ t('migrationConsole.actions') }}
                    </TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  <TableRow
                    v-for="ledger in referralRewardLedger"
                    :key="ledger.id"
                  >
                    <TableCell>
                      <div class="font-mono text-xs">
                        {{ ledger.order_id }}
                      </div>
                      <div class="mt-1 text-xs text-muted-foreground">
                        {{ t('migrationConsole.inviterValue', { value: ledger.inviter_user_id }) }}
                      </div>
                    </TableCell>
                    <TableCell>
                      <Badge :variant="reconciliationStatusVariant(ledger.status)">
                        {{ referralRewardLedgerStatusLabel(ledger.status) }}
                      </Badge>
                      <div
                        v-if="ledger.failure_reason"
                        class="mt-1 text-xs text-muted-foreground"
                      >
                        {{ ledger.failure_reason }}
                      </div>
                    </TableCell>
                    <TableCell>
                      {{ formatUsdAmount(ledger.reward_amount_usd) }}
                    </TableCell>
                    <TableCell class="text-right">
                      <div
                        v-if="canMutateReferralLedger(ledger)"
                        class="flex justify-end gap-2"
                      >
                        <Button
                          size="sm"
                          variant="outline"
                          class="admin-entry-action"
                          :disabled="referralRewardLedgerMutationId === ledger.id"
                          @click="retryReferralRewardLedger(ledger)"
                        >
                          {{ t('migrationConsole.retry') }}
                        </Button>
                        <Button
                          size="sm"
                          variant="ghost"
                          class="admin-entry-action"
                          :disabled="referralRewardLedgerMutationId === ledger.id"
                          @click="cancelReferralRewardLedger(ledger)"
                        >
                          {{ t('migrationConsole.cancel') }}
                        </Button>
                      </div>
                      <span
                        v-else
                        class="text-xs text-muted-foreground"
                      >
                        -
                      </span>
                    </TableCell>
                  </TableRow>
                </TableBody>
              </Table>
            </ObservedTable>
          </TabsContent>

          <TabsContent
            value="consistency"
            class="mt-5"
          >
            <ObservedTable
              :title="t('migrationConsole.consistencyBoard')"
              :loading="consistencyCheckLoading"
              :error="consistencyCheckError"
              :empty="consistencyChecks.length === 0"
              :empty-text="t('migrationConsole.noConsistency')"
              @refresh="loadConsistencyChecks"
            >
              <Table v-if="consistencyChecks.length > 0">
                <TableHeader>
                  <TableRow>
                    <TableHead>{{ t('migrationConsole.request') }}</TableHead>
                    <TableHead>{{ t('migrationConsole.status') }}</TableHead>
                    <TableHead>{{ t('migrationConsole.amount') }}</TableHead>
                    <TableHead>{{ t('migrationConsole.reservationRoute') }}</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  <TableRow
                    v-for="item in consistencyChecks"
                    :key="item.request_id"
                  >
                    <TableCell>
                      <div class="font-mono text-xs">
                        {{ item.request_id }}
                      </div>
                      <div class="mt-1 text-xs text-muted-foreground">
                        {{ t('migrationConsole.policyValue', { value: consistencyProductPlanLabel(item) }) }}
                      </div>
                    </TableCell>
                    <TableCell>
                      <Badge :variant="consistencyStatusVariant(item.consistency_status)">
                        {{ consistencyStatusLabel(item.consistency_status) }}
                      </Badge>
                      <div
                        v-if="item.issue_codes.length > 0"
                        class="mt-2 flex flex-wrap gap-1"
                      >
                        <Badge
                          v-for="issue in item.issue_codes"
                          :key="issue"
                          variant="secondary"
                        >
                          {{ consistencyIssueLabel(issue) }}
                        </Badge>
                      </div>
                    </TableCell>
                    <TableCell>
                      <div class="text-sm font-medium">
                        {{ t('migrationConsole.totalValue', { value: formatUsdAmount(item.niffler_total_charge_usd) }) }}
                      </div>
                      <div class="mt-1 text-xs text-muted-foreground">
                        {{ t('migrationConsole.walletValue', { value: formatUsdAmount(item.niffler_wallet_charge_usd) }) }}
                      </div>
                      <div class="text-xs text-muted-foreground">
                        {{ t('migrationConsole.planValue', { value: formatUsdAmount(item.niffler_entitlement_charge_usd) }) }}
                      </div>
                    </TableCell>
                    <TableCell>
                      <div class="text-xs">
                        {{ t('migrationConsole.reservationValue', { value: consistencyReservationLabel(item) }) }}
                      </div>
                      <div class="mt-1 text-xs text-muted-foreground">
                        {{ t('migrationConsole.routeSuccess', { success: item.successful_route_attempt_count, total: item.route_attempt_count }) }}
                      </div>
                    </TableCell>
                  </TableRow>
                </TableBody>
              </Table>
            </ObservedTable>
          </TabsContent>
        </Tabs>
      </Card>
    </div>
  </PageContainer>
</template>

<script setup lang="ts">
import { computed, defineComponent, h, onMounted, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'

const { t } = useI18n()
import {
  Gauge,
  Loader2,
  RefreshCw,
} from 'lucide-vue-next'
import { PageContainer, PageHeader } from '@/components/layout'
import {
  Badge,
  Button,
  Card,
  Label,
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
  Switch,
  Tabs,
  TabsContent,
  TabsList,
  TabsTrigger,
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui'
import { adminApi, type AdminApiKey } from '@/api/admin'
import {
  cancelNifflerReferralRewardLedger,
  getNifflerRuntimeRolloutPreview,
  listNifflerApiKeyProductPlanBindings,
  listNifflerBillingReservations,
  listNifflerConsistencyChecks,
  listNifflerProductPlans,
  listNifflerReferralRewardLedger,
  listNifflerRouteAttempts,
  listNifflerRuntimeRolloutSettings,
  listNifflerSettlementSnapshots,
  retryNifflerReferralRewardLedger,
  upsertNifflerRuntimeRolloutSetting,
  type NifflerApiKeyProductPlanBinding,
  type NifflerBillingReservation,
  type NifflerBillingReservationStatus,
  type NifflerConsistencyCheck,
  type NifflerProductPlan,
  type NifflerReferralRewardLedger,
  type NifflerReferralRewardLedgerStatus,
  type NifflerRouteAttempt,
  type NifflerRuntimeRolloutPreview,
  type NifflerRuntimeRolloutSetting,
  type NifflerRuntimeRolloutTargetScope,
  type NifflerSettlementSnapshot,
  type UpsertNifflerRuntimeRolloutSettingPayload,
} from '@/api/niffler-core'
import { useToast } from '@/composables/useToast'
import { extractErrorMessage } from '@/utils/error'
import { formatNifflerUnixMs } from './niffler-upstream-account-ui'

type RuntimeRolloutFlagKey = Exclude<
  keyof Pick<
    UpsertNifflerRuntimeRolloutSettingPayload,
    | 'enable_new_routing'
    | 'enable_settlement_snapshot'
    | 'enable_error_return_rules'
    | 'enable_billing_reservation'
    | 'enable_referral_ledger'
  >,
  undefined
>

type RuntimeRolloutForm = Required<
  Pick<
    UpsertNifflerRuntimeRolloutSettingPayload,
    | 'enable_new_routing'
    | 'enable_settlement_snapshot'
    | 'enable_error_return_rules'
    | 'enable_billing_reservation'
    | 'enable_referral_ledger'
    | 'is_active'
  >
>

const ObservedTable = defineComponent({
  name: 'ObservedTable',
  props: {
    title: { type: String, required: true },
    loading: { type: Boolean, required: true },
    error: { type: String, default: '' },
    empty: { type: Boolean, required: true },
    emptyText: { type: String, required: true },
  },
  emits: ['refresh'],
  setup(props, { emit, slots }) {
    return () => h('section', { class: 'rounded-lg border border-border/70' }, [
      h('div', { class: 'flex items-center justify-between border-b border-border/70 px-4 py-3' }, [
        h('h2', { class: 'text-sm font-semibold' }, props.title),
        h(Button, {
          variant: 'outline',
          size: 'sm',
          class: 'admin-filter-action',
          disabled: props.loading,
          onClick: () => emit('refresh'),
}, () => props.loading ? t('migrationConsole.refreshing') : t('migrationConsole.refresh')),
      ]),
      props.error
        ? h('p', { class: 'border-b border-destructive/20 bg-destructive/5 px-4 py-3 text-sm text-destructive' }, props.error)
        : null,
      props.loading
        ? h('div', { class: 'flex items-center justify-center py-12 text-sm text-muted-foreground' }, [
          h(Loader2, { class: 'mr-2 h-5 w-5 animate-spin' }),
 t('migrationConsole.loading'),
        ])
        : props.empty
          ? h('div', { class: 'py-12 text-center text-sm text-muted-foreground' }, props.emptyText)
          : slots.default?.(),
    ])
  },
})

const { success, error: showError } = useToast()

const runtimeRolloutFlagOptions: Array<{
  key: RuntimeRolloutFlagKey
  label: string
  description: string
}> = [
{ key: 'enable_new_routing', label: t('migrationConsole.newRouting'), description: t('migrationConsole.newRoutingHint') },
{ key: 'enable_settlement_snapshot', label: t('migrationConsole.settlementSnapshot'), description: t('migrationConsole.settlementSnapshotHint') },
{ key: 'enable_error_return_rules', label: t('migrationConsole.errorRules'), description: t('migrationConsole.errorRulesHint') },
{ key: 'enable_billing_reservation', label: t('migrationConsole.walletReservation'), description: t('migrationConsole.walletReservationHint') },
{ key: 'enable_referral_ledger', label: t('migrationConsole.referralLedger'), description: t('migrationConsole.referralLedgerHint') },
]

const defaultRuntimeRolloutForm = (): RuntimeRolloutForm => ({
  enable_new_routing: true,
  enable_settlement_snapshot: true,
  enable_error_return_rules: true,
  enable_billing_reservation: false,
  enable_referral_ledger: false,
  is_active: true,
})

const productPlans = ref<NifflerProductPlan[]>([])
const apiKeys = ref<AdminApiKey[]>([])
const apiKeyProductPlanBindings = ref<NifflerApiKeyProductPlanBinding[]>([])
const runtimeRolloutSettings = ref<NifflerRuntimeRolloutSetting[]>([])
const runtimeRolloutPreview = ref<NifflerRuntimeRolloutPreview | null>(null)
const billingReservations = ref<NifflerBillingReservation[]>([])
const settlementSnapshots = ref<NifflerSettlementSnapshot[]>([])
const referralRewardLedger = ref<NifflerReferralRewardLedger[]>([])
const routeAttempts = ref<NifflerRouteAttempt[]>([])
const consistencyChecks = ref<NifflerConsistencyCheck[]>([])
const seedDataLoading = ref(false)
const runtimeRolloutLoading = ref(false)
const runtimeRolloutPreviewLoading = ref(false)
const billingReservationLoading = ref(false)
const settlementSnapshotLoading = ref(false)
const referralRewardLedgerLoading = ref(false)
const routeAttemptLoading = ref(false)
const consistencyCheckLoading = ref(false)
const runtimeRolloutError = ref('')
const runtimeRolloutPreviewError = ref('')
const billingReservationError = ref('')
const settlementSnapshotError = ref('')
const referralRewardLedgerError = ref('')
const routeAttemptError = ref('')
const consistencyCheckError = ref('')
const selectedProductPlanId = ref('')
const selectedRuntimeRolloutApiKeyId = ref('')
const savingRuntimeRolloutTargetKey = ref<string | null>(null)
const referralRewardLedgerMutationId = ref<string | null>(null)
const runtimeRolloutForm = ref<RuntimeRolloutForm>(defaultRuntimeRolloutForm())

const usdFormatter = new Intl.NumberFormat('en-US', {
  style: 'currency',
  currency: 'USD',
  minimumFractionDigits: 4,
  maximumFractionDigits: 6,
})

const pageLoading = computed(() =>
  seedDataLoading.value
  || runtimeRolloutLoading.value
  || runtimeRolloutPreviewLoading.value
  || billingReservationLoading.value
  || settlementSnapshotLoading.value
  || referralRewardLedgerLoading.value
  || routeAttemptLoading.value
  || consistencyCheckLoading.value
)

const selectedProductPlan = computed(() =>
  productPlans.value.find(plan => plan.id === selectedProductPlanId.value) ?? null
)

const standaloneApiKeys = computed(() =>
  apiKeys.value.filter(apiKey => apiKey.is_standalone)
)

const selectedRuntimeRolloutApiKey = computed(() =>
  standaloneApiKeys.value.find(apiKey => apiKey.id === selectedRuntimeRolloutApiKeyId.value) ?? null
)

const productPlanNameById = computed(() =>
  new Map(productPlans.value.map(plan => [plan.id, plan.display_name]))
)

const apiKeyNameById = computed(() =>
  new Map(apiKeys.value.map(apiKey => [apiKey.id, formatApiKeyName(apiKey)]))
)

watch(selectedRuntimeRolloutApiKeyId, () => {
  runtimeRolloutPreview.value = null
  runtimeRolloutPreviewError.value = ''
})

async function refreshAll() {
  await Promise.all([
    loadSeedData(),
    loadRuntimeRolloutSettings(),
    loadBillingReservations(),
    loadSettlementSnapshots(),
    loadReferralRewardLedger(),
    loadRouteAttempts(),
    loadConsistencyChecks(),
  ])
}

async function loadSeedData() {
  seedDataLoading.value = true
  try {
    const [productPlanResponse, apiKeyResponse, bindingResponse] = await Promise.all([
      listNifflerProductPlans({ include_inactive: true, limit: 100 }),
      adminApi.getAllApiKeys({ skip: 0, limit: 200, include_usage_summary: false }),
      listNifflerApiKeyProductPlanBindings({ offset: 0, limit: 200 }),
    ])
    productPlans.value = productPlanResponse.items
    apiKeys.value = apiKeyResponse.api_keys
    apiKeyProductPlanBindings.value = bindingResponse.items
    selectedProductPlanId.value ||= productPlans.value[0]?.id ?? ''
    selectedRuntimeRolloutApiKeyId.value ||= standaloneApiKeys.value[0]?.id ?? ''
  } catch (err) {
    showError(extractErrorMessage(err, t('migrationConsole.loadTargetsFailed')))
  } finally {
    seedDataLoading.value = false
  }
}

async function loadRuntimeRolloutSettings() {
  runtimeRolloutLoading.value = true
  runtimeRolloutError.value = ''
  try {
    const response = await listNifflerRuntimeRolloutSettings({
      include_inactive: true,
      limit: 100,
    })
    runtimeRolloutSettings.value = response.items
  } catch (err) {
  runtimeRolloutError.value = extractErrorMessage(err, t('migrationConsole.loadRolloutFailed'))
    showError(runtimeRolloutError.value)
  } finally {
    runtimeRolloutLoading.value = false
  }
}

async function loadRuntimeRolloutPreview() {
  if (!selectedRuntimeRolloutApiKeyId.value) {
    showError(t('migrationConsole.selectKeyFirst'))
    return
  }
  runtimeRolloutPreviewLoading.value = true
  runtimeRolloutPreviewError.value = ''
  try {
    runtimeRolloutPreview.value = await getNifflerRuntimeRolloutPreview(selectedRuntimeRolloutApiKeyId.value)
  } catch (err) {
    runtimeRolloutPreview.value = null
    runtimeRolloutPreviewError.value = extractErrorMessage(err, t('migrationConsole.loadPreviewFailed'))
    showError(runtimeRolloutPreviewError.value)
  } finally {
    runtimeRolloutPreviewLoading.value = false
  }
}

async function saveSelectedProductPlanRuntimeRollout() {
  if (!selectedProductPlan.value) {
    showError(t('migrationConsole.selectPolicyFirst'))
    return
  }
  if (!selectedProductPlan.value.is_active) {
    showError(t('migrationConsole.activePolicyOnly'))
    return
  }
  await saveRuntimeRolloutSetting('product_plan', selectedProductPlan.value.id)
}

async function saveSelectedApiKeyRuntimeRollout() {
  if (!selectedRuntimeRolloutApiKey.value) {
    showError(t('migrationConsole.selectKeyFirst'))
    return
  }
  if (!selectedRuntimeRolloutApiKey.value.is_active) {
    showError(t('migrationConsole.activeKeyOnly'))
    return
  }
  await saveRuntimeRolloutSetting('api_key', selectedRuntimeRolloutApiKey.value.id)
}

async function saveRuntimeRolloutSetting(
  targetScope: NifflerRuntimeRolloutTargetScope,
  targetId: string
) {
  const targetKey = runtimeRolloutTargetKey(targetScope, targetId)
  savingRuntimeRolloutTargetKey.value = targetKey
  try {
    await upsertNifflerRuntimeRolloutSetting({
      target_scope: targetScope,
      target_id: targetId,
      enable_new_routing: runtimeRolloutForm.value.enable_new_routing,
      enable_settlement_snapshot: runtimeRolloutForm.value.enable_settlement_snapshot,
      enable_error_return_rules: runtimeRolloutForm.value.enable_error_return_rules,
      enable_billing_reservation: runtimeRolloutForm.value.enable_billing_reservation,
      enable_referral_ledger: runtimeRolloutForm.value.enable_referral_ledger,
      is_active: runtimeRolloutForm.value.is_active,
    })
    success(t('migrationConsole.rolloutSaved'))
    await loadRuntimeRolloutSettings()
    if (selectedRuntimeRolloutApiKeyId.value) {
      await loadRuntimeRolloutPreview()
    }
  } catch (err) {
    showError(extractErrorMessage(err, t('migrationConsole.saveRolloutFailed')))
  } finally {
    savingRuntimeRolloutTargetKey.value = null
  }
}

async function loadBillingReservations() {
  billingReservationLoading.value = true
  billingReservationError.value = ''
  try {
    const response = await listNifflerBillingReservations({ offset: 0, limit: 50 })
    billingReservations.value = response.items
  } catch (err) {
    billingReservationError.value = extractErrorMessage(err, t('migrationConsole.loadReservationFailed'))
    showError(billingReservationError.value)
  } finally {
    billingReservationLoading.value = false
  }
}

async function loadSettlementSnapshots() {
  settlementSnapshotLoading.value = true
  settlementSnapshotError.value = ''
  try {
    const response = await listNifflerSettlementSnapshots({ offset: 0, limit: 50 })
    settlementSnapshots.value = response.items
  } catch (err) {
    settlementSnapshotError.value = extractErrorMessage(err, t('migrationConsole.loadSettlementFailed'))
    showError(settlementSnapshotError.value)
  } finally {
    settlementSnapshotLoading.value = false
  }
}

async function loadReferralRewardLedger() {
  referralRewardLedgerLoading.value = true
  referralRewardLedgerError.value = ''
  try {
    const response = await listNifflerReferralRewardLedger({ offset: 0, limit: 50 })
    referralRewardLedger.value = response.items
  } catch (err) {
    referralRewardLedgerError.value = extractErrorMessage(err, t('migrationConsole.loadReferralFailed'))
    showError(referralRewardLedgerError.value)
  } finally {
    referralRewardLedgerLoading.value = false
  }
}

function canMutateReferralLedger(ledger: NifflerReferralRewardLedger): boolean {
  return ledger.status === 'pending' || ledger.status === 'failed'
}

async function retryReferralRewardLedger(ledger: NifflerReferralRewardLedger) {
  if (!canMutateReferralLedger(ledger)) return
  referralRewardLedgerMutationId.value = ledger.id
  try {
    await retryNifflerReferralRewardLedger(ledger.id)
    success(t('migrationConsole.referralRetried'))
    await loadReferralRewardLedger()
  } catch (err) {
    showError(extractErrorMessage(err, t('migrationConsole.retryReferralFailed')))
  } finally {
    referralRewardLedgerMutationId.value = null
  }
}

async function cancelReferralRewardLedger(ledger: NifflerReferralRewardLedger) {
  if (!canMutateReferralLedger(ledger)) return
  referralRewardLedgerMutationId.value = ledger.id
  try {
    await cancelNifflerReferralRewardLedger(ledger.id)
    success(t('migrationConsole.referralCancelled'))
    await loadReferralRewardLedger()
  } catch (err) {
    showError(extractErrorMessage(err, t('migrationConsole.cancelReferralFailed')))
  } finally {
    referralRewardLedgerMutationId.value = null
  }
}

async function loadRouteAttempts() {
  routeAttemptLoading.value = true
  routeAttemptError.value = ''
  try {
    const response = await listNifflerRouteAttempts({ offset: 0, limit: 50 })
    routeAttempts.value = response.items
  } catch (err) {
    routeAttemptError.value = extractErrorMessage(err, t('migrationConsole.loadRoutesFailed'))
    showError(routeAttemptError.value)
  } finally {
    routeAttemptLoading.value = false
  }
}

async function loadConsistencyChecks() {
  consistencyCheckLoading.value = true
  consistencyCheckError.value = ''
  try {
    const response = await listNifflerConsistencyChecks({ offset: 0, limit: 50 })
    consistencyChecks.value = response.items
  } catch (err) {
    consistencyCheckError.value = extractErrorMessage(err, t('migrationConsole.loadConsistencyFailed'))
    showError(consistencyCheckError.value)
  } finally {
    consistencyCheckLoading.value = false
  }
}

function runtimeRolloutTargetKey(scope: NifflerRuntimeRolloutTargetScope, targetId: string): string {
  return `${scope}:${targetId}`
}

function runtimeRolloutTargetScopeLabel(scope: NifflerRuntimeRolloutTargetScope): string {
  return scope === 'api_key' ? t('migrationConsole.apiKey') : t('migrationConsole.productPlan')
}

function runtimeRolloutTargetLabel(setting: NifflerRuntimeRolloutSetting): string {
  if (setting.target_scope === 'product_plan') {
    return productPlanNameById.value.get(setting.target_id) || setting.target_id
  }
  return apiKeyNameById.value.get(setting.target_id) || setting.target_id
}

function runtimeRolloutEnabledLabels(
  flags: Pick<
    NifflerRuntimeRolloutSetting | RuntimeRolloutForm,
    | 'enable_new_routing'
    | 'enable_settlement_snapshot'
    | 'enable_error_return_rules'
    | 'enable_billing_reservation'
    | 'enable_referral_ledger'
  >
): string[] {
  const labels: string[] = []
  if (flags.enable_new_routing) labels.push(t('migrationConsole.newRouting'))
  if (flags.enable_settlement_snapshot) labels.push(t('migrationConsole.settlementSnapshot'))
  if (flags.enable_error_return_rules) labels.push(t('migrationConsole.errorRules'))
  if (flags.enable_billing_reservation) labels.push(t('migrationConsole.walletReservation'))
  if (flags.enable_referral_ledger) labels.push(t('migrationConsole.referralLedger'))
  return labels
}

function formatApiKeyName(apiKey: AdminApiKey): string {
  return apiKey.name?.trim() || apiKey.key_display || apiKey.id
}

function formatApiKeyOwner(apiKey: AdminApiKey): string {
  return apiKey.user_email || apiKey.username || apiKey.user_id
}

function formatUsdAmount(value: number): string {
  if (!Number.isFinite(value)) return '$0.0000'
  return usdFormatter.format(value)
}

function reconciliationStatusVariant(status: string): 'default' | 'secondary' | 'destructive' | 'outline' {
  if (status === 'failed' || status === 'manual_review') return 'destructive'
  if (status === 'active' || status === 'pending') return 'outline'
  if (status === 'settled' || status === 'paid') return 'default'
  return 'secondary'
}

function billingReservationStatusLabel(status: NifflerBillingReservationStatus): string {
  const labels: Record<NifflerBillingReservationStatus, string> = {
    active: t('migrationConsole.reservationActive'),
    settled: t('migrationConsole.reservationSettled'),
    released: t('migrationConsole.reservationReleased'),
    expired: t('migrationConsole.reservationExpired'),
    manual_review: t('migrationConsole.reservationManual'),
  }
  return labels[status] ?? status
}

function referralRewardLedgerStatusLabel(status: NifflerReferralRewardLedgerStatus): string {
  const labels: Record<NifflerReferralRewardLedgerStatus, string> = {
    pending: t('migrationConsole.referralPending'),
    paid: t('migrationConsole.referralPaid'),
    failed: t('migrationConsole.failed'),
    cancelled: t('migrationConsole.cancelled'),
  }
  return labels[status] ?? status
}

function routeAttemptStatusLabel(status: string): string {
  const labels: Record<string, string> = {
    success: t('migrationConsole.success'),
    skipped: t('migrationConsole.skipped'),
    cancelled: t('migrationConsole.cancelled'),
    failed: t('migrationConsole.failed'),
  }
  return labels[status] ?? status
}

function routeAttemptStatusVariant(status: string): 'default' | 'secondary' | 'destructive' | 'outline' {
  if (status === 'failed') return 'destructive'
  if (status === 'success') return 'default'
  if (status === 'skipped') return 'outline'
  return 'secondary'
}

function routeAttemptServiceLabel(attempt: NifflerRouteAttempt): string {
  return attempt.upstream_service_name || attempt.upstream_service_id || t('migrationConsole.unrecordedService')
}

function routeAttemptAccountLabel(attempt: NifflerRouteAttempt): string {
  const contacts = [attempt.upstream_account_email, attempt.upstream_account_phone].filter(Boolean)
  if (contacts.length > 0) return contacts.join(' / ')
  return attempt.upstream_account_display_name || attempt.upstream_account_id || t('migrationConsole.unrecordedAccount')
}

function settlementSnapshotServiceLabel(snapshot: NifflerSettlementSnapshot): string {
  return snapshot.upstream_service_name || snapshot.upstream_service_id || t('migrationConsole.unrecordedService')
}

function settlementSnapshotAccountLabel(snapshot: NifflerSettlementSnapshot): string {
  const contacts = [snapshot.upstream_account_email, snapshot.upstream_account_phone].filter(Boolean)
  if (contacts.length > 0) return contacts.join(' / ')
  return snapshot.upstream_account_display_name || snapshot.upstream_account_id || t('migrationConsole.unrecordedAccount')
}

function formatLatencyMs(value?: number | null): string {
  if (value === null || value === undefined || !Number.isFinite(value)) return t('migrationConsole.none')
  return `${Math.round(value)} ms`
}

function consistencyStatusLabel(status: string): string {
  return status === 'ok' ? t('migrationConsole.consistent') : t('migrationConsole.needsReview')
}

function consistencyStatusVariant(status: string): 'default' | 'secondary' | 'destructive' | 'outline' {
  return status === 'ok' ? 'default' : 'destructive'
}

function consistencyProductPlanLabel(item: NifflerConsistencyCheck): string {
  return item.product_plan_name || item.product_plan_id || t('migrationConsole.unbound')
}

function consistencyReservationLabel(item: NifflerConsistencyCheck): string {
  if (!item.reservation_status) return t('migrationConsole.missing')
  const reason = item.reservation_release_reason ? ` / ${item.reservation_release_reason}` : ''
  return `${billingReservationStatusLabel(item.reservation_status)}${reason}`
}

function consistencyIssueLabel(issue: string): string {
  const labels: Record<string, string> = {
    missing_legacy_usage: t('migrationConsole.issueMissingLegacyUsage'),
    missing_legacy_settlement: t('migrationConsole.issueMissingLegacySettlement'),
    legacy_not_settled: t('migrationConsole.issueLegacyNotSettled'),
    missing_legacy_wallet_charge: t('migrationConsole.issueMissingWalletCharge'),
    wallet_charge_mismatch: t('migrationConsole.issueWalletMismatch'),
    entitlement_charge_mismatch: t('migrationConsole.issuePlanMismatch'),
    total_charge_mismatch: t('migrationConsole.issueTotalMismatch'),
    missing_billing_reservation: t('migrationConsole.issueMissingReservation'),
    reservation_not_finalized: t('migrationConsole.issueReservationNotFinalized'),
    reservation_manual_review: t('migrationConsole.issueReservationManual'),
    missing_route_attempt: t('migrationConsole.issueMissingRoute'),
  }
  return labels[issue] ?? issue
}

onMounted(() => {
  void refreshAll()
})
</script>
