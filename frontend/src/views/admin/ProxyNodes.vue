<template>
  <div class="space-y-6 pb-8">
    <Card
      variant="default"
      class="overflow-hidden"
    >
      <!-- 标题和筛选器 -->
      <div class="px-4 sm:px-6 py-3.5 border-b border-border/60">
        <!-- 移动端 -->
        <div class="flex flex-col gap-3 sm:hidden">
          <div class="flex items-center justify-between">
            <h3 class="text-base font-semibold">
              {{ t('proxyNodes.title') }}
            </h3>
            <div class="flex items-center gap-2">
              <Button
                size="sm"
                variant="outline"
                class="h-7 text-xs"
                @click="showBatchUpgradeDialog = true"
              >
                {{ t('proxyNodes.upgrade') }}
              </Button>
              <Button
                size="sm"
                class="h-7 text-xs"
                @click="openAddDialog"
              >
                <Plus class="w-3 h-3 mr-1" />
                {{ t('proxyNodes.add') }}
              </Button>
              <RefreshButton
                :loading="store.loading"
                @click="refresh"
              />
            </div>
          </div>
          <div class="flex flex-wrap items-center gap-2">
            <div class="relative min-w-0 basis-full">
              <Search class="absolute left-2.5 top-1/2 -translate-y-1/2 h-3.5 w-3.5 text-muted-foreground z-10 pointer-events-none" />
              <Input
                v-model="searchQuery"
                type="text"
                  :placeholder="t('proxyNodes.search')"
                class="w-full pl-8 pr-3 h-8 text-sm bg-background/50 border-border/60"
              />
            </div>
            <div class="min-w-0 flex-1">
              <Select v-model="filterStatus">
                <SelectTrigger class="w-full h-8 text-xs border-border/60">
                  <SelectValue :placeholder="t('proxyNodes.status')" />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="all">
                    {{ t('proxyNodes.all') }}
                  </SelectItem>
                  <SelectItem value="online">
                    {{ t('proxyNodes.online') }}
                  </SelectItem>
                  <SelectItem value="offline">
                    {{ t('proxyNodes.offline') }}
                  </SelectItem>
                </SelectContent>
              </Select>
            </div>
          </div>
        </div>

        <!-- 桌面端 -->
        <div class="hidden sm:flex items-center justify-between gap-4">
          <h3 class="text-base font-semibold">
            {{ t('proxyNodes.title') }}
          </h3>
          <div class="flex items-center gap-2">
            <div class="relative">
              <Search class="absolute left-2.5 top-1/2 -translate-y-1/2 h-3.5 w-3.5 text-muted-foreground z-10 pointer-events-none" />
              <Input
                v-model="searchQuery"
                type="text"
                :placeholder="t('proxyNodes.search')"
                class="w-48 pl-8 pr-3 h-8 text-sm bg-background/50 border-border/60"
              />
            </div>
            <div class="h-4 w-px bg-border" />
            <div class="xl:hidden">
              <Select v-model="filterStatus">
                <SelectTrigger class="w-28 h-8 text-xs border-border/60">
                  <SelectValue :placeholder="t('proxyNodes.allStatus')" />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="all">
                    {{ t('proxyNodes.allStatus') }}
                  </SelectItem>
                  <SelectItem value="online">
                    {{ t('proxyNodes.online') }}
                  </SelectItem>
                  <SelectItem value="offline">
                    {{ t('proxyNodes.offline') }}
                  </SelectItem>
                </SelectContent>
              </Select>
            </div>
            <div class="h-4 w-px bg-border" />
            <Button
              variant="outline"
              size="sm"
              class="h-8 text-xs"
              @click="showBatchUpgradeDialog = true"
            >
              {{ t('proxyNodes.batchUpgrade') }}
            </Button>
            <Button
              variant="ghost"
              size="icon"
              class="h-8 w-8"
              :title="t('proxyNodes.manualAdd')"
              @click="openAddDialog"
            >
              <Plus class="w-3.5 h-3.5" />
            </Button>
            <RefreshButton
              :loading="store.loading"
              @click="refresh"
            />
          </div>
        </div>
      </div>

      <!-- 桌面端表格 -->
      <div class="hidden xl:block overflow-x-auto">
        <Table>
          <TableHeader>
            <TableRow class="border-b border-border/60 hover:bg-transparent">
              <TableHead class="w-[28px] min-w-[28px] max-w-[28px] h-12 p-0 pl-2" />
              <TableHead class="w-[160px] h-12 font-semibold">
                {{ t('proxyNodes.name') }}
              </TableHead>
              <TableHead class="w-[180px] h-12 font-semibold">
                {{ t('proxyNodes.address') }}
              </TableHead>
              <TableHead class="w-[100px] h-12 font-semibold">
                {{ t('proxyNodes.region') }}
              </TableHead>
              <SortableTableHead
                class="w-[90px] h-12 font-semibold text-center"
                column-key="status"
                :sortable="false"
                align="center"
                :filter-active="filterStatus !== 'all'"
                :filter-title="t('proxyNodes.filterStatus')"
                filter-content-class="w-36 p-1 rounded-2xl border-border bg-card text-foreground shadow-2xl backdrop-blur-xl"
              >
                {{ t('proxyNodes.status') }}
                <template #filter="{ close }">
                  <TableFilterMenu
                    v-model="filterStatus"
                    :options="proxyNodeStatusFilterOptions"
                    @select="close"
                  />
                </template>
              </SortableTableHead>
              <TableHead class="w-[100px] h-12 font-semibold text-center">
                {{ t('proxyNodes.totalRequests') }}
              </TableHead>
              <TableHead class="w-[100px] h-12 font-semibold text-center">
                {{ t('proxyNodes.failureRate') }}
              </TableHead>
              <TableHead class="w-[100px] h-12 font-semibold text-center">
                {{ t('proxyNodes.latency') }}
              </TableHead>
              <TableHead class="w-[120px] h-12 font-semibold text-center">
                {{ t('proxyNodes.version') }}
              </TableHead>
              <TableHead class="w-[160px] h-12 font-semibold">
                {{ t('proxyNodes.lastHeartbeat') }}
              </TableHead>
              <TableHead class="w-[140px] h-12 font-semibold text-center">
                {{ t('proxyNodes.actions') }}
              </TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            <template
              v-for="node in paginatedNodes"
              :key="node.id"
            >
              <TableRow
                class="border-b border-border/40 hover:bg-muted/30 transition-colors"
                :class="isNodeExpanded(node.id) ? 'bg-muted/20' : ''"
              >
                <TableCell class="w-[28px] min-w-[28px] max-w-[28px] p-0 pl-2 text-center">
                  <button
                    type="button"
                    class="inline-flex h-5 w-5 items-center justify-center rounded-md text-muted-foreground transition-colors hover:text-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-1"
                    :title="isNodeExpanded(node.id) ? t('proxyNodes.collapse') : t('proxyNodes.expand')"
                    @click="toggleNodeDetails(node)"
                  >
                    <ChevronDown
                      v-if="isNodeExpanded(node.id)"
                      class="h-3.5 w-3.5"
                    />
                    <ChevronRight
                      v-else
                      class="h-3.5 w-3.5"
                    />
                  </button>
                </TableCell>
                <TableCell class="py-4">
                  <div class="flex items-center gap-1.5">
                    <span class="text-sm font-semibold">{{ node.name }}</span>
                    <Badge
                      v-if="node.is_manual"
                      variant="outline"
                      class="text-[10px] px-1.5 py-0"
                    >
                      {{ t('proxyNodes.manual') }}
                    </Badge>
                    <Badge
                      v-if="node.tunnel_mode"
                      variant="outline"
                      class="text-[10px] px-1.5 py-0"
                    >
                      Tunnel
                    </Badge>
                    <Badge
                      v-if="nodeSchedulingBadge(node)"
                      :variant="nodeSchedulingBadge(node)!.variant"
                      class="text-[10px] px-1.5 py-0"
                    >
                      {{ nodeSchedulingBadge(node)!.label }}
                    </Badge>
                    <HardwareTooltip :node="node" />
                  </div>
                </TableCell>
                <TableCell class="py-4">
                  <code class="text-xs text-muted-foreground">{{ nodeAddress(node) }}</code>
                </TableCell>
                <TableCell class="py-4">
                  <span class="text-sm text-muted-foreground">{{ formatRegion(node.region) }}</span>
                </TableCell>
                <TableCell class="py-4 text-center">
                  <Badge
                    :variant="statusVariant(node.status)"
                    :title="statusTitle(node)"
                    class="font-medium px-2.5 py-0.5 text-xs"
                  >
                    {{ statusLabel(node) }}
                  </Badge>
                </TableCell>
                <TableCell class="py-4 text-center">
                  <span class="text-sm tabular-nums">{{ formatNumber(node.total_requests) }}</span>
                </TableCell>
                <TableCell class="py-4 text-center">
                  <span
                    class="text-sm tabular-nums"
                    :class="failureRate(node) > 5 ? 'text-destructive font-medium' : ''"
                  >{{ formatFailureRate(node) }}</span>
                </TableCell>
                <TableCell class="py-4 text-center">
                  <span class="text-sm tabular-nums">{{ node.avg_latency_ms != null ? `${node.avg_latency_ms.toFixed(0)}ms` : '-' }}</span>
                </TableCell>
                <TableCell class="py-4 text-center">
                  <span class="text-sm tabular-nums">{{ node.is_manual ? '-' : nodeProxyVersion(node) }}</span>
                </TableCell>
                <TableCell class="py-4">
                  <span class="text-xs text-muted-foreground">{{ formatTime(node.last_heartbeat_at) }}</span>
                </TableCell>
                <TableCell class="py-4 text-center">
                  <div class="flex items-center justify-center gap-0.5">
                    <Button
                      variant="ghost"
                      size="icon"
                      class="h-8 w-8"
                      :title="testingNodes.has(node.id) ? t('proxyNodes.testing') : t('proxyNodes.test')"
                      :disabled="testingNodes.has(node.id)"
                      @click="handleTest(node)"
                    >
                      <Loader2
                        v-if="testingNodes.has(node.id)"
                        class="h-4 w-4 animate-spin"
                      />
                      <Activity
                        v-else
                        class="h-4 w-4"
                      />
                    </Button>
                    <Button
                      v-if="node.is_manual"
                      variant="ghost"
                      size="icon"
                      class="h-8 w-8"
                      :title="t('proxyNodes.edit')"
                      @click="handleEdit(node)"
                    >
                      <SquarePen class="h-4 w-4" />
                    </Button>
                    <Button
                      v-if="!node.is_manual"
                      variant="ghost"
                      size="icon"
                      class="h-8 w-8"
                      :title="t('proxyNodes.remoteConfig')"
                      @click="handleConfig(node)"
                    >
                      <Settings class="h-4 w-4" />
                    </Button>
                    <Button
                      v-if="!node.is_manual"
                      variant="ghost"
                      size="icon"
                      class="h-8 w-8"
                      :title="t('proxyNodes.connectionEvents')"
                      @click="handleViewEvents(node)"
                    >
                      <History class="h-4 w-4" />
                    </Button>
                    <Button
                      variant="ghost"
                      size="icon"
                      class="h-8 w-8"
                      :title="t('proxyNodes.delete')"
                      @click="handleDelete(node)"
                    >
                      <Trash2 class="h-4 w-4" />
                    </Button>
                  </div>
                </TableCell>
              </TableRow>
              <TableRow
                v-if="isNodeExpanded(node.id)"
                class="border-b border-border/40 hover:bg-transparent"
              >
                <TableCell
                  colspan="11"
                  class="p-0"
                >
                  <ProxyNodeDataPanel
                    :node="node"
                    :state="nodeDetails[node.id]"
                    @refresh="loadNodeDetails(node)"
                  />
                </TableCell>
              </TableRow>
            </template>
            <TableRow v-if="paginatedNodes.length === 0">
              <TableCell
                colspan="11"
                class="py-12 text-center text-muted-foreground text-sm"
              >
                {{ store.loading ? t('proxyNodes.loading') : t('proxyNodes.empty') }}
              </TableCell>
            </TableRow>
          </TableBody>
        </Table>
      </div>

      <!-- 移动端卡片列表 -->
      <div class="xl:hidden divide-y divide-border/40">
        <div
          v-for="node in paginatedNodes"
          :key="node.id"
          class="p-4 sm:p-5"
        >
          <div class="flex items-start justify-between mb-2">
            <div>
              <div class="flex items-center gap-1.5">
                <button
                  type="button"
                  class="inline-flex h-5 w-5 items-center justify-center rounded-md text-muted-foreground transition-colors hover:text-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 shrink-0"
                  :title="isNodeExpanded(node.id) ? t('proxyNodes.collapse') : t('proxyNodes.expand')"
                  @click="toggleNodeDetails(node)"
                >
                  <ChevronDown
                    v-if="isNodeExpanded(node.id)"
                    class="h-3.5 w-3.5"
                  />
                  <ChevronRight
                    v-else
                    class="h-3.5 w-3.5"
                  />
                </button>
                <span class="font-semibold text-sm">{{ node.name }}</span>
                <Badge
                  v-if="node.is_manual"
                  variant="outline"
                  class="text-[10px] px-1.5 py-0"
                >
                  {{ t('proxyNodes.manual') }}
                </Badge>
                <Badge
                  v-if="node.tunnel_mode"
                  variant="outline"
                  class="text-[10px] px-1.5 py-0"
                >
                  Tunnel
                </Badge>
                <Badge
                  v-if="nodeSchedulingBadge(node)"
                  :variant="nodeSchedulingBadge(node)!.variant"
                  class="text-[10px] px-1.5 py-0"
                >
                  {{ nodeSchedulingBadge(node)!.label }}
                </Badge>
                <HardwareTooltip :node="node" />
              </div>
              <code class="text-xs text-muted-foreground">{{ nodeAddress(node) }}</code>
              <div
                v-if="!node.is_manual"
                class="text-[11px] text-muted-foreground mt-1"
              >
                {{ t('proxyNodes.version') }}: {{ nodeProxyVersion(node) }}
              </div>
            </div>
            <Badge
              :variant="statusVariant(node.status)"
              :title="statusTitle(node)"
              class="text-xs"
            >
              {{ statusLabel(node) }}
            </Badge>
          </div>
          <div class="grid grid-cols-4 gap-2 text-xs text-muted-foreground mb-3">
            <div>
              <span class="block text-foreground/60">{{ t('proxyNodes.region') }}</span>
              <span>{{ formatRegion(node.region) }}</span>
            </div>
            <div>
              <span class="block text-foreground/60">{{ t('proxyNodes.totalRequests') }}</span>
              <span class="tabular-nums">{{ formatNumber(node.total_requests) }}</span>
            </div>
            <div>
              <span class="block text-foreground/60">{{ t('proxyNodes.failureRate') }}</span>
              <span
                class="tabular-nums"
                :class="failureRate(node) > 5 ? 'text-destructive font-medium' : ''"
              >{{ formatFailureRate(node) }}</span>
            </div>
            <div>
              <span class="block text-foreground/60">{{ t('proxyNodes.latency') }}</span>
              <span class="tabular-nums">{{ node.avg_latency_ms != null ? `${node.avg_latency_ms.toFixed(0)}ms` : '-' }}</span>
            </div>
          </div>
          <div class="flex items-center justify-between">
            <span class="text-xs text-muted-foreground">{{ formatTime(node.last_heartbeat_at) }}</span>
            <div class="flex flex-wrap items-center justify-end gap-1">
              <Button
                variant="ghost"
                size="sm"
                class="h-7 px-2 text-xs"
                :disabled="testingNodes.has(node.id)"
                @click="handleTest(node)"
              >
                <Loader2
                  v-if="testingNodes.has(node.id)"
                  class="h-3 w-3 mr-1 animate-spin"
                />
                <Activity
                  v-else
                  class="h-3 w-3 mr-1"
                />
                {{ testingNodes.has(node.id) ? t('proxyNodes.testingShort') : t('proxyNodes.testShort') }}
              </Button>
              <Button
                v-if="node.is_manual"
                variant="ghost"
                size="sm"
                class="h-7 px-2 text-xs"
                @click="handleEdit(node)"
              >
                <SquarePen class="h-3 w-3 mr-1" />
                {{ t('proxyNodes.edit') }}
              </Button>
              <Button
                v-if="!node.is_manual"
                variant="ghost"
                size="sm"
                class="h-7 px-2 text-xs"
                @click="handleConfig(node)"
              >
                <Settings class="h-3 w-3 mr-1" />
                {{ t('proxyNodes.config') }}
              </Button>
              <Button
                variant="ghost"
                size="sm"
                class="h-7 px-2 text-xs"
                @click="handleDelete(node)"
              >
                <Trash2 class="h-3 w-3 mr-1" />
                {{ t('proxyNodes.delete') }}
              </Button>
            </div>
          </div>
          <div
            v-if="isNodeExpanded(node.id)"
            class="mt-4 -mx-4 sm:-mx-5"
          >
            <ProxyNodeDataPanel
              :node="node"
              :state="nodeDetails[node.id]"
              @refresh="loadNodeDetails(node)"
            />
          </div>
        </div>
        <div
          v-if="paginatedNodes.length === 0"
          class="p-8 text-center text-muted-foreground text-sm"
        >
          {{ store.loading ? t('proxyNodes.loading') : t('proxyNodes.empty') }}
        </div>
      </div>

      <!-- 分页 -->
      <Pagination
        :current="currentPage"
        :total="filteredNodes.length"
        :page-size="pageSize"
        cache-key="proxy-nodes-page-size"
        @update:current="currentPage = $event"
        @update:page-size="pageSize = $event"
      />
    </Card>
    <!-- 手动添加/编辑代理节点对话框 -->
    <Dialog
      :model-value="showAddDialog"
      :title="editingNode ? t('proxyNodes.editTitle') : t('proxyNodes.addTitle')"
      :description="editingNode ? t('proxyNodes.editDescription') : t('proxyNodes.addDescription')"
      :icon="editingNode ? SquarePen : Plus"
      size="lg"
      @update:model-value="handleDialogClose"
    >
      <div
        v-if="!editingNode"
        class="mb-4 grid grid-cols-2 gap-2 rounded-lg border border-border/60 bg-muted/30 p-1"
      >
        <Button
          type="button"
          :variant="addMode === 'script' ? 'default' : 'ghost'"
          class="h-9"
          @click="addMode = 'script'"
        >
          <Terminal class="w-3.5 h-3.5 mr-1.5" />
          {{ t('proxyNodes.scriptAdd') }}
        </Button>
        <Button
          type="button"
          :variant="addMode === 'manual' ? 'default' : 'ghost'"
          class="h-9"
          @click="addMode = 'manual'"
        >
          <Plus class="w-3.5 h-3.5 mr-1.5" />
          {{ t('proxyNodes.manualAdd') }}
        </Button>
      </div>

      <div
        v-if="!editingNode && addMode === 'script'"
        class="space-y-4"
      >
        <div class="rounded-lg border border-border/60 bg-muted/30 p-3 text-xs text-muted-foreground">
          {{ t('proxyNodes.installHint') }}
        </div>

        <div class="space-y-1.5">
          <Label>{{ t('proxyNodes.nodeName') }} *</Label>
          <Input
            v-model="installForm.node_name"
            :placeholder="t('proxyNodes.nodeNamePlaceholder')"
            @keyup.enter="refreshProxyInstallCommand"
          />
        </div>

        <div class="space-y-2">
          <Label class="text-sm font-semibold">{{ t('proxyNodes.targetSystem') }}</Label>
          <div class="grid grid-cols-1 sm:grid-cols-2 gap-2">
            <Button
              type="button"
              :variant="installSystem === 'unix' ? 'default' : 'outline'"
              class="justify-start h-auto py-3"
              @click="installSystem = 'unix'"
            >
              macOS / Linux
            </Button>
            <Button
              type="button"
              :variant="installSystem === 'windows' ? 'default' : 'outline'"
              class="justify-start h-auto py-3"
              @click="installSystem = 'windows'"
            >
              Windows PowerShell
            </Button>
          </div>
        </div>

        <div class="space-y-2">
          <div class="flex items-center justify-between gap-2">
            <Label class="text-sm font-semibold">{{ t('proxyNodes.copyToMachine') }}</Label>
            <div class="flex items-center gap-2">
              <Button
                variant="outline"
                size="sm"
                class="gap-1.5"
                :disabled="installLoading || !proxyInstallCommand"
                @click="copyProxyInstallCommand"
              >
                <CheckCircle
                  v-if="installCopied"
                  class="h-3.5 w-3.5 text-emerald-600 dark:text-emerald-400"
                />
                <Copy
                  v-else
                  class="h-3.5 w-3.5"
                />
                {{ installCopied ? t('proxyNodes.copied') : t('proxyNodes.copyOneClick') }}
              </Button>
              <Button
                variant="ghost"
                size="sm"
                :disabled="installLoading || !installForm.node_name.trim()"
                @click="refreshProxyInstallCommand"
              >
                {{ installLoading ? t('proxyNodes.generating') : (proxyInstallSession ? t('proxyNodes.regenerate') : t('proxyNodes.generateCommand')) }}
              </Button>
            </div>
          </div>
          <div class="rounded-lg border border-border/60 bg-background overflow-hidden">
            <pre class="max-h-32 overflow-x-auto whitespace-pre-wrap break-all p-3 text-xs font-mono">{{ proxyInstallCommand || t('proxyNodes.commandPlaceholder') }}</pre>
          </div>
          <p class="text-xs text-muted-foreground">
            {{ proxyInstallHint }}
          </p>
        </div>
      </div>

      <form
        v-else
        class="space-y-4"
        @submit.prevent="handleAddManualNode"
      >
        <div class="space-y-1.5">
          <Label>{{ t('proxyNodes.name') }} *</Label>
          <Input
            v-model="addForm.name"
            :placeholder="t('proxyNodes.namePlaceholder')"
          />
        </div>
        <div class="space-y-1.5">
          <Label>{{ t('proxyNodes.proxyAddress') }} *</Label>
          <Input
            v-model="addForm.proxy_url"
            :placeholder="t('proxyNodes.addressPlaceholder')"
          />
        </div>
        <div class="grid grid-cols-2 gap-3">
          <div class="space-y-1.5">
            <Label>{{ t('proxyNodes.username') }}</Label>
            <Input
              v-model="addForm.username"
              :placeholder="t('proxyNodes.optional')"
              autocomplete="off"
              data-form-type="other"
              data-lpignore="true"
              data-1p-ignore="true"
            />
          </div>
          <div class="space-y-1.5">
            <Label>{{ t('proxyNodes.password') }}</Label>
            <Input
              v-model="addForm.password"
              type="text"
              masked
              :placeholder="t('proxyNodes.optional')"
              autocomplete="new-password"
              data-form-type="other"
              data-lpignore="true"
              data-1p-ignore="true"
            />
          </div>
        </div>
        <div class="space-y-1.5">
          <Label>{{ t('proxyNodes.region') }}</Label>
          <Input
            v-model="addForm.region"
            :placeholder="t('proxyNodes.regionPlaceholder')"
          />
        </div>
      </form>

      <template #footer>
        <div
          v-if="!editingNode && addMode === 'script'"
          class="flex items-center justify-end gap-2 w-full"
        >
          <Button
            variant="outline"
            @click="handleDialogClose(false)"
          >
            {{ t('proxyNodes.close') }}
          </Button>
          <Button
            :disabled="installLoading || !proxyInstallCommand"
            @click="copyProxyInstallCommand"
          >
            {{ installCopied ? t('proxyNodes.copied') : t('proxyNodes.copyCommand') }}
          </Button>
        </div>
        <div
          v-else
          class="flex items-center justify-between w-full"
        >
          <Button
            variant="outline"
            :disabled="testingUrl || !addForm.proxy_url"
            @click="handleTestUrl"
          >
            {{ testingUrl ? t('proxyNodes.testing') : t('proxyNodes.testShort') }}
          </Button>
          <div class="flex items-center gap-2">
            <Button
              variant="outline"
              @click="handleDialogClose(false)"
            >
              {{ t('proxyNodes.cancel') }}
            </Button>
            <Button
              :disabled="addingNode || !addForm.name || !addForm.proxy_url"
              @click="editingNode ? handleUpdateManualNode() : handleAddManualNode()"
            >
              {{ addingNode ? (editingNode ? t('proxyNodes.saving') : t('proxyNodes.adding')) : (editingNode ? t('proxyNodes.save') : t('proxyNodes.add')) }}
            </Button>
          </div>
        </div>
      </template>
    </Dialog>

    <!-- 远程配置对话框 (aether-tunnel 节点) -->
    <Dialog
      :model-value="showConfigDialog"
      :title="t('proxyNodes.remoteConfig')"
      :description="t('proxyNodes.remoteConfigHint')"
      :icon="Settings"
      size="md"
      @update:model-value="handleConfigDialogClose"
    >
      <form
        class="space-y-4"
        @submit.prevent
      >
        <div class="space-y-1.5">
          <Label>{{ t('proxyNodes.allowedPorts') }}</Label>
          <Input
            v-model="configForm.allowed_ports"
            placeholder="80, 443, 8080, 8443"
          />
          <p class="text-xs text-muted-foreground">
            {{ t('proxyNodes.allowedPortsHint') }}
          </p>
        </div>
        <div class="space-y-1.5">
          <Label>{{ t('proxyNodes.logLevel') }}</Label>
          <Select v-model="configForm.log_level">
            <SelectTrigger><SelectValue /></SelectTrigger>
            <SelectContent>
              <SelectItem value="trace">
                trace
              </SelectItem>
              <SelectItem value="debug">
                debug
              </SelectItem>
              <SelectItem value="info">
                info
              </SelectItem>
              <SelectItem value="warn">
                warn
              </SelectItem>
              <SelectItem value="error">
                error
              </SelectItem>
            </SelectContent>
          </Select>
        </div>
        <div class="grid grid-cols-2 gap-4">
          <div class="space-y-1.5">
            <Label>{{ t('proxyNodes.heartbeatInterval') }}</Label>
            <Input
              v-model="configForm.heartbeat_interval"
              type="number"
              min="5"
              max="600"
            />
          </div>
          <div class="space-y-1.5">
            <Label>{{ t('proxyNodes.schedulingState') }}</Label>
            <Select v-model="configForm.scheduling_state">
              <SelectTrigger><SelectValue /></SelectTrigger>
              <SelectContent>
                <SelectItem value="active">
                  active
                </SelectItem>
                <SelectItem value="draining">
                  draining
                </SelectItem>
                <SelectItem value="cordoned">
                  cordoned
                </SelectItem>
              </SelectContent>
            </Select>
            <p class="text-xs text-muted-foreground">
              {{ t('proxyNodes.schedulingStateHint') }}
            </p>
          </div>
        </div>
        <div class="space-y-1.5">
          <Label>{{ t('proxyNodes.upgradeTo') }}</Label>
          <Input
            v-model="configForm.upgrade_to"
            :placeholder="t('proxyNodes.versionPlaceholder')"
          />
          <p class="text-xs text-muted-foreground">
            {{ t('proxyNodes.clearUpgradeHint') }}
          </p>
        </div>
        <div
          v-if="configNode"
          class="text-xs text-muted-foreground"
        >
          {{ t('proxyNodes.configVersion') }}: v{{ configNode.config_version }}
        </div>
      </form>
      <template #footer>
        <Button
          variant="outline"
          @click="handleConfigDialogClose(false)"
        >
          {{ t('proxyNodes.cancel') }}
        </Button>
        <Button
          :disabled="savingConfig"
          @click="handleSaveConfig"
        >
          {{ savingConfig ? t('proxyNodes.saving') : t('proxyNodes.save') }}
        </Button>
      </template>
    </Dialog>

    <!-- 批量升级对话框 -->
    <Dialog
      :model-value="showBatchUpgradeDialog"
      :title="t('proxyNodes.batchUpgrade')"
      :description="t('proxyNodes.batchUpgradeHint')"
      :icon="Settings"
      size="sm"
      @update:model-value="(open: boolean) => { if (!open) { resetBatchUpgradeDialog() } }"
    >
      <form
        class="space-y-4"
        @submit.prevent="handleBatchUpgrade"
      >
        <div class="space-y-1.5">
          <Label>{{ t('proxyNodes.targetVersion') }}</Label>
          <Input
            v-model="batchUpgradeVersion"
            :placeholder="t('proxyNodes.versionPlaceholder')"
          />
          <p class="text-xs text-muted-foreground">
            {{ t('proxyNodes.batchUpgradeDetails') }}
          </p>
        </div>
      </form>
      <template #footer>
        <Button
          variant="outline"
          @click="resetBatchUpgradeDialog()"
        >
          {{ t('proxyNodes.cancel') }}
        </Button>
        <Button
          :disabled="batchUpgrading || !batchUpgradeVersion.trim()"
          @click="handleBatchUpgrade"
        >
          {{ batchUpgrading ? t('proxyNodes.dispatching') : t('proxyNodes.confirmDispatch') }}
        </Button>
      </template>
    </Dialog>

    <!-- 连接事件对话框 -->
    <Dialog
      :open="showEventsDialog"
      :title="t('proxyNodes.connectionEvents')"
      :description="eventsNode ? t('proxyNodes.connectionHistory', { name: eventsNode.name }) : ''"
      size="lg"
      @update:open="(v: boolean) => { if (!v) { showEventsDialog = false; eventsNode = null; nodeEvents = [] } }"
    >
      <div class="space-y-3">
        <!-- 可靠性指标摘要 -->
        <div
          v-if="eventsNode"
          class="grid grid-cols-3 gap-3 text-sm"
        >
          <div class="bg-muted/40 rounded-lg px-3 py-2 text-center">
            <span class="block text-foreground/60 text-xs">{{ t('proxyNodes.failedRequests') }}</span>
            <span class="tabular-nums font-medium">{{ formatNumber(eventsNode.failed_requests || 0) }}</span>
          </div>
          <div class="bg-muted/40 rounded-lg px-3 py-2 text-center">
            <span class="block text-foreground/60 text-xs">{{ t('proxyNodes.dnsFailures') }}</span>
            <span class="tabular-nums font-medium">{{ formatNumber(eventsNode.dns_failures || 0) }}</span>
          </div>
          <div class="bg-muted/40 rounded-lg px-3 py-2 text-center">
            <span class="block text-foreground/60 text-xs">{{ t('proxyNodes.streamErrors') }}</span>
            <span class="tabular-nums font-medium">{{ formatNumber(eventsNode.stream_errors || 0) }}</span>
          </div>
        </div>

        <!-- 事件列表 -->
        <div
          v-if="loadingEvents"
          class="py-8 text-center text-muted-foreground text-sm"
        >
          {{ t('proxyNodes.loading') }}
        </div>
        <div
          v-else-if="nodeEvents.length === 0"
          class="py-8 text-center text-muted-foreground text-sm"
        >
          {{ t('proxyNodes.noEvents') }}
        </div>
        <div
          v-else
          class="max-h-80 overflow-y-auto space-y-1.5"
        >
          <div
            v-for="event in nodeEvents"
            :key="event.id"
            class="flex items-center gap-2 px-3 py-2 rounded-lg bg-muted/30 text-sm"
          >
            <Badge
              :variant="eventTypeVariant(event.event_type)"
              class="text-[10px] px-1.5 py-0 shrink-0"
            >
              {{ eventTypeLabel(event.event_type) }}
            </Badge>
            <span class="text-muted-foreground truncate flex-1">{{ event.detail || '-' }}</span>
            <span class="text-xs text-muted-foreground/70 tabular-nums shrink-0">{{ formatTime(event.created_at) }}</span>
          </div>
        </div>
      </div>
      <template #footer>
        <Button
          variant="outline"
          @click="showEventsDialog = false; eventsNode = null; nodeEvents = []"
        >
          {{ t('proxyNodes.close') }}
        </Button>
      </template>
    </Dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onBeforeUnmount, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { useProxyNodesStore } from '@/stores/proxy-nodes'
import { useToast } from '@/composables/useToast'
import { useConfirm } from '@/composables/useConfirm'
import { useClipboard } from '@/composables/useClipboard'
import {
  proxyNodesApi,
  type ProxyNode,
  type ProxyNodeEvent,
  type ProxyNodeInstallSession,
  type ProxyNodeMetricsResponse,
  type ProxyNodeRemoteConfig,
  type ProxyNodeSchedulingState,
  type ProxyNodeTestResult,
} from '@/api/proxy-nodes'

import {
  Card,
  Button,
  Badge,
  Input,
  Label,
  Select,
  SelectTrigger,
  SelectValue,
  SelectContent,
  SelectItem,
  Table,
  TableHeader,
  TableBody,
  TableRow,
  TableHead,
  SortableTableHead,
  TableFilterMenu,
  TableCell,
  Pagination,
  RefreshButton,
  Dialog,
} from '@/components/ui'

import { Search, Trash2, Plus, SquarePen, Activity, Loader2, Settings, History, ChevronDown, ChevronRight, Terminal, Copy, CheckCircle } from 'lucide-vue-next'
import { parseApiError } from '@/utils/errorParser'
import { formatRegion } from '@/utils/region'
import HardwareTooltip from './components/HardwareTooltip.vue'
import ProxyNodeDataPanel from './components/ProxyNodeDataPanel.vue'

const { success, error: toastError } = useToast()
const { confirmDanger } = useConfirm()
const { copyToClipboard } = useClipboard()
const { t, locale } = useI18n()
const store = useProxyNodesStore()

const searchQuery = ref('')
const filterStatus = ref('all')
const proxyNodeStatusFilterOptions = computed(() => [
  { value: 'all', label: t('proxyNodes.allStatus') },
  { value: 'online', label: t('proxyNodes.online') },
  { value: 'offline', label: t('proxyNodes.offline') },
])
const currentPage = ref(1)
const pageSize = ref(20)

// 手动添加/编辑对话框
const showAddDialog = ref(false)
const addingNode = ref(false)
const editingNode = ref<ProxyNode | null>(null)
const addMode = ref<'script' | 'manual'>('script')
const addForm = ref({
  name: '',
  proxy_url: '',
  username: '',
  password: '',
  region: '',
})
const installForm = ref({
  node_name: '',
})
const installSystem = ref<'unix' | 'windows'>('unix')
const installLoading = ref(false)
const installCopied = ref(false)
const proxyInstallSession = ref<ProxyNodeInstallSession | null>(null)
let installCopiedResetTimer: ReturnType<typeof setTimeout> | null = null

const proxyInstallCommand = computed(() => {
  if (!proxyInstallSession.value) return ''
  return installSystem.value === 'windows'
    ? proxyInstallSession.value.powershell_command
    : proxyInstallSession.value.unix_command
})

const proxyInstallHint = computed(() => {
  if (!proxyInstallSession.value) {
    return t('proxyNodes.installDefaultHint')
  }
  return t('proxyNodes.installExpiryHint', { minutes: Math.floor(proxyInstallSession.value.expires_in_seconds / 60) })
})

// 远程配置对话框 (aether-tunnel 节点)
const showConfigDialog = ref(false)
const savingConfig = ref(false)
const configNode = ref<ProxyNode | null>(null)
const configForm = ref({
  allowed_ports: '',
  log_level: 'info',
  heartbeat_interval: '30',
  scheduling_state: 'active' as ProxyNodeSchedulingState,
  upgrade_to: '',
})
const showBatchUpgradeDialog = ref(false)
const batchUpgradeVersion = ref('')
const batchUpgrading = ref(false)

// 连接事件对话框
const showEventsDialog = ref(false)
const eventsNode = ref<ProxyNode | null>(null)
const nodeEvents = ref<ProxyNodeEvent[]>([])
const loadingEvents = ref(false)

interface ProxyNodeDetailState {
  loading: boolean
  error: string | null
  node: ProxyNode | null
  metrics: ProxyNodeMetricsResponse | null
  events: ProxyNodeEvent[]
  loadedAt: number | null
}

const expandedNodeIds = ref(new Set<string>())
const nodeDetails = ref<Record<string, ProxyNodeDetailState>>({})

// 测试连通性
const testingNodes = ref(new Set<string>())
const testingUrl = ref(false)

const filteredNodes = computed(() => {
  let filtered = [...store.nodes]

  if (searchQuery.value) {
    const keywords = searchQuery.value.toLowerCase().split(/\s+/).filter(k => k.length > 0)
    filtered = filtered.filter(node => {
      const text = `${node.name} ${node.ip} ${node.region || ''}`.toLowerCase()
      return keywords.every(kw => text.includes(kw))
    })
  }

  if (filterStatus.value !== 'all') {
    filtered = filtered.filter(node => node.status === filterStatus.value)
  }

  return filtered
})

const paginatedNodes = computed(() => {
  const start = (currentPage.value - 1) * pageSize.value
  return filteredNodes.value.slice(start, start + pageSize.value)
})

watch([searchQuery, filterStatus], () => {
  currentPage.value = 1
})

watch(() => installForm.value.node_name, () => {
  resetProxyInstallState()
})

watch(installSystem, () => {
  installCopied.value = false
  clearInstallCopiedResetTimer()
})

onMounted(async () => {
  await store.fetchNodes()
})

onBeforeUnmount(() => {
  clearInstallCopiedResetTimer()
})

async function refresh() {
  await store.fetchNodes()
}

function formatConnectivityTestParts(result: ProxyNodeTestResult): string[] {
  const parts = [
    t('proxyNodes.probeDetail', { value: formatConnectivityProbe(result.probe_url) }),
    t('proxyNodes.timeoutDetail', { value: `${result.timeout_secs}s` }),
    t('proxyNodes.latencyDetail', { value: result.latency_ms != null ? `${result.latency_ms}ms` : t('proxyNodes.noSample') }),
  ]
  if (result.exit_ip) parts.push(t('proxyNodes.exitIpDetail', { value: result.exit_ip }))
  return parts
}

function formatConnectivityProbe(probeUrl: string) {
  try {
    const url = new URL(probeUrl)
    return `${url.host}${url.pathname === '/' ? '' : url.pathname}`
  } catch {
    return probeUrl
  }
}

async function handleTestUrl() {
  if (!addForm.value.proxy_url || testingUrl.value) return
  testingUrl.value = true
  try {
    const result = await proxyNodesApi.testProxyUrl({
      proxy_url: addForm.value.proxy_url,
      username: addForm.value.username || undefined,
      password: addForm.value.password || undefined,
    })
    if (result.success) {
      success(t('proxyNodes.testPassed', { details: formatConnectivityTestParts(result).join(t('proxyNodes.detailsSeparator')) }))
    } else {
      toastError(t('proxyNodes.testFailed', { details: formatConnectivityTestParts(result).join(t('proxyNodes.detailsSeparator')), error: result.error || t('common.unknown') }))
    }
  } catch (err: unknown) {
    toastError(parseApiError(err, t('proxyNodes.testRequestFailed')))
  } finally {
    testingUrl.value = false
  }
}

function clearInstallCopiedResetTimer() {
  if (installCopiedResetTimer) {
    clearTimeout(installCopiedResetTimer)
    installCopiedResetTimer = null
  }
}

function resetProxyInstallState() {
  proxyInstallSession.value = null
  installCopied.value = false
  clearInstallCopiedResetTimer()
}

function openAddDialog() {
  editingNode.value = null
  addMode.value = 'script'
  addForm.value = { name: '', proxy_url: '', username: '', password: '', region: '' }
  installForm.value = { node_name: '' }
  resetProxyInstallState()
  showAddDialog.value = true
}

async function refreshProxyInstallCommand() {
  const nodeName = installForm.value.node_name.trim()
  if (!nodeName || installLoading.value) return
  installLoading.value = true
  resetProxyInstallState()
  try {
    proxyInstallSession.value = await store.createInstallSession({ node_name: nodeName })
    success(t('proxyNodes.installGenerated'))
  } catch (err: unknown) {
    toastError(parseApiError(err, t('proxyNodes.installGenerateFailed')))
  } finally {
    installLoading.value = false
  }
}

async function copyProxyInstallCommand() {
  if (!proxyInstallCommand.value) return
  const copied = await copyToClipboard(proxyInstallCommand.value, false)
  if (!copied) return
  installCopied.value = true
  success(t('proxyNodes.installCopied'))
  clearInstallCopiedResetTimer()
  installCopiedResetTimer = setTimeout(() => {
    installCopied.value = false
    installCopiedResetTimer = null
  }, 2000)
}

async function handleEdit(node: ProxyNode) {
  try {
    const { node: detail } = await proxyNodesApi.getNode(node.id)
    editingNode.value = detail
    addForm.value = {
      name: detail.name,
      proxy_url: detail.proxy_url || '',
      username: detail.proxy_username || '',
      password: detail.proxy_password || '',
      region: detail.region || '',
    }
    addMode.value = 'manual'
    resetProxyInstallState()
    showAddDialog.value = true
  } catch (err: unknown) {
    toastError(parseApiError(err, t('proxyNodes.loadDetailsFailed')))
  }
}

function handleDialogClose(open: boolean) {
  if (!open) {
    showAddDialog.value = false
    editingNode.value = null
    addMode.value = 'script'
    addForm.value = { name: '', proxy_url: '', username: '', password: '', region: '' }
    installForm.value = { node_name: '' }
    resetProxyInstallState()
  }
}

async function handleUpdateManualNode() {
  if (!editingNode.value || !addForm.value.name || !addForm.value.proxy_url) return

  addingNode.value = true
  try {
    await proxyNodesApi.updateManualNode(editingNode.value.id, {
      name: addForm.value.name,
      proxy_url: addForm.value.proxy_url,
      username: addForm.value.username || undefined,
      // 空密码不发送（保留原值）
      password: addForm.value.password || undefined,
      region: addForm.value.region || undefined,
    })
    success(t('proxyNodes.updated'))
    handleDialogClose(false)
    await store.fetchNodes()
  } catch (err: unknown) {
    toastError(parseApiError(err, t('proxyNodes.updateFailed')))
  } finally {
    addingNode.value = false
  }
}

async function handleAddManualNode() {
  if (!addForm.value.name || !addForm.value.proxy_url) return

  addingNode.value = true
  try {
    await store.createManualNode({
      name: addForm.value.name,
      proxy_url: addForm.value.proxy_url,
      username: addForm.value.username || undefined,
      password: addForm.value.password || undefined,
      region: addForm.value.region || undefined,
    })
    success(t('proxyNodes.added'))
    handleDialogClose(false)
  } catch (err: unknown) {
    toastError(parseApiError(err, t('proxyNodes.addFailed')))
  } finally {
    addingNode.value = false
  }
}

function handleConfig(node: ProxyNode) {
  configNode.value = node
  const rc: ProxyNodeRemoteConfig = node.remote_config ?? {}
  configForm.value = {
    allowed_ports: rc.allowed_ports?.join(', ') || '',
    log_level: rc.log_level || 'info',
    heartbeat_interval: String(rc.heartbeat_interval || node.heartbeat_interval || 30),
    scheduling_state: rc.scheduling_state || 'active',
    upgrade_to: rc.upgrade_to || '',
  }
  showConfigDialog.value = true
}

function handleConfigDialogClose(open: boolean) {
  if (!open) {
    showConfigDialog.value = false
    configNode.value = null
  }
}

async function handleSaveConfig() {
  if (!configNode.value) return
  savingConfig.value = true
  try {
    const data: Partial<ProxyNodeRemoteConfig> = {}
    const portsInput = configForm.value.allowed_ports.trim()
    if (portsInput) {
      data.allowed_ports = portsInput
        .split(',')
        .map((s: string) => parseInt(s.trim()))
        .filter((n: number) => !isNaN(n) && n >= 1 && n <= 65535)
    } else if (configNode.value.remote_config?.allowed_ports) {
      // 输入清空 → 显式发送空数组以清除已有端口白名单
      data.allowed_ports = []
    }
    if (configForm.value.log_level) {
      data.log_level = configForm.value.log_level
    }
    const hb = parseInt(configForm.value.heartbeat_interval)
    if (!isNaN(hb) && hb >= 5) {
      data.heartbeat_interval = hb
    }
    data.scheduling_state = configForm.value.scheduling_state
    const targetVersion = configForm.value.upgrade_to.trim()
    if (targetVersion) {
      data.upgrade_to = targetVersion
    } else if (configNode.value.remote_config?.upgrade_to) {
      data.upgrade_to = null
    }
    await proxyNodesApi.updateNodeConfig(configNode.value.id, data)
    success(t('proxyNodes.remoteConfigSaved'))
    handleConfigDialogClose(false)
    await store.fetchNodes()
  } catch (err: unknown) {
    toastError(parseApiError(err, t('proxyNodes.saveFailed')))
  } finally {
    savingConfig.value = false
  }
}

async function handleBatchUpgrade() {
  const version = batchUpgradeVersion.value.trim()
  if (!version || batchUpgrading.value) return
  batchUpgrading.value = true
  try {
    const result = await proxyNodesApi.batchUpgrade(version)
    if (result.updated > 0) {
      success(t('proxyNodes.batchUpgradeSuccess', { updated: result.updated, version: result.version, skipped: result.skipped }))
    } else {
      success(t('proxyNodes.batchUpgradeNoChanges', { version: result.version }))
    }
    resetBatchUpgradeDialog()
    await store.fetchNodes()
  } catch (err: unknown) {
    toastError(parseApiError(err, t('proxyNodes.batchUpgradeFailed')))
  } finally {
    batchUpgrading.value = false
  }
}

function resetBatchUpgradeDialog() {
  showBatchUpgradeDialog.value = false
  batchUpgradeVersion.value = ''
}

async function handleDelete(node: ProxyNode) {
  const confirmed = await confirmDanger(
    t('proxyNodes.deleteConfirm', { name: node.name, address: node.tunnel_mode ? node.ip : `${node.ip}:${node.port}` }),
    t('proxyNodes.deleteTitle')
  )
  if (!confirmed) return

  try {
    const result = await proxyNodesApi.deleteProxyNode(node.id)
    await store.fetchNodes()
    if (result.cleared_system_proxy) {
      success(t('proxyNodes.deletedAndCleared'))
    } else {
      success(t('proxyNodes.deleted'))
    }
  } catch (err: unknown) {
    toastError(parseApiError(err, t('proxyNodes.deleteFailed')))
  }
}

async function handleTest(node: ProxyNode) {
  if (testingNodes.value.has(node.id)) return

  testingNodes.value.add(node.id)
  try {
    const result = await proxyNodesApi.testNode(node.id)
    if (result.success) {
      success(t('proxyNodes.testPassed', { details: formatConnectivityTestParts(result).join(t('proxyNodes.detailsSeparator')) }))
    } else {
      toastError(t('proxyNodes.testFailed', { details: formatConnectivityTestParts(result).join(t('proxyNodes.detailsSeparator')), error: result.error || t('common.unknown') }))
    }
  } catch (err: unknown) {
    toastError(parseApiError(err, t('proxyNodes.testRequestFailed')))
  } finally {
    testingNodes.value.delete(node.id)
  }
}

function createNodeDetailState(): ProxyNodeDetailState {
  return {
    loading: false,
    error: null,
    node: null,
    metrics: null,
    events: [],
    loadedAt: null,
  }
}

function updateNodeDetailState(nodeId: string, patch: Partial<ProxyNodeDetailState>) {
  nodeDetails.value = {
    ...nodeDetails.value,
    [nodeId]: {
      ...(nodeDetails.value[nodeId] ?? createNodeDetailState()),
      ...patch,
    },
  }
}

function isNodeExpanded(nodeId: string) {
  return expandedNodeIds.value.has(nodeId)
}

function toggleNodeDetails(node: ProxyNode) {
  const next = new Set(expandedNodeIds.value)
  if (next.has(node.id)) {
    next.delete(node.id)
    expandedNodeIds.value = next
    return
  }

  next.add(node.id)
  expandedNodeIds.value = next

  const detailState = nodeDetails.value[node.id]
  if (!detailState?.loadedAt && !detailState?.loading) {
    void loadNodeDetails(node)
  }
}

async function loadNodeDetails(node: ProxyNode) {
  updateNodeDetailState(node.id, { loading: true, error: null })
  const to = Math.floor(Date.now() / 1000)
  const from = to - 24 * 60 * 60
  const eventsFrom = to - 7 * 24 * 60 * 60

  try {
    const [detail, metrics, events] = await Promise.all([
      proxyNodesApi.getNode(node.id),
      proxyNodesApi.listNodeMetrics(node.id, { from, to, step: '1h' }),
      proxyNodesApi.listNodeEvents(node.id, { limit: 8, from: eventsFrom, to }),
    ])
    updateNodeDetailState(node.id, {
      loading: false,
      error: null,
      node: detail.node,
      metrics,
      events: events.items,
      loadedAt: Date.now(),
    })
  } catch (err: unknown) {
    updateNodeDetailState(node.id, {
      loading: false,
      error: parseApiError(err, t('proxyNodes.loadNodeDataFailed')),
    })
  }
}

async function handleViewEvents(node: ProxyNode) {
  eventsNode.value = node
  showEventsDialog.value = true
  loadingEvents.value = true
  try {
    const res = await proxyNodesApi.listNodeEvents(node.id, { limit: 50 })
    nodeEvents.value = res.items
  } catch (err: unknown) {
    toastError(parseApiError(err, t('proxyNodes.loadEventsFailed')))
  } finally {
    loadingEvents.value = false
  }
}

function eventTypeLabel(type: string) {
  switch (type) {
    case 'connected': return t('proxyNodes.eventConnected')
    case 'disconnected': return t('proxyNodes.eventDisconnected')
    case 'error': return t('common.error')
    default: return type
  }
}

function eventTypeVariant(type: string) {
  switch (type) {
    case 'connected': return 'success' as const
    case 'disconnected': return 'destructive' as const
    case 'error': return 'destructive' as const
    default: return 'secondary' as const
  }
}

function statusVariant(status: string) {
  switch (status) {
    case 'online': return 'success' as const
    case 'offline': return 'destructive' as const
    default: return 'secondary' as const
  }
}

function statusLabel(node: ProxyNode) {
  if (node.tunnel_mode && !node.is_manual) {
    switch (node.status) {
      case 'online': return t('proxyNodes.tunnelOnline')
      case 'offline': return t('proxyNodes.tunnelOffline')
      default: return node.status
    }
  }

  switch (node.status) {
    case 'online': return t('proxyNodes.online')
    case 'offline': return t('proxyNodes.offline')
    default: return node.status
  }
}

function statusTitle(node: ProxyNode) {
  if (node.tunnel_mode && !node.is_manual) {
    if (node.status === 'online') {
      return t('proxyNodes.tunnelOnlineHint')
    }
    return t('proxyNodes.tunnelOfflineHint')
  }

  switch (node.status) {
    case 'online': return t('proxyNodes.onlineHint')
    case 'offline': return t('proxyNodes.offlineHint')
    default: return node.status
  }
}

function formatNumber(n: number) {
  if (n >= 1_000_000) return `${(n / 1_000_000).toFixed(1)}M`
  if (n >= 1_000) return `${(n / 1_000).toFixed(1)}K`
  return String(n)
}

function formatTime(iso: string | null) {
  if (!iso) return '-'
  const d = new Date(iso)
  const now = new Date()
  const diff = (now.getTime() - d.getTime()) / 1000
  if (diff < 60) return t('proxyNodes.justNow')
  if (diff < 3600) return t('proxyNodes.minutesAgo', { count: Math.floor(diff / 60) })
  if (diff < 86400) return t('proxyNodes.hoursAgo', { count: Math.floor(diff / 3600) })
  return d.toLocaleDateString(locale.value, { month: '2-digit', day: '2-digit', hour: '2-digit', minute: '2-digit' })
}

function failureRate(node: ProxyNode) {
  if (!node.total_requests) return 0
  const failed = (node.failed_requests || 0) + (node.dns_failures || 0) + (node.stream_errors || 0)
  return (failed / node.total_requests) * 100
}

function formatFailureRate(node: ProxyNode) {
  if (!node.total_requests) return '-'
  const rate = failureRate(node)
  if (rate === 0) return '0%'
  if (rate < 0.1) return '<0.1%'
  return `${rate.toFixed(1)}%`
}

function nodeAddress(node: ProxyNode) {
  if (node.is_manual) return node.proxy_url || `${node.ip}:${node.port}`
  if (node.tunnel_mode) return node.ip || 'WebSocket Tunnel'
  return `${node.ip}:${node.port}`
}

function nodeProxyVersion(node: ProxyNode) {
  const metadata = node.proxy_metadata
  if (!metadata || typeof metadata !== 'object') return '-'
  const version = (metadata as Record<string, unknown>).version
  if (typeof version !== 'string') return '-'
  const normalized = version.trim()
  return normalized || '-'
}

type BadgeVariant = 'default' | 'secondary' | 'destructive' | 'outline' | 'success' | 'warning' | 'dark'

function nodeSchedulingBadge(node: ProxyNode): { label: string; variant: BadgeVariant } | null {
  switch (node.remote_config?.scheduling_state) {
    case 'draining':
      return { label: t('proxyNodes.draining'), variant: 'warning' }
    case 'cordoned':
      return { label: t('proxyNodes.cordoned'), variant: 'dark' }
    default:
      return null
  }
}
</script>
