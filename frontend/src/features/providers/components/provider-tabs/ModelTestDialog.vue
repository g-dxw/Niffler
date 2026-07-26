<template>
  <Dialog
    :open="open"
    size="3xl"
    :close-on-backdrop="false"
    @update:open="(val: boolean) => { if (!val) emit('close') }"
  >
    <template #header>
      <div class="border-b border-border px-6 py-4">
        <div class="space-y-3">
          <div class="text-lg font-semibold text-foreground leading-tight">
            {{ dialogTitle }}
          </div>
          <p
            v-if="dialogDescription && !showResult"
            class="text-xs text-muted-foreground"
          >
            {{ dialogDescription }}
          </p>
        </div>
      </div>
    </template>

    <div
      v-if="showSetup"
      class="space-y-4"
    >
      <div
        v-if="endpoints.length > 0"
        class="space-y-2"
      >
        <div class="grid min-h-8 w-full items-center gap-2 sm:h-8 sm:grid-cols-2">
          <div class="text-sm font-medium text-foreground">
            {{ t('modelTest.selectEndpoint') }}
          </div>
          <div
            v-if="modelMappingAvailable"
            class="grid w-full grid-cols-[auto_1fr] items-center gap-2"
          >
            <div class="text-sm font-medium text-foreground">
              {{ t('modelTest.modelMapping') }}
            </div>
            <Select
              :model-value="selectedModelMappingValue"
              @update:model-value="handleModelMappingValueChange"
            >
              <SelectTrigger class="h-8 min-h-8 w-full shrink-0 overflow-hidden border-border/60 py-0 text-xs leading-none [transition-property:none] [&>span:first-child]:min-w-0 [&>span:first-child]:flex-1 [&>span:first-child]:text-left [&>span:first-child]:leading-none">
                <SelectValue :placeholder="requestedModelName || t('modelTest.currentModel')" />
              </SelectTrigger>
              <SelectContent
                class="w-[var(--radix-select-trigger-width)]"
                align="end"
                :disable-portal="false"
                :searchable="false"
              >
                <SelectItem
                  :value="requestedModelName"
                  :text-value="requestedModelName"
                >
                  {{ requestedModelName }}
                </SelectItem>
                <SelectItem
                  v-for="option in modelMappingOptions"
                  :key="option.name"
                  :value="option.name"
                  :text-value="option.name"
                >
                  {{ option.name }}
                </SelectItem>
              </SelectContent>
            </Select>
          </div>
        </div>
        <div class="grid gap-2 md:grid-cols-2">
          <button
            v-for="endpoint in endpoints"
            :key="endpoint.id"
            type="button"
            class="h-full w-full rounded-lg border px-3 py-3 text-left transition-colors"
            :class="selectedEndpoint?.id === endpoint.id
              ? 'border-primary bg-primary/5'
              : 'border-border/60 hover:bg-muted/40'"
            @click="emit('selectEndpoint', endpoint.id)"
          >
            <div class="flex items-center justify-between gap-3">
              <div class="min-w-0">
                <div class="text-sm font-medium">
                  {{ formatApiFormat(endpoint.api_format) }}
                </div>
                <div class="mt-1 text-xs text-muted-foreground break-all">
                  {{ endpoint.base_url }}
                </div>
              </div>
              <Badge :variant="selectedEndpoint?.id === endpoint.id ? 'success' : 'outline'">
                {{ selectedEndpoint?.id === endpoint.id ? t('modelTest.selected') : (endpoint.is_active ? t('modelTest.enabled') : t('modelTest.disabled')) }}
              </Badge>
            </div>
          </button>
        </div>
      </div>

      <div class="grid gap-4 lg:grid-cols-2 lg:items-start">
        <div class="space-y-2">
          <div class="flex items-center justify-between gap-3">
            <div class="text-sm font-medium">
              {{ t('modelTest.requestHeaders') }}
            </div>
            <div class="flex items-center gap-1">
              <Button
                variant="ghost"
                size="icon"
                class="h-8 w-8 rounded-lg text-muted-foreground"
                :title="t('modelTest.formatHeaders')"
                @click="formatRequestHeadersDraft"
              >
                <Code2 class="h-4 w-4" />
              </Button>
              <Button
                variant="ghost"
                size="icon"
                class="h-8 w-8 rounded-lg text-muted-foreground"
                :title="t('modelTest.resetHeaders')"
                @click="resetRequestHeadersDraft"
              >
                <RotateCcw class="h-4 w-4" />
              </Button>
            </div>
          </div>
          <Textarea
            :model-value="requestHeadersDraft"
            class="min-h-[220px] font-mono text-xs"
            :placeholder="t('modelTest.headersPlaceholder')"
            @update:model-value="emit('update:requestHeadersDraft', $event)"
          />
          <div
            v-if="requestHeadersError"
            class="rounded-md border border-destructive/30 bg-destructive/10 px-3 py-2 text-xs text-destructive"
          >
            {{ requestHeadersError }}
          </div>
          <div class="rounded-md border border-border/60 bg-muted/20 px-3 py-2 text-[11px] text-muted-foreground">
              {{ t('modelTest.headersHint') }}
          </div>
        </div>

        <div class="space-y-2">
          <div class="flex items-center justify-between gap-3">
            <div class="text-sm font-medium">
              {{ t('modelTest.requestBody') }}
            </div>
            <div class="flex items-center gap-1">
              <Button
                variant="ghost"
                size="icon"
                class="h-8 w-8 rounded-lg text-muted-foreground"
                :title="t('modelTest.formatBody')"
                @click="formatRequestBodyDraft"
              >
                <Code2 class="h-4 w-4" />
              </Button>
              <Button
                variant="ghost"
                size="icon"
                class="h-8 w-8 rounded-lg text-muted-foreground"
                :title="t('modelTest.resetBody')"
                @click="resetRequestBodyDraft"
              >
                <RotateCcw class="h-4 w-4" />
              </Button>
            </div>
          </div>
          <Textarea
            :model-value="requestBodyDraft"
            class="min-h-[220px] font-mono text-xs"
            :placeholder="t('modelTest.bodyPlaceholder')"
            @update:model-value="emit('update:requestBodyDraft', $event)"
          />
          <div
            v-if="requestBodyError"
            class="rounded-md border border-destructive/30 bg-destructive/10 px-3 py-2 text-xs text-destructive"
          >
            {{ requestBodyError }}
          </div>
          <div class="rounded-md border border-border/60 bg-muted/20 px-3 py-2 text-[11px] text-muted-foreground">
            {{ t('modelTest.bodyHint') }}
          </div>
        </div>
      </div>

      <Button
        class="w-full"
        :disabled="startDisabled"
        @click="emit('start')"
      >
        {{ t('modelTest.start') }}
      </Button>
    </div>

    <div
      v-else-if="testing"
      class="space-y-4 py-6"
    >
      <div class="flex flex-col items-center justify-center gap-3 text-center">
        <Loader2 class="h-8 w-8 animate-spin text-primary" />
        <div class="space-y-1">
          <p class="text-sm font-medium">
            {{ t('modelTest.testing') }}
          </p>
          <p class="text-xs text-muted-foreground">
            {{ selectingModelName || '-' }}
          </p>
          <p
            v-if="selectedEndpoint"
            class="text-xs text-muted-foreground"
          >
            {{ t('modelTest.endpointSummary', { format: formatApiFormat(selectedEndpoint.api_format), url: selectedEndpoint.base_url }) }}
          </p>
        </div>
      </div>

      <div class="rounded-lg border border-border/60 bg-muted/20 p-4 space-y-4">
        <div class="space-y-2">
          <div class="flex items-center justify-between gap-3 text-xs text-muted-foreground">
            <span>{{ t('modelTest.liveProgress') }}</span>
            <span>{{ liveProgressText }}</span>
          </div>
          <div class="h-2 overflow-hidden rounded-full bg-muted">
            <div
              class="h-full bg-primary transition-all duration-300"
              :style="{ width: `${liveProgressPercent}%` }"
            />
          </div>
          <div
            v-if="liveTraceSummary.total_candidates > 0"
            class="flex flex-wrap gap-1.5"
          >
            <Badge
              v-for="item in liveSummaryItems"
              :key="item.key"
              :variant="item.variant"
              class="px-1.5 py-0 text-[10px]"
            >
              {{ item.label }} {{ item.value }}
            </Badge>
          </div>
          <div
            v-else
            class="text-xs text-muted-foreground"
          >
              {{ t('modelTest.preparing') }}
          </div>
        </div>

        <div class="grid gap-3 sm:grid-cols-2">
          <div class="rounded-md border border-border/60 bg-background/80 p-3 space-y-1">
            <div class="text-xs text-muted-foreground">
              {{ liveEntityLabel }}
            </div>
            <div class="break-all text-sm font-medium">
              {{ liveAccountTitle }}
            </div>
            <div
              v-if="liveAccountMeta"
              class="break-all text-xs text-muted-foreground"
            >
              {{ liveAccountMeta }}
            </div>
          </div>
          <div class="rounded-md border border-border/60 bg-background/80 p-3 space-y-1">
            <div class="text-xs text-muted-foreground">
              {{ t('modelTest.status') }}
            </div>
            <div class="text-sm font-medium">
              {{ liveStatusTitle }}
            </div>
            <div
              v-if="liveStatusDetail"
              class="break-all text-xs text-muted-foreground"
            >
              {{ liveStatusDetail }}
            </div>
          </div>
        </div>

        <div
          v-if="requestId"
          class="break-all text-[11px] text-muted-foreground"
        >
          {{ t('modelTest.requestId') }}：<code class="rounded bg-muted px-1 py-0.5">{{ requestId }}</code>
        </div>

        <div
          v-if="liveRecentCandidates.length > 0"
          class="space-y-2"
        >
          <div class="text-xs font-medium text-muted-foreground">
            {{ t('modelTest.latest') }}
          </div>
          <div class="space-y-2">
            <div
              v-for="candidate in liveRecentCandidates"
              :key="`${candidate.id}-${candidate.status}`"
              class="flex items-start justify-between gap-3 rounded-md border border-border/50 bg-background/70 px-3 py-2 text-xs"
            >
              <div class="min-w-0 space-y-1">
                <div class="flex min-w-0 items-center gap-2">
                  <span class="shrink-0 text-muted-foreground">{{ formatTraceCandidateIndex(candidate) }}</span>
                  <Badge
                    :variant="statusVariant(candidate.status)"
                    class="shrink-0 px-1.5 py-0 text-[10px]"
                  >
                    {{ statusDisplay(candidate) }}
                  </Badge>
                  <span class="truncate font-medium">{{ formatTraceCandidateAccount(candidate) }}</span>
                </div>
                <div
                  v-if="traceCandidateDetail(candidate)"
                  class="break-all text-muted-foreground"
                >
                  {{ traceCandidateDetail(candidate) }}
                </div>
              </div>
              <div class="shrink-0 tabular-nums text-muted-foreground">
                {{ candidate.latency_ms != null ? `${candidate.latency_ms}ms` : '' }}
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>

    <div
      v-else-if="result"
      class="space-y-4"
    >
      <div class="rounded-lg border border-border/60 bg-muted/20 p-4 space-y-4">
        <div class="flex flex-wrap items-center justify-between gap-3">
          <div class="flex items-center gap-2">
            <Badge
              :variant="result.success ? 'success' : 'destructive'"
              class="px-2 py-0.5"
            >
              {{ result.success ? t('modelTest.success') : t('modelTest.failed') }}
            </Badge>
          </div>
          <div
            v-if="hasEffectiveModel"
            class="text-xs text-muted-foreground"
          >
            {{ t('modelTest.requestModel', { model: resultModelTitle }) }}
          </div>
        </div>

        <div class="grid gap-3 lg:grid-cols-3">
          <div class="rounded-md border border-border/60 bg-background/80 p-3 space-y-1">
            <div class="text-xs text-muted-foreground">
              {{ resultEntityLabel }}
            </div>
            <div class="break-all text-sm font-medium">
              {{ resultWinningTitle }}
            </div>
          </div>
          <div class="rounded-md border border-border/60 bg-background/80 p-3 space-y-1">
            <div class="text-xs text-muted-foreground">
              {{ t('modelTest.routingResult') }}
            </div>
            <div class="text-sm font-medium">
              {{ resultDispatchTitle }}
            </div>
          </div>
          <div class="rounded-md border border-border/60 bg-background/80 p-3 space-y-1">
            <div class="text-xs text-muted-foreground">
              {{ t('modelTest.actualRequest') }}
            </div>
            <div class="break-all text-sm font-medium">
              {{ resultModelTitle }}
            </div>
          </div>
        </div>

        <div
          v-if="showCandidateDiagnostics"
          class="flex flex-wrap gap-1.5"
        >
          <Badge
            v-for="item in resultSummaryItems"
            :key="item.key"
            :variant="item.variant"
            class="px-1.5 py-0 text-[10px]"
          >
            {{ item.label }} {{ item.value }}
          </Badge>
        </div>
      </div>

      <div
        v-if="shouldCollapseAttempts"
        class="flex items-center justify-between gap-3 text-xs text-muted-foreground"
      >
        <span>{{ t('modelTest.visibleAttempts', { visible: visibleAttempts.length, total: resultAttempts.length }) }}</span>
        <Button
          variant="ghost"
          size="sm"
          @click="showAllAttempts = !showAllAttempts"
        >
          {{ showAllAttempts ? t('modelTest.collapseDetails') : t('modelTest.expandAll', { count: resultAttempts.length }) }}
        </Button>
      </div>

      <div
        v-if="resultAttempts.length > 0"
        class="max-h-[360px] space-y-2 overflow-y-auto pr-1 sm:hidden"
      >
        <div
          v-for="(attempt, idx) in visibleAttempts"
          :key="'m' + idx"
          class="rounded-md border px-3 py-2 text-xs"
          :class="attemptRowClass(attempt.status, selectedInspectionKey === inspectionKey(attempt))"
          role="button"
          tabindex="0"
          @click="selectInspectionAttempt(attempt)"
          @keydown.enter.prevent="selectInspectionAttempt(attempt)"
          @keydown.space.prevent="selectInspectionAttempt(attempt)"
        >
          <div class="flex items-center justify-between gap-2">
            <div class="flex min-w-0 items-center gap-1.5">
              <span class="shrink-0 text-muted-foreground">{{ formatAttemptIndex(attempt) }}</span>
              <Badge
                :variant="statusVariant(attempt.status)"
                class="shrink-0 px-1.5 py-0 text-[10px]"
              >
                {{ statusDisplay(attempt) }}
              </Badge>
              <span
                v-if="attempt.latency_ms != null"
                class="shrink-0 tabular-nums text-muted-foreground"
              >
                {{ attempt.latency_ms }}ms
              </span>
            </div>
          </div>
          <div class="mt-1.5 space-y-0.5">
            <div class="truncate font-medium">
              {{ formatAttemptAccountDisplay(attempt) }}
            </div>
            <div
              v-if="attemptImagePreviews(attempt).length > 0"
              class="mt-2 flex flex-wrap gap-2"
            >
              <button
                v-for="(preview, imageIndex) in attemptImagePreviews(attempt).slice(0, 3)"
                :key="`${preview.src}-${imageIndex}`"
                type="button"
                class="h-16 w-16 overflow-hidden rounded-md border border-border/60 bg-muted/30 transition-colors hover:border-primary/60"
                :title="preview.label"
                @click.stop="openImagePreview(preview)"
              >
                <img
                  :src="preview.src"
                  :alt="preview.label"
                  class="h-full w-full object-contain"
                  loading="lazy"
                >
              </button>
            </div>
            <div
              v-else-if="attemptDetail(attempt) !== '-'"
              class="mt-1 break-all text-muted-foreground"
            >
              {{ attemptDetail(attempt) }}
            </div>
          </div>
        </div>
      </div>

      <div
        v-if="resultAttempts.length > 0"
        class="hidden max-h-[360px] overflow-y-auto rounded-md border sm:block"
      >
        <table class="w-full table-fixed text-xs">
          <colgroup>
            <col class="w-8">
            <col class="w-[28%]">
            <col class="w-16">
            <col class="w-16">
            <col>
          </colgroup>
          <thead class="sticky top-0 z-10">
            <tr class="border-b bg-muted/30">
              <th class="py-2 pl-3 pr-1 text-left font-medium">
                #
              </th>
              <th class="px-3 py-2 text-left font-medium">
                Key
              </th>
              <th class="px-3 py-2 text-left font-medium">
                {{ t('modelTest.status') }}
              </th>
              <th class="px-3 py-2 text-right font-medium">
                {{ t('modelTest.latency') }}
              </th>
              <th class="px-3 py-2 text-left font-medium">
                {{ t('modelTest.details') }}
              </th>
            </tr>
          </thead>
          <tbody>
            <tr
              v-for="(attempt, idx) in visibleAttempts"
              :key="idx"
              class="last:border-b-0 align-top border-b"
              :class="attemptRowClass(attempt.status, selectedInspectionKey === inspectionKey(attempt))"
              tabindex="0"
              @click="selectInspectionAttempt(attempt)"
              @keydown.enter.prevent="selectInspectionAttempt(attempt)"
              @keydown.space.prevent="selectInspectionAttempt(attempt)"
            >
              <td class="py-2 pl-3 pr-1 text-muted-foreground">
                {{ formatAttemptIndex(attempt) }}
              </td>
              <td class="px-3 py-2">
                <div
                  class="truncate font-medium"
                  :title="formatAttemptAccountTitle(attempt)"
                >
                  {{ formatAttemptAccountDisplay(attempt) }}
                </div>
              </td>
              <td class="px-3 py-2">
                <Badge
                  :variant="statusVariant(attempt.status)"
                  class="px-1.5 py-0 text-[10px]"
                >
                  {{ statusDisplay(attempt) }}
                </Badge>
              </td>
              <td class="px-3 py-2 text-right tabular-nums text-muted-foreground">
                {{ attempt.latency_ms != null ? attempt.latency_ms + 'ms' : '-' }}
              </td>
              <td class="px-3 py-2 text-muted-foreground">
                <div
                  v-if="attemptImagePreviews(attempt).length > 0"
                  class="flex flex-wrap gap-2"
                >
                  <button
                    v-for="(preview, imageIndex) in attemptImagePreviews(attempt).slice(0, 4)"
                    :key="`${preview.src}-${imageIndex}`"
                    type="button"
                    class="h-14 w-14 overflow-hidden rounded-md border border-border/60 bg-muted/30 transition-colors hover:border-primary/60 focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-primary/70"
                    :title="preview.label"
                    @click.stop="openImagePreview(preview)"
                  >
                    <img
                      :src="preview.src"
                      :alt="preview.label"
                      class="h-full w-full object-contain"
                      loading="lazy"
                    >
                  </button>
                  <span
                    v-if="attemptImagePreviews(attempt).length > 4"
                    class="flex h-14 items-center rounded-md border border-border/60 px-2 text-xs text-muted-foreground"
                  >
                    +{{ attemptImagePreviews(attempt).length - 4 }}
                  </span>
                </div>
                <div
                  v-else
                  class="line-clamp-2 break-all"
                  :title="attemptDetail(attempt)"
                >
                  {{ attemptDetail(attempt) }}
                </div>
              </td>
            </tr>
          </tbody>
        </table>
      </div>

      <div
        v-else-if="resultAttempts.length === 0"
        class="py-4 text-center text-sm text-muted-foreground"
      >
        {{ resultEmptyMessage }}
      </div>

      <div
        v-if="showDebugInspector"
        class="space-y-3"
      >
        <div
          v-if="selectedInspectionAttempt"
          class="space-y-3"
        >
          <div
            v-if="showAttemptDiagnostics"
            class="flex flex-wrap items-center gap-2 text-xs text-muted-foreground"
          >
            <span class="font-medium text-foreground">
              {{ formatAttemptIndex(selectedInspectionAttempt) }}
            </span>
            <span>{{ formatAttemptAccountDisplay(selectedInspectionAttempt) }}</span>
            <span>·</span>
            <span>{{ formatApiFormat(selectedInspectionAttempt.endpoint_api_format) }}</span>
          </div>

          <Card>
            <div class="p-3 sm:p-4">
              <Tabs
                v-model="inspectionTab"
                :default-value="inspectionTab"
              >
                <div class="flex items-center border-b pb-2 mb-3">
                  <button
                    v-for="tab in detailTabs"
                    :key="tab.name"
                    class="px-2 sm:px-3 py-1.5 text-sm transition-colors border-b-2 -mb-[9px] whitespace-nowrap"
                    :class="inspectionTab === tab.name
                      ? 'border-primary text-foreground font-medium'
                      : 'border-transparent text-muted-foreground hover:text-foreground'"
                    @click="inspectionTab = tab.name"
                  >
                    {{ tab.label }}
                  </button>
                </div>

                <div class="content-block rounded-md border overflow-hidden">
                  <div class="flex items-center justify-end gap-0.5 px-3 py-1 border-b bg-muted/40">
                    <button
                      :title="inspectionExpandDepth === 0 ? t('modelTest.expandAllShort') : t('modelTest.collapseAll')"
                      class="p-1 rounded transition-colors text-muted-foreground hover:bg-muted"
                      @click="inspectionExpandDepth === 0 ? expandInspectionContent() : collapseInspectionContent()"
                    >
                      <Maximize2
                        v-if="inspectionExpandDepth === 0"
                        class="w-3.5 h-3.5"
                      />
                      <Minimize2
                        v-else
                        class="w-3.5 h-3.5"
                      />
                    </button>

                    <button
                      :title="inspectionCopiedStates[inspectionTab] ? t('modelTest.copied') : t('modelTest.copy')"
                      class="p-1 rounded transition-colors text-muted-foreground hover:bg-muted"
                      @click="copyInspectionContent(inspectionTab)"
                    >
                      <Check
                        v-if="inspectionCopiedStates[inspectionTab]"
                        class="w-3.5 h-3.5 text-green-500"
                      />
                      <Copy
                        v-else
                        class="w-3.5 h-3.5"
                      />
                    </button>
                  </div>

                  <TabsContent value="request-headers">
                    <JsonContent
                      :data="selectedInspectionAttempt.request_headers"
                      view-mode="formatted"
                      :expand-depth="inspectionExpandDepth"
                      :is-dark="isDark"
                      :empty-message="t('modelTest.noRequestHeaders')"
                    />
                  </TabsContent>

                  <TabsContent value="request-body">
                    <JsonContent
                      :data="toJsonContentData(selectedInspectionAttempt.request_body)"
                      view-mode="formatted"
                      :expand-depth="inspectionExpandDepth"
                      :is-dark="isDark"
                      :empty-message="t('modelTest.noRequestBody')"
                    />
                  </TabsContent>

                  <TabsContent value="response-headers">
                    <JsonContent
                      :data="selectedInspectionAttempt.response_headers"
                      view-mode="formatted"
                      :expand-depth="inspectionExpandDepth"
                      :is-dark="isDark"
                      :empty-message="t('modelTest.noResponseHeaders')"
                    />
                  </TabsContent>

                  <TabsContent value="response-body">
                    <div
                      v-if="selectedInspectionImagePreviews.length > 0"
                      class="mb-3 rounded-md border border-border/60 bg-muted/20 p-3"
                    >
                      <div class="mb-3 flex items-center justify-between gap-3 text-xs text-muted-foreground">
                        <span>{{ t('modelTest.imagePreview') }}</span>
                        <span>{{ t('modelTest.imageCount', { count: selectedInspectionImagePreviews.length }) }}</span>
                      </div>
                      <div class="grid gap-3 sm:grid-cols-2 xl:grid-cols-3">
                        <button
                          v-for="(preview, index) in selectedInspectionImagePreviews"
                          :key="`${preview.src}-${index}`"
                          type="button"
                          class="group block overflow-hidden rounded-md border border-border/60 bg-background text-left transition-colors hover:border-primary/60 focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-primary/70"
                          @click="openImagePreview(preview)"
                        >
                          <div class="aspect-square w-full overflow-hidden bg-muted/30">
                            <img
                              :src="preview.src"
                              :alt="preview.label"
                              class="h-full w-full object-contain"
                              loading="lazy"
                            >
                          </div>
                          <div class="border-t border-border/60 px-2 py-1 text-[11px] text-muted-foreground">
                            {{ preview.label }}
                          </div>
                        </button>
                      </div>
                    </div>
                    <JsonContent
                      :data="toJsonContentData(selectedInspectionAttempt.response_body)"
                      view-mode="formatted"
                      :expand-depth="inspectionExpandDepth"
                      :is-dark="isDark"
                      :empty-message="t('modelTest.noResponseBody')"
                    />
                  </TabsContent>
                </div>
              </Tabs>
            </div>
          </Card>
        </div>
      </div>
    </div>

    <template #footer>
      <Button
        variant="outline"
        @click="emit('close')"
      >
        {{ showSetup ? t('modelTest.cancel') : t('modelTest.close') }}
      </Button>
      <Button
        v-if="showResult"
        variant="outline"
        @click="emit('back')"
      >
          {{ t('modelTest.back') }}
      </Button>
    </template>
  </Dialog>

  <Dialog
    :open="Boolean(activeImagePreview)"
    size="6xl"
    :z-index="120"
    @update:open="(val: boolean) => { if (!val) activeImagePreview = null }"
  >
    <template #header>
      <div class="border-b border-border px-6 py-4">
        <div class="text-lg font-semibold text-foreground leading-tight">
          {{ activeImagePreview?.label || t('modelTest.imagePreview') }}
        </div>
      </div>
    </template>

    <div class="flex max-h-[76vh] items-center justify-center overflow-auto rounded-md bg-muted/20 p-3">
      <img
        v-if="activeImagePreview"
        :src="activeImagePreview.src"
        :alt="activeImagePreview.label"
        class="max-h-[72vh] max-w-full rounded-md object-contain"
      >
    </div>

    <template #footer>
      <Button
        variant="outline"
        @click="activeImagePreview = null"
      >
        {{ t('modelTest.close') }}
      </Button>
    </template>
  </Dialog>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { Check, Code2, Copy, Loader2, Maximize2, Minimize2, RotateCcw } from 'lucide-vue-next'
import {
  Badge,
  Card,
  Dialog,
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
  Tabs,
  TabsContent,
} from '@/components/ui'
import Button from '@/components/ui/button.vue'
import Textarea from '@/components/ui/textarea.vue'
import { formatApiFormat } from '@/api/endpoints/types/api-format'
import type { TestAttemptDetail, TestCandidateSummary, TestModelFailoverResponse } from '@/api/endpoints/providers'
import type { CandidateRecord, RequestTrace } from '@/api/requestTrace'
import JsonContent from '@/features/usage/components/RequestDetailDrawer/JsonContent.vue'
import { useClipboard } from '@/composables/useClipboard'
import { useDarkMode } from '@/composables/useDarkMode'
import {
  extractModelTestImagePreviews,
  extractModelTestResponsePreview,
  formatModelTestDiagnostic,
} from './model-test-request'
import type { ModelTestImagePreview } from './model-test-request'

const { t } = useI18n()

type JsonContentData = Record<string, unknown> | unknown[] | string | number | boolean | null | undefined

type TestEndpointOption = {
  id: string
  api_format: string
  base_url: string
  is_active: boolean
}

type TestModelMappingOption = {
  name: string
  priority?: number
}

const props = defineProps<{
  open: boolean
  result: TestModelFailoverResponse | null
  mode?: 'global' | 'direct' | 'pool'
  providerType?: string | null
  selectingModelName?: string | null
  requestedModelName?: string | null
  endpoints?: TestEndpointOption[]
  selectedEndpoint?: TestEndpointOption | null
  testing?: boolean
  trace?: RequestTrace | null
  requestId?: string | null
  requestHeadersDraft?: string
  requestHeadersResetValue?: string
  requestHeadersError?: string | null
  requestBodyDraft?: string
  requestBodyResetValue?: string
  requestBodyError?: string | null
  modelMappingAvailable?: boolean
  modelMappingOptions?: TestModelMappingOption[]
  selectedModelMapping?: string | null
  startDisabled?: boolean
}>()

const emit = defineEmits<{
  close: []
  back: []
  start: []
  selectEndpoint: [endpointId: string]
  selectModelMapping: [modelName: string]
  'update:requestHeadersDraft': [value: string]
  'update:requestBodyDraft': [value: string]
}>()

const endpoints = computed(() => props.endpoints ?? [])
const modelMappingOptions = computed(() => props.modelMappingOptions ?? [])
const modelMappingAvailable = computed(
  () => props.modelMappingAvailable === true && modelMappingOptions.value.length > 0,
)
const requestedModelName = computed(() => props.requestedModelName?.trim() || '')
const selectedModelMapping = computed(() => props.selectedModelMapping?.trim() || '')
const selectedModelMappingValue = computed(() => (
  selectedModelMapping.value || requestedModelName.value
))
const requestHeadersDraft = computed(() => props.requestHeadersDraft ?? '')
const requestBodyDraft = computed(() => props.requestBodyDraft ?? '')
const traceCandidates = computed(() => props.trace?.candidates ?? [])
const showSetup = computed(() => props.open && !props.testing && !props.result)
const showResult = computed(() => !!props.result)
const resultSummary = computed(() => deriveSummaryFromAttempts(props.result))
const showCandidateDiagnostics = computed(() => resultSummary.value.total_candidates > 1)
const showAttemptDiagnostics = computed(() => (
  showCandidateDiagnostics.value
))
const { isDark } = useDarkMode()
const { copyToClipboard } = useClipboard()

function handleModelMappingValueChange(value: string) {
  emit(
    'selectModelMapping',
    value === requestedModelName.value ? '' : value,
  )
}

const dialogTitle = computed(() => {
  if (props.result) return t('modelTest.resultTitle')
  return t('modelTest.title')
})

const dialogDescription = computed(() => {
  if (showSetup.value && props.selectingModelName) {
    return t('modelTest.setupDescription', { model: props.selectingModelName })
  }
  if (props.testing && props.selectedEndpoint) {
    return t('modelTest.testingDescription', { format: formatApiFormat(props.selectedEndpoint.api_format), model: props.selectingModelName || t('modelTest.model') })
  }
  return ''
})

const resultAttempts = computed(() => props.result?.attempts ?? [])

const hasEffectiveModel = computed(() => {
  if (!props.result) return false
  return resultAttempts.value.some(
    attempt => attempt.effective_model && attempt.effective_model !== props.result?.model,
  )
})

const resultEmptyMessage = computed(() => {
  if (!props.result) return t('modelTest.noCandidates')
  if (typeof props.result.error === 'string' && props.result.error.trim()) {
    return formatModelTestDiagnostic(props.result.error)
  }
  const rawResult = props.result as TestModelFailoverResponse & { message?: string }
  if (typeof rawResult.message === 'string' && rawResult.message.trim()) {
    return formatModelTestDiagnostic(rawResult.message)
  }
  return t('modelTest.noCandidates')
})
const showAllAttempts = ref(false)
const inspectionTab = ref<'request-headers' | 'request-body' | 'response-headers' | 'response-body'>('request-body')
const selectedInspectionKey = ref<string | null>(null)
const inspectionExpandDepth = ref(0)
const inspectionCopiedStates = ref<Record<string, boolean>>({})
const activeImagePreview = ref<ModelTestImagePreview | null>(null)

watch(() => props.result, () => {
  showAllAttempts.value = false
  inspectionTab.value = 'request-body'
  inspectionExpandDepth.value = 0
  inspectionCopiedStates.value = {}
  const defaultAttempt = inspectableAttempts.value[0] ?? resultAttempts.value[0] ?? null
  selectedInspectionKey.value = defaultAttempt ? inspectionKey(defaultAttempt) : null
})

const shouldCollapseAttempts = computed(() => resultAttempts.value.length > 20)

const visibleAttempts = computed(() => {
  if (!shouldCollapseAttempts.value || showAllAttempts.value) {
    return resultAttempts.value
  }
  return resultAttempts.value.slice(0, 20)
})

type SummaryView = {
  total_candidates: number
  attempted: number
  success: number
  failed: number
  skipped: number
  unused: number
  pending: number
  available: number
  completed: number
  stop_reason?: string | null
  winning_candidate_index?: number | null
  winning_key_name?: string | null
  winning_key_account_label?: string | null
  winning_key_id?: string | null
  winning_auth_type?: string | null
  winning_effective_model?: string | null
  winning_endpoint_api_format?: string | null
  winning_endpoint_base_url?: string | null
  winning_latency_ms?: number | null
  winning_status_code?: number | null
}

type SummaryBadgeItem = {
  key: string
  label: string
  value: number
  variant: 'default' | 'secondary' | 'outline' | 'success' | 'destructive'
}

function toCount(value: unknown): number {
  return typeof value === 'number' && Number.isFinite(value) ? Math.max(0, value) : 0
}

function normalizeCandidateSummary(summary: TestCandidateSummary | null | undefined): SummaryView | null {
  if (!summary) return null
  const success = toCount(summary.success)
  const failed = toCount(summary.failed)
  const skipped = toCount(summary.skipped)
  const unused = toCount(summary.unused)
  const pending = toCount(summary.pending)
  const available = toCount(summary.available)
  return {
    total_candidates: toCount(summary.total_candidates),
    attempted: toCount(summary.attempted),
    success,
    failed,
    skipped,
    unused,
    pending,
    available,
    completed: toCount(summary.completed) || success + failed + skipped + unused,
    stop_reason: summary.stop_reason ?? null,
    winning_candidate_index: summary.winning_candidate_index ?? null,
    winning_key_name: summary.winning_key_name ?? null,
    winning_key_account_label: summary.winning_key_account_label ?? null,
    winning_key_id: summary.winning_key_id ?? null,
    winning_auth_type: summary.winning_auth_type ?? null,
    winning_effective_model: summary.winning_effective_model ?? null,
    winning_endpoint_api_format: summary.winning_endpoint_api_format ?? null,
    winning_endpoint_base_url: summary.winning_endpoint_base_url ?? null,
    winning_latency_ms: summary.winning_latency_ms ?? null,
    winning_status_code: summary.winning_status_code ?? null,
  }
}

function deriveSummaryFromAttempts(result: TestModelFailoverResponse | null): SummaryView {
  const normalized = normalizeCandidateSummary(result?.candidate_summary)
  if (normalized) return normalized

  const attempts = result?.attempts ?? []
  const success = attempts.filter(attempt => attempt.status === 'success').length
  const failed = attempts.filter(attempt => ['failed', 'cancelled', 'stream_interrupted'].includes(attempt.status)).length
  const skipped = attempts.filter(attempt => attempt.status === 'skipped').length
  const pending = attempts.filter(attempt => ['pending', 'streaming'].includes(attempt.status)).length
  const available = attempts.filter(attempt => attempt.status === 'available').length
  const totalCandidates = Math.max(result?.total_candidates ?? 0, attempts.length)
  const explicitUnused = attempts.filter(attempt => attempt.status === 'unused').length
  const winningAttempt = attempts.find(attempt => attempt.status === 'success') ?? null
  const unused = explicitUnused > 0
    ? explicitUnused
    : success > 0 && winningAttempt
      ? Math.max(0, totalCandidates - winningAttempt.candidate_index - 1)
      : 0
  const attempted = result?.total_attempts ?? attempts.filter(attempt => !['skipped', 'available', 'unused'].includes(attempt.status)).length
  const stopReason = totalCandidates === 0
    ? 'no_candidate'
    : success > 0
      ? 'first_success'
      : attempted === 0 && skipped > 0
        ? 'all_skipped'
        : failed > 0 || skipped > 0
          ? 'exhausted'
          : 'pending'

  return {
    total_candidates: totalCandidates,
    attempted,
    success,
    failed,
    skipped,
    unused,
    pending,
    available,
    completed: success + failed + skipped + unused,
    stop_reason: stopReason,
    winning_candidate_index: winningAttempt?.candidate_index ?? null,
    winning_key_name: winningAttempt?.key_name ?? null,
    winning_key_account_label: winningAttempt?.key_account_label ?? null,
    winning_key_id: winningAttempt?.key_id ?? null,
    winning_auth_type: winningAttempt?.auth_type ?? null,
    winning_effective_model: winningAttempt?.effective_model ?? null,
    winning_endpoint_api_format: winningAttempt?.endpoint_api_format ?? null,
    winning_endpoint_base_url: winningAttempt?.endpoint_base_url ?? null,
    winning_latency_ms: winningAttempt?.latency_ms ?? null,
    winning_status_code: winningAttempt?.status_code ?? null,
  }
}

const liveTraceSummary = computed(() => {
  const summary: SummaryView = {
    total_candidates: traceCandidates.value.length,
    attempted: 0,
    available: 0,
    pending: 0,
    success: 0,
    failed: 0,
    skipped: 0,
    unused: 0,
    completed: 0,
    stop_reason: traceCandidates.value.length > 0 ? 'pending' : 'no_candidate',
  }

  for (const candidate of traceCandidates.value) {
    if (candidate.status === 'available') summary.available += 1
    if (candidate.status === 'unused') summary.unused += 1
    if (candidate.status === 'pending' || candidate.status === 'streaming') summary.pending += 1
    if (candidate.status === 'success') summary.success += 1
    if (candidate.status === 'failed' || candidate.status === 'cancelled' || candidate.status === 'stream_interrupted') summary.failed += 1
    if (candidate.status === 'skipped') summary.skipped += 1
  }

  summary.attempted = summary.pending + summary.success + summary.failed
  summary.completed = summary.success + summary.failed + summary.skipped + summary.unused
  if (summary.success > 0) summary.stop_reason = 'first_success'
  if (summary.success === 0 && summary.pending === 0 && summary.failed + summary.skipped > 0) summary.stop_reason = 'exhausted'
  return summary
})

const normalizedProviderType = computed(() => (
  props.providerType
  || props.result?.provider?.provider_type
  || ''
).toLowerCase())

const testScenario = computed<'pool' | 'custom' | 'direct' | 'provider'>(() => {
  if (props.mode === 'pool') return 'pool'
  if (normalizedProviderType.value === 'custom') return 'custom'
  if (props.mode === 'direct') return 'direct'
  return 'provider'
})

const candidateNoun = computed(() => {
  if (testScenario.value === 'pool') return t('modelTest.account')
  if (testScenario.value === 'custom') return 'Key'
  return t('modelTest.candidate')
})

const liveEntityLabel = computed(() => {
  if (testScenario.value === 'pool') return t('modelTest.currentAccount')
  if (testScenario.value === 'custom') return t('modelTest.currentKey')
  return t('modelTest.currentCandidate')
})

const resultEntityLabel = computed(() => {
  if (testScenario.value === 'pool') return t('modelTest.hitAccount')
  if (testScenario.value === 'custom') return t('modelTest.hitKey')
  return t('modelTest.hitCandidate')
})

function buildSummaryItems(summary: SummaryView): SummaryBadgeItem[] {
  const totalLabel = testScenario.value === 'pool'
    ? t('modelTest.candidateAccount')
    : testScenario.value === 'custom'
      ? t('modelTest.candidateKey')
      : t('modelTest.candidate')
  const successLabel = testScenario.value === 'pool'
    ? t('modelTest.hitAccount')
    : testScenario.value === 'custom'
      ? t('modelTest.hitKey')
      : t('modelTest.success')
  const failedLabel = testScenario.value === 'pool' ? t('modelTest.failover') : t('modelTest.failed')
  const skippedLabel = testScenario.value === 'pool' ? t('modelTest.schedulingSkipped') : t('modelTest.skipped')
  const unusedLabel = testScenario.value === 'pool' ? t('modelTest.unusedAfterSuccess') : t('modelTest.unused')

  return [
    { key: 'total', label: totalLabel, value: summary.total_candidates, variant: 'secondary' },
    { key: 'attempted', label: t('modelTest.attempted'), value: summary.attempted, variant: 'outline' },
    { key: 'success', label: successLabel, value: summary.success, variant: 'success' },
    { key: 'failed', label: failedLabel, value: summary.failed, variant: 'destructive' },
    { key: 'skipped', label: skippedLabel, value: summary.skipped, variant: 'secondary' },
    { key: 'unused', label: unusedLabel, value: summary.unused, variant: 'secondary' },
  ]
}

const liveSummaryItems = computed(() => buildSummaryItems(liveTraceSummary.value))
const resultSummaryItems = computed(() => buildSummaryItems(resultSummary.value))

const liveProgressText = computed(() => {
  if (liveTraceSummary.value.total_candidates <= 0) return t('modelTest.preparingShort')
  return `${liveTraceSummary.value.completed}/${liveTraceSummary.value.total_candidates}`
})

const liveProgressPercent = computed(() => {
  if (liveTraceSummary.value.total_candidates <= 0) return 8
  const raw = Math.round((liveTraceSummary.value.completed / liveTraceSummary.value.total_candidates) * 100)
  return Math.min(100, Math.max(raw, liveTraceSummary.value.pending > 0 ? 12 : 6))
})

const activeTraceCandidate = computed(() => {
  const preferredStatuses = ['pending', 'streaming', 'failed', 'success', 'skipped', 'cancelled']
  for (let index = traceCandidates.value.length - 1; index >= 0; index -= 1) {
    const candidate = traceCandidates.value[index]
    if (preferredStatuses.includes(candidate.status)) return candidate
  }
  return traceCandidates.value[0] ?? null
})

const liveAccountTitle = computed(() => {
  const candidate = activeTraceCandidate.value
  if (!candidate) return t('modelTest.waitingCandidate')
  return candidate.key_account_label || candidate.key_name || candidate.key_preview || '-'
})

const liveAccountMeta = computed(() => {
  const candidate = activeTraceCandidate.value
  if (!candidate) return ''
  const parts: string[] = []
  if (candidate.key_auth_type) parts.push(formatAuthType(candidate.key_auth_type))
  if (candidate.key_oauth_plan_type) parts.push(candidate.key_oauth_plan_type)
  if (candidate.key_preview && candidate.key_preview !== candidate.key_account_label) parts.push(candidate.key_preview)
  return parts.join(' · ')
})

const liveStatusTitle = computed(() => {
  const candidate = activeTraceCandidate.value
  if (!candidate) return t('modelTest.preparing')
  if (candidate.status === 'pending' || candidate.status === 'streaming') {
    return t('modelTest.requestingCandidate', { candidate: formatTraceCandidateIndex(candidate) })
  }
  if (candidate.status === 'success') {
    return t('modelTest.hitCandidateIndex', { candidate: formatTraceCandidateIndex(candidate) })
  }
  if (candidate.status === 'unused') {
    return t('modelTest.unusedAfterSuccess')
  }
  return statusLabel(candidate.status)
})

const liveStatusDetail = computed(() => {
  const candidate = activeTraceCandidate.value
  if (!candidate) return ''
  return traceCandidateDetail(candidate)
})

const liveRecentCandidates = computed(() => {
  return traceCandidates.value
    .filter(candidate => candidate.status !== 'available')
    .slice(-4)
    .reverse()
})

const inspectableAttempts = computed(() => {
  return resultAttempts.value.filter(hasDebugData)
})

const showDebugInspector = computed(() => {
  return inspectableAttempts.value.length > 0
})

const detailTabs = [
  { name: 'request-headers', label: t('modelTest.requestHeadersShort') },
  { name: 'request-body', label: t('modelTest.requestBodyShort') },
  { name: 'response-headers', label: t('modelTest.responseHeaders') },
  { name: 'response-body', label: t('modelTest.responseBody') },
] as const

const selectedInspectionAttempt = computed(() => {
  const key = selectedInspectionKey.value
  if (key) {
    const fromKey = resultAttempts.value.find(attempt => inspectionKey(attempt) === key)
    if (fromKey) return fromKey
  }

  return inspectableAttempts.value[0] ?? resultAttempts.value[0] ?? null
})

function toJsonContentData(value: unknown): JsonContentData {
  return value as JsonContentData
}

const selectedInspectionImagePreviews = computed(() => (
  selectedInspectionAttempt.value
    ? extractModelTestImagePreviews(selectedInspectionAttempt.value.response_body)
    : []
))

const resultWinningTitle = computed(() => {
  const summary = resultSummary.value
  const keyName = summary.winning_key_account_label || summary.winning_key_name || summary.winning_key_id
  if (keyName) return keyName
  if (props.result?.success) return t('modelTest.hitEntity', { entity: candidateNoun.value })
  return t('modelTest.noEntity', { entity: resultEntityLabel.value })
})

const resultDispatchTitle = computed(() => {
  const summary = resultSummary.value
  return t('modelTest.attemptedSummary', { attempted: summary.attempted, total: summary.total_candidates })
})

const attemptedEffectiveModelTitle = computed(() => {
  const successAttempt = resultAttempts.value.find(
    attempt => attempt.status === 'success' && attempt.effective_model,
  )
  if (successAttempt?.effective_model) return successAttempt.effective_model

  const attempted = resultAttempts.value.find(
    attempt => !['skipped', 'available', 'unused'].includes(attempt.status) && attempt.effective_model,
  )
  if (attempted?.effective_model) return attempted.effective_model

  return resultAttempts.value.find(attempt => attempt.effective_model)?.effective_model ?? null
})

const attemptedRequestModelTitle = computed(() => {
  const successAttempt = resultAttempts.value.find(
    attempt => attempt.status === 'success' && extractAttemptRequestModel(attempt),
  )
  const successRequestModel = successAttempt ? extractAttemptRequestModel(successAttempt) : null
  if (successRequestModel) return successRequestModel

  const attempted = resultAttempts.value.find(
    attempt => !['skipped', 'available', 'unused'].includes(attempt.status)
      && extractAttemptRequestModel(attempt),
  )
  const attemptedRequestModel = attempted ? extractAttemptRequestModel(attempted) : null
  if (attemptedRequestModel) return attemptedRequestModel

  const anyAttempt = resultAttempts.value.find(attempt => extractAttemptRequestModel(attempt))
  return anyAttempt ? extractAttemptRequestModel(anyAttempt) : null
})

const resultModelTitle = computed(() => {
  return attemptedRequestModelTitle.value
    || resultSummary.value.winning_effective_model
    || attemptedEffectiveModelTitle.value
    || props.result?.model
    || props.selectingModelName
    || '-'
})

function statusVariant(status: string) {
  if (status === 'success') return 'success' as const
  if (status === 'failed' || status === 'stream_interrupted') return 'destructive' as const
  return 'secondary' as const
}

function statusLabel(status: string) {
  if (status === 'success') return t('modelTest.success')
  if (status === 'failed') return t('modelTest.failed')
  if (status === 'skipped') return t('modelTest.skipped')
  if (status === 'pending') return t('modelTest.pending')
  if (status === 'streaming') return t('modelTest.testingShort')
  if (status === 'cancelled') return t('modelTest.cancelled')
  if (status === 'stream_interrupted') return t('modelTest.streamInterrupted')
  if (status === 'available') return t('modelTest.available')
  if (status === 'unused') return t('modelTest.unused')
  return status
}

function statusDisplay(item: { status: string; status_code?: number | null }): string {
  const code = item.status_code
  const status = item.status
  if (!code) return statusLabel(status)
  if (status === 'failed' && code >= 200 && code < 300) {
    return t('modelTest.bodyError', { code })
  }
  return String(code)
}

function attemptRowClass(status: string, selected = false) {
  const statusClass = (() => {
    if (status === 'success') return 'bg-green-500/5'
    if (status === 'failed') return 'bg-red-500/5'
    if (status === 'cancelled') return 'bg-amber-500/5'
    if (status === 'skipped') return 'bg-muted/20'
    return ''
  })()
  const interactionClass = 'cursor-pointer transition-colors hover:bg-muted/40 focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-primary/70'
  const selectedClass = selected ? 'ring-1 ring-primary/70 bg-primary/5' : ''
  return [statusClass, interactionClass, selectedClass].filter(Boolean).join(' ')
}

function maskKey(key: string): string {
  if (key.length <= 8) return key
  return `${key.slice(0, 4)}...${key.slice(-4)}`
}

function formatAttemptAccountDisplay(attempt: TestAttemptDetail): string {
  return attempt.key_account_label || attempt.key_name || (attempt.key_id ? maskKey(attempt.key_id) : '-')
}

function formatAttemptAccountTitle(attempt: TestAttemptDetail): string {
  return attempt.key_account_label || attempt.key_name || attempt.key_id || '-'
}

function formatAuthType(authType: string): string {
  const lowered = authType.toLowerCase()
  if (lowered === 'api_key') return 'API Key'
  if (lowered === 'service_account') return 'Service Account'
  if (lowered === 'oauth') return 'OAuth'
  if (lowered === 'codex') return 'Codex OAuth'
  if (lowered === 'antigravity') return 'Antigravity OAuth'
  if (lowered === 'kiro') return 'Kiro OAuth'
  if (lowered === 'grok') return 'Grok OAuth'
  return authType
}

function formatAttemptIndex(attempt: TestAttemptDetail): string {
  const retryIndex = attempt.retry_index ?? 0
  return retryIndex > 0 ? `#${attempt.candidate_index}.${retryIndex}` : `#${attempt.candidate_index}`
}

function formatTraceCandidateIndex(candidate: CandidateRecord): string {
  return candidate.retry_index > 0 ? `#${candidate.candidate_index}.${candidate.retry_index}` : `#${candidate.candidate_index}`
}

function formatTraceCandidateAccount(candidate: CandidateRecord): string {
  return candidate.key_account_label || candidate.key_name || candidate.key_preview || '-'
}

function extractAttemptRequestModel(attempt: TestAttemptDetail): string | null {
  const body = attempt.request_body
  if (!body || typeof body !== 'object' || Array.isArray(body)) return null

  const model = (body as Record<string, unknown>).model
  if (typeof model === 'string' && model.trim()) return model.trim()

  const conversationState = (body as Record<string, unknown>).conversationState
  if (!conversationState || typeof conversationState !== 'object' || Array.isArray(conversationState)) return null

  const currentMessage = (conversationState as Record<string, unknown>).currentMessage
  if (!currentMessage || typeof currentMessage !== 'object' || Array.isArray(currentMessage)) return null

  const userInputMessage = (currentMessage as Record<string, unknown>).userInputMessage
  if (!userInputMessage || typeof userInputMessage !== 'object' || Array.isArray(userInputMessage)) return null

  const modelId = (userInputMessage as Record<string, unknown>).modelId
  return typeof modelId === 'string' && modelId.trim() ? modelId.trim() : null
}

function traceCandidateDetail(candidate: CandidateRecord): string {
  if (candidate.skip_reason) return formatModelTestDiagnostic(candidate.skip_reason)
  if (candidate.error_message) return formatModelTestDiagnostic(candidate.error_message)
  if (candidate.status === 'unused') return t('modelTest.unusedAfterSuccess')
  if (candidate.status === 'available') return t('modelTest.available')
  if (candidate.endpoint_name) return t('modelTest.endpointLabel', { endpoint: formatApiFormat(candidate.endpoint_name) })
  return ''
}

function attemptDetail(attempt: TestAttemptDetail): string {
  if (attempt.status === 'cancelled') return t('modelTest.testCancelled')
  if (attempt.status === 'unused') return t('modelTest.unusedAfterSuccess')
  if (attempt.skip_reason) return formatModelTestDiagnostic(attempt.skip_reason)
  if (attempt.error_message) return formatModelTestDiagnostic(attempt.error_message)
  if (attempt.status === 'success') {
    return extractModelTestResponsePreview(attempt.response_body)
      ?? (attempt.effective_model ? t('modelTest.requestModel', { model: attempt.effective_model }) : attempt.endpoint_base_url)
  }
  return '-'
}

function attemptImagePreviews(attempt: TestAttemptDetail): ModelTestImagePreview[] {
  return extractModelTestImagePreviews(attempt.response_body)
}

function openImagePreview(preview: ModelTestImagePreview) {
  activeImagePreview.value = preview
}

function inspectionKey(attempt: TestAttemptDetail): string {
  return `${attempt.candidate_index}:${attempt.retry_index ?? 0}:${attempt.key_id}`
}

function selectInspectionAttempt(attempt: TestAttemptDetail) {
  selectedInspectionKey.value = inspectionKey(attempt)
  inspectionTab.value = 'request-body'
}

function hasDebugData(attempt: TestAttemptDetail): boolean {
  return Boolean(
    attempt.request_url
    || attempt.request_headers
    || attempt.request_body != null
    || attempt.response_headers
    || attempt.response_body != null,
  )
}

function getInspectionTabData(
  tabName: typeof detailTabs[number]['name'],
  attempt: TestAttemptDetail | null,
): unknown {
  if (!attempt) return null
  switch (tabName) {
    case 'request-headers':
      return attempt.request_headers
    case 'request-body':
      return attempt.request_body
    case 'response-headers':
      return attempt.response_headers
    case 'response-body':
      return attempt.response_body
  }
}

function copyInspectionContent(tabName: typeof detailTabs[number]['name']) {
  const data = getInspectionTabData(tabName, selectedInspectionAttempt.value)
  if (data === null || data === undefined || data === '') return

  const text = typeof data === 'string' ? data : JSON.stringify(data, null, 2)
  copyToClipboard(text, false)
  inspectionCopiedStates.value[tabName] = true
  setTimeout(() => {
    inspectionCopiedStates.value[tabName] = false
  }, 2000)
}

function expandInspectionContent() {
  inspectionExpandDepth.value = 999
}

function collapseInspectionContent() {
  inspectionExpandDepth.value = 0
}

function formatRequestHeadersDraft() {
  formatJsonDraft(requestHeadersDraft.value, value => emit('update:requestHeadersDraft', value), '{}')
}

function formatRequestBodyDraft() {
  formatJsonDraft(requestBodyDraft.value, value => emit('update:requestBodyDraft', value))
}

function resetRequestHeadersDraft() {
  emit('update:requestHeadersDraft', props.requestHeadersResetValue ?? '{}')
}

function resetRequestBodyDraft() {
  emit('update:requestBodyDraft', props.requestBodyResetValue ?? '')
}

function formatJsonDraft(
  draft: string,
  onFormatted: (value: string) => void,
  emptyFallback?: string,
) {
  const normalized = draft.trim()
  if (!normalized) {
    if (emptyFallback !== undefined) {
      onFormatted(emptyFallback)
    }
    return
  }

  try {
    const parsed = JSON.parse(normalized)
    if (!parsed || typeof parsed !== 'object' || Array.isArray(parsed)) return
    onFormatted(JSON.stringify(parsed, null, 2))
  } catch {
    // keep user input untouched when JSON is invalid
  }
}
</script>

<style scoped>
.content-block :deep(.rounded-2xl) {
  border: none !important;
  border-radius: 0 !important;
  box-shadow: none !important;
}
</style>
