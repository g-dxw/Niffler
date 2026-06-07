<template>
  <PageContainer>
    <PageHeader
      title="上游接入"
      description="登记新 Niffler 模型里的上游服务和账号。当前只写新表，不影响线上调度。"
      :icon="Server"
    >
      <template #actions>
        <Button
          variant="outline"
          :disabled="serviceLoading || accountLoading"
          @click="refreshAll"
        >
          <RefreshCw
            class="mr-2 h-4 w-4"
            :class="{ 'animate-spin': serviceLoading || accountLoading }"
          />
          刷新
        </Button>
      </template>
    </PageHeader>

    <div class="mt-6 space-y-5">
      <Card class="overflow-hidden border-amber-200/80 bg-amber-50/70 dark:border-amber-900/50 dark:bg-amber-950/20">
        <div class="flex flex-col gap-3 p-5 md:flex-row md:items-start">
          <AlertTriangle class="mt-0.5 h-5 w-5 shrink-0 text-amber-600" />
          <div class="space-y-1">
            <p class="font-medium text-amber-900 dark:text-amber-200">
              这是新模型入口，不会改动当前线上请求。
            </p>
            <p class="text-sm text-amber-800/80 dark:text-amber-100/75">
              本页只写入新表：上游服务、上游账号、服务能力。账号不保存真实密钥内容，也不会进入旧 Provider、号池、计费或结算链路。
            </p>
          </div>
        </div>
      </Card>

      <div class="grid gap-5 xl:grid-cols-[minmax(0,1.08fr)_minmax(360px,0.92fr)]">
        <Card class="overflow-hidden">
          <div class="flex flex-col gap-4 border-b border-border/70 p-5 lg:flex-row lg:items-center lg:justify-between">
            <div>
              <h2 class="text-lg font-semibold">
                上游服务
              </h2>
              <p class="mt-1 text-sm text-muted-foreground">
                例如 Codex、Claude、OpenAI、第三方 OpenAI 兼容服务。
              </p>
            </div>
            <div class="flex flex-col gap-2 sm:flex-row sm:items-center">
              <Input
                v-model="serviceSearch"
                class="h-9 sm:w-64"
                placeholder="搜索服务名称"
                @keyup.enter="loadServices"
              />
              <Button
                variant="outline"
                class="h-9"
                :disabled="serviceLoading"
                @click="loadServices"
              >
                <Search class="mr-2 h-4 w-4" />
                搜索
              </Button>
              <Button
                class="h-9"
                @click="serviceDialogOpen = true"
              >
                <Plus class="mr-2 h-4 w-4" />
                新增服务
              </Button>
            </div>
          </div>

          <div
            v-if="serviceError"
            class="border-b border-destructive/20 bg-destructive/5 px-5 py-3 text-sm text-destructive"
          >
            {{ serviceError }}
          </div>

          <div
            v-if="serviceLoading && services.length === 0"
            class="flex items-center justify-center py-16 text-sm text-muted-foreground"
          >
            <Loader2 class="mr-2 h-5 w-5 animate-spin" />
            正在读取上游服务...
          </div>

          <div
            v-else-if="services.length === 0"
            class="py-16 text-center"
          >
            <Server class="mx-auto h-10 w-10 text-muted-foreground/50" />
            <p class="mt-3 font-medium">
              还没有上游服务
            </p>
            <p class="mt-1 text-sm text-muted-foreground">
              先登记服务，再在服务下登记账号。
            </p>
          </div>

          <Table v-else>
            <TableHeader>
              <TableRow>
                <TableHead>服务名称</TableHead>
                <TableHead>类型</TableHead>
                <TableHead>协议</TableHead>
                <TableHead>成本倍率</TableHead>
                <TableHead>状态</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              <TableRow
                v-for="service in services"
                :key="service.id"
                class="cursor-pointer"
                :class="selectedServiceId === service.id ? 'bg-primary/5' : 'hover:bg-muted/40'"
                @click="selectService(service.id)"
              >
                <TableCell>
                  <div class="font-medium">
                    {{ service.display_name }}
                  </div>
                  <div
                    v-if="service.base_url"
                    class="mt-1 max-w-[360px] truncate text-xs text-muted-foreground"
                  >
                    {{ service.base_url }}
                  </div>
                </TableCell>
                <TableCell>
                  {{ service.service_kind }}
                </TableCell>
                <TableCell>
                  {{ service.default_api_format || '-' }}
                </TableCell>
                <TableCell>
                  {{ formatMultiplier(service.cost_multiplier) }}
                </TableCell>
                <TableCell>
                  <Badge :variant="service.is_active ? 'outline' : 'secondary'">
                    {{ service.is_active ? '启用' : '停用' }}
                  </Badge>
                </TableCell>
              </TableRow>
            </TableBody>
          </Table>
        </Card>

        <Card class="overflow-hidden">
          <div class="flex flex-col gap-4 border-b border-border/70 p-5 sm:flex-row sm:items-start sm:justify-between">
            <div>
              <h2 class="text-lg font-semibold">
                上游账号
              </h2>
              <p class="mt-1 text-sm text-muted-foreground">
                {{ selectedService ? `当前服务：${selectedService.display_name}` : '先选择左侧服务' }}
              </p>
            </div>
            <Button
              class="h-9"
              :disabled="!selectedService"
              @click="accountDialogOpen = true"
            >
              <Plus class="mr-2 h-4 w-4" />
              新增账号
            </Button>
          </div>

          <div
            v-if="accountError"
            class="border-b border-destructive/20 bg-destructive/5 px-5 py-3 text-sm text-destructive"
          >
            {{ accountError }}
          </div>

          <div
            v-if="!selectedService"
            class="py-16 text-center"
          >
            <KeyRound class="mx-auto h-10 w-10 text-muted-foreground/50" />
            <p class="mt-3 font-medium">
              请选择一个上游服务
            </p>
            <p class="mt-1 text-sm text-muted-foreground">
              账号会登记到选中的服务下面。
            </p>
          </div>

          <div
            v-else-if="accountLoading && accounts.length === 0"
            class="flex items-center justify-center py-16 text-sm text-muted-foreground"
          >
            <Loader2 class="mr-2 h-5 w-5 animate-spin" />
            正在读取账号...
          </div>

          <div
            v-else-if="accounts.length === 0"
            class="py-16 text-center"
          >
            <KeyRound class="mx-auto h-10 w-10 text-muted-foreground/50" />
            <p class="mt-3 font-medium">
              这个服务下还没有账号
            </p>
            <p class="mt-1 text-sm text-muted-foreground">
              当前版本只登记账号身份，不保存真实凭证。
            </p>
          </div>

          <div
            v-else
            class="divide-y divide-border/70"
          >
            <div
              v-for="account in accounts"
              :key="account.id"
              class="p-5"
            >
              <div class="flex items-start justify-between gap-3">
                <div class="min-w-0">
                  <p class="truncate font-medium">
                    {{ account.display_name }}
                  </p>
                  <p class="mt-1 text-sm text-muted-foreground">
                    {{ accountContactLabel(account) }}
                  </p>
                </div>
                <Badge variant="outline">
                  {{ accountStatusLabel(account.status) }}
                </Badge>
              </div>
              <div class="mt-3 grid gap-2 text-xs text-muted-foreground sm:grid-cols-3">
                <span>认证：{{ authKindLabel(account.auth_kind) }}</span>
                <span>成本倍率：{{ formatMultiplier(account.cost_multiplier) }}</span>
                <span>优先级：{{ account.priority }}</span>
              </div>
            </div>
          </div>
        </Card>
      </div>
    </div>

    <Dialog
      v-model="serviceDialogOpen"
      size="2xl"
      title="新增上游服务"
      description="只登记服务基础信息和能力，不接入旧运行时。"
      :icon="Server"
    >
      <form
        class="space-y-5"
        @submit.prevent="submitService"
      >
        <div class="grid gap-4 sm:grid-cols-2">
          <div class="space-y-2">
            <Label for="service-name">服务名称</Label>
            <Input
              id="service-name"
              v-model="serviceForm.display_name"
              placeholder="例如 cc-max(zzshu)1.0"
              required
            />
          </div>
          <div class="space-y-2">
            <Label for="service-kind">服务类型</Label>
            <Select v-model="serviceForm.service_kind">
              <SelectTrigger id="service-kind">
                <SelectValue placeholder="选择服务类型" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="codex">Codex / ChatGPT OAuth</SelectItem>
                <SelectItem value="claude">Claude</SelectItem>
                <SelectItem value="openai">OpenAI</SelectItem>
                <SelectItem value="custom_openai">自定义 OpenAI 兼容</SelectItem>
                <SelectItem value="custom">自定义服务</SelectItem>
              </SelectContent>
            </Select>
          </div>
          <div class="space-y-2">
            <Label for="protocol-kind">协议</Label>
            <Select v-model="serviceForm.protocol_kind">
              <SelectTrigger id="protocol-kind">
                <SelectValue placeholder="选择协议" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="openai">OpenAI</SelectItem>
                <SelectItem value="anthropic">Anthropic</SelectItem>
                <SelectItem value="gemini">Gemini</SelectItem>
                <SelectItem value="codex">Codex</SelectItem>
                <SelectItem value="custom">自定义</SelectItem>
              </SelectContent>
            </Select>
          </div>
          <div class="space-y-2">
            <Label for="api-format">默认 API 格式</Label>
            <Input
              id="api-format"
              v-model="serviceForm.default_api_format"
              placeholder="例如 openai、codex"
            />
          </div>
          <div class="space-y-2 sm:col-span-2">
            <Label for="base-url">Base URL</Label>
            <Input
              id="base-url"
              v-model="serviceForm.base_url"
              placeholder="https://api.example.com"
            />
          </div>
          <div class="space-y-2">
            <Label for="cost-multiplier">成本倍率</Label>
            <Input
              id="cost-multiplier"
              v-model.number="serviceForm.cost_multiplier"
              type="number"
              min="0"
              step="0.0001"
            />
          </div>
          <div class="flex items-center gap-3 pt-7">
            <Switch
              id="service-active"
              v-model="serviceForm.is_active"
            />
            <Label for="service-active">启用服务</Label>
          </div>
        </div>

        <div class="rounded-xl border border-border/70 p-4">
          <p class="text-sm font-medium">
            服务能力
          </p>
          <div class="mt-3 grid gap-3 sm:grid-cols-2">
            <label
              v-for="item in capabilityOptions"
              :key="item.key"
              class="flex items-start gap-3 rounded-lg border border-border/50 p-3"
            >
              <Checkbox v-model:checked="serviceForm.capabilities[item.key]" />
              <span>
                <span class="block text-sm font-medium">{{ item.label }}</span>
                <span class="block text-xs text-muted-foreground">{{ item.description }}</span>
              </span>
            </label>
          </div>
        </div>
      </form>

      <template #footer>
        <Button
          type="submit"
          :disabled="savingService"
          @click="submitService"
        >
          {{ savingService ? '保存中...' : '保存服务' }}
        </Button>
        <Button
          type="button"
          variant="outline"
          :disabled="savingService"
          @click="serviceDialogOpen = false"
        >
          取消
        </Button>
      </template>
    </Dialog>

    <Dialog
      v-model="accountDialogOpen"
      size="lg"
      title="新增上游账号"
      description="当前版本只登记账号身份，不保存真实密钥。"
      :icon="KeyRound"
    >
      <form
        class="space-y-4"
        @submit.prevent="submitAccount"
      >
        <div class="space-y-2">
          <Label for="account-name">账号名称</Label>
          <Input
            id="account-name"
            v-model="accountForm.display_name"
            placeholder="例如 codex-plus 主账号"
            required
          />
        </div>
        <div class="grid gap-4 sm:grid-cols-2">
          <div class="space-y-2">
            <Label for="account-email">邮箱</Label>
            <Input
              id="account-email"
              v-model="accountForm.email"
              placeholder="name@example.com"
            />
          </div>
          <div class="space-y-2">
            <Label for="account-phone">手机号</Label>
            <Input
              id="account-phone"
              v-model="accountForm.phone"
              placeholder="可选"
            />
          </div>
        </div>
        <div class="grid gap-4 sm:grid-cols-3">
          <div class="space-y-2">
            <Label for="account-auth">认证方式</Label>
            <Select v-model="accountForm.auth_kind">
              <SelectTrigger id="account-auth">
                <SelectValue placeholder="选择认证方式" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="oauth">OAuth</SelectItem>
                <SelectItem value="api_key">API Key</SelectItem>
                <SelectItem value="custom_header">自定义 Header</SelectItem>
              </SelectContent>
            </Select>
          </div>
          <div class="space-y-2">
            <Label for="account-cost">成本倍率</Label>
            <Input
              id="account-cost"
              v-model.number="accountForm.cost_multiplier"
              type="number"
              min="0"
              step="0.0001"
            />
          </div>
          <div class="space-y-2">
            <Label for="account-priority">优先级</Label>
            <Input
              id="account-priority"
              v-model.number="accountForm.priority"
              type="number"
              step="1"
            />
          </div>
        </div>
      </form>

      <template #footer>
        <Button
          type="submit"
          :disabled="savingAccount || !selectedService"
          @click="submitAccount"
        >
          {{ savingAccount ? '保存中...' : '保存账号' }}
        </Button>
        <Button
          type="button"
          variant="outline"
          :disabled="savingAccount"
          @click="accountDialogOpen = false"
        >
          取消
        </Button>
      </template>
    </Dialog>
  </PageContainer>
</template>

<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue'
import {
  AlertTriangle,
  KeyRound,
  Loader2,
  Plus,
  RefreshCw,
  Search,
  Server,
} from 'lucide-vue-next'
import { PageContainer, PageHeader } from '@/components/layout'
import {
  Badge,
  Button,
  Card,
  Checkbox,
  Dialog,
  Input,
  Label,
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
  Switch,
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui'
import {
  createNifflerUpstreamAccount,
  createNifflerUpstreamService,
  listNifflerUpstreamAccounts,
  listNifflerUpstreamServices,
  type CreateNifflerUpstreamAccountPayload,
  type CreateNifflerUpstreamServicePayload,
  type NifflerAccountStatus,
  type NifflerProtocolKind,
  type NifflerUpstreamAccount,
  type NifflerUpstreamService,
} from '@/api/niffler-core'
import { useToast } from '@/composables/useToast'
import { extractErrorMessage } from '@/utils/error'

type CapabilityKey = keyof NonNullable<CreateNifflerUpstreamServicePayload['capabilities']>

const { success, error: showError } = useToast()

const services = ref<NifflerUpstreamService[]>([])
const accounts = ref<NifflerUpstreamAccount[]>([])
const serviceLoading = ref(false)
const accountLoading = ref(false)
const savingService = ref(false)
const savingAccount = ref(false)
const serviceError = ref('')
const accountError = ref('')
const serviceSearch = ref('')
const selectedServiceId = ref<string | null>(null)
const serviceDialogOpen = ref(false)
const accountDialogOpen = ref(false)

const defaultServiceForm = (): CreateNifflerUpstreamServicePayload => ({
  display_name: '',
  service_kind: 'custom_openai',
  protocol_kind: 'openai',
  default_api_format: 'openai',
  base_url: '',
  cost_multiplier: 1,
  is_active: true,
  capabilities: {
    text: true,
    streaming: true,
    images_endpoint: false,
    openai_responses_image_tool: false,
    model_list: true,
    model_test: true,
  },
})

const defaultAccountForm = (): CreateNifflerUpstreamAccountPayload => ({
  display_name: '',
  email: '',
  phone: '',
  auth_kind: 'oauth',
  cost_multiplier: 1,
  priority: 0,
})

const serviceForm = ref<CreateNifflerUpstreamServicePayload>(defaultServiceForm())
const accountForm = ref<CreateNifflerUpstreamAccountPayload>(defaultAccountForm())
let accountLoadSeq = 0

const capabilityOptions: Array<{
  key: CapabilityKey
  label: string
  description: string
}> = [
  { key: 'text', label: '文本对话', description: '支持普通文本请求' },
  { key: 'streaming', label: '流式响应', description: '支持边生成边返回' },
  { key: 'images_endpoint', label: '图片接口', description: '支持 /images 生图接口' },
  { key: 'openai_responses_image_tool', label: 'Responses 生图工具', description: '支持对话内调用图片工具' },
  { key: 'model_list', label: '模型列表', description: '支持读取模型列表' },
  { key: 'model_test', label: '模型测试', description: '支持后台测试模型' },
]

const selectedService = computed(() =>
  services.value.find(service => service.id === selectedServiceId.value) ?? null
)

watch(serviceDialogOpen, (open) => {
  if (!open) {
    serviceForm.value = defaultServiceForm()
  }
})

watch(accountDialogOpen, (open) => {
  if (!open) {
    accountForm.value = defaultAccountForm()
  }
})

async function refreshAll() {
  await loadServices()
  if (selectedServiceId.value) {
    await loadAccounts(selectedServiceId.value)
  }
}

async function loadServices() {
  serviceLoading.value = true
  serviceError.value = ''
  try {
    const response = await listNifflerUpstreamServices({
      include_inactive: true,
      search: serviceSearch.value.trim() || undefined,
      limit: 100,
    })
    services.value = response.items
    if (!selectedServiceId.value && services.value.length > 0) {
      selectedServiceId.value = services.value[0].id
      await loadAccounts(services.value[0].id)
    } else if (selectedServiceId.value && !services.value.some(item => item.id === selectedServiceId.value)) {
      selectedServiceId.value = services.value[0]?.id ?? null
      accounts.value = []
      if (selectedServiceId.value) {
        await loadAccounts(selectedServiceId.value)
      }
    }
  } catch (err) {
    serviceError.value = extractErrorMessage(err, '读取上游服务失败')
    showError(serviceError.value)
  } finally {
    serviceLoading.value = false
  }
}

async function loadAccounts(serviceId: string) {
  const seq = ++accountLoadSeq
  accountLoading.value = true
  accountError.value = ''
  try {
    const response = await listNifflerUpstreamAccounts(serviceId, { limit: 100 })
    if (seq !== accountLoadSeq) return
    accounts.value = response.items
  } catch (err) {
    if (seq !== accountLoadSeq) return
    accountError.value = extractErrorMessage(err, '读取上游账号失败')
    showError(accountError.value)
  } finally {
    if (seq === accountLoadSeq) {
      accountLoading.value = false
    }
  }
}

async function selectService(serviceId: string) {
  selectedServiceId.value = serviceId
  await loadAccounts(serviceId)
}

async function submitService() {
  const payload = normalizeServicePayload(serviceForm.value)
  if (!payload) return

  savingService.value = true
  try {
    const created = await createNifflerUpstreamService(payload)
    success('上游服务已登记')
    serviceDialogOpen.value = false
    await loadServices()
    selectedServiceId.value = created.id
    await loadAccounts(created.id)
  } catch (err) {
    showError(extractErrorMessage(err, '新增上游服务失败'))
  } finally {
    savingService.value = false
  }
}

async function submitAccount() {
  if (!selectedServiceId.value) return
  const payload = normalizeAccountPayload(accountForm.value)
  if (!payload) return

  savingAccount.value = true
  try {
    await createNifflerUpstreamAccount(selectedServiceId.value, payload)
    success('上游账号已登记')
    accountDialogOpen.value = false
    await loadAccounts(selectedServiceId.value)
  } catch (err) {
    showError(extractErrorMessage(err, '新增上游账号失败'))
  } finally {
    savingAccount.value = false
  }
}

function normalizeServicePayload(
  form: CreateNifflerUpstreamServicePayload
): CreateNifflerUpstreamServicePayload | null {
  const displayName = form.display_name.trim()
  if (!displayName) {
    showError('服务名称不能为空')
    return null
  }

  const costMultiplier = Number(form.cost_multiplier ?? 1)
  if (!Number.isFinite(costMultiplier) || costMultiplier < 0) {
    showError('成本倍率必须是非负数字')
    return null
  }

  const protocolKind = (form.protocol_kind || 'openai') as NifflerProtocolKind
  return {
    display_name: displayName,
    service_kind: form.service_kind.trim() || 'custom',
    protocol_kind: protocolKind,
    default_api_format: emptyToNull(form.default_api_format),
    base_url: emptyToNull(form.base_url),
    cost_multiplier: costMultiplier,
    is_active: form.is_active ?? true,
    capabilities: {
      text: Boolean(form.capabilities?.text),
      streaming: Boolean(form.capabilities?.streaming),
      images_endpoint: Boolean(form.capabilities?.images_endpoint),
      openai_responses_image_tool: Boolean(form.capabilities?.openai_responses_image_tool),
      model_list: Boolean(form.capabilities?.model_list),
      model_test: Boolean(form.capabilities?.model_test),
    },
  }
}

function normalizeAccountPayload(
  form: CreateNifflerUpstreamAccountPayload
): CreateNifflerUpstreamAccountPayload | null {
  const displayName = form.display_name.trim()
  if (!displayName) {
    showError('账号名称不能为空')
    return null
  }

  const costMultiplier = Number(form.cost_multiplier ?? 1)
  if (!Number.isFinite(costMultiplier) || costMultiplier < 0) {
    showError('成本倍率必须是非负数字')
    return null
  }

  const priority = Number(form.priority ?? 0)
  if (!Number.isFinite(priority)) {
    showError('优先级必须是数字')
    return null
  }

  return {
    display_name: displayName,
    email: emptyToNull(form.email),
    phone: emptyToNull(form.phone),
    auth_kind: form.auth_kind,
    cost_multiplier: costMultiplier,
    priority,
  }
}

function emptyToNull(value?: string | null): string | null {
  const normalized = value?.trim() ?? ''
  return normalized ? normalized : null
}

function formatMultiplier(value: number): string {
  return `${Number(value || 0).toFixed(4).replace(/\.?0+$/, '')}x`
}

function accountContactLabel(account: NifflerUpstreamAccount): string {
  const contacts = [account.email, account.phone].filter(Boolean)
  return contacts.length > 0 ? contacts.join(' / ') : '未填写邮箱或手机号'
}

function authKindLabel(value: string): string {
  const labels: Record<string, string> = {
    api_key: 'API Key',
    oauth: 'OAuth',
    custom_header: '自定义 Header',
  }
  return labels[value] ?? value
}

function accountStatusLabel(status: NifflerAccountStatus): string {
  const labels: Record<NifflerAccountStatus, string> = {
    available: '可用',
    disabled: '停用',
    invalid: '失效',
    quota_exhausted: '额度耗尽',
    cooling_down: '冷却中',
  }
  return labels[status] ?? status
}

onMounted(() => {
  void loadServices()
})
</script>
