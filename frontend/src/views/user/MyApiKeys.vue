<template>
  <div class="space-y-6 pb-8">
    <!-- API Keys 表格 -->
    <Card
      variant="default"
      class="overflow-hidden"
    >
      <!-- 标题和操作栏 -->
      <div class="px-4 sm:px-6 py-3 sm:py-3.5 border-b border-border/60">
        <div class="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-3 sm:gap-4">
          <h3 class="text-sm sm:text-base font-semibold shrink-0">
            {{ t('apiKeys.title') }}
          </h3>

          <!-- 操作按钮 -->
          <div class="flex items-center gap-2">
            <!-- 新增 API Key 按钮 -->
            <Button
              variant="ghost"
              size="icon"
              class="h-8 w-8"
              :title="t('apiKeys.create')"
              @click="openCreateApiKeyDialog"
            >
              <Plus class="w-3.5 h-3.5" />
            </Button>

            <!-- 刷新按钮 -->
            <RefreshButton
              :loading="loading"
              @click="loadApiKeys"
            />
          </div>
        </div>
      </div>

      <!-- 加载状态 -->
      <div
        v-if="loading"
        class="flex items-center justify-center py-12"
      >
        <LoadingState :message="t('apiKeys.loading')" />
      </div>

      <!-- 空状态 -->
      <div
        v-else-if="apiKeys.length === 0"
        class="flex items-center justify-center py-12"
      >
        <EmptyState
          :title="t('apiKeys.empty')"
          :description="t('apiKeys.emptyHint')"
          :icon="Key"
        >
          <template #actions>
            <Button
              size="lg"
              class="shadow-lg shadow-primary/20"
              @click="openCreateApiKeyDialog"
            >
              <Plus class="mr-2 h-4 w-4" />
              {{ t('apiKeys.create') }}
            </Button>
          </template>
        </EmptyState>
      </div>

      <!-- 桌面端表格 -->
      <div
        v-else
        class="hidden md:block overflow-x-auto"
      >
        <Table>
          <TableHeader>
            <TableRow class="border-b border-border/60 hover:bg-transparent">
              <TableHead class="min-w-[200px] h-12 font-semibold">
                {{ t('apiKeys.name') }}
              </TableHead>
              <TableHead class="min-w-[160px] h-12 font-semibold">
                {{ t('apiKeys.key') }}
              </TableHead>
              <TableHead class="min-w-[100px] h-12 font-semibold">
                {{ t('apiKeys.cost') }}
              </TableHead>
              <TableHead class="min-w-[100px] h-12 font-semibold">
                {{ t('apiKeys.requests') }}
              </TableHead>
              <TableHead class="min-w-[70px] h-12 font-semibold text-center">
                {{ t('apiKeys.status') }}
              </TableHead>
              <TableHead class="min-w-[100px] h-12 font-semibold">
                {{ t('apiKeys.lastUsed') }}
              </TableHead>
              <TableHead class="min-w-[80px] h-12 font-semibold text-center">
                {{ t('apiKeys.actions') }}
              </TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            <TableRow
              v-for="apiKey in paginatedApiKeys"
              :key="apiKey.id"
              class="border-b border-border/40 hover:bg-muted/30 transition-colors"
            >
              <!-- 密钥名称 -->
              <TableCell class="py-4">
                <div class="flex-1 min-w-0">
                  <div
                    class="text-sm font-semibold truncate"
                    :title="apiKey.name"
                  >
                    {{ apiKey.name }}
                  </div>
                  <div class="text-xs text-muted-foreground mt-0.5">
                    {{ t('apiKeys.createdAt') }} {{ formatDate(apiKey.created_at) }}
                  </div>
                  <div class="text-xs text-muted-foreground mt-0.5">
                    {{ t('apiKeys.group') }}：{{ apiKey.group_name || t('apiKeys.defaultGroup') }}
                  </div>
                </div>
              </TableCell>

              <!-- 密钥显示 -->
              <TableCell class="py-4">
                <div class="flex items-center gap-1.5">
                  <code class="text-xs font-mono text-muted-foreground bg-muted/30 px-2 py-1 rounded">
                    {{ apiKey.key_display || 'sk-••••••••' }}
                  </code>
                  <Button
                    variant="ghost"
                    size="icon"
                    class="h-6 w-6"
                    :title="t('apiKeys.copy')"
                    @click="copyApiKey(apiKey)"
                  >
                    <Copy class="h-3.5 w-3.5" />
                  </Button>
                </div>
              </TableCell>

              <!-- 费用 -->
              <TableCell class="py-4">
                <span class="text-sm font-semibold text-amber-600 dark:text-amber-500">
                  ${{ (apiKey.total_cost_usd || 0).toFixed(4) }}
                </span>
              </TableCell>

              <!-- 请求次数 -->
              <TableCell class="py-4">
                <div class="flex items-center gap-1.5">
                  <Activity class="h-3.5 w-3.5 text-muted-foreground" />
                  <span class="text-sm font-medium text-foreground">
                    {{ formatNumber(apiKey.total_requests || 0) }}
                  </span>
                </div>
              </TableCell>

              <!-- 状态 -->
              <TableCell class="py-4 text-center">
                <div class="flex flex-col items-center gap-1">
                  <Badge
                    :variant="apiKey.is_active ? 'success' : 'secondary'"
                    class="h-5 px-2 py-0 text-[10px] font-medium"
                  >
                    {{ apiKey.is_active ? t('apiKeys.active') : t('apiKeys.disabled') }}
                  </Badge>
                  <Badge
                    v-if="apiKey.is_locked"
                    variant="warning"
                    class="h-5 px-2 py-0 text-[10px] font-medium"
                  >
                    {{ t('apiKeys.locked') }}
                  </Badge>
                  <Badge
                    variant="secondary"
                    class="h-5 px-2 py-0 text-[10px] font-medium"
                  >
                    {{ formatRateLimitSimple(apiKey.rate_limit) }}
                  </Badge>
                  <Badge
                    variant="secondary"
                    class="h-5 px-2 py-0 text-[10px] font-medium"
                  >
                    {{ formatConcurrentLimitSimple(apiKey.concurrent_limit) }}
                  </Badge>
                </div>
              </TableCell>

              <!-- 最后使用时间 -->
              <TableCell class="py-4 text-sm text-muted-foreground">
                {{ apiKey.last_used_at ? formatRelativeTime(apiKey.last_used_at) : t('apiKeys.neverUsed') }}
              </TableCell>

              <!-- 操作按钮 -->
              <TableCell class="py-4">
                <div class="flex justify-center gap-1">
                  <Button
                    variant="ghost"
                    size="icon"
                    class="h-8 w-8"
                    :title="t('apiKeys.oneClick')"
                    @click="openInstallDialog(apiKey)"
                  >
                    <Terminal class="h-4 w-4" />
                  </Button>
                  <Button
                    variant="ghost"
                    size="icon"
                    class="h-8 w-8"
                    :title="apiKey.is_locked ? t('apiKeys.locked') : t('apiKeys.importCc')"
                    :disabled="apiKey.is_locked"
                    @click="openCcSwitchDialog(apiKey)"
                  >
                    <ExternalLink class="h-4 w-4" />
                  </Button>
                  <Button
                    variant="ghost"
                    size="icon"
                    class="h-8 w-8"
                    :title="apiKey.is_locked ? t('apiKeys.locked') : t('apiKeys.edit')"
                    :disabled="apiKey.is_locked"
                    @click="openEditApiKeyDialog(apiKey)"
                  >
                    <SquarePen class="h-4 w-4" />
                  </Button>
                  <Button
                    variant="ghost"
                    size="icon"
                    class="h-8 w-8"
                    :title="apiKey.is_locked ? t('apiKeys.locked') : (apiKey.is_active ? t('apiKeys.disabled') : t('apiKeys.active'))"
                    :disabled="apiKey.is_locked"
                    @click="toggleApiKey(apiKey)"
                  >
                    <Power class="h-4 w-4" />
                  </Button>
                  <Button
                    variant="ghost"
                    size="icon"
                    class="h-8 w-8"
                    :title="apiKey.is_locked ? t('apiKeys.locked') : t('apiKeys.delete')"
                    :disabled="apiKey.is_locked"
                    @click="confirmDelete(apiKey)"
                  >
                    <Trash2 class="h-4 w-4" />
                  </Button>
                </div>
              </TableCell>
            </TableRow>
          </TableBody>
        </Table>
      </div>

      <!-- 移动端卡片列表 -->
      <div
        v-if="!loading && apiKeys.length > 0"
        class="md:hidden space-y-3 overflow-x-auto p-4"
      >
        <Card
          v-for="apiKey in paginatedApiKeys"
          :key="apiKey.id"
          variant="default"
          class="group min-w-[560px] hover:shadow-md hover:border-primary/30 transition-all duration-200"
        >
          <div class="p-4">
            <!-- 第一行：名称、状态、操作 -->
            <div class="flex items-center justify-between mb-2">
              <div class="flex items-center gap-2 min-w-0 flex-1">
                <h3 class="text-sm font-semibold text-foreground truncate">
                  {{ apiKey.name }}
                </h3>
                <Badge
                  :variant="apiKey.is_active ? 'success' : 'secondary'"
                  class="text-xs px-1.5 py-0"
                >
                  {{ apiKey.is_active ? t('apiKeys.active') : t('apiKeys.disabled') }}
                </Badge>
                <Badge
                  v-if="apiKey.is_locked"
                  variant="warning"
                  class="text-[10px] px-1.5 py-0"
                >
                  {{ t('apiKeys.locked') }}
                </Badge>
                <Badge
                  variant="secondary"
                  class="text-[10px] px-1.5 py-0"
                >
                  {{ formatRateLimitSimple(apiKey.rate_limit) }}
                </Badge>
                <Badge
                  variant="secondary"
                  class="text-[10px] px-1.5 py-0"
                >
                  {{ formatConcurrentLimitSimple(apiKey.concurrent_limit) }}
                </Badge>
              </div>
              <div class="flex items-center gap-0.5 flex-shrink-0">
                <Button
                  variant="ghost"
                  size="icon"
                  class="h-7 w-7"
                  :title="t('apiKeys.oneClick')"
                  @click="openInstallDialog(apiKey)"
                >
                  <Terminal class="h-3.5 w-3.5" />
                </Button>
                <Button
                  variant="ghost"
                  size="icon"
                  class="h-7 w-7"
                  :title="apiKey.is_locked ? t('apiKeys.locked') : t('apiKeys.importCc')"
                  :disabled="apiKey.is_locked"
                  @click="openCcSwitchDialog(apiKey)"
                >
                  <ExternalLink class="h-3.5 w-3.5" />
                </Button>
                <Button
                  variant="ghost"
                  size="icon"
                  class="h-7 w-7"
                  :title="apiKey.is_locked ? t('apiKeys.locked') : t('apiKeys.edit')"
                  :disabled="apiKey.is_locked"
                  @click="openEditApiKeyDialog(apiKey)"
                >
                  <SquarePen class="h-3.5 w-3.5" />
                </Button>
                <Button
                  variant="ghost"
                  size="icon"
                  class="h-7 w-7"
                  :title="t('apiKeys.copy')"
                  @click="copyApiKey(apiKey)"
                >
                  <Copy class="h-3.5 w-3.5" />
                </Button>
                <Button
                  variant="ghost"
                  size="icon"
                  class="h-7 w-7"
                  :title="apiKey.is_locked ? t('apiKeys.locked') : (apiKey.is_active ? t('apiKeys.disabled') : t('apiKeys.active'))"
                  :disabled="apiKey.is_locked"
                  @click="toggleApiKey(apiKey)"
                >
                  <Power class="h-3.5 w-3.5" />
                </Button>
                <Button
                  variant="ghost"
                  size="icon"
                  class="h-7 w-7"
                  :title="apiKey.is_locked ? t('apiKeys.locked') : t('apiKeys.delete')"
                  :disabled="apiKey.is_locked"
                  @click="confirmDelete(apiKey)"
                >
                  <Trash2 class="h-3.5 w-3.5" />
                </Button>
              </div>
            </div>

            <!-- 第二行：密钥、时间、统计 -->
            <div class="space-y-1.5">
              <div class="flex items-center gap-2 text-xs">
                <code class="font-mono text-muted-foreground">{{ apiKey.key_display || 'sk-••••••••' }}</code>
                <span class="text-muted-foreground">•</span>
                <span class="text-muted-foreground">
                  {{ apiKey.last_used_at ? formatRelativeTime(apiKey.last_used_at) : t('apiKeys.neverUsed') }}
                </span>
              </div>
              <div class="text-xs text-muted-foreground">
                {{ t('apiKeys.group') }}：{{ apiKey.group_name || t('apiKeys.defaultGroup') }}
              </div>
              <div class="flex items-center gap-3 text-xs">
                <span class="text-amber-600 dark:text-amber-500 font-semibold">
                  ${{ (apiKey.total_cost_usd || 0).toFixed(4) }}
                </span>
                <span class="text-muted-foreground">•</span>
                <span class="text-foreground font-medium">
                  {{ t('apiKeys.requestCount', { count: formatNumber(apiKey.total_requests || 0) }) }}
                </span>
                <span class="text-muted-foreground">•</span>
                <span class="text-muted-foreground">
                  {{ formatRateLimitSimple(apiKey.rate_limit) }}
                </span>
              </div>
            </div>
          </div>
        </Card>
      </div>

      <!-- 分页 -->
      <Pagination
        v-if="apiKeys.length > 0"
        :current="currentPage"
        :total="apiKeys.length"
        :page-size="pageSize"
        cache-key="my-api-keys-page-size"
        @update:current="currentPage = $event"
        @update:page-size="pageSize = $event"
      />
    </Card>

    <!-- 创建 API 密钥对话框 -->
    <Dialog v-model="showCreateDialog">
      <template #header>
        <div class="border-b border-border px-6 py-4">
          <div class="flex items-center gap-3">
            <div class="flex h-9 w-9 items-center justify-center rounded-lg bg-primary/10 flex-shrink-0">
              <Key class="h-5 w-5 text-primary" />
            </div>
            <div class="flex-1 min-w-0">
              <h3 class="text-lg font-semibold text-foreground leading-tight">
                {{ editingApiKey ? t('apiKeys.edit') : t('apiKeys.createTitle') }}
              </h3>
            </div>
          </div>
        </div>
      </template>

      <div class="space-y-4">
        <div class="space-y-2">
          <Label
            for="key-name"
            class="text-sm font-semibold"
          >{{ t('apiKeys.name') }}</Label>
          <Input
            id="key-name"
            v-model="newKeyName"
            :placeholder="t('apiKeys.name')"
            class="h-11 border-border/60"
            autocomplete="off"
            required
          />
          <p class="text-xs text-muted-foreground">
            {{ t('apiKeys.nameHint') }}
          </p>
        </div>

        <div class="space-y-2">
          <Label
            for="key-rate-limit"
            class="text-sm font-semibold"
          >{{ t('apiKeys.rateLimit') }}</Label>
          <Input
            id="key-rate-limit"
            :model-value="newKeyRateLimit ?? ''"
            type="number"
            min="0"
            max="10000"
            :placeholder="t('apiKeys.leaveUnlimited')"
            class="h-11 border-border/60"
            @update:model-value="(v) => newKeyRateLimit = parseNumberInput(v, { min: 0, max: 10000 })"
          />
          <p class="text-xs text-muted-foreground">
            {{ t('apiKeys.leaveUnlimited') }}
          </p>
        </div>

        <div class="space-y-2">
          <Label
            for="key-group"
            class="text-sm font-semibold"
          >{{ t('apiKeys.group') }}</Label>
          <select
            id="key-group"
            v-model="selectedGroupId"
            class="h-11 w-full rounded-md border border-border/60 bg-background px-3 text-sm"
            :disabled="apiKeyGroups.length === 0"
          >
            <option
              v-if="apiKeyGroups.length === 0"
              value=""
            >
              {{ t('apiKeys.groupEmpty') }}
            </option>
            <option
              v-for="group in apiKeyGroups"
              :key="group.id"
              :value="group.id"
            >
              {{ group.name }}{{ group.visibility === 'internal' ? t('apiKeys.internalGroup') : '' }}
            </option>
          </select>
          <p class="text-xs text-muted-foreground">
            {{ t('apiKeys.groupDetailHint') }}
          </p>
        </div>

        <div class="space-y-2">
          <Label
            for="key-concurrent-limit"
            class="text-sm font-semibold"
          >{{ t('apiKeys.concurrencyLimit') }}</Label>
          <Input
            id="key-concurrent-limit"
            :model-value="newKeyConcurrentLimit ?? ''"
            type="number"
            min="0"
            max="10000"
            :placeholder="t('apiKeys.concurrencyPlaceholder')"
            class="h-11 border-border/60"
            @update:model-value="(v) => newKeyConcurrentLimit = parseNumberInput(v, { min: 0, max: 10000 })"
          />
          <p class="text-xs text-muted-foreground">
            {{ editingApiKey ? t('apiKeys.editConcurrencyHint') : t('apiKeys.createConcurrencyHint') }}
          </p>
        </div>

        <div class="rounded-lg border border-border/60 bg-muted/30 p-4">
          <div class="flex items-center justify-between gap-4">
            <div>
              <Label class="text-sm font-semibold">{{ t('apiKeys.piiProtection') }}</Label>
              <p class="mt-1 text-xs text-muted-foreground">
                {{ keyRedactionMode === 'inherit' ? t('apiKeys.followAccountDefault') : t('apiKeys.adminFeatureRequired') }}
              </p>
            </div>
            <div class="flex items-center gap-2">
              <Button
                size="sm"
                :variant="keyRedactionMode === 'inherit' ? 'default' : 'outline'"
                @click="keyRedactionMode = 'inherit'"
              >
                {{ t('apiKeys.followAccount') }}
              </Button>
              <Button
                size="sm"
                :variant="keyRedactionMode === 'custom' ? 'default' : 'outline'"
                @click="keyRedactionMode = 'custom'"
              >
                {{ t('apiKeys.customConfig') }}
              </Button>
            </div>
          </div>
          <div
            v-if="keyRedactionMode === 'custom'"
            class="mt-4 flex items-center justify-between gap-4 border-t border-border/50 pt-4"
          >
            <div>
              <Label class="text-sm font-medium">{{ t('apiKeys.enableProtection') }}</Label>
              <p class="mt-1 text-xs text-muted-foreground">
                {{ t('apiKeys.onlyThisKey') }}
              </p>
            </div>
            <Switch v-model="newKeyRedactionEnabled" />
          </div>
          <div
            v-if="keyRedactionMode === 'custom' && newKeyRedactionEnabled"
            class="mt-4 flex items-center justify-between gap-4 border-t border-border/50 pt-4"
          >
            <div>
              <Label class="text-sm font-medium">{{ t('apiKeys.placeholderNotice') }}</Label>
              <p class="mt-1 text-xs text-muted-foreground">
                {{ t('apiKeys.placeholderNoticeHint') }}
              </p>
            </div>
            <Switch v-model="newKeyRedactionInjectNotice" />
          </div>
        </div>
      </div>

      <template #footer>
        <Button
          variant="outline"
          class="h-11 px-6"
          @click="closeApiKeyDialog"
        >
          {{ t('apiKeys.cancel') }}
        </Button>
        <Button
          class="h-11 px-6 shadow-lg shadow-primary/20"
          :disabled="creating || apiKeyGroups.length === 0"
          @click="saveApiKey"
        >
          <Loader2
            v-if="creating"
            class="animate-spin h-4 w-4 mr-2"
          />
          {{ creating ? (editingApiKey ? t('apiKeys.saving') : t('apiKeys.creating')) : (editingApiKey ? t('apiKeys.save') : t('apiKeys.create')) }}
        </Button>
      </template>
    </Dialog>

    <!-- 新密钥创建成功对话框 -->
    <Dialog
      v-model="showKeyDialog"
      size="lg"
    >
      <template #header>
        <div class="border-b border-border px-6 py-4">
          <div class="flex items-center gap-3">
            <div class="flex h-9 w-9 items-center justify-center rounded-lg bg-emerald-100 dark:bg-emerald-900/30 flex-shrink-0">
              <CheckCircle class="h-5 w-5 text-emerald-600 dark:text-emerald-400" />
            </div>
            <div class="flex-1 min-w-0">
              <h3 class="text-lg font-semibold text-foreground leading-tight">
                {{ t('apiKeys.success') }}
              </h3>
              <p class="text-xs text-muted-foreground">
                {{ t('apiKeys.keepSecret') }}
              </p>
            </div>
          </div>
        </div>
      </template>

      <div class="space-y-4">
        <div class="space-y-2">
          <Label class="text-sm font-medium">{{ t('apiKeys.key') }}</Label>
          <div class="flex items-center gap-2">
            <Input
              type="text"
              :value="newKeyValue"
              readonly
              class="flex-1 font-mono text-sm bg-muted/50 h-11"
              @click="($event.target as HTMLInputElement)?.select()"
            />
            <Button
              class="h-11"
              @click="copyTextToClipboard(newKeyValue)"
            >
              {{ t('apiKeys.copy') }}
            </Button>
          </div>
        </div>
      </div>

      <template #footer>
        <Button
          class="h-10 px-5"
          @click="closeCreatedKeyDialog"
        >
          {{ t('apiKeys.confirm') }}
        </Button>
      </template>
    </Dialog>

    <!-- 接入方式选择对话框 -->
    <Dialog
      v-model="showSetupChoiceDialog"
      size="lg"
    >
      <template #header>
        <div class="border-b border-border px-6 py-4">
          <div class="flex items-center gap-3">
            <div class="flex h-9 w-9 items-center justify-center rounded-lg bg-primary/10 flex-shrink-0">
              <Key class="h-5 w-5 text-primary" />
            </div>
            <div class="flex-1 min-w-0">
              <h3 class="text-lg font-semibold text-foreground leading-tight">
                {{ t('apiKeys.setupTitle') }}
              </h3>
              <p class="text-xs text-muted-foreground truncate">
                {{ t('apiKeys.key') }}: {{ selectedSetupApiKey?.name || t('apiKeys.unselected') }}
              </p>
            </div>
          </div>
        </div>
      </template>

      <div class="space-y-4">
        <div class="rounded-lg border border-border/60 bg-muted/30 p-3 text-xs text-muted-foreground">
          {{ t('apiKeys.setupChoiceHint') }}
        </div>

        <div class="grid grid-cols-1 sm:grid-cols-2 gap-3">
          <button
            type="button"
            class="group rounded-xl border border-primary/45 bg-primary/10 p-4 text-left transition hover:border-primary hover:bg-primary/15 focus:outline-none focus:ring-2 focus:ring-primary/35"
            @click="chooseSetupCcSwitch"
          >
            <span class="mb-3 inline-flex items-center rounded-full bg-primary px-2.5 py-1 text-[11px] font-semibold text-primary-foreground">
              {{ t('apiKeys.recommended') }}
            </span>
            <span class="flex items-center gap-2 text-base font-semibold text-foreground">
              <ExternalLink class="h-4 w-4 text-primary" />
              {{ t('apiKeys.importCc') }}
            </span>
            <span class="mt-2 block text-sm leading-6 text-muted-foreground">
              {{ t('apiKeys.ccSwitchChoiceHint') }}
            </span>
          </button>

          <button
            type="button"
            class="group rounded-xl border border-border/70 bg-background p-4 text-left transition hover:border-primary/70 hover:bg-muted/30 focus:outline-none focus:ring-2 focus:ring-primary/25"
            @click="chooseSetupInstall"
          >
            <span class="mb-3 inline-flex items-center rounded-full border border-border px-2.5 py-1 text-[11px] font-semibold text-muted-foreground">
              {{ t('apiKeys.commandSetup') }}
            </span>
            <span class="flex items-center gap-2 text-base font-semibold text-foreground">
              <Terminal class="h-4 w-4 text-primary" />
              {{ t('apiKeys.oneClick') }}
            </span>
            <span class="mt-2 block text-sm leading-6 text-muted-foreground">
              {{ t('apiKeys.commandChoiceHint') }}
            </span>
          </button>
        </div>
      </div>

      <template #footer>
        <Button
          variant="outline"
          class="h-10 px-5"
          @click="showSetupChoiceDialog = false"
        >
          {{ t('apiKeys.later') }}
        </Button>
      </template>
    </Dialog>

    <!-- 一键配置对话框 -->
    <Dialog
      v-model="showInstallDialog"
      size="lg"
    >
      <template #header>
        <div class="border-b border-border px-6 py-4">
          <div class="flex items-center gap-3">
            <div class="flex h-9 w-9 items-center justify-center rounded-lg bg-primary/10 flex-shrink-0">
              <Terminal class="h-5 w-5 text-primary" />
            </div>
            <div class="flex-1 min-w-0">
              <h3 class="text-lg font-semibold text-foreground leading-tight">
                {{ t('apiKeys.oneClick') }}
              </h3>
              <p class="text-xs text-muted-foreground truncate">
                {{ t('apiKeys.key') }}: {{ selectedInstallApiKey?.name || t('apiKeys.unselected') }}
              </p>
            </div>
          </div>
        </div>
      </template>

      <div class="space-y-5">
        <div class="rounded-lg border border-border/60 bg-muted/30 p-3 text-xs text-muted-foreground">
          {{ t('apiKeys.installHint') }}
        </div>

        <div class="space-y-2">
          <Label class="text-sm font-semibold">{{ t('apiKeys.targetTool') }}</Label>
          <div class="grid grid-cols-1 sm:grid-cols-3 gap-2">
            <Button
              v-for="option in installCliOptions"
              :key="option.value"
              :variant="installCli === option.value ? 'default' : 'outline'"
              class="justify-start h-auto py-3"
              @click="selectInstallCli(option.value)"
            >
              {{ option.label }}
            </Button>
          </div>
        </div>

        <div class="space-y-2">
          <Label class="text-sm font-semibold">{{ t('apiKeys.targetSystem') }}</Label>
          <div class="grid grid-cols-1 sm:grid-cols-3 gap-2">
            <Button
              v-for="option in installSystemOptions"
              :key="option.value"
              :variant="installSystem === option.value ? 'default' : 'outline'"
              class="justify-start h-auto py-3"
              @click="selectInstallSystem(option.value)"
            >
              {{ option.label }}
            </Button>
          </div>
        </div>

        <div class="space-y-2">
          <div class="flex items-center justify-between gap-2">
            <Label class="text-sm font-semibold">{{ t('apiKeys.executeHint') }}</Label>
            <div class="flex items-center gap-2">
              <Button
                variant="outline"
                size="sm"
                class="gap-1.5"
                :disabled="installLoading || !installCommand"
                :title="installCopied ? t('apiKeys.copied') : t('apiKeys.copyInstall')"
                @click="copyInstallCommand"
              >
                <CheckCircle
                  v-if="installCopied"
                  class="h-3.5 w-3.5 text-emerald-600 dark:text-emerald-400"
                />
                <Copy
                  v-else
                  class="h-3.5 w-3.5"
                />
                {{ installCopied ? t('apiKeys.copied') : t('apiKeys.copyOnce') }}
              </Button>
              <Button
                variant="ghost"
                size="sm"
                :disabled="installLoading || !selectedInstallApiKey"
                @click="refreshInstallCommand"
              >
                {{ installLoading ? t('apiKeys.generating') : t('apiKeys.regenerate') }}
              </Button>
            </div>
          </div>
          <div class="rounded-lg border border-border/60 bg-background overflow-hidden">
            <pre class="max-h-32 overflow-x-auto whitespace-pre-wrap break-all p-3 text-xs font-mono">{{ installCommand || t('apiKeys.generatingCommand') }}</pre>
          </div>
          <p class="text-xs text-muted-foreground">
            {{ installCommandHint }}
          </p>
        </div>
      </div>

      <template #footer>
        <Button
          variant="outline"
          class="h-10 px-5"
          @click="showInstallDialog = false"
        >
          {{ t('apiKeys.close') }}
        </Button>
        <Button
          class="h-10 px-5 shadow-lg shadow-primary/20"
          :disabled="!installCommand || installLoading"
          @click="copyInstallCommand"
        >
          {{ installCopied ? t('apiKeys.copied') : t('apiKeys.copyCommand') }}
        </Button>
      </template>
    </Dialog>

    <!-- 导入 CC Switch 对话框 -->
    <Dialog
      v-model="showCcSwitchDialog"
      size="lg"
    >
      <template #header>
        <div class="border-b border-border px-6 py-4">
          <div class="flex items-center gap-3">
            <div class="flex h-9 w-9 items-center justify-center rounded-lg bg-primary/10 flex-shrink-0">
              <ExternalLink class="h-5 w-5 text-primary" />
            </div>
            <div class="flex-1 min-w-0">
              <h3 class="text-lg font-semibold text-foreground leading-tight">
                {{ t('apiKeys.importCc') }}
              </h3>
              <p class="text-xs text-muted-foreground truncate">
                {{ t('apiKeys.key') }}: {{ selectedCcSwitchApiKey?.name || t('apiKeys.unselected') }}
              </p>
            </div>
          </div>
        </div>
      </template>

      <div class="space-y-5">
        <div class="rounded-lg border border-border/60 bg-muted/30 p-3 text-xs text-muted-foreground">
          {{ t('apiKeys.ccSwitchAddressHint') }}
        </div>

        <div class="space-y-2">
          <Label class="text-sm font-semibold">{{ t('apiKeys.importTo') }}</Label>
          <div class="grid grid-cols-1 sm:grid-cols-3 gap-2">
            <Button
              v-for="option in ccSwitchAppOptions"
              :key="option.value"
              :variant="ccSwitchApp === option.value ? 'default' : 'outline'"
              class="justify-start h-auto py-3"
              @click="selectCcSwitchApp(option.value)"
            >
              <span class="flex flex-col items-start gap-0.5 text-left">
                <span>{{ option.label }}</span>
                <span class="text-xs opacity-70 font-normal">{{ option.description }}</span>
              </span>
            </Button>
          </div>
        </div>

        <div class="space-y-2">
          <Label
            for="ccswitch-provider-name"
            class="text-sm font-semibold"
          >{{ t('apiKeys.serviceName') }}</Label>
          <Input
            id="ccswitch-provider-name"
            v-model="ccSwitchProviderName"
            placeholder="Niffler"
            class="h-11 border-border/60"
            autocomplete="off"
          />
          <p class="text-xs text-muted-foreground">
            {{ t('apiKeys.ccSwitchNameHint') }}
          </p>
        </div>

        <div class="space-y-2">
          <Label
            for="ccswitch-model"
            class="text-sm font-semibold"
          >{{ t('apiKeys.primaryModel') }}</Label>
          <Input
            id="ccswitch-model"
            v-model="ccSwitchModel"
            :placeholder="t('apiKeys.modelExample')"
            class="h-11 border-border/60"
            autocomplete="off"
          />
          <p
            v-if="ccSwitchApp === 'codex'"
            class="text-xs text-muted-foreground"
          >
            {{ t('apiKeys.codexDefaultHint', { model: DEFAULT_CCSWITCH_CODEX_MODEL, effort: DEFAULT_CCSWITCH_CODEX_REASONING_EFFORT }) }}
          </p>
          <p
            v-else
            class="text-xs text-muted-foreground"
          >
            {{ t('apiKeys.balanceModelHint') }}
          </p>
        </div>

        <div class="rounded-lg border border-border/60 bg-background overflow-hidden">
          <div class="border-b border-border/60 px-3 py-2 text-xs font-semibold text-muted-foreground">
            {{ t('apiKeys.importEndpoint') }}
          </div>
          <pre class="max-h-24 overflow-x-auto whitespace-pre-wrap break-all p-3 text-xs font-mono">{{ ccSwitchEndpointPreview }}</pre>
        </div>

        <p class="text-xs text-muted-foreground">
          {{ t('apiKeys.importSecurityHint') }}
        </p>
      </div>

      <template #footer>
        <Button
          variant="outline"
          class="h-10 px-5"
          @click="showCcSwitchDialog = false"
        >
          {{ t('apiKeys.cancel') }}
        </Button>
        <Button
          class="h-10 px-5 shadow-lg shadow-primary/20"
          :disabled="ccSwitchImportLoading || !selectedCcSwitchApiKey"
          @click="importToCcSwitch"
        >
          <Loader2
            v-if="ccSwitchImportLoading"
            class="animate-spin h-4 w-4 mr-2"
          />
          {{ ccSwitchImportLoading ? t('apiKeys.preparing') : t('apiKeys.import') }}
        </Button>
      </template>
    </Dialog>

    <!-- 删除确认对话框 -->
    <AlertDialog
      v-model="showDeleteDialog"
      type="danger"
      :title="t('apiKeys.deleteTitle')"
      :description="t('apiKeys.deleteConfirm', { name: keyToDelete?.name || '' })"
      :confirm-text="t('apiKeys.delete')"
      :loading="deleting"
      @confirm="deleteApiKey"
      @cancel="showDeleteDialog = false"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onBeforeUnmount, computed, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { meApi, type ApiKey, type ApiKeyGroupOption, type InstallSessionTargetSystem, type InstallTargetCli, type ApiKeyInstallSession } from '@/api/me'
import Card from '@/components/ui/card.vue'
import Button from '@/components/ui/button.vue'
import Input from '@/components/ui/input.vue'
import Label from '@/components/ui/label.vue'
import Badge from '@/components/ui/badge.vue'
import Switch from '@/components/ui/switch.vue'
import { Dialog, Pagination } from '@/components/ui'
import { LoadingState, AlertDialog, EmptyState } from '@/components/common'
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow
} from '@/components/ui'
import RefreshButton from '@/components/ui/refresh-button.vue'
import { Plus, Key, Copy, Trash2, Loader2, Activity, CheckCircle, Power, SquarePen, Terminal, ExternalLink } from 'lucide-vue-next'
import { useToast } from '@/composables/useToast'
import { log } from '@/utils/logger'
import { parseApiError } from '@/utils/errorParser'
import { formatRateLimitSimple } from '@/utils/format'
import { parseNumberInput } from '@/utils/form'
import { getApiBaseOrigin } from '@/utils/url'
import { getErrorStatus } from '@/types/api-error'
import {
  buildCcSwitchImportUrl,
  ccSwitchEndpoint,
  DEFAULT_CCSWITCH_CODEX_MODEL,
  DEFAULT_CCSWITCH_CODEX_REASONING_EFFORT,
  type CcSwitchApp,
} from '@/features/api-keys/utils/ccswitchImport'
import {
  hasChatPiiRedactionFeatureSettings,
  mergeChatPiiRedactionFeatureSettings,
  readChatPiiRedactionFeatureSettings,
} from '@/utils/featureSettings'

const { t, locale } = useI18n()
const { success, error: showError } = useToast()

const installCliOptions: Array<{ value: InstallTargetCli; label: string }> = [
  { value: 'claude_code', label: 'Claude Code' },
  { value: 'codex_cli', label: 'Codex' },
  { value: 'gemini_cli', label: 'Gemini CLI' }
]

const installSystemOptions: Array<{ value: InstallSessionTargetSystem; label: string }> = [
  { value: 'macos', label: 'macOS' },
  { value: 'linux', label: 'Linux' },
  { value: 'windows', label: 'Windows' }
]

const ccSwitchAppOptions = computed<Array<{ value: CcSwitchApp; label: string; description: string }>>(() => [
  { value: 'claude', label: 'Claude Code', description: t('apiKeys.rootAddress') },
  { value: 'codex', label: 'Codex', description: t('apiKeys.appendV1') },
  { value: 'gemini', label: 'Gemini CLI', description: t('apiKeys.rootAddress') },
])

const apiKeys = ref<ApiKey[]>([])
const apiKeyGroups = ref<ApiKeyGroupOption[]>([])
const loading = ref(false)
const creating = ref(false)
const deleting = ref(false)

// 分页相关
const currentPage = ref(1)
const pageSize = ref(10)

const paginatedApiKeys = computed(() => {
  const start = (currentPage.value - 1) * pageSize.value
  return apiKeys.value.slice(start, start + pageSize.value)
})

const showCreateDialog = ref(false)
const showKeyDialog = ref(false)
const showDeleteDialog = ref(false)
const showSetupChoiceDialog = ref(false)
const showInstallDialog = ref(false)
const showCcSwitchDialog = ref(false)

const newKeyName = ref('')
const selectedGroupId = ref('')
const newKeyRateLimit = ref<number | undefined>(undefined)
const newKeyConcurrentLimit = ref<number | undefined>(undefined)
const keyRedactionMode = ref<'inherit' | 'custom'>('inherit')
const newKeyRedactionEnabled = ref(false)
const newKeyRedactionInjectNotice = ref(true)
const newKeyValue = ref('')
const keyToDelete = ref<ApiKey | null>(null)
const editingApiKey = ref<ApiKey | null>(null)
const selectedSetupApiKey = ref<ApiKey | null>(null)
const selectedInstallApiKey = ref<ApiKey | null>(null)
const pendingSetupApiKey = ref<ApiKey | null>(null)
const installCli = ref<InstallTargetCli>('claude_code')
const installSystem = ref<InstallSessionTargetSystem>('linux')
const installSession = ref<ApiKeyInstallSession | null>(null)
const installLoading = ref(false)
const installCopied = ref(false)
const selectedCcSwitchApiKey = ref<ApiKey | null>(null)
const ccSwitchApp = ref<CcSwitchApp>('claude')
const ccSwitchProviderName = ref('Niffler')
const ccSwitchModel = ref('')
const ccSwitchImportLoading = ref(false)
let installCopiedResetTimer: ReturnType<typeof setTimeout> | null = null

const installCommand = computed(() => {
  if (!installSession.value) return ''
  return installSystem.value === 'windows'
    ? installSession.value.powershell_command
    : installSession.value.unix_command
})

const installCommandHint = computed(() => {
  if (installSystem.value === 'windows') {
    return t('apiKeys.windowsInstallHint')
  }
  return t('apiKeys.unixInstallHint')
})

const ccSwitchBaseUrl = ref(getApiBaseOrigin())

const ccSwitchEndpointPreview = computed(() => {
  return ccSwitchEndpoint(ccSwitchApp.value, ccSwitchBaseUrl.value)
})

onMounted(() => {
  installSystem.value = detectCurrentSystem()
  loadApiKeyGroups()
  loadApiKeys()
})

onBeforeUnmount(() => {
  resetInstallCopiedState()
})

watch(showInstallDialog, (isOpen) => {
  if (!isOpen) {
    resetInstallCopiedState()
  }
})

watch(showKeyDialog, (isOpen) => {
  if (!isOpen && pendingSetupApiKey.value) {
    closeCreatedKeyDialog()
  }
})

async function loadApiKeys() {
  loading.value = true
  try {
    apiKeys.value = await meApi.getApiKeys()
  } catch (error: unknown) {
    log.error('加载 API 密钥失败:', error)
    const status = getErrorStatus(error)
    if (status === undefined) {
      showError(t('apiKeys.serverUnavailable'))
    } else if (status === 401) {
      showError(t('apiKeys.authFailed'))
    } else {
      showError(parseApiError(error, t('apiKeys.loadFailed')))
    }
  } finally {
    loading.value = false
  }
}

async function loadApiKeyGroups() {
  try {
    apiKeyGroups.value = await meApi.getApiKeyGroups()
    if (!selectedGroupId.value && apiKeyGroups.value.length > 0) {
      selectedGroupId.value = apiKeyGroups.value[0].id
    }
  } catch (error: unknown) {
    log.error('加载 API Key 分组失败:', error)
    showError(parseApiError(error, t('apiKeys.loadGroupsFailed')))
  }
}

function clearInstallCopiedResetTimer() {
  if (installCopiedResetTimer) {
    clearTimeout(installCopiedResetTimer)
    installCopiedResetTimer = null
  }
}

function resetInstallCopiedState() {
  clearInstallCopiedResetTimer()
  installCopied.value = false
}

function openEditApiKeyDialog(apiKey: ApiKey) {
  const hasRedactionFeature = hasChatPiiRedactionFeatureSettings(apiKey.feature_settings)
  const redactionFeature = readChatPiiRedactionFeatureSettings(apiKey.feature_settings)
  editingApiKey.value = apiKey
  newKeyName.value = apiKey.name || ''
  selectedGroupId.value = apiKey.group_id || apiKeyGroups.value[0]?.id || ''
  newKeyRateLimit.value = apiKey.rate_limit ?? undefined
  newKeyConcurrentLimit.value = apiKey.concurrent_limit ?? undefined
  keyRedactionMode.value = hasRedactionFeature ? 'custom' : 'inherit'
  newKeyRedactionEnabled.value = redactionFeature.enabled
  newKeyRedactionInjectNotice.value = redactionFeature.inject_model_instruction
  showCreateDialog.value = true
}

function openCreateApiKeyDialog() {
  editingApiKey.value = null
  newKeyName.value = ''
  selectedGroupId.value = apiKeyGroups.value[0]?.id || ''
  newKeyRateLimit.value = undefined
  newKeyConcurrentLimit.value = undefined
  keyRedactionMode.value = 'inherit'
  newKeyRedactionEnabled.value = false
  newKeyRedactionInjectNotice.value = true
  showCreateDialog.value = true
}

function detectCurrentSystem(): InstallSessionTargetSystem {
  const platform = window.navigator.platform.toLowerCase()
  const userAgent = window.navigator.userAgent.toLowerCase()
  if (platform.includes('mac')) return 'macos'
  if (platform.includes('win') || userAgent.includes('windows')) return 'windows'
  return 'linux'
}

async function openInstallDialog(apiKey: ApiKey) {
  selectedInstallApiKey.value = apiKey
  installSession.value = null
  resetInstallCopiedState()
  showInstallDialog.value = true
  await refreshInstallCommand()
}

function openSetupChoiceDialog(apiKey: ApiKey) {
  selectedSetupApiKey.value = apiKey
  showSetupChoiceDialog.value = true
}

function chooseSetupCcSwitch() {
  if (!selectedSetupApiKey.value) return
  const apiKey = selectedSetupApiKey.value
  showSetupChoiceDialog.value = false
  openCcSwitchDialog(apiKey)
}

function chooseSetupInstall() {
  if (!selectedSetupApiKey.value) return
  const apiKey = selectedSetupApiKey.value
  showSetupChoiceDialog.value = false
  void openInstallDialog(apiKey)
}

async function selectInstallCli(value: InstallTargetCli) {
  installCli.value = value
  await refreshInstallCommand()
}

async function selectInstallSystem(value: InstallSessionTargetSystem) {
  installSystem.value = value
  await refreshInstallCommand()
}

async function refreshInstallCommand() {
  if (!selectedInstallApiKey.value) return
  installLoading.value = true
  installSession.value = null
  resetInstallCopiedState()
  try {
    installSession.value = await meApi.createApiKeyInstallSession(selectedInstallApiKey.value.id, {
      target_cli: installCli.value,
      target_system: installSystem.value,
    })
  } catch (error) {
    log.error('生成 CLI 安装命令失败:', error)
    showError(parseApiError(error, t('apiKeys.generateInstallFailed')))
  } finally {
    installLoading.value = false
  }
}

async function copyInstallCommand() {
  if (!installCommand.value) return
  const copied = await copyTextToClipboard(installCommand.value, false)
  if (!copied) return

  installCopied.value = true
  success(t('apiKeys.installCommandCopied'))
  clearInstallCopiedResetTimer()
  installCopiedResetTimer = setTimeout(() => {
    installCopied.value = false
    installCopiedResetTimer = null
  }, 2000)
}

function openCcSwitchDialog(apiKey: ApiKey) {
  selectedCcSwitchApiKey.value = apiKey
  ccSwitchApp.value = 'claude'
  ccSwitchProviderName.value = `Niffler - ${apiKey.name || 'API Key'}`
  ccSwitchModel.value = ''
  showCcSwitchDialog.value = true
  void refreshCcSwitchBaseUrl()
}

function selectCcSwitchApp(value: CcSwitchApp) {
  ccSwitchApp.value = value
  if (value === 'codex') {
    ccSwitchModel.value = ccSwitchModel.value.trim() || DEFAULT_CCSWITCH_CODEX_MODEL
  } else {
    ccSwitchModel.value = ''
  }
}

async function refreshCcSwitchBaseUrl() {
  try {
    const response = await meApi.getPublicBaseUrl()
    const value = response.public_base_url?.trim().replace(/\/+$/, '')
    if (value) {
      ccSwitchBaseUrl.value = value
    }
  } catch (error) {
    log.warn('获取公开 API 地址失败，使用前端推断地址:', error)
  }
}

async function importToCcSwitch() {
  if (!selectedCcSwitchApiKey.value) return
  ccSwitchImportLoading.value = true
  try {
    await refreshCcSwitchBaseUrl()
    const response = await meApi.getFullApiKey(selectedCcSwitchApiKey.value.id)
    const deeplink = buildCcSwitchImportUrl({
      app: ccSwitchApp.value,
      baseUrl: ccSwitchBaseUrl.value,
      providerName: ccSwitchProviderName.value,
      apiKey: response.key,
      model: ccSwitchModel.value,
    })
    window.location.href = deeplink
    success(t('apiKeys.ccSwitchOpened'))
  } catch (error) {
    log.error('导入 CC Switch 失败:', error)
    showError(parseApiError(error, t('apiKeys.ccSwitchImportFailed')))
  } finally {
    ccSwitchImportLoading.value = false
  }
}

function closeCreatedKeyDialog() {
  showKeyDialog.value = false
  const pending = pendingSetupApiKey.value
  pendingSetupApiKey.value = null
  if (pending) {
    openSetupChoiceDialog(pending)
  }
}

function closeApiKeyDialog() {
  showCreateDialog.value = false
  editingApiKey.value = null
  newKeyName.value = ''
  selectedGroupId.value = apiKeyGroups.value[0]?.id || ''
  newKeyRateLimit.value = undefined
  newKeyConcurrentLimit.value = undefined
  keyRedactionMode.value = 'inherit'
  newKeyRedactionEnabled.value = false
  newKeyRedactionInjectNotice.value = true
}

async function saveApiKey() {
  if (!newKeyName.value.trim()) {
    showError(t('apiKeys.enterName'))
    return
  }
  if (apiKeyGroups.value.length === 0 || !selectedGroupId.value) {
    showError(t('apiKeys.noGroupsContactAdmin'))
    return
  }

  creating.value = true
  try {
    if (editingApiKey.value) {
      await meApi.updateApiKey(editingApiKey.value.id, {
        name: newKeyName.value,
        group_id: selectedGroupId.value || undefined,
        rate_limit: newKeyRateLimit.value ?? 0,
        concurrent_limit: newKeyConcurrentLimit.value,
        feature_settings: keyRedactionMode.value === 'custom'
          ? mergeChatPiiRedactionFeatureSettings(editingApiKey.value.feature_settings, {
                enabled: newKeyRedactionEnabled.value,
                inject_model_instruction: newKeyRedactionInjectNotice.value,
            })
          : null,
      })
      success(t('apiKeys.updated'))
    } else {
      const newKey = await meApi.createApiKey({
        name: newKeyName.value,
        group_id: selectedGroupId.value || undefined,
        rate_limit: newKeyRateLimit.value ?? 0,
        concurrent_limit: newKeyConcurrentLimit.value,
        ...(keyRedactionMode.value === 'custom'
          ? {
              feature_settings: mergeChatPiiRedactionFeatureSettings(null, {
                enabled: newKeyRedactionEnabled.value,
                inject_model_instruction: newKeyRedactionInjectNotice.value,
              }),
            }
          : {}),
      })
      newKeyValue.value = newKey.key || ''
      pendingSetupApiKey.value = newKey
      showKeyDialog.value = true
      success(t('apiKeys.created'))
    }
    closeApiKeyDialog()
    await loadApiKeys()
  } catch (error) {
    log.error(editingApiKey.value ? '更新 API 密钥失败:' : '创建 API 密钥失败:', error)
    showError(editingApiKey.value ? t('apiKeys.updateFailed') : t('apiKeys.createFailed'))
  } finally {
    creating.value = false
  }
}

function confirmDelete(apiKey: ApiKey) {
  keyToDelete.value = apiKey
  showDeleteDialog.value = true
}

async function deleteApiKey() {
  if (!keyToDelete.value) return

  deleting.value = true
  try {
    await meApi.deleteApiKey(keyToDelete.value.id)
    apiKeys.value = apiKeys.value.filter(k => k.id !== keyToDelete.value?.id)
    showDeleteDialog.value = false
    success(t('apiKeys.deleted'))
  } catch (error) {
    log.error('删除 API 密钥失败:', error)
    showError(t('apiKeys.deleteFailed'))
  } finally {
    deleting.value = false
    keyToDelete.value = null
  }
}

async function toggleApiKey(apiKey: ApiKey) {
  try {
    const updated = await meApi.toggleApiKey(apiKey.id)
    const index = apiKeys.value.findIndex(k => k.id === apiKey.id)
    if (index !== -1) {
      apiKeys.value[index].is_active = updated.is_active
    }
    success(updated.is_active ? t('apiKeys.enabled') : t('apiKeys.disabledSuccess'))
  } catch (error) {
    log.error('切换密钥状态失败:', error)
    showError(t('apiKeys.operationFailed'))
  }
}

async function copyApiKey(apiKey: ApiKey) {
  try {
    // 调用后端 API 获取完整密钥
    const response = await meApi.getFullApiKey(apiKey.id)
    const copied = await copyTextToClipboard(response.key, false) // 不显示内部提示
    if (copied) {
      success(t('apiKeys.fullKeyCopied'))
    }
  } catch (error) {
    log.error('复制密钥失败:', error)
    showError(t('apiKeys.copyFailedRetry'))
  }
}

async function copyTextToClipboard(text: string, showToast: boolean = true): Promise<boolean> {
  try {
    if (navigator.clipboard && window.isSecureContext) {
      await navigator.clipboard.writeText(text)
      if (showToast) success(t('apiKeys.copiedToClipboard'))
      return true
    } else {
      const textArea = document.createElement('textarea')
      textArea.value = text
      textArea.style.position = 'fixed'
      textArea.style.left = '-999999px'
      textArea.style.top = '-999999px'
      document.body.appendChild(textArea)
      textArea.focus()
      textArea.select()

      try {
        const successful = document.execCommand('copy')
        if (successful && showToast) {
          success(t('apiKeys.copiedToClipboard'))
        }
        if (successful) {
          return true
        } else {
          showError(t('apiKeys.copyManually'))
          return false
        }
      } finally {
        document.body.removeChild(textArea)
      }
    }
  } catch (error) {
    log.error('复制失败:', error)
    showError(t('apiKeys.copySelectManually'))
    return false
  }
}

function formatNumber(num: number | undefined | null): string {
  if (num === undefined || num === null) {
    return '0'
  }
  return num.toLocaleString(locale.value)
}

function formatConcurrentLimitSimple(concurrentLimit?: number | null): string {
  if (concurrentLimit == null || concurrentLimit === 0) {
    return t('apiKeys.unlimitedConcurrency')
  }
  return t('apiKeys.concurrencyValue', { count: concurrentLimit })
}

function formatDate(dateString?: string | null): string {
  if (!dateString) return t('common.unknown')
  const date = new Date(dateString)
  if (Number.isNaN(date.getTime())) return t('common.unknown')
  return date.toLocaleDateString(locale.value, {
    year: 'numeric',
    month: '2-digit',
    day: '2-digit'
  })
}

function formatRelativeTime(dateString: string): string {
  const date = new Date(dateString)
  if (Number.isNaN(date.getTime())) return t('common.unknown')
  const now = new Date()
  const diffMs = now.getTime() - date.getTime()
  const diffMins = Math.floor(diffMs / (1000 * 60))
  const diffHours = Math.floor(diffMs / (1000 * 60 * 60))
  const diffDays = Math.floor(diffMs / (1000 * 60 * 60 * 24))

  if (diffMins < 1) return t('apiKeys.justNow')
  if (diffMins < 60) return t('apiKeys.minutesAgo', { count: diffMins })
  if (diffHours < 24) return t('apiKeys.hoursAgo', { count: diffHours })
  if (diffDays < 7) return t('apiKeys.daysAgo', { count: diffDays })

  return formatDate(dateString)
}

</script>
