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
          :disabled="serviceLoading || accountLoading || productPlanLoading || productPlanModelLoading || errorReturnSettingLoading"
          @click="refreshAll"
        >
          <RefreshCw
            class="mr-2 h-4 w-4"
            :class="{ 'animate-spin': serviceLoading || accountLoading || productPlanLoading || productPlanModelLoading || errorReturnSettingLoading }"
          />
          刷新
        </Button>
      </template>
    </PageHeader>

    <div class="mt-6 space-y-5">
      <Card class="overflow-hidden border-warning/30 bg-warning/10">
        <div class="flex flex-col gap-3 p-5 md:flex-row md:items-start">
          <AlertTriangle class="mt-0.5 h-5 w-5 shrink-0 text-warning" />
          <div class="space-y-1">
            <p class="font-medium text-foreground">
              这是新模型入口，不会改动当前线上请求。
            </p>
            <p class="text-sm text-muted-foreground">
              本页只写入新表：上游服务、上游账号、服务能力、产品策略、可售模型、错误文案规则。账号不保存真实密钥内容，也不会进入旧 Provider、号池、用户模型、计费、结算或错误返回链路。
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
                @click="openServiceDialog"
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
                  {{ serviceKindLabel(service.service_kind) }}
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

      <Card class="overflow-hidden">
        <div class="flex flex-col gap-4 border-b border-border/70 p-5 lg:flex-row lg:items-center lg:justify-between">
          <div>
            <h2 class="text-lg font-semibold">
              错误文案规则
            </h2>
            <p class="mt-1 text-sm text-muted-foreground">
              保存平台和上游错误返回文案。当前只写新配置，不影响线上错误返回。
            </p>
          </div>
          <Button
            class="h-9"
            @click="errorReturnSettingDialogOpen = true"
          >
            <Plus class="mr-2 h-4 w-4" />
            新增规则
          </Button>
        </div>

        <div
          v-if="errorReturnSettingError"
          class="border-b border-destructive/20 bg-destructive/5 px-5 py-3 text-sm text-destructive"
        >
          {{ errorReturnSettingError }}
        </div>

        <div
          v-if="errorReturnSettingLoading && errorReturnSettings.length === 0"
          class="flex items-center justify-center py-16 text-sm text-muted-foreground"
        >
          <Loader2 class="mr-2 h-5 w-5 animate-spin" />
          正在读取错误文案规则...
        </div>

        <div
          v-else-if="errorReturnSettings.length === 0"
          class="py-16 text-center"
        >
          <AlertTriangle class="mx-auto h-10 w-10 text-muted-foreground/50" />
          <p class="mt-3 font-medium">
            还没有错误文案规则
          </p>
          <p class="mt-1 text-sm text-muted-foreground">
            可以先登记常见平台错误和上游错误文案，后续灰度时再接入运行时。
          </p>
        </div>

        <Table v-else>
          <TableHeader>
            <TableRow>
              <TableHead>范围</TableHead>
              <TableHead>匹配条件</TableHead>
              <TableHead>返回方式</TableHead>
              <TableHead>用户文案</TableHead>
              <TableHead>账号保护</TableHead>
              <TableHead>状态</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            <TableRow
              v-for="rule in errorReturnSettings"
              :key="rule.id"
            >
              <TableCell>
                <div class="font-medium">
                  {{ errorScopeLabel(rule.scope) }}
                </div>
                <div class="mt-1 text-xs text-muted-foreground">
                  {{ upstreamServiceLabel(rule.upstream_service_id) }}
                </div>
              </TableCell>
              <TableCell>
                <div>{{ matchLabel(rule) }}</div>
                <div
                  v-if="rule.handling_step"
                  class="mt-1 text-xs text-muted-foreground"
                >
                  {{ handlingStepLabel(rule.handling_step) }}
                </div>
              </TableCell>
              <TableCell>
                {{ responseModeLabel(rule.response_mode) }}
              </TableCell>
              <TableCell>
                <div class="max-w-[420px] truncate">
                  {{ rule.user_message }}
                </div>
              </TableCell>
              <TableCell>
                <div>{{ protectionActionLabel(rule.account_protection_action) }}</div>
                <div
                  v-if="rule.pause_duration"
                  class="mt-1 text-xs text-muted-foreground"
                >
                  {{ pauseDurationLabel(rule.pause_duration) }}
                </div>
              </TableCell>
              <TableCell>
                <Badge :variant="rule.is_active ? 'outline' : 'secondary'">
                  {{ rule.is_active ? '启用' : '停用' }}
                </Badge>
              </TableCell>
            </TableRow>
          </TableBody>
        </Table>
      </Card>
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
            <Label for="service-template">接入类型</Label>
            <Select v-model="selectedServiceTemplateKey">
              <SelectTrigger id="service-template">
                <SelectValue placeholder="选择接入类型" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem
                  v-for="template in nifflerServiceTemplates"
                  :key="template.key"
                  :value="template.key"
                >
                  {{ template.label }}
                </SelectItem>
              </SelectContent>
            </Select>
          </div>

          <div class="rounded-xl border border-border/70 bg-muted/30 p-4 sm:col-span-2">
            <div class="flex flex-col gap-3 sm:flex-row sm:items-start sm:justify-between">
              <p class="text-sm text-muted-foreground">
                {{ selectedServiceTemplate.description }}
              </p>
              <Badge variant="outline">
                账号默认：{{ authKindLabel(selectedServiceTemplate.defaultAuthKind) }}
              </Badge>
            </div>
            <div class="mt-3 flex flex-wrap gap-2 text-xs text-muted-foreground">
              <span class="rounded-md bg-background px-2 py-1">服务类型：{{ serviceForm.service_kind }}</span>
              <span class="rounded-md bg-background px-2 py-1">协议：{{ serviceForm.protocol_kind }}</span>
              <span class="rounded-md bg-background px-2 py-1">API 格式：{{ serviceForm.default_api_format }}</span>
            </div>
          </div>
          <div class="space-y-2 sm:col-span-2">
            <Label for="base-url">Base URL</Label>
            <Input
              id="base-url"
              v-model="serviceForm.base_url"
              :placeholder="selectedServiceTemplate.baseUrlPlaceholder"
              :required="selectedServiceTemplate.baseUrlRequired"
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

          <details class="rounded-xl border border-border/70 p-4 sm:col-span-2">
            <summary class="cursor-pointer text-sm font-medium">
              高级字段
            </summary>
            <div class="mt-4 grid gap-4 sm:grid-cols-3">
              <div class="space-y-2">
                <Label for="service-kind">服务类型</Label>
                <Input
                  id="service-kind"
                  v-model="serviceForm.service_kind"
                  placeholder="例如 openai_compatible"
                />
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
            </div>
          </details>
        </div>

        <div class="rounded-xl border border-border/70 p-4">
          <p class="text-sm font-medium">
            服务能力
          </p>
          <p class="mt-1 text-xs text-muted-foreground">
            只显示当前协议可配置的能力；OpenAI Responses 生图工具不会出现在 Anthropic 或 Gemini 协议里。
          </p>
          <div class="mt-3 grid gap-3 sm:grid-cols-2">
            <label
              v-for="item in visibleCapabilityOptions"
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
          <Label for="product-plan-description">备注</Label>
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

    <Dialog
      v-model="errorReturnSettingDialogOpen"
      size="2xl"
      title="新增错误文案规则"
      description="只保存新模型里的错误文案配置，不改变当前线上返回内容。"
      :icon="AlertTriangle"
    >
      <form
        class="space-y-5"
        @submit.prevent="submitErrorReturnSetting"
      >
        <div class="grid gap-4 sm:grid-cols-2">
          <div class="space-y-2">
            <Label for="error-scope">规则范围</Label>
            <Select v-model="errorReturnSettingForm.scope">
              <SelectTrigger id="error-scope">
                <SelectValue placeholder="选择规则范围" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="platform">平台本地错误</SelectItem>
                <SelectItem value="upstream">上游返回错误</SelectItem>
              </SelectContent>
            </Select>
          </div>
          <div
            v-if="errorReturnSettingForm.scope === 'upstream'"
            class="space-y-2"
          >
            <Label for="error-upstream">上游服务</Label>
            <Select v-model="errorReturnSettingForm.upstream_service_id">
              <SelectTrigger id="error-upstream">
                <SelectValue placeholder="选择上游服务" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="__all__">全部上游</SelectItem>
                <SelectItem
                  v-for="service in services"
                  :key="service.id"
                  :value="service.id"
                >
                  {{ service.display_name }}
                </SelectItem>
              </SelectContent>
            </Select>
          </div>
          <div
            v-if="errorReturnSettingForm.scope === 'upstream'"
            class="space-y-2"
          >
            <Label for="error-step">处理类型</Label>
            <Select v-model="errorReturnSettingForm.handling_step">
              <SelectTrigger id="error-step">
                <SelectValue placeholder="选择处理类型" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="risk_keyword">风控关键词</SelectItem>
                <SelectItem value="contact_or_marketing_replacement">广告或客服内容替换</SelectItem>
                <SelectItem value="status_code_message">状态码文案</SelectItem>
                <SelectItem value="default_upstream_message">默认上游错误文案</SelectItem>
              </SelectContent>
            </Select>
          </div>
          <div class="space-y-2">
            <Label for="error-status-code">状态码</Label>
            <Input
              id="error-status-code"
              v-model="errorReturnSettingForm.match_status_code"
              type="number"
              min="100"
              max="599"
              step="1"
              placeholder="可选，例如 403"
            />
          </div>
          <div class="space-y-2 sm:col-span-2">
            <Label for="error-match-text">
              {{ errorReturnSettingForm.scope === 'platform' ? '平台错误代码' : '匹配关键词' }}
            </Label>
            <Input
              id="error-match-text"
              v-model="errorReturnSettingForm.match_text"
              :placeholder="errorReturnSettingForm.scope === 'platform' ? '例如 insufficient_balance，可选' : '例如 abuse、support@example.com，可选'"
            />
          </div>
          <div class="space-y-2">
            <Label for="error-response-mode">返回方式</Label>
            <Select v-model="errorReturnSettingForm.response_mode">
              <SelectTrigger id="error-response-mode">
                <SelectValue placeholder="选择返回方式" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="replace">完全替换</SelectItem>
                <SelectItem value="append">追加说明</SelectItem>
                <SelectItem value="redact">部分脱敏</SelectItem>
              </SelectContent>
            </Select>
          </div>
          <div
            v-if="errorReturnSettingForm.scope === 'upstream'"
            class="space-y-2"
          >
            <Label for="error-protection">账号保护</Label>
            <Select v-model="errorReturnSettingForm.account_protection_action">
              <SelectTrigger id="error-protection">
                <SelectValue placeholder="选择账号保护" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="record_only">只记录</SelectItem>
                <SelectItem value="pause_scheduling">暂停调度</SelectItem>
                <SelectItem value="disable_account">停用账号</SelectItem>
              </SelectContent>
            </Select>
          </div>
          <div
            v-if="errorReturnSettingForm.scope === 'upstream' && errorReturnSettingForm.account_protection_action === 'pause_scheduling'"
            class="space-y-2"
          >
            <Label for="error-pause-duration">暂停时长</Label>
            <Select v-model="errorReturnSettingForm.pause_duration">
              <SelectTrigger id="error-pause-duration">
                <SelectValue placeholder="选择暂停时长" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="ten_minutes">10 分钟</SelectItem>
                <SelectItem value="one_hour">1 小时</SelectItem>
                <SelectItem value="twenty_four_hours">24 小时</SelectItem>
                <SelectItem value="manual_restore">手动恢复</SelectItem>
              </SelectContent>
            </Select>
          </div>
          <div class="flex items-center gap-3 pt-7">
            <Switch
              id="error-active"
              v-model="errorReturnSettingForm.is_active"
            />
            <Label for="error-active">启用规则</Label>
          </div>
        </div>

        <div class="space-y-2">
          <Label for="error-user-message">返回给用户的文案</Label>
          <Textarea
            id="error-user-message"
            v-model="errorReturnSettingForm.user_message"
            rows="4"
            placeholder="例如：请求内容触发上游安全限制，请调整任务后重试。如需帮助，请联系平台客服。"
            required
          />
        </div>
      </form>

      <template #footer>
        <Button
          type="submit"
          :disabled="savingErrorReturnSetting"
          @click="submitErrorReturnSetting"
        >
          {{ savingErrorReturnSetting ? '保存中...' : '保存规则' }}
        </Button>
        <Button
          type="button"
          variant="outline"
          :disabled="savingErrorReturnSetting"
          @click="errorReturnSettingDialogOpen = false"
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
  Textarea,
} from '@/components/ui'
import {
  createNifflerErrorReturnSetting,
  createNifflerProductPlan,
  createNifflerUpstreamAccount,
  createNifflerUpstreamService,
  listNifflerErrorReturnSettings,
  listNifflerProductPlanModels,
  listNifflerProductPlans,
  listNifflerUpstreamAccounts,
  listNifflerUpstreamServices,
  upsertNifflerProductPlanModel,
  type CreateNifflerErrorReturnSettingPayload,
  type CreateNifflerProductPlanPayload,
  type CreateNifflerUpstreamAccountPayload,
  type CreateNifflerUpstreamServicePayload,
  type NifflerAccountProtectionAction,
  type NifflerAccountStatus,
  type NifflerErrorResponseScope,
  type NifflerErrorReturnSetting,
  type NifflerPauseDuration,
  type NifflerProductPlan,
  type NifflerProductPlanModel,
  type NifflerProtocolKind,
  type NifflerUpstreamErrorHandlingStep,
  type NifflerUpstreamAccount,
  type NifflerUpstreamService,
  type NifflerUserResponseMode,
  type UpsertNifflerProductPlanModelPayload,
} from '@/api/niffler-core'
import { useToast } from '@/composables/useToast'
import { extractErrorMessage } from '@/utils/error'
import {
  DEFAULT_NIFFLER_SERVICE_TEMPLATE_KEY,
  buildNifflerServiceFormFromTemplate,
  filterCapabilityOptionsForProtocol,
  getDefaultAuthKindForService,
  getServiceKindLabel,
  getNifflerServiceTemplate,
  nifflerServiceTemplates,
  type NifflerServiceCapabilityKey,
  type NifflerServiceTemplateKey,
} from './niffler-upstream-service-templates'

type ProductPlanForm = Required<Pick<CreateNifflerProductPlanPayload, 'display_name' | 'is_public' | 'is_active'>> & {
  sales_multiplier: number | string
  description: string
}
type ProductPlanModelForm = Omit<UpsertNifflerProductPlanModelPayload, 'sales_multiplier_override'> & {
  sales_multiplier_override: number | string | null
}
type ErrorReturnSettingForm = {
  scope: NifflerErrorResponseScope
  upstream_service_id: string
  match_status_code: number | string | null
  match_text: string
  handling_step: NifflerUpstreamErrorHandlingStep | ''
  response_mode: NifflerUserResponseMode
  user_message: string
  account_protection_action: NifflerAccountProtectionAction
  pause_duration: NifflerPauseDuration | ''
  is_active: boolean
}

const { success, error: showError } = useToast()

const services = ref<NifflerUpstreamService[]>([])
const accounts = ref<NifflerUpstreamAccount[]>([])
const productPlans = ref<NifflerProductPlan[]>([])
const productPlanModels = ref<NifflerProductPlanModel[]>([])
const errorReturnSettings = ref<NifflerErrorReturnSetting[]>([])
const serviceLoading = ref(false)
const accountLoading = ref(false)
const productPlanLoading = ref(false)
const productPlanModelLoading = ref(false)
const errorReturnSettingLoading = ref(false)
const savingService = ref(false)
const savingAccount = ref(false)
const savingProductPlan = ref(false)
const savingProductPlanModel = ref(false)
const savingErrorReturnSetting = ref(false)
const serviceError = ref('')
const accountError = ref('')
const productPlanError = ref('')
const productPlanModelError = ref('')
const errorReturnSettingError = ref('')
const serviceSearch = ref('')
const productPlanSearch = ref('')
const selectedServiceId = ref<string | null>(null)
const selectedProductPlanId = ref<string | null>(null)
const serviceDialogOpen = ref(false)
const accountDialogOpen = ref(false)
const productPlanDialogOpen = ref(false)
const productPlanModelDialogOpen = ref(false)
const errorReturnSettingDialogOpen = ref(false)
const selectedServiceTemplateKey = ref<NifflerServiceTemplateKey>(DEFAULT_NIFFLER_SERVICE_TEMPLATE_KEY)

const defaultServiceForm = (): CreateNifflerUpstreamServicePayload =>
  buildNifflerServiceFormFromTemplate(DEFAULT_NIFFLER_SERVICE_TEMPLATE_KEY)

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

const defaultErrorReturnSettingForm = (): ErrorReturnSettingForm => ({
  scope: 'platform',
  upstream_service_id: '__all__',
  match_status_code: null,
  match_text: '',
  handling_step: '',
  response_mode: 'replace',
  user_message: '',
  account_protection_action: 'record_only',
  pause_duration: '',
  is_active: true,
})

const serviceForm = ref<CreateNifflerUpstreamServicePayload>(defaultServiceForm())
const accountForm = ref<CreateNifflerUpstreamAccountPayload>(defaultAccountForm())
const productPlanForm = ref<ProductPlanForm>(defaultProductPlanForm())
const productPlanModelForm = ref<ProductPlanModelForm>(defaultProductPlanModelForm())
const errorReturnSettingForm = ref<ErrorReturnSettingForm>(defaultErrorReturnSettingForm())
let accountLoadSeq = 0
let productPlanModelLoadSeq = 0

const capabilityOptions: Array<{
  key: NifflerServiceCapabilityKey
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

const selectedServiceTemplate = computed(() =>
  getNifflerServiceTemplate(selectedServiceTemplateKey.value)
)

const visibleCapabilityOptions = computed(() =>
  filterCapabilityOptionsForProtocol(
    capabilityOptions,
    (serviceForm.value.protocol_kind || selectedServiceTemplate.value.protocolKind) as NifflerProtocolKind
  )
)

watch(serviceDialogOpen, (open) => {
  if (!open) {
    selectedServiceTemplateKey.value = DEFAULT_NIFFLER_SERVICE_TEMPLATE_KEY
    serviceForm.value = defaultServiceForm()
  }
})

watch(selectedServiceTemplateKey, (templateKey) => {
  serviceForm.value = buildNifflerServiceFormFromTemplate(templateKey, serviceForm.value)
})

watch(
  () => serviceForm.value.protocol_kind,
  (protocolKind) => {
    clearHiddenCapabilities((protocolKind || selectedServiceTemplate.value.protocolKind) as NifflerProtocolKind)
  }
)

watch(accountDialogOpen, (open) => {
  if (!open) {
    accountForm.value = defaultAccountForm()
    return
  }
  if (selectedService.value) {
    accountForm.value = {
      ...defaultAccountForm(),
      auth_kind: getDefaultAuthKindForService(selectedService.value),
    }
  }
})

function openServiceDialog() {
  selectedServiceTemplateKey.value = DEFAULT_NIFFLER_SERVICE_TEMPLATE_KEY
  serviceForm.value = defaultServiceForm()
  serviceDialogOpen.value = true
}

function clearHiddenCapabilities(protocolKind: NifflerProtocolKind) {
  const visibleKeys = new Set(
    filterCapabilityOptionsForProtocol(capabilityOptions, protocolKind).map(option => option.key)
  )
  const capabilities = serviceForm.value.capabilities ?? {}
  for (const option of capabilityOptions) {
    if (!visibleKeys.has(option.key)) {
      capabilities[option.key] = false
    }
  }
  serviceForm.value.capabilities = capabilities
}

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

watch(errorReturnSettingDialogOpen, (open) => {
  if (!open) {
    errorReturnSettingForm.value = defaultErrorReturnSettingForm()
  }
})

async function refreshAll() {
  await Promise.all([loadServices(), loadProductPlans(), loadErrorReturnSettings()])
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

async function loadErrorReturnSettings() {
  errorReturnSettingLoading.value = true
  errorReturnSettingError.value = ''
  try {
    const response = await listNifflerErrorReturnSettings({
      include_inactive: true,
      limit: 100,
    })
    errorReturnSettings.value = response.items
  } catch (err) {
    errorReturnSettingError.value = extractErrorMessage(err, '读取错误文案规则失败')
    showError(errorReturnSettingError.value)
  } finally {
    errorReturnSettingLoading.value = false
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

async function submitErrorReturnSetting() {
  const payload = normalizeErrorReturnSettingPayload(errorReturnSettingForm.value)
  if (!payload) return

  savingErrorReturnSetting.value = true
  try {
    await createNifflerErrorReturnSetting(payload)
    success('错误文案规则已保存')
    errorReturnSettingDialogOpen.value = false
    await loadErrorReturnSettings()
  } catch (err) {
    showError(extractErrorMessage(err, '保存错误文案规则失败'))
  } finally {
    savingErrorReturnSetting.value = false
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

function normalizeErrorReturnSettingPayload(
  form: ErrorReturnSettingForm
): CreateNifflerErrorReturnSettingPayload | null {
  const userMessage = form.user_message.trim()
  if (!userMessage) {
    showError('返回给用户的文案不能为空')
    return null
  }

  const rawStatusCode = form.match_status_code
  let matchStatusCode: number | null = null
  if (rawStatusCode !== null && rawStatusCode !== '') {
    const parsed = Number(rawStatusCode)
    if (!Number.isInteger(parsed) || parsed < 100 || parsed > 599) {
      showError('状态码必须是 100 到 599 之间的整数')
      return null
    }
    matchStatusCode = parsed
  }

  if (form.scope === 'upstream' && !form.handling_step) {
    showError('上游级规则必须选择处理类型')
    return null
  }

  if (
    form.scope === 'upstream'
    && form.account_protection_action === 'pause_scheduling'
    && !form.pause_duration
  ) {
    showError('暂停调度必须选择暂停时长')
    return null
  }

  return {
    scope: form.scope,
    upstream_service_id:
      form.scope === 'upstream' && form.upstream_service_id !== '__all__'
        ? form.upstream_service_id
        : null,
    match_status_code: matchStatusCode,
    match_text: emptyToNull(form.match_text),
    handling_step:
      form.scope === 'upstream'
        ? (form.handling_step as NifflerUpstreamErrorHandlingStep)
        : null,
    response_mode: form.response_mode,
    user_message: userMessage,
    account_protection_action:
      form.scope === 'upstream' ? form.account_protection_action : 'record_only',
    pause_duration:
      form.scope === 'upstream' && form.account_protection_action === 'pause_scheduling'
        ? (form.pause_duration as NifflerPauseDuration)
        : null,
    is_active: form.is_active,
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

function serviceKindLabel(value: string): string {
  return getServiceKindLabel(value)
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

function errorScopeLabel(scope: NifflerErrorResponseScope): string {
  const labels: Record<NifflerErrorResponseScope, string> = {
    platform: '平台本地错误',
    upstream: '上游返回错误',
  }
  return labels[scope] ?? scope
}

function upstreamServiceLabel(serviceId?: string | null): string {
  if (!serviceId) {
    return '全部上游'
  }
  return services.value.find(service => service.id === serviceId)?.display_name ?? '未知上游服务'
}

function matchLabel(rule: NifflerErrorReturnSetting): string {
  const parts: string[] = []
  if (rule.match_status_code) {
    parts.push(`状态码 ${rule.match_status_code}`)
  }
  if (rule.match_text) {
    parts.push(rule.scope === 'platform' ? `错误代码：${rule.match_text}` : `关键词：${rule.match_text}`)
  }
  return parts.length > 0 ? parts.join(' / ') : '默认规则'
}

function handlingStepLabel(step: NifflerUpstreamErrorHandlingStep): string {
  const labels: Record<NifflerUpstreamErrorHandlingStep, string> = {
    risk_keyword: '风控关键词',
    contact_or_marketing_replacement: '广告或客服内容替换',
    status_code_message: '状态码文案',
    default_upstream_message: '默认上游错误文案',
  }
  return labels[step] ?? step
}

function responseModeLabel(mode: NifflerUserResponseMode): string {
  const labels: Record<NifflerUserResponseMode, string> = {
    replace: '完全替换',
    append: '追加说明',
    redact: '部分脱敏',
  }
  return labels[mode] ?? mode
}

function protectionActionLabel(action: NifflerAccountProtectionAction): string {
  const labels: Record<NifflerAccountProtectionAction, string> = {
    record_only: '只记录',
    pause_scheduling: '暂停调度',
    disable_account: '停用账号',
  }
  return labels[action] ?? action
}

function pauseDurationLabel(duration: NifflerPauseDuration): string {
  const labels: Record<NifflerPauseDuration, string> = {
    ten_minutes: '10 分钟',
    one_hour: '1 小时',
    twenty_four_hours: '24 小时',
    manual_restore: '手动恢复',
  }
  return labels[duration] ?? duration
}

onMounted(() => {
  void loadServices()
  void loadProductPlans()
  void loadErrorReturnSettings()
})
</script>
