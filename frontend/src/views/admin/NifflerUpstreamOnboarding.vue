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
          :disabled="serviceLoading || accountLoading || productPlanLoading || productPlanModelLoading"
          @click="refreshAll"
        >
          <RefreshCw
            class="mr-2 h-4 w-4"
            :class="{ 'animate-spin': serviceLoading || accountLoading || productPlanLoading || productPlanModelLoading }"
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
              本页只写入新表：上游服务、上游账号、服务能力、产品策略、可售模型。账号不保存真实密钥内容，也不会进入旧 Provider、号池、用户模型、计费或结算链路。
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

      <div class="grid gap-5 xl:grid-cols-[minmax(0,1.08fr)_minmax(360px,0.92fr)]">
        <Card class="overflow-hidden">
          <div class="flex flex-col gap-4 border-b border-border/70 p-5 lg:flex-row lg:items-center lg:justify-between">
            <div>
              <h2 class="text-lg font-semibold">
                产品策略
              </h2>
              <p class="mt-1 text-sm text-muted-foreground">
                登记以后要卖给用户的策略，不会影响当前用户 Key 和旧分组。
              </p>
            </div>
            <div class="flex flex-col gap-2 sm:flex-row sm:items-center">
              <Input
                v-model="productPlanSearch"
                class="h-9 sm:w-64"
                placeholder="搜索策略名称"
                @keyup.enter="loadProductPlans"
              />
              <Button
                variant="outline"
                class="h-9"
                :disabled="productPlanLoading"
                @click="loadProductPlans"
              >
                <Search class="mr-2 h-4 w-4" />
                搜索
              </Button>
              <Button
                class="h-9"
                @click="productPlanDialogOpen = true"
              >
                <Plus class="mr-2 h-4 w-4" />
                新增策略
              </Button>
            </div>
          </div>

          <div
            v-if="productPlanError"
            class="border-b border-destructive/20 bg-destructive/5 px-5 py-3 text-sm text-destructive"
          >
            {{ productPlanError }}
          </div>

          <div
            v-if="productPlanLoading && productPlans.length === 0"
            class="flex items-center justify-center py-16 text-sm text-muted-foreground"
          >
            <Loader2 class="mr-2 h-5 w-5 animate-spin" />
            正在读取产品策略...
          </div>

          <div
            v-else-if="productPlans.length === 0"
            class="py-16 text-center"
          >
            <Tags class="mx-auto h-10 w-10 text-muted-foreground/50" />
            <p class="mt-3 font-medium">
              还没有产品策略
            </p>
            <p class="mt-1 text-sm text-muted-foreground">
              先登记策略，再登记这个策略下可售的模型。
            </p>
          </div>

          <Table v-else>
            <TableHeader>
              <TableRow>
                <TableHead>策略名称</TableHead>
                <TableHead>默认销售倍率</TableHead>
                <TableHead>公开</TableHead>
                <TableHead>状态</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              <TableRow
                v-for="plan in productPlans"
                :key="plan.id"
                class="cursor-pointer"
                :class="selectedProductPlanId === plan.id ? 'bg-primary/5' : 'hover:bg-muted/40'"
                @click="selectProductPlan(plan.id)"
              >
                <TableCell>
                  <div class="font-medium">
                    {{ plan.display_name }}
                  </div>
                  <div
                    v-if="plan.description"
                    class="mt-1 max-w-[360px] truncate text-xs text-muted-foreground"
                  >
                    {{ plan.description }}
                  </div>
                </TableCell>
                <TableCell>
                  {{ formatMultiplier(plan.sales_multiplier) }}
                </TableCell>
                <TableCell>
                  <Badge :variant="plan.is_public ? 'outline' : 'secondary'">
                    {{ plan.is_public ? '公开' : '内部' }}
                  </Badge>
                </TableCell>
                <TableCell>
                  <Badge :variant="plan.is_active ? 'outline' : 'secondary'">
                    {{ plan.is_active ? '启用' : '停用' }}
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
                可售模型
              </h2>
              <p class="mt-1 text-sm text-muted-foreground">
                {{ selectedProductPlan ? `当前策略：${selectedProductPlan.display_name}` : '先选择左侧产品策略' }}
              </p>
            </div>
            <Button
              class="h-9"
              :disabled="!selectedProductPlan"
              @click="productPlanModelDialogOpen = true"
            >
              <Plus class="mr-2 h-4 w-4" />
              新增模型
            </Button>
          </div>

          <div
            v-if="productPlanModelError"
            class="border-b border-destructive/20 bg-destructive/5 px-5 py-3 text-sm text-destructive"
          >
            {{ productPlanModelError }}
          </div>

          <div
            v-if="!selectedProductPlan"
            class="py-16 text-center"
          >
            <PackageCheck class="mx-auto h-10 w-10 text-muted-foreground/50" />
            <p class="mt-3 font-medium">
              请选择一个产品策略
            </p>
            <p class="mt-1 text-sm text-muted-foreground">
              可售模型会登记到选中的策略下面。
            </p>
          </div>

          <div
            v-else-if="productPlanModelLoading && productPlanModels.length === 0"
            class="flex items-center justify-center py-16 text-sm text-muted-foreground"
          >
            <Loader2 class="mr-2 h-5 w-5 animate-spin" />
            正在读取可售模型...
          </div>

          <div
            v-else-if="productPlanModels.length === 0"
            class="py-16 text-center"
          >
            <PackageCheck class="mx-auto h-10 w-10 text-muted-foreground/50" />
            <p class="mt-3 font-medium">
              这个策略下还没有模型
            </p>
            <p class="mt-1 text-sm text-muted-foreground">
              这里登记的是新模型配置，不影响当前用户可用模型。
            </p>
          </div>

          <Table v-else>
            <TableHeader>
              <TableRow>
                <TableHead>模型名称</TableHead>
                <TableHead>销售倍率覆盖</TableHead>
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

    <Dialog
      v-model="productPlanDialogOpen"
      size="lg"
      title="新增产品策略"
      description="只登记新模型里的产品策略，不绑定用户 Key。"
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
            placeholder="例如 标准套餐策略"
            required
          />
        </div>
        <div class="space-y-2">
          <Label for="product-plan-description">说明</Label>
          <Input
            id="product-plan-description"
            v-model="productPlanForm.description"
            placeholder="给管理员看的备注，可选"
          />
        </div>
        <div class="grid gap-4 sm:grid-cols-3">
          <div class="space-y-2">
            <Label for="product-plan-sales">钱包销售倍率</Label>
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
            <Label for="product-plan-public">公开策略</Label>
          </div>
          <div class="flex items-center gap-3 pt-7">
            <Switch
              id="product-plan-active"
              v-model="productPlanForm.is_active"
            />
            <Label for="product-plan-active">启用策略</Label>
          </div>
        </div>
      </form>

      <template #footer>
        <Button
          type="submit"
          :disabled="savingProductPlan"
          @click="submitProductPlan"
        >
          {{ savingProductPlan ? '保存中...' : '保存策略' }}
        </Button>
        <Button
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
      title="新增可售模型"
      description="只登记这个产品策略里的模型和销售倍率覆盖。"
      :icon="PackageCheck"
    >
      <form
        class="space-y-4"
        @submit.prevent="submitProductPlanModel"
      >
        <div class="space-y-2">
          <Label for="product-plan-model-name">模型名称</Label>
          <Input
            id="product-plan-model-name"
            v-model="productPlanModelForm.model_name"
            placeholder="例如 gpt-5.5"
            required
          />
        </div>
        <div class="grid gap-4 sm:grid-cols-2">
          <div class="space-y-2">
            <Label for="product-plan-model-sales">模型级销售倍率覆盖</Label>
            <Input
              id="product-plan-model-sales"
              v-model="productPlanModelForm.sales_multiplier_override"
              type="number"
              min="0"
              step="0.0001"
              placeholder="留空则使用策略默认倍率"
            />
          </div>
          <div class="flex items-center gap-3 pt-7">
            <Switch
              id="product-plan-model-enabled"
              v-model="productPlanModelForm.is_enabled"
            />
            <Label for="product-plan-model-enabled">启用模型</Label>
          </div>
        </div>
      </form>

      <template #footer>
        <Button
          type="submit"
          :disabled="savingProductPlanModel || !selectedProductPlan"
          @click="submitProductPlanModel"
        >
          {{ savingProductPlanModel ? '保存中...' : '保存模型' }}
        </Button>
        <Button
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
  AlertTriangle,
  KeyRound,
  Loader2,
  PackageCheck,
  Plus,
  RefreshCw,
  Search,
  Server,
  Tags,
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
  createNifflerProductPlan,
  createNifflerUpstreamAccount,
  createNifflerUpstreamService,
  listNifflerProductPlanModels,
  listNifflerProductPlans,
  listNifflerUpstreamAccounts,
  listNifflerUpstreamServices,
  upsertNifflerProductPlanModel,
  type CreateNifflerProductPlanPayload,
  type CreateNifflerUpstreamAccountPayload,
  type CreateNifflerUpstreamServicePayload,
  type NifflerAccountStatus,
  type NifflerProductPlan,
  type NifflerProductPlanModel,
  type NifflerProtocolKind,
  type NifflerUpstreamAccount,
  type NifflerUpstreamService,
  type UpsertNifflerProductPlanModelPayload,
} from '@/api/niffler-core'
import { useToast } from '@/composables/useToast'
import { extractErrorMessage } from '@/utils/error'

type CapabilityKey = keyof NonNullable<CreateNifflerUpstreamServicePayload['capabilities']>
type ProductPlanForm = Required<Pick<CreateNifflerProductPlanPayload, 'display_name' | 'is_public' | 'is_active'>> & {
  sales_multiplier: number | string
  description: string
}
type ProductPlanModelForm = Omit<UpsertNifflerProductPlanModelPayload, 'sales_multiplier_override'> & {
  sales_multiplier_override: number | string | null
}

const { success, error: showError } = useToast()

const services = ref<NifflerUpstreamService[]>([])
const accounts = ref<NifflerUpstreamAccount[]>([])
const productPlans = ref<NifflerProductPlan[]>([])
const productPlanModels = ref<NifflerProductPlanModel[]>([])
const serviceLoading = ref(false)
const accountLoading = ref(false)
const productPlanLoading = ref(false)
const productPlanModelLoading = ref(false)
const savingService = ref(false)
const savingAccount = ref(false)
const savingProductPlan = ref(false)
const savingProductPlanModel = ref(false)
const serviceError = ref('')
const accountError = ref('')
const productPlanError = ref('')
const productPlanModelError = ref('')
const serviceSearch = ref('')
const productPlanSearch = ref('')
const selectedServiceId = ref<string | null>(null)
const selectedProductPlanId = ref<string | null>(null)
const serviceDialogOpen = ref(false)
const accountDialogOpen = ref(false)
const productPlanDialogOpen = ref(false)
const productPlanModelDialogOpen = ref(false)

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

const serviceForm = ref<CreateNifflerUpstreamServicePayload>(defaultServiceForm())
const accountForm = ref<CreateNifflerUpstreamAccountPayload>(defaultAccountForm())
const productPlanForm = ref<ProductPlanForm>(defaultProductPlanForm())
const productPlanModelForm = ref<ProductPlanModelForm>(defaultProductPlanModelForm())
let accountLoadSeq = 0
let productPlanModelLoadSeq = 0

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

const selectedProductPlan = computed(() =>
  productPlans.value.find(plan => plan.id === selectedProductPlanId.value) ?? null
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

watch(productPlanDialogOpen, (open) => {
  if (!open) {
    productPlanForm.value = defaultProductPlanForm()
  }
})

watch(productPlanModelDialogOpen, (open) => {
  if (!open) {
    productPlanModelForm.value = defaultProductPlanModelForm()
  }
})

async function refreshAll() {
  await Promise.all([loadServices(), loadProductPlans()])
  if (selectedServiceId.value) {
    await loadAccounts(selectedServiceId.value)
  }
  if (selectedProductPlanId.value) {
    await loadProductPlanModels(selectedProductPlanId.value)
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
      selectedProductPlanId.value = productPlans.value[0].id
      await loadProductPlanModels(productPlans.value[0].id)
    } else if (selectedProductPlanId.value && !productPlans.value.some(item => item.id === selectedProductPlanId.value)) {
      selectedProductPlanId.value = productPlans.value[0]?.id ?? null
      productPlanModels.value = []
      if (selectedProductPlanId.value) {
        await loadProductPlanModels(selectedProductPlanId.value)
      }
    }
  } catch (err) {
    productPlanError.value = extractErrorMessage(err, '读取产品策略失败')
    showError(productPlanError.value)
  } finally {
    productPlanLoading.value = false
  }
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
    productPlanModelError.value = extractErrorMessage(err, '读取可售模型失败')
    showError(productPlanModelError.value)
  } finally {
    if (seq === productPlanModelLoadSeq) {
      productPlanModelLoading.value = false
    }
  }
}

async function selectProductPlan(productPlanId: string) {
  selectedProductPlanId.value = productPlanId
  await loadProductPlanModels(productPlanId)
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

async function submitProductPlan() {
  const payload = normalizeProductPlanPayload(productPlanForm.value)
  if (!payload) return

  savingProductPlan.value = true
  try {
    const created = await createNifflerProductPlan(payload)
    success('产品策略已登记')
    productPlanDialogOpen.value = false
    await loadProductPlans()
    selectedProductPlanId.value = created.id
    await loadProductPlanModels(created.id)
  } catch (err) {
    showError(extractErrorMessage(err, '新增产品策略失败'))
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
    success('可售模型已保存')
    productPlanModelDialogOpen.value = false
    await loadProductPlanModels(selectedProductPlanId.value)
  } catch (err) {
    showError(extractErrorMessage(err, '保存可售模型失败'))
  } finally {
    savingProductPlanModel.value = false
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

function normalizeProductPlanPayload(form: ProductPlanForm): CreateNifflerProductPlanPayload | null {
  const displayName = form.display_name.trim()
  if (!displayName) {
    showError('策略名称不能为空')
    return null
  }

  const salesMultiplier = Number(form.sales_multiplier ?? 1)
  if (!Number.isFinite(salesMultiplier) || salesMultiplier < 0) {
    showError('钱包销售倍率必须是非负数字')
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
      showError('模型级销售倍率覆盖必须是非负数字')
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
  return value === null || value === undefined ? '使用策略默认倍率' : formatMultiplier(value)
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
  void loadProductPlans()
})
</script>
