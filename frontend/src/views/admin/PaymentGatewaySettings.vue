<template>
  <PageContainer>
    <PageHeader
      :title="t('paymentGateway.title')"
      :description="t('paymentGateway.description')"
    >
      <template #actions>
        <div class="flex items-center gap-2">
          <Button
            variant="outline"
            size="sm"
            :disabled="testing || loading"
            @click="testGateway"
          >
            <PlugZap class="mr-2 h-4 w-4" />
            {{ testing ? t('paymentGateway.testing') : t('paymentGateway.test') }}
          </Button>
          <Button
            size="sm"
            :disabled="saving || loading"
            @click="saveConfig"
          >
            <Save class="mr-2 h-4 w-4" />
            {{ saving ? t('paymentGateway.saving') : t('paymentGateway.save') }}
          </Button>
        </div>
      </template>
    </PageHeader>

    <div class="mt-6 space-y-6">
      <div class="flex flex-wrap gap-2">
        <Button
          v-for="provider in providerTabs"
          :key="provider.id"
          :variant="activeProvider === provider.id ? 'default' : 'outline'"
          size="sm"
          :disabled="loading || saving"
          @click="selectProvider(provider.id)"
        >
          {{ provider.name }}
        </Button>
      </div>

      <div
        v-if="loading"
        class="py-16"
      >
        <LoadingState :message="t('paymentGateway.loading')" />
      </div>

      <template v-else>
        <div class="grid grid-cols-1 gap-4 lg:grid-cols-3">
          <Card class="p-5">
            <div class="text-xs uppercase tracking-wider text-muted-foreground">
              {{ t('paymentGateway.gatewayStatus') }}
            </div>
            <div class="mt-3 flex items-center gap-3">
              <Badge :variant="form.enabled ? 'success' : 'secondary'">
                {{ form.enabled ? t('paymentGateway.enabled') : t('paymentGateway.disabled') }}
              </Badge>
              <Switch v-model="form.enabled" />
            </div>
          </Card>
          <Card class="p-5">
            <div class="text-xs uppercase tracking-wider text-muted-foreground">
              {{ t('paymentGateway.secretStatus') }}
            </div>
            <div class="mt-3 flex flex-wrap gap-2">
              <Badge :variant="hasSecret ? 'success' : 'warning'">
                {{ activeProviderMeta.secretLabel }}：{{ hasSecret ? t('paymentGateway.saved') : t('paymentGateway.notSet') }}
              </Badge>
            </div>
          </Card>
          <Card class="p-5">
            <div class="text-xs uppercase tracking-wider text-muted-foreground">
              {{ t('paymentGateway.currency') }}
            </div>
            <div class="mt-2 text-2xl font-semibold tabular-nums">
              1 USD = {{ Number(form.usd_exchange_rate || 0).toFixed(4) }} {{ form.pay_currency }}
            </div>
          </Card>
        </div>

        <CardSection
          :title="`${activeProviderMeta.name} ${t('paymentGateway.paymentInfo')}`"
          :description="activeProviderMeta.description"
        >
          <div class="grid grid-cols-1 gap-5 md:grid-cols-2">
            <div class="space-y-1.5">
              <Label for="gateway-endpoint">{{ activeProviderMeta.endpointLabel }}</Label>
              <Input
                id="gateway-endpoint"
                v-model="form.endpoint_url"
                :placeholder="activeProviderMeta.endpointPlaceholder"
              />
            </div>

            <div class="space-y-1.5">
              <Label for="gateway-callback-base">{{ t('paymentGateway.callbackBase') }}</Label>
              <Input
                id="gateway-callback-base"
                v-model="form.callback_base_url"
                :placeholder="defaultCallbackBaseUrl || 'https://aether.example.com'"
              />
              <p class="text-xs text-muted-foreground">
                {{ t('paymentGateway.callbackHint') }}
              </p>
            </div>

            <div class="space-y-1.5">
              <Label for="gateway-merchant-id">{{ activeProviderMeta.merchantIdLabel }}</Label>
              <Input
                id="gateway-merchant-id"
                v-model="form.merchant_id"
                :placeholder="activeProviderMeta.merchantIdPlaceholder"
                autocomplete="off"
              />
            </div>

            <div class="space-y-1.5">
              <Label for="gateway-merchant-key">
                {{ activeProviderMeta.secretLabel }}
                <span class="text-xs font-normal text-muted-foreground">
                  {{ hasSecret ? t('paymentGateway.keepSecret') : '' }}
                </span>
              </Label>
              <Input
                id="gateway-merchant-key"
                v-model="form.merchant_key"
                masked
                :placeholder="hasSecret ? t('paymentGateway.secretOverridePlaceholder') : activeProviderMeta.secretPlaceholder"
              />
            </div>
          </div>
        </CardSection>

        <CardSection
          :title="t('paymentGateway.billing')"
          :description="t('paymentGateway.billingHint')"
        >
          <div class="grid grid-cols-1 gap-5 md:grid-cols-3">
            <div class="space-y-1.5">
              <Label for="gateway-currency">{{ t('paymentGateway.paymentCurrency') }}</Label>
              <Input
                id="gateway-currency"
                v-model="form.pay_currency"
                maxlength="16"
                placeholder="CNY"
              />
            </div>
            <div class="space-y-1.5">
              <Label for="gateway-rate">{{ t('paymentGateway.usdRate') }}</Label>
              <Input
                id="gateway-rate"
                v-model.number="form.usd_exchange_rate"
                type="number"
                min="0.0001"
                step="0.0001"
              />
            </div>
            <div class="space-y-1.5">
              <Label for="gateway-min">{{ t('paymentGateway.minimum') }}</Label>
              <Input
                id="gateway-min"
                v-model.number="form.min_recharge_usd"
                type="number"
                min="0.01"
                step="0.01"
              />
            </div>
          </div>
        </CardSection>

        <CardSection
          v-if="activeProvider === 'epay'"
          :title="t('paymentGateway.channels')"
          :description="t('paymentGateway.channelsHint')"
        >
          <template #actions>
            <Button
              variant="outline"
              size="sm"
              @click="addChannel"
            >
              <Plus class="mr-2 h-4 w-4" />
                {{ t('paymentGateway.addChannel') }}
            </Button>
          </template>

          <div class="space-y-3">
            <div
              v-for="(channel, index) in form.channels"
              :key="index"
              class="grid grid-cols-1 gap-3 rounded-lg border border-border/60 bg-muted/20 p-3 md:grid-cols-[1fr_1fr_auto]"
            >
              <div class="space-y-1.5">
                <Label :for="`epay-channel-${index}`">{{ t('paymentGateway.channel') }}</Label>
                <Input
                  :id="`epay-channel-${index}`"
                  v-model="channel.channel"
                  placeholder="alipay"
                />
              </div>
              <div class="space-y-1.5">
                <Label :for="`epay-channel-name-${index}`">{{ t('paymentGateway.displayName') }}</Label>
                <Input
                  :id="`epay-channel-name-${index}`"
                  v-model="channel.display_name"
                  :placeholder="t('paymentGateway.alipayDisplayName')"
                />
              </div>
              <div class="flex items-end">
                <Button
                  variant="ghost"
                  size="icon"
                  :title="t('paymentGateway.remove')"
                  :disabled="form.channels.length <= 1"
                  @click="removeChannel(index)"
                >
                  <Trash2 class="h-4 w-4" />
                </Button>
              </div>
            </div>
          </div>
        </CardSection>

        <p
          v-if="updatedAtText"
          class="text-xs text-muted-foreground"
        >
          {{ t('paymentGateway.updated') }}：{{ updatedAtText }}
        </p>
      </template>
    </div>
  </PageContainer>
</template>

<script setup lang="ts">
import { computed, onMounted, reactive, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { PlugZap, Plus, Save, Trash2 } from 'lucide-vue-next'
import {
  dodopayGatewayApi,
  epayGatewayApi,
  type EpayChannelConfig,
  type PaymentGatewayProvider,
  type UpdatePaymentGatewayConfigRequest,
} from '@/api/billing'
import {
  Badge,
  Button,
  Card,
  Input,
  Label,
  Switch,
} from '@/components/ui'
import { LoadingState } from '@/components/common'
import { CardSection, PageContainer, PageHeader } from '@/components/layout'
import { useToast } from '@/composables/useToast'
import { parseApiError } from '@/utils/errorParser'
import { log } from '@/utils/logger'

const { success, error: showError } = useToast()

const loading = ref(true)
const saving = ref(false)
const testing = ref(false)
const { t, locale } = useI18n()
const hasSecret = ref(false)
const updatedAt = ref<number | null>(null)
const activeProvider = ref<PaymentGatewayProvider>('epay')

const providerTabs = computed<Array<{
  id: PaymentGatewayProvider
  name: string
  endpointLabel: string
  endpointPlaceholder: string
  merchantIdLabel: string
  merchantIdPlaceholder: string
  secretLabel: string
  secretPlaceholder: string
  description: string
}>>(() => [
  {
    id: 'epay',
    name: t('paymentGateway.epayName'),
    endpointLabel: t('paymentGateway.epayEndpoint'),
    endpointPlaceholder: 'https://pay.example.com/submit.php',
    merchantIdLabel: t('paymentGateway.merchantId'),
    merchantIdPlaceholder: '1000',
    secretLabel: t('paymentGateway.merchantSecret'),
    secretPlaceholder: t('paymentGateway.merchantSecretPlaceholder'),
    description: t('paymentGateway.epayDescription'),
  },
  {
    id: 'dodopay',
    name: 'DoDoPay',
    endpointLabel: t('paymentGateway.dodopayEndpoint'),
    endpointPlaceholder: 'https://pay.dodododo.org',
    merchantIdLabel: 'App ID',
    merchantIdPlaceholder: 'app_xxxxx',
    secretLabel: 'App Secret',
    secretPlaceholder: t('paymentGateway.dodopaySecretPlaceholder'),
    description: t('paymentGateway.dodopayDescription'),
  },
])

const activeProviderMeta = computed(
  () => providerTabs.value.find((provider) => provider.id === activeProvider.value) || providerTabs.value[0]
)

const form = reactive({
  enabled: false,
  endpoint_url: '',
  callback_base_url: '',
  merchant_id: '',
  merchant_key: '',
  pay_currency: 'CNY',
  usd_exchange_rate: 7.2,
  min_recharge_usd: 1,
  channels: [
    { channel: 'alipay', display_name: t('paymentGateway.alipay') },
    { channel: 'wxpay', display_name: t('paymentGateway.wechatPay') },
  ] as EpayChannelConfig[],
})

function activeGatewayApi() {
  return activeProvider.value === 'dodopay' ? dodopayGatewayApi : epayGatewayApi
}

const updatedAtText = computed(() => {
  if (!updatedAt.value) return ''
  return new Date(updatedAt.value * 1000).toLocaleString(locale.value === 'en-US' ? 'en-US' : 'zh-CN')
})

const defaultCallbackBaseUrl = computed(() => {
  if (typeof window === 'undefined') return ''
  return window.location.origin
})

onMounted(() => {
  void loadConfig()
})

async function selectProvider(provider: PaymentGatewayProvider) {
  if (activeProvider.value === provider) return
  activeProvider.value = provider
  await loadConfig()
}

async function loadConfig() {
  loading.value = true
  try {
    const config = await activeGatewayApi().get()
    form.enabled = config.enabled
    form.endpoint_url = config.endpoint_url || ''
    form.callback_base_url = config.callback_base_url || ''
    form.merchant_id = config.merchant_id || ''
    form.merchant_key = ''
    form.pay_currency = config.pay_currency || 'CNY'
    form.usd_exchange_rate = Number(config.usd_exchange_rate || 7.2)
    form.min_recharge_usd = Number(config.min_recharge_usd || 1)
    form.channels = config.channels?.length
      ? config.channels.map((item) => ({ ...item }))
      : activeProvider.value === 'epay'
        ? [
            { channel: 'alipay', display_name: t('paymentGateway.alipay') },
            { channel: 'wxpay', display_name: t('paymentGateway.wechatPay') },
          ]
        : []
    hasSecret.value = config.has_secret
    updatedAt.value = config.updated_at ?? null
  } catch (err) {
    log.error(`Failed to load ${activeProviderMeta.value.name} configuration:`, err)
    showError(parseApiError(err, t('paymentGateway.loadFailed', { provider: activeProviderMeta.value.name })))
  } finally {
    loading.value = false
  }
}

function normalizeChannels(): EpayChannelConfig[] {
  return form.channels
    .map((item) => ({
      channel: item.channel.trim(),
      display_name: item.display_name.trim(),
    }))
    .filter((item) => item.channel && item.display_name)
}

function validateForm(): string | null {
  if (!form.endpoint_url.trim()) return t('paymentGateway.requiredField', { field: activeProviderMeta.value.endpointLabel })
  if (!form.merchant_id.trim()) return t('paymentGateway.requiredField', { field: activeProviderMeta.value.merchantIdLabel })
  if (!hasSecret.value && !form.merchant_key.trim()) {
    return t('paymentGateway.secretRequired', { field: activeProviderMeta.value.secretLabel })
  }
  if (!form.pay_currency.trim()) return t('paymentGateway.currencyRequired')
  if (!Number.isFinite(Number(form.usd_exchange_rate)) || Number(form.usd_exchange_rate) <= 0) {
    return t('paymentGateway.ratePositive')
  }
  if (!Number.isFinite(Number(form.min_recharge_usd)) || Number(form.min_recharge_usd) <= 0) {
    return t('paymentGateway.minimumPositive')
  }
  if (activeProvider.value === 'epay' && normalizeChannels().length === 0) return t('paymentGateway.channelRequired')
  return null
}

async function saveConfig() {
  const validationError = validateForm()
  if (validationError) {
    showError(validationError)
    return
  }

  saving.value = true
  try {
    const callbackBaseUrl = form.callback_base_url.trim()
    const payload: UpdatePaymentGatewayConfigRequest = {
      enabled: form.enabled,
      endpoint_url: form.endpoint_url.trim(),
      callback_base_url: callbackBaseUrl || null,
      merchant_id: form.merchant_id.trim(),
      pay_currency: form.pay_currency.trim().toUpperCase(),
      usd_exchange_rate: Number(form.usd_exchange_rate),
      min_recharge_usd: Number(form.min_recharge_usd),
      channels: activeProvider.value === 'epay' ? normalizeChannels() : [],
      ...(form.merchant_key.trim() ? { merchant_key: form.merchant_key.trim() } : {}),
    }
    const config = await activeGatewayApi().update(payload)
    hasSecret.value = config.has_secret
    updatedAt.value = config.updated_at ?? null
    form.callback_base_url = config.callback_base_url || ''
    form.merchant_key = ''
    success(t('paymentGateway.savedProvider', { provider: activeProviderMeta.value.name }))
  } catch (err) {
    log.error(`Failed to save ${activeProviderMeta.value.name} configuration:`, err)
    showError(parseApiError(err, t('paymentGateway.saveFailedProvider', { provider: activeProviderMeta.value.name })))
  } finally {
    saving.value = false
  }
}

async function testGateway() {
  testing.value = true
  try {
    await activeGatewayApi().test()
    success(t('paymentGateway.testPassed', { provider: activeProviderMeta.value.name }))
  } catch (err) {
    log.error(`Failed to test ${activeProviderMeta.value.name} configuration:`, err)
    showError(parseApiError(err, t('paymentGateway.testFailed', { provider: activeProviderMeta.value.name })))
  } finally {
    testing.value = false
  }
}

function addChannel() {
  form.channels.push({ channel: '', display_name: '' })
}

function removeChannel(index: number) {
  if (form.channels.length <= 1) return
  form.channels.splice(index, 1)
}
</script>
