<template>
  <PageContainer>
    <PageHeader
      title="产品策略"
      description="决定用户能用哪些模型，以及钱包按什么价格扣费。"
      :icon="Tags"
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
          刷新
        </Button>
        <Button
          class="admin-entry-action"
          @click="productPlanDialogOpen = true"
        >
          <Plus class="mr-2 h-4 w-4" />
          新增策略
        </Button>
      </template>
    </PageHeader>

    <Card class="mt-6 overflow-hidden">
      <div class="grid min-h-[560px] xl:grid-cols-[360px_minmax(0,1fr)]">
        <section class="border-b border-border/70 p-4 xl:border-b-0 xl:border-r">
          <div class="flex items-center justify-between gap-3">
            <div>
              <h2 class="text-sm font-semibold">
                策略
              </h2>
              <p class="mt-1 text-xs text-muted-foreground">
                给用户密钥使用的模型和价格规则。
              </p>
            </div>
            <Badge variant="secondary">
              {{ productPlans.length }}
            </Badge>
          </div>

          <div class="mt-3 flex gap-2">
            <Input
              v-model="productPlanSearch"
              class="h-9"
              placeholder="搜索策略"
              @keyup.enter="loadProductPlans"
            />
            <Button
              variant="outline"
              size="icon"
              class="admin-filter-action h-9 w-9 shrink-0"
              :disabled="productPlanLoading"
              title="搜索"
              @click="loadProductPlans"
            >
              <Search class="h-4 w-4" />
            </Button>
          </div>

          <p
            v-if="productPlanError"
            class="mt-3 rounded-md border border-destructive/20 bg-destructive/5 px-3 py-2 text-sm text-destructive"
          >
            {{ productPlanError }}
          </p>

          <div
            v-if="productPlanLoading && productPlans.length === 0"
            class="flex items-center justify-center py-12 text-sm text-muted-foreground"
          >
            <Loader2 class="mr-2 h-5 w-5 animate-spin" />
            正在读取策略
          </div>

          <div
            v-else-if="productPlans.length === 0"
            class="mt-4 rounded-lg border border-dashed border-border/70 p-4"
          >
            <p class="text-sm font-medium">
              先新增一个策略
            </p>
            <p class="mt-1 text-xs text-muted-foreground">
              策略建好后，再添加可用模型和适用密钥。
            </p>
            <Button
              class="admin-entry-action mt-3 h-8"
              size="sm"
              @click="productPlanDialogOpen = true"
            >
              新增策略
            </Button>
          </div>

          <div
            v-else
            class="mt-3 space-y-2"
          >
            <button
              v-for="plan in productPlans"
              :key="plan.id"
              type="button"
              class="admin-entry-row w-full rounded-lg border px-3 py-2 text-left transition-colors"
              :class="selectedProductPlanId === plan.id ? 'border-primary/50 bg-primary/10' : 'border-border/70 hover:bg-muted/40'"
              @click="selectProductPlan(plan.id)"
            >
              <div class="flex items-start justify-between gap-2">
                <div class="min-w-0">
                  <div class="truncate text-sm font-medium">
                    {{ plan.display_name }}
                  </div>
                  <div class="mt-1 truncate text-xs text-muted-foreground">
                    {{ formatMultiplier(plan.sales_multiplier) }} · {{ plan.is_public ? '公开' : '内部' }}
                  </div>
                </div>
                <Badge :variant="plan.is_active ? 'outline' : 'secondary'">
                  {{ plan.is_active ? '启用' : '停用' }}
                </Badge>
              </div>
            </button>
          </div>
        </section>

        <section class="p-4">
          <div
            v-if="!selectedProductPlan"
            class="flex min-h-[440px] flex-col justify-center rounded-lg border border-dashed border-border/70 p-6"
          >
            <p class="text-base font-semibold">
              先新增策略
            </p>
            <p class="mt-2 max-w-md text-sm text-muted-foreground">
              策略决定用户密钥能用哪些模型，以及钱包扣费价格。
            </p>
            <Button
              class="admin-entry-action mt-4 w-fit"
              @click="productPlanDialogOpen = true"
            >
              <Plus class="mr-2 h-4 w-4" />
              新增策略
            </Button>
          </div>

          <div
            v-else
            class="space-y-5"
          >
            <div class="flex flex-col gap-3 border-b border-border/70 pb-4 lg:flex-row lg:items-start lg:justify-between">
              <div class="min-w-0">
                <div class="flex flex-wrap items-center gap-2">
                  <h2 class="truncate text-lg font-semibold">
                    {{ selectedProductPlan.display_name }}
                  </h2>
                  <Badge :variant="selectedProductPlan.is_active ? 'outline' : 'secondary'">
                    {{ selectedProductPlan.is_active ? '启用' : '停用' }}
                  </Badge>
                  <Badge :variant="selectedProductPlan.is_public ? 'outline' : 'secondary'">
                    {{ selectedProductPlan.is_public ? '公开' : '内部' }}
                  </Badge>
                </div>
                <p
                  v-if="selectedProductPlan.description"
                  class="mt-2 text-sm text-muted-foreground"
                >
                  {{ selectedProductPlan.description }}
                </p>
                <div class="mt-2 flex flex-wrap gap-x-4 gap-y-1 text-xs text-muted-foreground">
                  <span>默认倍率 {{ formatMultiplier(selectedProductPlan.sales_multiplier) }}</span>
                  <span>模型 {{ productPlanModels.length }}</span>
                  <span>密钥 {{ selectedPlanKeyCount }}</span>
                </div>
              </div>
              <Button
                class="admin-entry-action h-9"
                @click="openProductPlanModelDialog"
              >
                <Plus class="mr-2 h-4 w-4" />
                添加模型
              </Button>
            </div>

            <div class="grid gap-5 xl:grid-cols-[minmax(0,1fr)_minmax(0,1fr)]">
              <section class="rounded-lg border border-border/70">
                <div class="border-b border-border/70 px-4 py-3">
                  <h3 class="text-sm font-semibold">
                    可用模型
                  </h3>
                  <p class="mt-1 text-xs text-muted-foreground">
                    这些模型会按当前策略的价格规则扣费。
                  </p>
                </div>

                <p
                  v-if="productPlanModelError"
                  class="border-b border-destructive/20 bg-destructive/5 px-4 py-3 text-sm text-destructive"
                >
                  {{ productPlanModelError }}
                </p>

                <div
                  v-if="productPlanModelLoading && productPlanModels.length === 0"
                  class="flex items-center justify-center py-12 text-sm text-muted-foreground"
                >
                  <Loader2 class="mr-2 h-5 w-5 animate-spin" />
                  正在读取模型
                </div>

                <div
                  v-else-if="productPlanModels.length === 0"
                  class="p-4"
                >
                  <div class="rounded-lg border border-dashed border-border/70 p-4">
                    <p class="text-sm font-medium">
                      还没有模型
                    </p>
                    <p class="mt-1 text-xs text-muted-foreground">
                      添加后，使用该策略的密钥才能看到这些模型。
                    </p>
                    <Button
                      class="admin-entry-action mt-3 h-8"
                      size="sm"
                      @click="openProductPlanModelDialog"
                    >
                      添加模型
                    </Button>
                  </div>
                </div>

                <Table v-else>
                  <TableHeader>
                    <TableRow>
                      <TableHead>模型</TableHead>
                      <TableHead>倍率</TableHead>
                      <TableHead>状态</TableHead>
                    </TableRow>
                  </TableHeader>
                  <TableBody>
                    <TableRow
                      v-for="model in productPlanModels"
                      :key="model.id"
                    >
                      <TableCell>
                        <div class="font-medium">
                          {{ model.model_name }}
                        </div>
                      </TableCell>
                      <TableCell>
                        {{ formatOptionalMultiplier(model.sales_multiplier_override) }}
                      </TableCell>
                      <TableCell>
                        <Badge :variant="model.is_enabled ? 'outline' : 'secondary'">
                          {{ model.is_enabled ? '启用' : '停用' }}
                        </Badge>
                      </TableCell>
                    </TableRow>
                  </TableBody>
                </Table>
              </section>

              <section class="rounded-lg border border-border/70">
                <div class="border-b border-border/70 px-4 py-3">
                  <h3 class="text-sm font-semibold">
                    适用密钥
                  </h3>
                  <p class="mt-1 text-xs text-muted-foreground">
                    这些密钥会按当前策略的模型和价格生效。
                  </p>
                </div>

                <p
                  v-if="apiKeyBindingError"
                  class="border-b border-destructive/20 bg-destructive/5 px-4 py-3 text-sm text-destructive"
                >
                  {{ apiKeyBindingError }}
                </p>

                <div
                  v-if="(apiKeyLoading || apiKeyBindingLoading) && standaloneApiKeys.length === 0"
                  class="flex items-center justify-center py-12 text-sm text-muted-foreground"
                >
                  <Loader2 class="mr-2 h-5 w-5 animate-spin" />
                  正在读取密钥
                </div>

                <div
                  v-else-if="standaloneApiKeys.length === 0"
                  class="p-4"
                >
                  <div class="rounded-lg border border-dashed border-border/70 p-4">
                    <p class="text-sm font-medium">
                      还没有独立密钥
                    </p>
                    <p class="mt-1 text-xs text-muted-foreground">
                      先在独立密钥页面创建密钥。
                    </p>
                  </div>
                </div>

                <Table v-else>
                  <TableHeader>
                    <TableRow>
                      <TableHead>密钥</TableHead>
                      <TableHead>当前策略</TableHead>
                      <TableHead class="text-right">
                        操作
                      </TableHead>
                    </TableRow>
                  </TableHeader>
                  <TableBody>
                    <TableRow
                      v-for="apiKey in standaloneApiKeys"
                      :key="apiKey.id"
                    >
                      <TableCell>
                        <div class="font-medium">
                          {{ formatApiKeyName(apiKey) }}
                        </div>
                        <div class="mt-1 text-xs text-muted-foreground">
                          {{ formatApiKeyOwner(apiKey) }}
                        </div>
                      </TableCell>
                      <TableCell>
                        <Badge :variant="apiKeyBindingByApiKeyId.get(apiKey.id) ? 'outline' : 'secondary'">
                          {{ apiKeyBindingPlanLabel(apiKey.id) }}
                        </Badge>
                      </TableCell>
                      <TableCell class="text-right">
                        <Button
                          size="sm"
                          variant="outline"
                          class="admin-entry-action"
                          :disabled="!selectedProductPlan.is_active || !apiKey.is_active || apiKeyIsBoundToSelectedPlan(apiKey.id) || savingApiKeyBindingId === apiKey.id"
                          @click="bindApiKeyToSelectedProductPlan(apiKey.id)"
                        >
                          <Loader2
                            v-if="savingApiKeyBindingId === apiKey.id"
                            class="mr-2 h-4 w-4 animate-spin"
                          />
                          {{ apiKeyIsBoundToSelectedPlan(apiKey.id) ? '已适用' : '改策略' }}
                        </Button>
                      </TableCell>
                    </TableRow>
                  </TableBody>
                </Table>
              </section>
            </div>
          </div>
        </section>
      </div>
    </Card>

    <Dialog
      v-model="productPlanDialogOpen"
      size="lg"
      title="新增策略"
      description="设置默认销售倍率、公开状态和备注。"
      :icon="Tags"
    >
      <form
        class="space-y-4"
        @submit.prevent="submitProductPlan"
      >
        <div class="space-y-2">
          <Label for="product-plan-name">策略名称</Label>
          <Input
            id="product-plan-name"
            v-model="productPlanForm.display_name"
            placeholder="例如 标准策略"
            required
          />
        </div>
        <div class="space-y-2">
          <Label for="product-plan-description">备注</Label>
          <Input
            id="product-plan-description"
            v-model="productPlanForm.description"
            placeholder="给管理员看的备注，可选"
          />
        </div>
        <div class="grid gap-4 sm:grid-cols-3">
          <div class="space-y-2">
            <Label for="product-plan-sales">销售倍率</Label>
            <Input
              id="product-plan-sales"
              v-model.number="productPlanForm.sales_multiplier"
              type="number"
              min="0"
              step="0.0001"
            />
          </div>
          <div class="flex items-center gap-3 pt-7">
            <Switch
              id="product-plan-public"
              v-model="productPlanForm.is_public"
            />
            <Label for="product-plan-public">公开</Label>
          </div>
          <div class="flex items-center gap-3 pt-7">
            <Switch
              id="product-plan-active"
              v-model="productPlanForm.is_active"
            />
            <Label for="product-plan-active">启用</Label>
          </div>
        </div>
      </form>

      <template #footer>
        <Button
          class="admin-entry-action"
          type="submit"
          :disabled="savingProductPlan"
          @click="submitProductPlan"
        >
          {{ savingProductPlan ? '保存中...' : '保存策略' }}
        </Button>
        <Button
          class="admin-entry-action"
          type="button"
          variant="outline"
          :disabled="savingProductPlan"
          @click="productPlanDialogOpen = false"
        >
          取消
        </Button>
      </template>
    </Dialog>

    <Dialog
      v-model="productPlanModelDialogOpen"
      size="lg"
      title="添加模型"
      description="从全局模型选择并预览钱包价格。"
      :icon="PackageCheck"
    >
      <form
        class="space-y-4"
        @submit.prevent="submitProductPlanModel"
      >
        <div class="space-y-2">
          <Label for="product-plan-global-model">全局模型</Label>
          <Select
            :model-value="selectedProductPlanModelGlobalModelId"
            :disabled="globalModelsLoading"
            @update:model-value="selectProductPlanGlobalModel"
          >
            <SelectTrigger id="product-plan-global-model">
              <SelectValue :placeholder="globalModelsLoading ? '正在读取模型...' : '选择模型'" />
            </SelectTrigger>
            <SelectContent search-placeholder="搜索模型名称...">
              <SelectItem
                v-for="model in globalModels"
                :key="model.id"
                :value="model.id"
                :text-value="`${model.display_name} ${model.name}`"
              >
                {{ model.display_name }} ({{ model.name }})
              </SelectItem>
            </SelectContent>
          </Select>
          <p
            v-if="globalModelsError"
            class="text-xs text-destructive"
          >
            {{ globalModelsError }}
          </p>
        </div>

        <div class="space-y-2">
          <Label for="product-plan-model-name">模型名称</Label>
          <Input
            id="product-plan-model-name"
            v-model="productPlanModelForm.model_name"
            placeholder="选择后自动填写，也可手动输入"
            required
          />
        </div>

        <div class="grid gap-4 sm:grid-cols-2">
          <div class="space-y-2">
            <Label for="product-plan-model-sales">倍率覆盖</Label>
            <Input
              id="product-plan-model-sales"
              v-model="productPlanModelForm.sales_multiplier_override"
              type="number"
              min="0"
              step="0.0001"
              placeholder="留空使用默认倍率"
            />
          </div>
          <div class="flex items-center gap-3 pt-7">
            <Switch
              id="product-plan-model-enabled"
              v-model="productPlanModelForm.is_enabled"
            />
            <Label for="product-plan-model-enabled">启用</Label>
          </div>
        </div>

        <div
          v-if="selectedProductPlanModelGlobalModel"
          class="rounded-xl border border-border/70 bg-muted/25 p-4"
        >
          <div class="flex flex-col gap-2 sm:flex-row sm:items-start sm:justify-between">
            <div>
              <p class="text-sm font-medium">
                钱包价格
              </p>
              <p class="mt-1 text-xs text-muted-foreground">
                基础价格 × 实际销售倍率。
              </p>
            </div>
            <div class="flex flex-wrap gap-2 text-xs text-muted-foreground">
              <span class="rounded-md bg-background px-2 py-1">
                默认：{{ formatMultiplier(selectedProductPlan?.sales_multiplier ?? 1) }}
              </span>
              <span class="rounded-md bg-background px-2 py-1">
                实际：{{ formatMultiplier(productPlanModelEffectiveMultiplier) }}
              </span>
            </div>
          </div>

          <div
            v-if="productPlanModelPriceRows.length > 0"
            class="mt-3 divide-y divide-border/60 rounded-lg border border-border/60 bg-background"
          >
            <div class="grid grid-cols-[1fr_auto_auto] items-center gap-3 px-3 py-2 text-xs text-muted-foreground">
              <span>计费项</span>
              <span>基础价</span>
              <span>钱包价</span>
            </div>
            <div
              v-for="row in productPlanModelPriceRows"
              :key="row.key"
              class="grid grid-cols-[1fr_auto_auto] items-center gap-3 px-3 py-2 text-sm"
            >
              <span class="text-muted-foreground">{{ row.label }}</span>
              <span>{{ formatProductPlanModelPrice(row.basePrice, row.unit) }}</span>
              <span class="font-medium">
                {{ formatProductPlanModelPrice(row.salesPrice, row.unit) }}
              </span>
            </div>
          </div>
        </div>
      </form>

      <template #footer>
        <Button
          class="admin-entry-action"
          type="submit"
          :disabled="savingProductPlanModel || !selectedProductPlan"
          @click="submitProductPlanModel"
        >
          {{ savingProductPlanModel ? '保存中...' : '保存模型' }}
        </Button>
        <Button
          class="admin-entry-action"
          type="button"
          variant="outline"
          :disabled="savingProductPlanModel"
          @click="productPlanModelDialogOpen = false"
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
  Loader2,
  PackageCheck,
  Plus,
  RefreshCw,
  Search,
  Tags,
} from 'lucide-vue-next'
import { PageContainer, PageHeader } from '@/components/layout'
import {
  Badge,
  Button,
  Card,
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
import { adminApi, type AdminApiKey } from '@/api/admin'
import {
  createNifflerProductPlan,
  listNifflerApiKeyProductPlanBindings,
  listNifflerProductPlanModels,
  listNifflerProductPlans,
  upsertNifflerApiKeyProductPlanBinding,
  upsertNifflerProductPlanModel,
  type CreateNifflerProductPlanPayload,
  type NifflerApiKeyProductPlanBinding,
  type NifflerProductPlan,
  type NifflerProductPlanModel,
  type UpsertNifflerProductPlanModelPayload,
} from '@/api/niffler-core'
import {
  listGlobalModels,
  type GlobalModelResponse,
} from '@/api/global-models'
import { useToast } from '@/composables/useToast'
import { extractErrorMessage } from '@/utils/error'
import {
  buildProductPlanModelPriceRows,
  formatProductPlanModelPrice,
  getProductPlanModelEffectiveMultiplier,
} from './niffler-product-plan-pricing'

type ProductPlanForm = Required<Pick<CreateNifflerProductPlanPayload, 'display_name' | 'is_public' | 'is_active'>> & {
  sales_multiplier: number | string
  description: string
}

type ProductPlanModelForm = Omit<UpsertNifflerProductPlanModelPayload, 'sales_multiplier_override'> & {
  sales_multiplier_override: number | string | null
}

const { success, error: showError } = useToast()

const productPlans = ref<NifflerProductPlan[]>([])
const productPlanModels = ref<NifflerProductPlanModel[]>([])
const apiKeys = ref<AdminApiKey[]>([])
const apiKeyProductPlanBindings = ref<NifflerApiKeyProductPlanBinding[]>([])
const globalModels = ref<GlobalModelResponse[]>([])
const productPlanLoading = ref(false)
const productPlanModelLoading = ref(false)
const apiKeyLoading = ref(false)
const apiKeyBindingLoading = ref(false)
const globalModelsLoading = ref(false)
const savingProductPlan = ref(false)
const savingProductPlanModel = ref(false)
const savingApiKeyBindingId = ref<string | null>(null)
const productPlanError = ref('')
const productPlanModelError = ref('')
const apiKeyBindingError = ref('')
const globalModelsError = ref('')
const productPlanSearch = ref('')
const selectedProductPlanId = ref<string | null>(null)
const productPlanDialogOpen = ref(false)
const productPlanModelDialogOpen = ref(false)
const selectedProductPlanModelGlobalModelId = ref('')
let productPlanModelLoadSeq = 0
let apiKeyBindingLoadSeq = 0

const defaultProductPlanForm = (): ProductPlanForm => ({
  display_name: '',
  is_public: false,
  is_active: true,
  sales_multiplier: 1,
  description: '',
})

const defaultProductPlanModelForm = (): ProductPlanModelForm => ({
  model_name: '',
  is_enabled: true,
  sales_multiplier_override: null,
})

const productPlanForm = ref<ProductPlanForm>(defaultProductPlanForm())
const productPlanModelForm = ref<ProductPlanModelForm>(defaultProductPlanModelForm())

const pageLoading = computed(() =>
  productPlanLoading.value
  || productPlanModelLoading.value
  || apiKeyLoading.value
  || apiKeyBindingLoading.value
)

const selectedProductPlan = computed(() =>
  productPlans.value.find(plan => plan.id === selectedProductPlanId.value) ?? null
)

const productPlanNameById = computed(() =>
  new Map(productPlans.value.map(plan => [plan.id, plan.display_name]))
)

const apiKeyBindingByApiKeyId = computed(() =>
  new Map(apiKeyProductPlanBindings.value.map(binding => [binding.api_key_id, binding]))
)

const standaloneApiKeys = computed(() =>
  apiKeys.value.filter(apiKey => apiKey.is_standalone)
)

const selectedPlanKeyCount = computed(() =>
  apiKeyProductPlanBindings.value.filter(binding => binding.product_plan_id === selectedProductPlanId.value).length
)

const selectedProductPlanModelGlobalModel = computed(() =>
  globalModels.value.find(model => model.id === selectedProductPlanModelGlobalModelId.value) ?? null
)

const productPlanModelEffectiveMultiplier = computed(() =>
  getProductPlanModelEffectiveMultiplier(
    selectedProductPlan.value?.sales_multiplier,
    productPlanModelForm.value.sales_multiplier_override
  )
)

const productPlanModelPriceRows = computed(() =>
  buildProductPlanModelPriceRows(
    selectedProductPlanModelGlobalModel.value,
    productPlanModelEffectiveMultiplier.value
  )
)

watch(productPlanDialogOpen, (open) => {
  if (!open) {
    productPlanForm.value = defaultProductPlanForm()
  }
})

watch(productPlanModelDialogOpen, (open) => {
  if (!open) {
    productPlanModelForm.value = defaultProductPlanModelForm()
    selectedProductPlanModelGlobalModelId.value = ''
    globalModelsError.value = ''
  }
})

async function refreshAll() {
  await Promise.all([
    loadProductPlans(),
    loadApiKeyBindingData(),
  ])
  if (selectedProductPlanId.value) {
    await loadProductPlanModels(selectedProductPlanId.value)
  }
}

async function loadProductPlans() {
  productPlanLoading.value = true
  productPlanError.value = ''
  try {
    const response = await listNifflerProductPlans({
      include_inactive: true,
      search: productPlanSearch.value.trim() || undefined,
      limit: 100,
    })
    productPlans.value = response.items
    if (!selectedProductPlanId.value && productPlans.value.length > 0) {
      await selectProductPlan(productPlans.value[0].id)
    } else if (selectedProductPlanId.value && !productPlans.value.some(item => item.id === selectedProductPlanId.value)) {
      selectedProductPlanId.value = productPlans.value[0]?.id ?? null
      productPlanModels.value = []
      if (selectedProductPlanId.value) {
        await selectProductPlan(selectedProductPlanId.value)
      }
    }
  } catch (err) {
    productPlanError.value = extractErrorMessage(err, '读取策略失败')
    showError(productPlanError.value)
  } finally {
    productPlanLoading.value = false
  }
}

async function selectProductPlan(productPlanId: string) {
  selectedProductPlanId.value = productPlanId
  await loadProductPlanModels(productPlanId)
}

async function loadProductPlanModels(productPlanId: string) {
  const seq = ++productPlanModelLoadSeq
  productPlanModelLoading.value = true
  productPlanModelError.value = ''
  try {
    const response = await listNifflerProductPlanModels(productPlanId, { limit: 100 })
    if (seq !== productPlanModelLoadSeq) return
    productPlanModels.value = response.items
  } catch (err) {
    if (seq !== productPlanModelLoadSeq) return
    productPlanModelError.value = extractErrorMessage(err, '读取模型失败')
    showError(productPlanModelError.value)
  } finally {
    if (seq === productPlanModelLoadSeq) {
      productPlanModelLoading.value = false
    }
  }
}

async function loadApiKeyBindingData() {
  const seq = ++apiKeyBindingLoadSeq
  apiKeyLoading.value = true
  apiKeyBindingLoading.value = true
  apiKeyBindingError.value = ''
  try {
    const [apiKeyResponse, bindingResponse] = await Promise.all([
      adminApi.getAllApiKeys({ skip: 0, limit: 200, include_usage_summary: false }),
      listNifflerApiKeyProductPlanBindings({ offset: 0, limit: 200 }),
    ])
    if (seq !== apiKeyBindingLoadSeq) return
    apiKeys.value = apiKeyResponse.api_keys
    apiKeyProductPlanBindings.value = bindingResponse.items
  } catch (err) {
    if (seq !== apiKeyBindingLoadSeq) return
    apiKeyBindingError.value = extractErrorMessage(err, '读取密钥失败')
    showError(apiKeyBindingError.value)
  } finally {
    if (seq === apiKeyBindingLoadSeq) {
      apiKeyLoading.value = false
      apiKeyBindingLoading.value = false
    }
  }
}

async function loadGlobalModels() {
  if (globalModels.value.length > 0) return
  globalModelsLoading.value = true
  globalModelsError.value = ''
  try {
    const response = await listGlobalModels(
      { skip: 0, limit: 1000, is_active: true },
      { cacheTtlMs: 60_000 }
    )
    globalModels.value = response.models
  } catch (err) {
    globalModelsError.value = extractErrorMessage(err, '读取全局模型失败')
    showError(globalModelsError.value)
  } finally {
    globalModelsLoading.value = false
  }
}

function openProductPlanModelDialog() {
  if (!selectedProductPlan.value) return
  productPlanModelForm.value = defaultProductPlanModelForm()
  selectedProductPlanModelGlobalModelId.value = ''
  globalModelsError.value = ''
  productPlanModelDialogOpen.value = true
  void loadGlobalModels()
}

function selectProductPlanGlobalModel(modelId: string) {
  selectedProductPlanModelGlobalModelId.value = modelId
  const model = globalModels.value.find(item => item.id === modelId)
  if (model) {
    productPlanModelForm.value.model_name = model.name
  }
}

async function submitProductPlan() {
  const payload = normalizeProductPlanPayload(productPlanForm.value)
  if (!payload) return

  savingProductPlan.value = true
  try {
    const created = await createNifflerProductPlan(payload)
    success('策略已保存')
    productPlanDialogOpen.value = false
    await loadProductPlans()
    await selectProductPlan(created.id)
  } catch (err) {
    showError(extractErrorMessage(err, '新增策略失败'))
  } finally {
    savingProductPlan.value = false
  }
}

async function submitProductPlanModel() {
  if (!selectedProductPlanId.value) return
  const payload = normalizeProductPlanModelPayload(productPlanModelForm.value)
  if (!payload) return

  savingProductPlanModel.value = true
  try {
    await upsertNifflerProductPlanModel(selectedProductPlanId.value, payload)
    success('模型已保存')
    productPlanModelDialogOpen.value = false
    await loadProductPlanModels(selectedProductPlanId.value)
  } catch (err) {
    showError(extractErrorMessage(err, '保存模型失败'))
  } finally {
    savingProductPlanModel.value = false
  }
}

async function bindApiKeyToSelectedProductPlan(apiKeyId: string) {
  if (!selectedProductPlanId.value || !selectedProductPlan.value) {
    showError('请先选择策略')
    return
  }
  if (!selectedProductPlan.value.is_active) {
    showError('只能使用启用的策略')
    return
  }
  savingApiKeyBindingId.value = apiKeyId
  try {
    await upsertNifflerApiKeyProductPlanBinding(selectedProductPlanId.value, { api_key_id: apiKeyId })
    success('策略已应用')
    await loadApiKeyBindingData()
  } catch (err) {
    showError(extractErrorMessage(err, '保存适用密钥失败'))
  } finally {
    savingApiKeyBindingId.value = null
  }
}

function normalizeProductPlanPayload(form: ProductPlanForm): CreateNifflerProductPlanPayload | null {
  const displayName = form.display_name.trim()
  if (!displayName) {
    showError('策略名称不能为空')
    return null
  }

  const salesMultiplier = Number(form.sales_multiplier ?? 1)
  if (!Number.isFinite(salesMultiplier) || salesMultiplier < 0) {
    showError('销售倍率必须是非负数字')
    return null
  }

  return {
    display_name: displayName,
    is_public: form.is_public,
    is_active: form.is_active,
    sales_multiplier: salesMultiplier,
    description: emptyToNull(form.description),
  }
}

function normalizeProductPlanModelPayload(
  form: ProductPlanModelForm
): UpsertNifflerProductPlanModelPayload | null {
  const modelName = form.model_name.trim()
  if (!modelName) {
    showError('模型名称不能为空')
    return null
  }

  const rawOverride = form.sales_multiplier_override
  let salesMultiplierOverride: number | null = null
  if (rawOverride !== null && rawOverride !== '') {
    const parsed = Number(rawOverride)
    if (!Number.isFinite(parsed) || parsed < 0) {
      showError('倍率覆盖必须是非负数字')
      return null
    }
    salesMultiplierOverride = parsed
  }

  return {
    model_name: modelName,
    is_enabled: form.is_enabled ?? true,
    sales_multiplier_override: salesMultiplierOverride,
  }
}

function emptyToNull(value?: string | null): string | null {
  const normalized = value?.trim() ?? ''
  return normalized ? normalized : null
}

function formatMultiplier(value: number): string {
  return `${Number(value || 0).toFixed(4).replace(/\.?0+$/, '')}x`
}

function formatOptionalMultiplier(value?: number | null): string {
  return value === null || value === undefined ? '默认倍率' : formatMultiplier(value)
}

function formatApiKeyName(apiKey: AdminApiKey): string {
  return apiKey.name?.trim() || apiKey.key_display || apiKey.id
}

function formatApiKeyOwner(apiKey: AdminApiKey): string {
  return apiKey.user_email || apiKey.username || apiKey.user_id
}

function apiKeyBindingPlanLabel(apiKeyId: string): string {
  const binding = apiKeyBindingByApiKeyId.value.get(apiKeyId)
  if (!binding) return '未设置'
  return productPlanNameById.value.get(binding.product_plan_id) || '未知策略'
}

function apiKeyIsBoundToSelectedPlan(apiKeyId: string): boolean {
  return apiKeyBindingByApiKeyId.value.get(apiKeyId)?.product_plan_id === selectedProductPlanId.value
}

onMounted(() => {
  void loadProductPlans()
  void loadApiKeyBindingData()
})
</script>
