<template>
  <div class="space-y-6 px-4 sm:px-6 lg:px-0">
    <div class="flex flex-col gap-3 xl:flex-row xl:items-center xl:justify-between">
      <div>
        <h1 class="text-lg font-semibold">
          {{ t('performance.title') }}
        </h1>
        <p class="text-xs text-muted-foreground">
          {{ t('performance.description') }}
        </p>
      </div>
      <div class="flex flex-wrap items-center gap-2">
        <Badge variant="outline">
          {{ t('performance.live') }}
        </Badge>
        <span class="text-xs text-muted-foreground">
          {{ t('performance.updated') }} {{ liveLastUpdatedLabel }}
        </span>
        <RefreshButton
          :loading="isRefreshing"
          :title="t('performance.refresh')"
          @click="handleManualRefresh"
        />
        <TimeRangePicker
          v-model="timeRange"
          :show-granularity="false"
        />
      </div>
    </div>

    <Card class="overflow-hidden">
      <div class="border-b border-border/70 bg-muted/20 px-4 py-3 sm:px-5">
        <div class="flex flex-col gap-2 sm:flex-row sm:items-center sm:justify-between">
          <div>
            <h2 class="text-sm font-semibold">
              {{ t('performance.runtime') }}
            </h2>
            <p class="text-xs text-muted-foreground">
              {{ t('performance.runtimeHint') }}
            </p>
          </div>
          <div class="flex flex-wrap items-center gap-2">
            <Badge :variant="healthStatusVariant">
              {{ healthStatusText }}
            </Badge>
            <Badge variant="outline">
              {{ metricsAvailabilityText }}
            </Badge>
          </div>
        </div>
      </div>

      <div class="p-4 sm:p-5">
        <div
          v-if="liveLoading && !liveReady"
          class="py-6"
        >
          <LoadingState :message="t('performance.loading')" />
        </div>

        <div
          v-else-if="!liveReady"
          class="rounded-xl border border-dashed border-border/70 bg-muted/15 px-4 py-6 text-sm text-muted-foreground"
        >
          {{ t('performance.unavailable') }}
        </div>

        <div
          v-else
          class="space-y-4"
        >
          <div
            v-if="liveLoadError"
            class="rounded-lg border border-yellow-300/70 bg-yellow-50/80 px-3 py-2 text-xs text-yellow-900 dark:border-yellow-900/60 dark:bg-yellow-950/30 dark:text-yellow-100"
          >
            {{ liveLoadError }}
          </div>

          <div class="grid grid-cols-1 gap-3 sm:grid-cols-2 xl:grid-cols-3 2xl:grid-cols-6">
            <div
              v-for="card in liveSummaryCards"
              :key="card.title"
              class="rounded-xl border border-border/70 bg-card/70 px-4 py-3"
            >
              <div class="flex items-center justify-between gap-3">
                <span class="text-xs text-muted-foreground">{{ card.title }}</span>
                <component
                  :is="card.icon"
                  class="h-4 w-4"
                  :class="card.iconClass"
                />
              </div>
              <div class="mt-3 text-2xl font-semibold tracking-tight">
                {{ card.value }}
              </div>
              <div class="mt-2 text-xs text-muted-foreground">
                {{ card.hint }}
              </div>
            </div>
          </div>

          <div class="grid grid-cols-1 gap-4 xl:grid-cols-3">
            <div class="grid grid-cols-1 gap-4 lg:grid-cols-2 xl:col-span-2">
              <section class="rounded-xl border border-border/70 bg-card/60 p-4 lg:col-span-2">
                <div class="flex items-center justify-between gap-3">
                  <h3 class="text-sm font-semibold">
                    {{ t('performance.concurrency') }}
                  </h3>
                  <Badge variant="outline">
                    {{ t('performance.global') }} {{ distributedGateText }}
                  </Badge>
                </div>

                <div class="mt-4 grid grid-cols-1 gap-3 lg:grid-cols-2">
                  <div class="rounded-lg border border-border/60 bg-background/50 px-3 py-3">
                    <div class="flex items-center justify-between gap-3">
                      <div class="text-sm font-medium">
                        {{ t('performance.currentNode') }}
                      </div>
                      <Badge variant="outline">
                        {{ t('performance.local') }}
                      </Badge>
                    </div>
                    <div class="mt-4 grid grid-cols-2 gap-3 text-sm">
                      <div>
                        <div class="text-xs text-muted-foreground">
                          {{ t('performance.processing') }}
                        </div>
                        <div class="mt-1 text-lg font-semibold">
                          {{ formatMetricNumber(gatewayMetrics?.local.inFlight) }}
                        </div>
                      </div>
                      <div>
                        <div class="text-xs text-muted-foreground">
                          {{ t('performance.capacity') }}
                        </div>
                        <div class="mt-1 text-lg font-semibold">
                          {{ formatMetricNumber(gatewayMetrics?.local.availablePermits) }}
                        </div>
                      </div>
                      <div>
                        <div class="text-xs text-muted-foreground">
                          {{ t('performance.peak') }}
                        </div>
                        <div class="mt-1 text-lg font-semibold">
                          {{ formatMetricNumber(gatewayMetrics?.local.highWatermark) }}
                        </div>
                      </div>
                      <div>
                        <div class="text-xs text-muted-foreground">
                          {{ t('performance.throttled') }}
                        </div>
                        <div class="mt-1 text-lg font-semibold">
                          {{ formatMetricNumber(gatewayMetrics?.local.rejectedTotal) }}
                        </div>
                      </div>
                    </div>
                  </div>

                  <div class="rounded-lg border border-border/60 bg-background/50 px-3 py-3">
                    <div class="flex items-center justify-between gap-3">
                      <div class="text-sm font-medium">
                        {{ t('performance.global') }}
                      </div>
                      <Badge :variant="distributedGateVariant">
                        {{ distributedGateText }}
                      </Badge>
                    </div>
                    <div class="mt-4 grid grid-cols-2 gap-3 text-sm">
                      <div>
                        <div class="text-xs text-muted-foreground">
                          {{ t('performance.processing') }}
                        </div>
                        <div class="mt-1 text-lg font-semibold">
                          {{ formatMetricNumber(gatewayMetrics?.distributed.inFlight) }}
                        </div>
                      </div>
                      <div>
                        <div class="text-xs text-muted-foreground">
                          {{ t('performance.capacity') }}
                        </div>
                        <div class="mt-1 text-lg font-semibold">
                          {{ formatMetricNumber(gatewayMetrics?.distributed.availablePermits) }}
                        </div>
                      </div>
                      <div>
                        <div class="text-xs text-muted-foreground">
                          {{ t('performance.peak') }}
                        </div>
                        <div class="mt-1 text-lg font-semibold">
                          {{ formatMetricNumber(gatewayMetrics?.distributed.highWatermark) }}
                        </div>
                      </div>
                      <div>
                        <div class="text-xs text-muted-foreground">
                          {{ t('performance.throttled') }}
                        </div>
                        <div class="mt-1 text-lg font-semibold">
                          {{ formatMetricNumber(gatewayMetrics?.distributed.rejectedTotal) }}
                        </div>
                      </div>
                    </div>
                  </div>
                </div>
                <p
                  v-if="gatewayMetrics?.distributed.unavailable"
                  class="mt-3 text-xs text-yellow-700 dark:text-yellow-300"
                >
                  {{ t('performanceExtra.globalUnavailable') }}
                </p>
              </section>

              <section class="rounded-xl border border-border/70 bg-card/60 p-4">
                <div class="flex items-center justify-between gap-3">
                  <h3 class="text-sm font-semibold">
                    {{ t('performance.proxyTunnel') }}
                  </h3>
                  <Badge variant="outline">
                    {{ t('performance.liveConnections') }}
                  </Badge>
                </div>
                <div class="mt-4 grid grid-cols-2 gap-3 text-sm">
                  <div>
                    <div class="text-xs text-muted-foreground">
                      {{ t('performance.nodes') }}
                    </div>
                    <div class="mt-1 text-lg font-semibold">
                      {{ formatMetricNumber(currentTunnelNodes) }}
                    </div>
                  </div>
                  <div>
                    <div class="text-xs text-muted-foreground">
                      {{ t('performance.availableConnections') }}
                    </div>
                    <div class="mt-1 text-lg font-semibold">
                      {{ formatMetricNumber(gatewayMetrics?.tunnel.availableProxyConnections) }}
                    </div>
                  </div>
                  <div>
                    <div class="text-xs text-muted-foreground">
                      {{ t('performance.activeFlows') }}
                    </div>
                    <div class="mt-1 text-lg font-semibold">
                      {{ formatMetricNumber(currentActiveStreams) }}
                    </div>
                  </div>
                  <div>
                    <div class="text-xs text-muted-foreground">
                      {{ t('performance.avoiding') }}
                    </div>
                    <div class="mt-1 text-lg font-semibold">
                      {{ formatMetricNumber(gatewayMetrics?.tunnel.softAvoidProxyConnections) }}
                    </div>
                  </div>
                </div>
              </section>

              <section class="rounded-xl border border-border/70 bg-card/60 p-4">
                <div class="flex items-center justify-between gap-3">
                  <h3 class="text-sm font-semibold">
                    {{ t('performance.tunnelPressure') }}
                  </h3>
                  <Badge variant="outline">
                    {{ t('performanceExtra.queueWithValue', { value: tunnelQueueUtilizationText }) }}
                  </Badge>
                </div>
                <div class="mt-4 grid grid-cols-2 gap-3 text-sm">
                  <div>
                    <div class="text-xs text-muted-foreground">
                      {{ t('performance.queued') }}
                    </div>
                    <div class="mt-1 text-lg font-semibold">
                      {{ formatMetricNumber(gatewayMetrics?.tunnel.outboundQueueDepthTotal) }}
                    </div>
                  </div>
                  <div>
                    <div class="text-xs text-muted-foreground">
                      {{ t('performance.peakQueue') }}
                    </div>
                    <div class="mt-1 text-lg font-semibold">
                      {{ formatMetricNumber(gatewayMetrics?.tunnel.outboundQueueDepthMax) }}
                    </div>
                  </div>
                  <div>
                    <div class="text-xs text-muted-foreground">
                      {{ t('performance.queueRejected') }}
                    </div>
                    <div class="mt-1 text-lg font-semibold">
                      {{ formatMetricNumber(tunnelQueueRejectedTotal) }}
                    </div>
                  </div>
                  <div>
                    <div class="text-xs text-muted-foreground">
                      {{ t('performance.unavailableTunnel') }}
                    </div>
                    <div class="mt-1 text-lg font-semibold">
                      {{ formatMetricNumber(tunnelSelectionPressureTotal) }}
                    </div>
                  </div>
                </div>
              </section>
            </div>

            <section class="rounded-xl border border-border/70 bg-card/60 p-4">
              <div class="flex items-center justify-between gap-3">
                <h3 class="text-sm font-semibold">
                  {{ t('performance.fallback') }}
                </h3>
                <span class="text-xs text-muted-foreground">
                  {{ t('performanceExtra.totalWithValue', { value: formatMetricNumber(gatewayMetrics?.fallbackTotal) }) }}
                </span>
              </div>

              <div
                v-if="!fallbackRows.length"
                class="mt-4 rounded-lg border border-dashed border-border/70 px-3 py-4 text-sm text-muted-foreground"
              >
                {{ t('performance.noFallback') }}
              </div>

              <div
                v-else
                class="mt-4 space-y-3"
              >
                <div
                  v-for="item in fallbackRows"
                  :key="item.name"
                  class="rounded-lg border border-border/60 bg-background/50 px-3 py-3"
                >
                  <div class="flex items-center justify-between gap-3">
                    <span class="text-sm font-medium">{{ item.label }}</span>
                    <span class="text-sm font-semibold">{{ formatMetricNumber(item.total) }}</span>
                  </div>
                  <div class="mt-2 h-2 overflow-hidden rounded-full bg-muted">
                    <div
                      class="h-full rounded-full bg-primary/80"
                      :style="{ width: `${Math.max(item.ratio, 8)}%` }"
                    />
                  </div>
                </div>
              </div>
            </section>
          </div>

          <div class="grid grid-cols-1 items-stretch gap-4 xl:grid-cols-2">
            <section class="flex h-full flex-col rounded-xl border border-border/70 bg-card/60 p-4">
              <div class="flex items-center justify-between gap-3">
                <h3 class="text-sm font-semibold">
                  {{ t('performance.recentErrors') }}
                </h3>
                <div class="flex items-center gap-2">
                  <Button
                    v-if="hasMoreRecentErrors"
                    variant="ghost"
                    size="sm"
                    class="h-7 gap-1 px-2 text-xs"
                    :title="recentErrorsExpanded ? t('performance.collapse') : t('performance.expand')"
                    @click="recentErrorsExpanded = !recentErrorsExpanded"
                  >
                    <component
                      :is="recentErrorsExpanded ? ChevronUp : ChevronDown"
                      class="h-3.5 w-3.5"
                    />
                    {{ recentErrorsExpanded ? t('performance.collapse') : `${t('performance.expand')} ${recentErrors.length}` }}
                  </Button>
                  <span class="text-xs text-muted-foreground">
                    {{ formatMetricNumber(resilienceStatus?.error_statistics.total_errors) }} / 24h
                  </span>
                </div>
              </div>

              <div
                v-if="!recentErrors.length"
                class="mt-4 flex-1 rounded-lg border border-dashed border-border/70 px-3 py-4 text-sm text-muted-foreground"
              >
                {{ t('performance.noErrors') }}
              </div>

              <div
                v-else
                class="mt-4 min-h-0 flex-1"
              >
                <div :class="recentErrorsListClass">
                  <article
                    v-for="item in visibleRecentErrors"
                    :key="item.error_id"
                    class="rounded-lg border border-border/60 bg-background/50 px-3 py-3"
                  >
                    <div class="flex items-start justify-between gap-4">
                      <div>
                        <div class="text-sm font-medium">
                          {{ item.error_type }}
                        </div>
                        <div class="mt-1 text-xs text-muted-foreground">
                          {{ item.operation }}
                        </div>
                      </div>
                      <span class="shrink-0 text-xs text-muted-foreground">
                        {{ formatDate(item.timestamp) }}
                      </span>
                    </div>

                    <div class="mt-2 flex flex-wrap gap-2">
                      <Badge variant="outline">
                        HTTP {{ item.context.status_code ?? '-' }}
                      </Badge>
                      <Badge variant="outline">
                        {{ item.context.provider_name || item.context.provider_id || t('performanceExtra.unknownUpstream') }}
                      </Badge>
                      <Badge variant="outline">
                        {{ item.context.api_format || item.context.model || t('performanceExtra.unknownFormat') }}
                      </Badge>
                    </div>

                    <p
                      v-if="item.context.error_message"
                      class="mt-2 line-clamp-2 break-words text-xs text-muted-foreground"
                    >
                      {{ item.context.error_message }}
                    </p>
                  </article>
                </div>
              </div>
            </section>

            <section class="flex h-full flex-col rounded-xl border border-border/70 bg-card/60 p-4">
              <div class="flex items-center justify-between gap-3">
                <h3 class="text-sm font-semibold">
                  {{ t('performance.circuitHistory') }}
                </h3>
                <span class="text-xs text-muted-foreground">
                  {{ t('performanceExtra.openCircuits', { count: formatMetricNumber(resilienceStatus?.error_statistics.open_circuit_breakers) }) }}
                </span>
              </div>

              <div
                v-if="!circuitHistory.length"
                class="mt-4 rounded-lg border border-dashed border-border/70 px-3 py-4 text-sm text-muted-foreground"
              >
                {{ t('performanceExtra.noCircuitEvents') }}
              </div>

              <div
                v-else
                class="mt-4 space-y-3"
              >
                <article
                  v-for="item in circuitHistory"
                  :key="`${item.key_id}-${item.api_format}-${item.timestamp}`"
                  class="rounded-lg border border-border/60 bg-background/50 px-3 py-3"
                >
                  <div class="flex items-start justify-between gap-4">
                    <div>
                      <div class="flex flex-wrap items-center gap-2">
                        <span class="text-sm font-medium">
                          {{ item.provider_name || item.provider_id }}
                        </span>
                        <Badge :variant="item.event === 'opened' ? 'destructive' : 'warning'">
                          {{ item.event === 'opened' ? t('performanceExtra.opened') : t('performanceExtra.halfOpen') }}
                        </Badge>
                      </div>
                      <div class="mt-1 text-xs text-muted-foreground">
                        <button
                          type="button"
                          class="transition-colors hover:text-foreground"
                          :title="`${getCircuitHistoryAccountDisplay(item)}\n${t('performanceExtra.clickToCopy')}`"
                          @click="copyCircuitHistoryAccount(item)"
                        >
                          {{ getCircuitHistoryAccountDisplay(item) }}
                          <Copy class="ml-1 inline h-3 w-3" />
                        </button>
                        · {{ item.api_format || t('performanceExtra.unknownFormat') }}
                      </div>
                    </div>
                    <span class="shrink-0 text-xs text-muted-foreground">
                      {{ formatDate(item.timestamp) }}
                    </span>
                  </div>

                  <div class="mt-2 text-xs text-muted-foreground">
                    {{ t('performanceExtra.reason', { reason: item.reason || t('performanceExtra.notProvided') }) }}
                  </div>
                  <div class="mt-1 text-xs text-muted-foreground">
                    {{ t('performanceExtra.recoveryWindow', { value: item.recovery_seconds != null ? `${item.recovery_seconds}s` : '-' }) }}
                  </div>
                </article>
              </div>

              <div class="mt-4 border-t border-border/70 pt-4">
                <h4 class="text-xs font-semibold uppercase tracking-wide text-muted-foreground">
                  {{ t('performanceExtra.recommendations') }}
                </h4>
                <ul
                  v-if="resilienceRecommendations.length"
                  class="mt-3 space-y-2 text-sm"
                >
                  <li
                    v-for="item in resilienceRecommendations"
                    :key="item"
                    class="rounded-lg border border-border/60 bg-background/50 px-3 py-2"
                  >
                    {{ item }}
                  </li>
                </ul>
                <div
                  v-else
                  class="mt-3 text-sm text-muted-foreground"
                >
                  {{ t('performanceExtra.noRecommendations') }}
                </div>
              </div>
            </section>
          </div>
        </div>
      </div>
    </Card>

    <Card class="space-y-4 p-4">
      <div class="flex flex-col gap-2 sm:flex-row sm:items-center sm:justify-between">
        <div>
          <h3 class="text-sm font-semibold">
            {{ t('performanceExtra.upstreamPerformance') }}
          </h3>
          <p class="text-xs text-muted-foreground">
            {{ providerPerformanceSubtitle }}
          </p>
        </div>
        <div class="flex items-center gap-2">
          <Badge variant="outline">
            Top {{ providerPerformanceRows.length || 0 }}
          </Badge>
          <Button
            v-if="hasProviderPerformanceFilters"
            variant="ghost"
            size="sm"
            class="h-8 gap-1 px-2 text-xs"
            :title="t('performanceExtra.clearFiltersTitle')"
            @click="resetProviderPerformanceFilters"
          >
            <FilterX class="h-3.5 w-3.5" />
            {{ t('performanceExtra.clear') }}
          </Button>
        </div>
      </div>
      <div class="grid grid-cols-1 gap-2 pt-1 sm:grid-cols-2 xl:grid-cols-4 2xl:grid-cols-7">
        <Input
          v-model="providerPerformanceProviderId"
          size="sm"
          :placeholder="t('performanceExtra.upstreamId')"
        />
        <Input
          v-model="providerPerformanceModel"
          size="sm"
          :placeholder="t('performanceExtra.model')"
        />
        <Input
          v-model="providerPerformanceApiFormat"
          size="sm"
          :placeholder="t('performanceExtra.apiFormat')"
        />
        <Input
          v-model="providerPerformanceEndpointKind"
          size="sm"
          :placeholder="t('performanceExtra.endpointType')"
        />
        <Select v-model="providerPerformanceIsStream">
          <SelectTrigger class="h-8 text-xs border-border/60">
            <SelectValue :placeholder="t('performanceExtra.streaming')" />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="all">
              {{ t('performanceExtra.allStreaming') }}
            </SelectItem>
            <SelectItem value="true">
              {{ t('performanceExtra.streamingOnly') }}
            </SelectItem>
            <SelectItem value="false">
              {{ t('performanceExtra.nonStreaming') }}
            </SelectItem>
          </SelectContent>
        </Select>
        <Select v-model="providerPerformanceHasFormatConversion">
          <SelectTrigger class="h-8 text-xs border-border/60">
            <SelectValue :placeholder="t('performanceExtra.conversion')" />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="all">
              {{ t('performanceExtra.allConversions') }}
            </SelectItem>
            <SelectItem value="true">
              {{ t('performanceExtra.convertedOnly') }}
            </SelectItem>
            <SelectItem value="false">
              {{ t('performanceExtra.notConverted') }}
            </SelectItem>
          </SelectContent>
        </Select>
        <Input
          v-model="providerPerformanceSlowThresholdMs"
          size="sm"
          type="number"
          min="1"
          max="600000"
          :placeholder="t('performanceExtra.slowThreshold')"
        />
      </div>

      <div
        v-if="providerPerformanceLoading"
        class="p-6"
      >
        <LoadingState />
      </div>

      <div
        v-else
        class="space-y-4"
      >
        <div class="grid grid-cols-1 gap-3 sm:grid-cols-2 xl:grid-cols-4">
          <div
            v-for="card in providerPerformanceSummaryCards"
            :key="card.title"
            class="rounded-xl border border-border/70 bg-card/70 px-4 py-3"
          >
            <div class="flex items-center justify-between gap-3">
              <span class="text-xs text-muted-foreground">{{ card.title }}</span>
              <component
                :is="card.icon"
                class="h-4 w-4"
                :class="card.iconClass"
              />
            </div>
            <div class="mt-3 text-2xl font-semibold tracking-tight">
              {{ card.value }}
            </div>
            <div class="mt-2 text-xs text-muted-foreground">
              {{ card.hint }}
            </div>
          </div>
        </div>

        <div
          v-if="providerPerformanceRows.length"
          class="overflow-x-auto rounded-lg border border-border/70"
        >
          <table class="min-w-full divide-y divide-border/70 text-sm">
            <thead class="bg-muted/30 text-xs text-muted-foreground">
              <tr>
                <th class="px-3 py-2 text-left font-medium">
                  {{ t('performanceExtra.upstreamService') }}
                </th>
                <th class="px-3 py-2 text-right font-medium">
                  {{ t('performanceExtra.requests') }}
                </th>
                <th class="px-3 py-2 text-right font-medium">
                  {{ t('performanceExtra.successRate') }}
                </th>
                <th class="px-3 py-2 text-right font-medium">
                  {{ t('performanceExtra.errorRate') }}
                </th>
                <th class="px-3 py-2 text-right font-medium">
                  {{ t('performanceExtra.outputTps') }}
                </th>
                <th class="px-3 py-2 text-right font-medium">
                  {{ t('performanceExtra.avgFirstByte') }}
                </th>
                <th class="px-3 py-2 text-right font-medium">
                  {{ t('performanceExtra.avgResponse') }}
                </th>
                <th class="px-3 py-2 text-right font-medium">
                  {{ t('performanceExtra.responsePercentiles') }}
                </th>
                <th class="px-3 py-2 text-right font-medium">
                  {{ t('performanceExtra.firstBytePercentiles') }}
                </th>
                <th class="px-3 py-2 text-right font-medium">
                  {{ t('performanceExtra.slowRequests') }}
                </th>
                <th class="px-3 py-2 text-right font-medium">
                  {{ t('performanceExtra.sampleCoverage') }}
                </th>
              </tr>
            </thead>
            <tbody class="divide-y divide-border/60">
              <tr
                v-for="provider in providerPerformanceRows"
                :key="provider.provider_id"
                class="bg-background/40"
              >
                <td class="max-w-[220px] px-3 py-2">
                  <div class="truncate font-medium">
                    {{ provider.provider }}
                  </div>
                  <div class="truncate text-xs text-muted-foreground">
                    {{ provider.provider_id }}
                  </div>
                </td>
                <td class="px-3 py-2 text-right">
                  {{ formatMetricNumber(provider.request_count) }}
                </td>
                <td class="px-3 py-2 text-right">
                  {{ formatProviderPerformanceMetric(provider.success_rate, '%') }}
                </td>
                <td class="px-3 py-2 text-right">
                  {{ formatErrorRate(provider.success_rate) }}
                </td>
                <td class="px-3 py-2 text-right">
                  {{ formatProviderPerformanceMetric(provider.avg_output_tps, ' tps') }}
                </td>
                <td class="px-3 py-2 text-right">
                  {{ formatProviderPerformanceMetric(provider.avg_first_byte_time_ms, 'ms') }}
                </td>
                <td class="px-3 py-2 text-right">
                  {{ formatProviderPerformanceMetric(provider.avg_response_time_ms, 'ms') }}
                </td>
                <td class="px-3 py-2 text-right">
                  {{ formatProviderPerformanceMetric(provider.p90_response_time_ms, 'ms', 0) }}
                  /
                  {{ formatProviderPerformanceMetric(provider.p99_response_time_ms, 'ms', 0) }}
                </td>
                <td class="px-3 py-2 text-right">
                  {{ formatProviderPerformanceMetric(provider.p90_first_byte_time_ms, 'ms', 0) }}
                  /
                  {{ formatProviderPerformanceMetric(provider.p99_first_byte_time_ms, 'ms', 0) }}
                </td>
                <td class="px-3 py-2 text-right">
                  {{ formatMetricNumber(provider.slow_request_count) }}
                </td>
                <td class="px-3 py-2 text-right text-xs text-muted-foreground">
                  {{ providerSampleCoverageText(provider) }}
                </td>
              </tr>
            </tbody>
          </table>
        </div>

        <div
          v-else
          class="rounded-lg border border-dashed border-border/70 px-3 py-4 text-sm text-muted-foreground"
        >
          {{ t('performanceExtra.noUpstreamData') }}
        </div>
      </div>
    </Card>

    <div class="grid grid-cols-1 gap-4 lg:grid-cols-2">
      <Card class="space-y-3 p-4">
        <h3 class="text-sm font-semibold">
          {{ t('performanceExtra.outputTpsTrend') }}
        </h3>
        <div
          v-if="providerPerformanceLoading"
          class="p-6"
        >
          <LoadingState />
        </div>
        <div
          v-else
          class="h-[260px]"
        >
          <LineChart
            :data="providerTpsChartData"
            :options="providerTpsChartOptions"
          />
        </div>
      </Card>
      <Card class="space-y-3 p-4">
        <h3 class="text-sm font-semibold">
          {{ t('performanceExtra.avgFirstByteTrend') }}
        </h3>
        <div
          v-if="providerPerformanceLoading"
          class="p-6"
        >
          <LoadingState />
        </div>
        <div
          v-else
          class="h-[260px]"
        >
          <LineChart
            :data="providerFirstByteChartData"
            :options="providerLatencyChartOptions"
          />
        </div>
      </Card>
    </div>

    <div class="grid grid-cols-1 gap-4 lg:grid-cols-2">
      <Card class="p-4">
        <PercentileChart
          :title="t('performanceExtra.responseLatencyPercentiles')"
          :series="percentiles"
          mode="response"
          :loading="percentileLoading"
        />
      </Card>
      <Card class="p-4">
        <PercentileChart
          :title="t('performanceExtra.firstByteLatencyPercentiles')"
          :series="percentiles"
          mode="ttfb"
          :loading="percentileLoading"
        />
      </Card>
    </div>

    <div class="grid grid-cols-1 gap-4 lg:grid-cols-2">
      <Card class="p-4">
        <ErrorDistributionChart
          :title="t('performanceExtra.errorDistribution')"
          :distribution="errorDistribution"
          :loading="errorLoading"
        />
      </Card>
      <Card class="space-y-3 p-4">
        <h3 class="text-sm font-semibold">
          {{ t('performanceExtra.errorTrend') }}
        </h3>
        <div
          v-if="errorLoading"
          class="p-6"
        >
          <LoadingState />
        </div>
        <div
          v-else
          class="h-[260px]"
        >
          <LineChart :data="errorTrendChartData" />
        </div>
      </Card>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'

const { t } = useI18n()
import {
  Activity,
  AlertTriangle,
  Cable,
  CheckCircle2,
  ChevronDown,
  ChevronUp,
  Copy,
  FilterX,
  GitBranch,
  Gauge,
  ShieldCheck,
  Timer,
  Workflow,
  Zap,
} from 'lucide-vue-next'
import {
  adminApi,
  type ErrorDistributionResponse,
  type PercentileItem,
  type ProviderPerformanceItem,
  type ProviderPerformanceResponse,
} from '@/api/admin'
import {
  monitoringApi,
  type AdminMonitoringCircuitHistoryItem,
  type AdminMonitoringResilienceStatus,
  type AdminMonitoringSystemStatus,
  type GatewayMetricsSummary,
} from '@/api/monitoring'
import LineChart from '@/components/charts/LineChart.vue'
import { LoadingState, TimeRangePicker } from '@/components/common'
import { ErrorDistributionChart, PercentileChart } from '@/components/stats'
import Badge from '@/components/ui/badge.vue'
import Button from '@/components/ui/button.vue'
import Card from '@/components/ui/card.vue'
import Input from '@/components/ui/input.vue'
import RefreshButton from '@/components/ui/refresh-button.vue'
import Select from '@/components/ui/select.vue'
import SelectContent from '@/components/ui/select-content.vue'
import SelectItem from '@/components/ui/select-item.vue'
import SelectTrigger from '@/components/ui/select-trigger.vue'
import SelectValue from '@/components/ui/select-value.vue'
import { useClipboard } from '@/composables/useClipboard'
import { useToast } from '@/composables/useToast'
import { getDateRangeFromPeriod } from '@/features/usage/composables'
import type { DateRangeParams } from '@/features/usage/types'
import { formatDate, formatNumber } from '@/utils/format'
import { log } from '@/utils/logger'
import {
  buildProviderPerformanceChartData,
  formatDurationMs,
  formatProviderPerformanceMetric,
} from './performanceAnalysisHelpers'

const LIVE_REFRESH_INTERVAL_MS = 10_000
const RECENT_ERRORS_COLLAPSED_LIMIT = 3
const DEFAULT_PROVIDER_PERFORMANCE_SLOW_THRESHOLD_MS = 10_000

type ProviderPerformanceBooleanFilter = 'all' | 'true' | 'false'
type ProviderPerformanceParams = NonNullable<Parameters<typeof adminApi.getProviderPerformance>[0]>

const timeRange = ref<DateRangeParams>(getDateRangeFromPeriod('last7days'))
const { error: showError } = useToast()
const { copyToClipboard } = useClipboard()

const percentiles = ref<PercentileItem[]>([])
const percentileLoading = ref(false)

const errorDistribution = ref<ErrorDistributionResponse['distribution']>([])
const errorTrend = ref<ErrorDistributionResponse['trend']>([])
const errorLoading = ref(false)

const providerPerformance = ref<ProviderPerformanceResponse | null>(null)
const providerPerformanceLoading = ref(false)
const providerPerformanceProviderId = ref('')
const providerPerformanceModel = ref('')
const providerPerformanceApiFormat = ref('')
const providerPerformanceEndpointKind = ref('')
const providerPerformanceIsStream = ref<ProviderPerformanceBooleanFilter>('all')
const providerPerformanceHasFormatConversion = ref<ProviderPerformanceBooleanFilter>('all')
const providerPerformanceSlowThresholdMs = ref(String(DEFAULT_PROVIDER_PERFORMANCE_SLOW_THRESHOLD_MS))

const systemStatus = ref<AdminMonitoringSystemStatus | null>(null)
const resilienceStatus = ref<AdminMonitoringResilienceStatus | null>(null)
const circuitHistory = ref<AdminMonitoringCircuitHistoryItem[]>([])
const gatewayMetrics = ref<GatewayMetricsSummary | null>(null)
const liveLoading = ref(false)
const liveRefreshing = ref(false)
const liveReady = ref(false)
const liveLoadError = ref<string | null>(null)
const liveLastUpdatedAt = ref<string | null>(null)
const recentErrorsExpanded = ref(false)

let percentilesRequestId = 0
let errorsRequestId = 0
let providerPerformanceRequestId = 0
let liveRequestId = 0
let loadAllPromise: Promise<void> | null = null
let hasPendingLoadAll = false
let loadAllDebounceTimer: ReturnType<typeof setTimeout> | null = null
let providerPerformanceDebounceTimer: ReturnType<typeof setTimeout> | null = null
let liveRefreshTimer: ReturnType<typeof setInterval> | null = null

function getCircuitHistoryAccountDisplay(item: AdminMonitoringCircuitHistoryItem): string {
  return item.key_account_label || item.key_name || item.key_id
}

async function copyCircuitHistoryAccount(item: AdminMonitoringCircuitHistoryItem): Promise<void> {
  await copyToClipboard(getCircuitHistoryAccountDisplay(item))
}

function buildTimeRangeParams() {
  return {
    start_date: timeRange.value.start_date,
    end_date: timeRange.value.end_date,
    preset: timeRange.value.preset,
    timezone: timeRange.value.timezone,
    tz_offset_minutes: timeRange.value.tz_offset_minutes
  }
}

function normalizeProviderPerformanceFilter(value: string): string | undefined {
  const trimmed = value.trim()
  return trimmed ? trimmed : undefined
}

function parseProviderPerformanceBooleanFilter(
  value: ProviderPerformanceBooleanFilter
): boolean | undefined {
  if (value === 'true') return true
  if (value === 'false') return false
  return undefined
}

function clampProviderPerformanceSlowThreshold(value: string): number {
  const parsed = Number(value)
  if (!Number.isFinite(parsed)) {
    return DEFAULT_PROVIDER_PERFORMANCE_SLOW_THRESHOLD_MS
  }
  return Math.min(600_000, Math.max(1, Math.round(parsed)))
}

function resolveProviderPerformanceGranularity(): 'day' | 'hour' {
  if (timeRange.value.preset === 'today' || timeRange.value.preset === 'yesterday') {
    return 'hour'
  }

  if (!timeRange.value.preset && timeRange.value.start_date && timeRange.value.end_date) {
    return timeRange.value.start_date === timeRange.value.end_date ? 'hour' : 'day'
  }

  return 'day'
}

const providerPerformanceSlowThresholdValue = computed(() => (
  clampProviderPerformanceSlowThreshold(providerPerformanceSlowThresholdMs.value)
))

const providerPerformanceSlowThresholdLabel = computed(() => {
  const value = providerPerformanceSlowThresholdValue.value
  if (value >= 1000) {
    const seconds = value / 1000
    return `${Number.isInteger(seconds) ? seconds.toFixed(0) : seconds.toFixed(1)}s`
  }
  return `${value}ms`
})

const providerPerformanceActiveFilterCount = computed(() => {
  const textFilterCount = [
    providerPerformanceProviderId.value,
    providerPerformanceModel.value,
    providerPerformanceApiFormat.value,
    providerPerformanceEndpointKind.value,
  ].filter(value => normalizeProviderPerformanceFilter(value)).length

  const booleanFilterCount = [
    providerPerformanceIsStream.value,
    providerPerformanceHasFormatConversion.value,
  ].filter(value => value !== 'all').length

  const thresholdFilterCount = providerPerformanceSlowThresholdValue.value === DEFAULT_PROVIDER_PERFORMANCE_SLOW_THRESHOLD_MS
    ? 0
    : 1

  return textFilterCount + booleanFilterCount + thresholdFilterCount
})

const hasProviderPerformanceFilters = computed(() => providerPerformanceActiveFilterCount.value > 0)

function buildProviderPerformanceParams(): ProviderPerformanceParams {
  const params: ProviderPerformanceParams = {
    ...buildTimeRangeParams(),
    granularity: resolveProviderPerformanceGranularity(),
    limit: 8,
    slow_threshold_ms: providerPerformanceSlowThresholdValue.value,
  }

  const providerId = normalizeProviderPerformanceFilter(providerPerformanceProviderId.value)
  if (providerId) params.provider_id = providerId

  const model = normalizeProviderPerformanceFilter(providerPerformanceModel.value)
  if (model) params.model = model

  const apiFormat = normalizeProviderPerformanceFilter(providerPerformanceApiFormat.value)
  if (apiFormat) params.api_format = apiFormat

  const endpointKind = normalizeProviderPerformanceFilter(providerPerformanceEndpointKind.value)
  if (endpointKind) params.endpoint_kind = endpointKind

  const isStream = parseProviderPerformanceBooleanFilter(providerPerformanceIsStream.value)
  if (isStream !== undefined) params.is_stream = isStream

  const hasFormatConversion = parseProviderPerformanceBooleanFilter(
    providerPerformanceHasFormatConversion.value
  )
  if (hasFormatConversion !== undefined) {
    params.has_format_conversion = hasFormatConversion
  }

  return params
}

function resetProviderPerformanceFilters() {
  providerPerformanceProviderId.value = ''
  providerPerformanceModel.value = ''
  providerPerformanceApiFormat.value = ''
  providerPerformanceEndpointKind.value = ''
  providerPerformanceIsStream.value = 'all'
  providerPerformanceHasFormatConversion.value = 'all'
  providerPerformanceSlowThresholdMs.value = String(DEFAULT_PROVIDER_PERFORMANCE_SLOW_THRESHOLD_MS)
}

function formatMetricNumber(value: number | null | undefined): string {
  if (value == null || Number.isNaN(value)) {
    return '-'
  }

  if (!Number.isInteger(value)) {
    return value.toFixed(2)
  }

  return formatNumber(value)
}

function formatErrorRate(successRate: number | null | undefined): string {
  if (successRate == null || Number.isNaN(successRate)) {
    return '-'
  }
  return `${Math.max(0, 100 - successRate).toFixed(2)}%`
}

function providerSampleCoverageText(provider: ProviderPerformanceItem): string {
  if (!provider.request_count) {
    return '-'
  }
  const responseSamples = provider.response_time_sample_count ?? 0
  const firstByteSamples = provider.first_byte_sample_count ?? 0
  const responseCoverage = Math.round(responseSamples / provider.request_count * 100)
  const firstByteCoverage = Math.round(firstByteSamples / provider.request_count * 100)
  return `${responseCoverage}% / ${firstByteCoverage}%`
}

async function loadPercentiles() {
  const requestId = ++percentilesRequestId
  percentileLoading.value = true
  try {
    const data = await adminApi.getPercentiles(buildTimeRangeParams())
    if (requestId !== percentilesRequestId) return
    percentiles.value = data
  } catch (error) {
    if (requestId !== percentilesRequestId) return
    percentiles.value = []
    log.error('加载延迟百分位失败', error)
  } finally {
    if (requestId === percentilesRequestId) {
      percentileLoading.value = false
    }
  }
}

async function loadErrors() {
  const requestId = ++errorsRequestId
  errorLoading.value = true
  try {
    const response = await adminApi.getErrorDistribution(buildTimeRangeParams())
    if (requestId !== errorsRequestId) return
    errorDistribution.value = response.distribution
    errorTrend.value = response.trend
  } catch (error) {
    if (requestId !== errorsRequestId) return
    errorDistribution.value = []
    errorTrend.value = []
    log.error('加载错误分布失败', error)
  } finally {
    if (requestId === errorsRequestId) {
      errorLoading.value = false
    }
  }
}

async function loadProviderPerformance() {
  const requestId = ++providerPerformanceRequestId
  providerPerformanceLoading.value = true
  try {
    const data = await adminApi.getProviderPerformance(buildProviderPerformanceParams())
    if (requestId !== providerPerformanceRequestId) return
    providerPerformance.value = data
  } catch (error) {
    if (requestId !== providerPerformanceRequestId) return
    providerPerformance.value = null
    log.error('加载上游服务性能统计失败', error)
  } finally {
    if (requestId === providerPerformanceRequestId) {
      providerPerformanceLoading.value = false
    }
  }
}

async function loadLiveData(options: { silent?: boolean } = {}) {
  const requestId = ++liveRequestId
  const initialLoad = !liveReady.value

  if (initialLoad) {
    liveLoading.value = true
  } else {
    liveRefreshing.value = true
  }

  const results = await Promise.allSettled([
    monitoringApi.getSystemStatus(),
    monitoringApi.getResilienceStatus(),
    monitoringApi.getCircuitHistory(8),
    monitoringApi.getGatewayMetricsSummary(),
  ])

  if (requestId !== liveRequestId) {
    return
  }

  const failedScopes: string[] = []
  let successCount = 0

  const [systemResult, resilienceResult, circuitResult, metricsResult] = results

  if (systemResult.status === 'fulfilled') {
    systemStatus.value = systemResult.value
    successCount += 1
  } else {
    failedScopes.push(t('performanceExtra.systemStatus'))
    log.error('加载系统状态失败', systemResult.reason)
  }

  if (resilienceResult.status === 'fulfilled') {
    resilienceStatus.value = resilienceResult.value
    successCount += 1
  } else {
    failedScopes.push(t('performanceExtra.resilienceStatus'))
    log.error('加载韧性状态失败', resilienceResult.reason)
  }

  if (circuitResult.status === 'fulfilled') {
    circuitHistory.value = circuitResult.value.items
    successCount += 1
  } else {
    failedScopes.push(t('performanceExtra.circuitHistory'))
    log.error('加载熔断历史失败', circuitResult.reason)
  }

  if (metricsResult.status === 'fulfilled') {
    gatewayMetrics.value = metricsResult.value
    successCount += 1
  } else {
    failedScopes.push(t('performanceExtra.gatewayMetrics'))
    log.error('加载网关指标失败', metricsResult.reason)
  }

  liveReady.value = successCount > 0
  if (successCount > 0) {
    liveLastUpdatedAt.value = new Date().toISOString()
  }
  liveLoadError.value = failedScopes.length
    ? t('performanceExtra.partialLoadFailed', { scopes: failedScopes.join(t('performanceExtra.listSeparator')) })
    : null

  if (failedScopes.length && !options.silent) {
    showError(liveLoadError.value ?? t('performanceExtra.liveLoadFailed'))
  }

  if (requestId === liveRequestId) {
    liveLoading.value = false
    liveRefreshing.value = false
  }
}

const errorTrendChartData = computed(() => ({
  labels: errorTrend.value.map(item => item.date),
  datasets: [
    {
      label: t('performanceExtra.errorCount'),
      data: errorTrend.value.map(item => item.total),
      borderColor: 'rgb(239, 68, 68)',
      tension: 0.25,
      pointRadius: 2
    }
  ]
}))

const recentErrors = computed(() => resilienceStatus.value?.recent_errors ?? [])
const hasMoreRecentErrors = computed(() => recentErrors.value.length > RECENT_ERRORS_COLLAPSED_LIMIT)
const visibleRecentErrors = computed(() => (
  recentErrorsExpanded.value
    ? recentErrors.value
    : recentErrors.value.slice(0, RECENT_ERRORS_COLLAPSED_LIMIT)
))
const recentErrorsListClass = computed(() => [
  'space-y-3',
  recentErrorsExpanded.value ? 'max-h-[360px] overflow-y-auto pr-1' : '',
])
const resilienceRecommendations = computed(() => resilienceStatus.value?.recommendations ?? [])

const healthStatusVariant = computed<'success' | 'warning' | 'destructive' | 'outline'>(() => {
  switch (resilienceStatus.value?.status) {
    case 'healthy':
      return 'success'
    case 'degraded':
      return 'warning'
    case 'critical':
      return 'destructive'
    default:
      return 'outline'
  }
})

const healthStatusText = computed(() => {
  if (!resilienceStatus.value) {
    return t('performanceExtra.healthUnknown')
  }

  const statusMap: Record<string, string> = {
    healthy: t('performanceExtra.systemHealthy'),
    degraded: t('performanceExtra.systemDegraded'),
    critical: t('performanceExtra.systemCritical'),
  }

  return `${statusMap[resilienceStatus.value.status] ?? resilienceStatus.value.status} · ${resilienceStatus.value.health_score}/100`
})

const metricsAvailabilityText = computed(() => (
  gatewayMetrics.value ? t('performanceExtra.gatewayOnline') : t('performanceExtra.gatewayUnavailable')
))

const distributedGateVariant = computed<'warning' | 'outline'>(() => (
  gatewayMetrics.value?.distributed.unavailable ? 'warning' : 'outline'
))

const distributedGateText = computed(() => (
  gatewayMetrics.value?.distributed.unavailable ? t('performanceExtra.unavailable') : t('performanceExtra.online')
))

const liveLastUpdatedLabel = computed(() => (
  liveLastUpdatedAt.value ? formatDate(liveLastUpdatedAt.value) : t('performanceExtra.notRefreshed')
))

const currentActiveStreams = computed(() => (
  gatewayMetrics.value?.tunnel.activeStreams ?? systemStatus.value?.tunnel.active_streams ?? null
))

const currentProxyConnections = computed(() => (
  gatewayMetrics.value?.tunnel.proxyConnections ?? systemStatus.value?.tunnel.proxy_connections ?? null
))

const currentTunnelNodes = computed(() => (
  gatewayMetrics.value?.tunnel.nodes ?? systemStatus.value?.tunnel.nodes ?? null
))

const tunnelQueueRejectedTotal = computed(() => {
  const full = gatewayMetrics.value?.tunnel.outboundQueueRejectedFullTotal ?? 0
  const closed = gatewayMetrics.value?.tunnel.outboundQueueRejectedClosedTotal ?? 0
  return full + closed
})

const tunnelSelectionPressureTotal = computed(() => {
  const congested = gatewayMetrics.value?.tunnel.proxyConnectionCongestedTotal ?? 0
  const retry = gatewayMetrics.value?.tunnel.selectionRetryTotal ?? 0
  const unavailable = gatewayMetrics.value?.tunnel.selectionUnavailableTotal ?? 0
  return congested + retry + unavailable
})

const tunnelQueueUtilizationText = computed(() => {
  const depth = gatewayMetrics.value?.tunnel.outboundQueueDepthTotal
  const capacity = gatewayMetrics.value?.tunnel.outboundQueueCapacityTotal
  if (depth == null || capacity == null || capacity <= 0) {
    return '-'
  }
  return `${Math.round(depth / capacity * 100)}%`
})

const fallbackRows = computed(() => {
  const items = gatewayMetrics.value?.fallbacks ?? []
  const maxValue = Math.max(...items.map(item => item.total), 0)

  return items
    .filter(item => item.total > 0)
    .sort((left, right) => right.total - left.total)
    .map(item => ({
      ...item,
      ratio: maxValue > 0 ? item.total / maxValue * 100 : 0,
    }))
})

const providerPerformanceRows = computed(() => providerPerformance.value?.providers ?? [])

const providerPerformanceSubtitle = computed(() => {
  const requests = providerPerformance.value?.summary.request_count ?? 0
  const filters = providerPerformanceActiveFilterCount.value
  const filterText = filters > 0 ? t('performanceExtra.filterCountSuffix', { count: filters }) : ''
  return t('performanceExtra.sampleSummary', { count: formatMetricNumber(requests), filters: filterText })
})

const providerPerformanceSummaryCards = computed(() => {
  const summary = providerPerformance.value?.summary
  return [
    {
      title: t('performanceExtra.requestSamples'),
      value: formatMetricNumber(summary?.request_count),
      hint: t('performanceExtra.upstreamServiceCount', { count: formatMetricNumber(providerPerformanceRows.value.length) }),
      icon: Activity,
      iconClass: 'text-blue-500',
    },
    {
      title: t('performanceExtra.successRate'),
      value: formatProviderPerformanceMetric(summary?.success_rate, '%'),
      hint: t('performanceExtra.errorRateValue', { value: formatErrorRate(summary?.success_rate) }),
      icon: CheckCircle2,
      iconClass: 'text-emerald-500',
    },
    {
      title: t('performanceExtra.p99Response'),
      value: formatProviderPerformanceMetric(summary?.p99_response_time_ms, 'ms', 0),
      hint: `P90 ${formatProviderPerformanceMetric(summary?.p90_response_time_ms, 'ms', 0)}`,
      icon: Gauge,
      iconClass: 'text-violet-500',
    },
    {
      title: t('performanceExtra.p99FirstByte'),
      value: formatProviderPerformanceMetric(summary?.p99_first_byte_time_ms, 'ms', 0),
      hint: `P90 ${formatProviderPerformanceMetric(summary?.p90_first_byte_time_ms, 'ms', 0)}`,
      icon: Timer,
      iconClass: 'text-sky-500',
    },
    {
      title: t('performanceExtra.outputTps'),
      value: formatProviderPerformanceMetric(summary?.avg_output_tps, ' tps'),
      hint: t('performanceExtra.tpsSamples', { count: formatMetricNumber(summary?.tps_sample_count) }),
      icon: Zap,
      iconClass: 'text-amber-500',
    },
    {
      title: t('performanceExtra.avgFirstByte'),
      value: formatProviderPerformanceMetric(summary?.avg_first_byte_time_ms, 'ms'),
      hint: t('performanceExtra.firstByteSamples', { count: formatMetricNumber(summary?.first_byte_sample_count) }),
      icon: Timer,
      iconClass: 'text-sky-500',
    },
    {
      title: t('performanceExtra.avgResponse'),
      value: formatProviderPerformanceMetric(summary?.avg_response_time_ms, 'ms'),
      hint: t('performanceExtra.responseSamples', { count: formatMetricNumber(summary?.response_time_sample_count) }),
      icon: Gauge,
      iconClass: 'text-violet-500',
    },
    {
      title: t('performanceExtra.slowRequests'),
      value: formatMetricNumber(summary?.slow_request_count),
      hint: t('performanceExtra.slowThresholdHint', { value: providerPerformanceSlowThresholdLabel.value }),
      icon: AlertTriangle,
      iconClass: 'text-yellow-500',
    },
  ]
})

const providerTpsChartData = computed(() => (
  buildProviderPerformanceChartData(
    providerPerformance.value?.timeline ?? [],
    'avg_output_tps',
    providerPerformanceRows.value,
  )
))

const providerFirstByteChartData = computed(() => (
  buildProviderPerformanceChartData(
    providerPerformance.value?.timeline ?? [],
    'avg_first_byte_time_ms',
    providerPerformanceRows.value,
  )
))

const providerTpsChartOptions = computed(() => ({
  scales: {
    y: {
      ticks: {
        callback: (value: string | number) => `${value} tps`,
      },
    },
  },
}))

const providerLatencyChartOptions = computed(() => ({
  scales: {
    y: {
      ticks: {
        callback: (value: string | number) => formatDurationMs(Number(value), 0),
      },
    },
  },
}))

const liveSummaryCards = computed(() => [
  {
    title: t('performanceExtra.systemHealthy'),
    value: resilienceStatus.value ? `${resilienceStatus.value.health_score}/100` : '-',
    hint: t('performanceExtra.healthCircuitHint', { status: healthStatusText.value, count: formatMetricNumber(resilienceStatus.value?.error_statistics.open_circuit_breakers) }),
    icon: ShieldCheck,
    iconClass: 'text-emerald-500',
  },
  {
    title: t('performanceExtra.lastHourErrors'),
    value: formatMetricNumber(systemStatus.value?.recent_errors),
    hint: t('performanceExtra.totalErrors24h', { count: formatMetricNumber(resilienceStatus.value?.error_statistics.total_errors) }),
    icon: AlertTriangle,
    iconClass: 'text-yellow-500',
  },
  {
    title: t('performanceExtra.proxyActiveFlows'),
    value: formatMetricNumber(currentActiveStreams.value),
    hint: t('performanceExtra.proxyConnections', { count: formatMetricNumber(currentProxyConnections.value) }),
    icon: Cable,
    iconClass: 'text-sky-500',
  },
  {
    title: t('performanceExtra.localProcessing'),
    value: formatMetricNumber(gatewayMetrics.value?.local.inFlight),
    hint: t('performanceExtra.availablePermits', { count: formatMetricNumber(gatewayMetrics.value?.local.availablePermits) }),
    icon: Activity,
    iconClass: 'text-blue-500',
  },
  {
    title: t('performanceExtra.globalProcessing'),
    value: gatewayMetrics.value?.distributed.unavailable
      ? t('performanceExtra.unavailable')
      : formatMetricNumber(gatewayMetrics.value?.distributed.inFlight),
    hint: gatewayMetrics.value?.distributed.unavailable
      ? t('performanceExtra.checkRedis')
      : t('performanceExtra.availablePermits', { count: formatMetricNumber(gatewayMetrics.value?.distributed.availablePermits) }),
    icon: Workflow,
    iconClass: 'text-violet-500',
  },
  {
    title: t('performanceExtra.fallbackSwitches'),
    value: formatMetricNumber(gatewayMetrics.value?.fallbackTotal),
    hint: t('performanceExtra.currentProcessTotal'),
    icon: GitBranch,
    iconClass: 'text-rose-500',
  },
])

const isRefreshing = computed(() => (
  liveLoading.value ||
  liveRefreshing.value ||
  percentileLoading.value ||
  errorLoading.value ||
  providerPerformanceLoading.value
))

async function loadAll() {
  if (loadAllPromise) {
    hasPendingLoadAll = true
    return loadAllPromise
  }

  loadAllPromise = Promise.all([
    loadPercentiles(),
    loadErrors(),
    loadProviderPerformance(),
  ])
    .then(() => undefined)
    .finally(() => {
      loadAllPromise = null
      if (hasPendingLoadAll) {
        hasPendingLoadAll = false
        void loadAll()
      }
    })

  return loadAllPromise
}

async function handleManualRefresh() {
  await Promise.allSettled([loadLiveData(), loadAll()])
}

function scheduleLoadAll() {
  if (loadAllDebounceTimer) {
    clearTimeout(loadAllDebounceTimer)
  }

  loadAllDebounceTimer = setTimeout(() => {
    loadAllDebounceTimer = null
    void loadAll()
  }, 120)
}

function scheduleProviderPerformanceLoad() {
  if (providerPerformanceDebounceTimer) {
    clearTimeout(providerPerformanceDebounceTimer)
  }

  providerPerformanceDebounceTimer = setTimeout(() => {
    providerPerformanceDebounceTimer = null
    void loadProviderPerformance()
  }, 180)
}

watch(timeRange, scheduleLoadAll, { deep: true })
watch(
  [
    providerPerformanceProviderId,
    providerPerformanceModel,
    providerPerformanceApiFormat,
    providerPerformanceEndpointKind,
    providerPerformanceIsStream,
    providerPerformanceHasFormatConversion,
    providerPerformanceSlowThresholdMs,
  ],
  scheduleProviderPerformanceLoad
)

onMounted(() => {
  void loadLiveData()
  void loadAll()
  liveRefreshTimer = setInterval(() => {
    void loadLiveData({ silent: true })
  }, LIVE_REFRESH_INTERVAL_MS)
})

onUnmounted(() => {
  if (loadAllDebounceTimer) {
    clearTimeout(loadAllDebounceTimer)
    loadAllDebounceTimer = null
  }

  if (providerPerformanceDebounceTimer) {
    clearTimeout(providerPerformanceDebounceTimer)
    providerPerformanceDebounceTimer = null
  }

  if (liveRefreshTimer) {
    clearInterval(liveRefreshTimer)
    liveRefreshTimer = null
  }

  hasPendingLoadAll = false
  loadAllPromise = null
  percentilesRequestId += 1
  errorsRequestId += 1
  providerPerformanceRequestId += 1
  liveRequestId += 1
})
</script>
