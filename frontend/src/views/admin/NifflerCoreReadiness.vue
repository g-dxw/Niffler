<template>
  <div class="space-y-6 pb-8">
    <Card class="p-5 border-border/70 bg-card/95">
      <div class="flex flex-col gap-4 lg:flex-row lg:items-start lg:justify-between">
        <div class="space-y-2">
          <div class="flex flex-wrap items-center gap-2">
            <h2 class="text-xl font-semibold">
              {{ t('coreReadiness.title') }}
            </h2>
            <Badge variant="outline">
              {{ t('coreReadiness.readonly') }}
            </Badge>
          </div>
          <p class="max-w-3xl text-sm text-muted-foreground">
            {{ t('coreReadiness.description') }}
          </p>
        </div>
        <div class="flex items-center gap-2">
          <Select v-model="recentDays">
            <SelectTrigger class="h-9 w-32">
              <SelectValue :placeholder="t('coreReadiness.range')" />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="7">
                {{ t('coreReadiness.days7') }}
              </SelectItem>
              <SelectItem value="30">
                {{ t('coreReadiness.days30') }}
              </SelectItem>
              <SelectItem value="90">
                {{ t('coreReadiness.days90') }}
              </SelectItem>
            </SelectContent>
          </Select>
          <RefreshButton
            :loading="loading || stabilityLoading"
            @click="loadReadinessPage"
          />
        </div>
      </div>
    </Card>

    <Card
      v-if="error"
      class="p-4 border-destructive/30 bg-destructive/5"
    >
      <div class="flex items-start gap-3">
        <AlertCircle class="mt-0.5 h-5 w-5 shrink-0 text-destructive" />
        <div>
          <p class="font-medium text-destructive">
            {{ t('coreReadiness.failed') }}
          </p>
          <p class="mt-1 text-sm text-muted-foreground">
            {{ error }}
          </p>
        </div>
      </div>
    </Card>

    <div
      v-if="loading && !report"
      class="py-16 text-center text-muted-foreground"
    >
      <Loader2 class="mx-auto h-8 w-8 animate-spin" />
      <p class="mt-3 text-sm">
        {{ t('coreReadiness.loading') }}
      </p>
    </div>

    <template v-else-if="report">
      <div class="grid gap-4 md:grid-cols-2 xl:grid-cols-5">
        <MetricCard
          :title="t('coreReadiness.shadowTables')"
          :value="`${report.shadow_tables.existing_tables}/${report.shadow_tables.expected_tables}`"
          :description="report.shadow_tables.all_present ? t('coreReadiness.complete') : t('coreReadiness.missing')"
          :tone="report.shadow_tables.all_present ? 'success' : 'danger'"
        />
        <MetricCard
          :title="t('coreReadiness.providerMapping')"
          :value="`${report.provider_mapping.mapped_count}/${report.provider_mapping.legacy_count}`"
          :description="t('coreReadiness.blockedProviders', { count: report.provider_mapping.blocked_count })"
          :tone="report.provider_mapping.blocked_count ? 'warning' : 'success'"
        />
        <MetricCard
          :title="t('coreReadiness.accountMapping')"
          :value="`${report.account_mapping.mapped_count}/${report.account_mapping.legacy_count}`"
          :description="t('coreReadiness.blockedAccounts', { count: report.account_mapping.blocked_count })"
          :tone="report.account_mapping.blocked_count ? 'warning' : 'success'"
        />
        <MetricCard
          :title="t('coreReadiness.productMapping')"
          :value="`${report.product_plan_mapping.mapped_count}/${report.product_plan_mapping.legacy_count}`"
          :description="t('coreReadiness.publicInternalPlans', { public: report.summary.product_plans_public, internal: report.summary.product_plans_total - report.summary.product_plans_public })"
          :tone="report.product_plan_mapping.blocked_count ? 'warning' : 'success'"
        />
        <MetricCard
          :title="t('coreReadiness.requestIssues')"
          :value="String(report.summary.recent_problem_usage_sample_count)"
          :description="t('coreReadiness.recentDaysSample', { days: report.recent_days })"
          :tone="report.summary.recent_problem_usage_sample_count ? 'warning' : 'success'"
        />
      </div>

      <Card class="overflow-hidden">
        <div class="flex flex-col gap-3 border-b border-border/60 px-5 py-4 lg:flex-row lg:items-start lg:justify-between">
          <div>
            <div class="flex flex-wrap items-center gap-2">
              <h3 class="font-semibold">
                {{ t('coreReadiness.stability') }}
              </h3>
              <Badge variant="outline">
                {{ t('coreReadiness.max15') }}
              </Badge>
            </div>
            <p class="mt-1 text-sm text-muted-foreground">
              {{ t('coreReadiness.stabilityHint') }}
            </p>
          </div>
          <RefreshButton
            :loading="stabilityLoading"
            @click="loadStabilityObservations"
          />
        </div>
        <div
          v-if="stabilityLoading && !latestStabilityObservation"
          class="p-6 text-center text-sm text-muted-foreground"
        >
          {{ t('coreReadiness.loadingStability') }}
        </div>
        <div
          v-else-if="stabilityError"
          class="flex items-start gap-3 p-5 text-sm text-destructive"
        >
          <AlertCircle class="mt-0.5 h-5 w-5 shrink-0" />
          <span>{{ stabilityError }}</span>
        </div>
        <div
          v-else-if="latestStabilityObservation"
          class="space-y-5 p-5"
        >
          <div class="grid gap-4 md:grid-cols-2 xl:grid-cols-5">
            <MetricCard
              :title="t('coreReadiness.observationStatus')"
              :value="stabilityStatusLabel(latestStabilityObservation.status)"
              :description="formatWindow(latestStabilityObservation.window_start_unix_ms, latestStabilityObservation.window_end_unix_ms)"
              :tone="stabilityStatusTone(latestStabilityObservation.status)"
            />
            <MetricCard
              :title="t('coreReadiness.progress')"
              :value="t('coreReadiness.daysProgress', { current: stabilityConsecutivePassDays, required: STABILITY_REQUIRED_PASS_DAYS })"
              :description="stabilityGateDescription"
              :tone="stabilityReadyForLegacyRemoval ? 'success' : 'warning'"
            />
            <MetricCard
              :title="t('coreReadiness.rollback')"
              :value="rollbackDrillLabel(latestStabilityObservation.rollback_drill_status)"
              :description="t('coreReadiness.rollbackStatusConfig')"
              :tone="rollbackDrillTone(latestStabilityObservation.rollback_drill_status)"
            />
            <MetricCard
              :title="t('coreReadiness.unknownUpstream')"
              :value="String(latestStabilityObservation.unknown_upstream_count)"
              :description="t('coreReadiness.unknownUpstreamHint')"
              :tone="latestStabilityObservation.unknown_upstream_count ? 'danger' : 'success'"
            />
            <MetricCard
              :title="t('coreReadiness.consistencyIssues')"
              :value="String(latestStabilityObservation.consistency_issue_count)"
              :description="t('coreReadiness.checkedSamples', { count: latestStabilityObservation.consistency_checked_count })"
              :tone="latestStabilityObservation.consistency_issue_count ? 'danger' : 'success'"
            />
          </div>

          <div class="grid gap-4 xl:grid-cols-2">
            <div class="rounded-lg border border-border/60">
              <div class="border-b border-border/60 px-4 py-3">
                <p class="font-medium">
                  {{ t('coreReadiness.blockers') }}
                </p>
              </div>
              <div
                v-if="stabilityBlockerItems.length"
                class="divide-y divide-border/60"
              >
                <div
                  v-for="item in stabilityBlockerItems"
                  :key="item.code"
                  class="space-y-1 px-4 py-3"
                >
                  <p class="text-sm font-medium">
                    {{ item.title }}
                  </p>
                  <p class="text-sm text-muted-foreground">
                    {{ item.description }}
                  </p>
                </div>
              </div>
              <div
                v-else
                class="p-4 text-sm text-muted-foreground"
              >
                {{ t('coreReadiness.noBlockers') }}
              </div>
            </div>

            <div class="rounded-lg border border-border/60">
              <div class="border-b border-border/60 px-4 py-3">
                <p class="font-medium">
                  {{ t('coreReadiness.recent') }}
                </p>
              </div>
              <div class="divide-y divide-border/60">
                <div
                  v-for="item in stabilityObservations"
                  :key="item.id"
                  class="flex items-start justify-between gap-4 px-4 py-3 text-sm"
                >
                  <div>
                    <p class="font-medium">
                      {{ stabilityStatusLabel(item.status) }}
                    </p>
                    <p class="text-xs text-muted-foreground">
                      {{ formatWindow(item.window_start_unix_ms, item.window_end_unix_ms) }}
                    </p>
                  </div>
                  <div class="text-right text-xs text-muted-foreground">
                    <p>{{ t('coreReadiness.blockerCount', { count: item.blocker_codes.length }) }}</p>
                    <p>{{ t('coreReadiness.unknownUpstreamCount', { count: item.unknown_upstream_count }) }}</p>
                  </div>
                </div>
              </div>
            </div>
          </div>

          <div class="rounded-lg border border-border/60">
            <div class="flex flex-col gap-3 border-b border-border/60 px-4 py-3 lg:flex-row lg:items-start lg:justify-between">
              <div>
                <div class="flex flex-wrap items-center gap-2">
                  <p class="font-medium">
                    {{ t('coreReadiness.rollbackEvidence') }}
                  </p>
                  <Badge :variant="rollbackEvidence?.evidence_complete ? 'outline' : 'secondary'">
                    {{ rollbackEvidenceStatusText }}
                  </Badge>
                </div>
                <p class="mt-1 text-sm text-muted-foreground">
                  {{ t('coreReadiness.rollbackEvidenceHint') }}
                </p>
              </div>
              <RefreshButton
                :loading="rollbackEvidenceLoading"
                @click="loadRollbackDrillEvidence"
              />
            </div>

            <div
              v-if="rollbackEvidenceLoading && !rollbackEvidence"
              class="p-5 text-sm text-muted-foreground"
            >
              {{ t('coreReadiness.loadingRollbackEvidence') }}
            </div>
            <div
              v-else
              class="space-y-4 p-4"
            >
              <div
                v-if="rollbackEvidenceError"
                class="flex items-start gap-2 rounded-md border border-destructive/30 bg-destructive/5 p-3 text-sm text-destructive"
              >
                <AlertCircle class="mt-0.5 h-4 w-4 shrink-0" />
                <span>{{ rollbackEvidenceError }}</span>
              </div>

              <div
                v-if="rollbackEvidence"
                class="rounded-md border px-3 py-3 text-sm"
                :class="rollbackEvidenceHintClass"
              >
                <div class="flex items-start gap-2">
                  <component
                    :is="rollbackEvidenceHintIcon"
                    class="mt-0.5 h-4 w-4 shrink-0"
                    :class="toneClass(rollbackEvidenceHint.tone)"
                  />
                  <div class="space-y-1">
                    <p class="font-medium text-foreground">
                      {{ rollbackEvidenceHint.title }}
                    </p>
                    <p class="text-muted-foreground">
                      {{ rollbackEvidenceHint.description }}
                    </p>
                    <p
                      v-if="rollbackEvidenceMissingLabels.length"
                      class="text-xs text-muted-foreground"
                    >
                      {{ t('coreReadiness.missingFields', { fields: rollbackEvidenceMissingLabels.join('、') }) }}
                    </p>
                  </div>
                </div>
              </div>

              <div class="grid gap-3 text-sm md:grid-cols-3">
                <div>
                  <p class="text-xs text-muted-foreground">
                    {{ t('coreReadiness.statusConfig') }}
                  </p>
                  <p class="mt-1 break-all font-mono text-xs">
                    {{ rollbackEvidence?.status_config_key || '-' }}
                  </p>
                </div>
                <div>
                  <p class="text-xs text-muted-foreground">
                    {{ t('coreReadiness.evidenceConfig') }}
                  </p>
                  <p class="mt-1 break-all font-mono text-xs">
                    {{ rollbackEvidence?.evidence_config_key || '-' }}
                  </p>
                </div>
                <div>
                  <p class="text-xs text-muted-foreground">
                    {{ t('coreReadiness.latestRecord') }}
                  </p>
                  <p class="mt-1">
                    {{ formatUnixMs(rollbackEvidence?.evidence.recorded_at_unix_ms || 0) }}
                  </p>
                </div>
              </div>

              <div class="grid gap-4 lg:grid-cols-2">
                <div class="space-y-2">
                  <Label for="rollback-drill-status">
                    {{ t('coreReadiness.drillStatus') }}
                  </Label>
                  <Select
                    id="rollback-drill-status"
                    v-model="rollbackEvidenceForm.status"
                  >
                    <SelectTrigger>
                    <SelectValue :placeholder="t('coreReadiness.selectDrillStatus')" />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="not_recorded">
                        {{ t('coreReadiness.notRecorded') }}
                      </SelectItem>
                      <SelectItem value="failed">
                        {{ t('coreReadiness.drillFailed') }}
                      </SelectItem>
                      <SelectItem value="passed">
                        {{ t('coreReadiness.passed') }}
                      </SelectItem>
                    </SelectContent>
                  </Select>
                </div>
                <div class="space-y-2">
                  <Label for="rollback-backup-reference">
                    {{ t('coreReadiness.backupReference') }}
                  </Label>
                  <Input
                    id="rollback-backup-reference"
                    v-model="rollbackEvidenceForm.backup_reference"
                    :placeholder="t('coreReadiness.backupReferencePlaceholder')"
                  />
                </div>
                <div class="space-y-2">
                  <Label for="rollback-image-tag">
                    {{ t('coreReadiness.rollbackImageTag') }}
                  </Label>
                  <Input
                    id="rollback-image-tag"
                    v-model="rollbackEvidenceForm.rollback_image_tag"
                    :placeholder="t('coreReadiness.rollbackImageTagPlaceholder')"
                  />
                </div>
                <div class="space-y-2">
                  <Label>
                    {{ t('coreReadiness.recordedBy') }}
                  </Label>
                  <Input
                    :model-value="rollbackEvidence?.evidence.recorded_by || '-'"
                    disabled
                  />
                </div>
              </div>

              <div class="space-y-2">
                <Label for="rollback-drill-summary">
                  {{ t('coreReadiness.drillSummary') }}
                </Label>
                <Textarea
                  id="rollback-drill-summary"
                  v-model="rollbackEvidenceForm.drill_summary"
                  rows="4"
                  :placeholder="t('coreReadiness.drillSummaryPlaceholder')"
                />
              </div>

              <div class="flex flex-col gap-3 border-t border-border/60 pt-4 sm:flex-row sm:items-center sm:justify-between">
                <p class="text-xs text-muted-foreground">
                  {{ t('coreReadiness.saveEvidenceHint') }}
                </p>
                <Button
                  :disabled="rollbackEvidenceSubmitDisabled"
                  @click="saveRollbackDrillEvidence"
                >
                  <Loader2
                    v-if="rollbackEvidenceSaving"
                    class="mr-2 h-4 w-4 animate-spin"
                  />
                  {{ t('coreReadiness.saveEvidence') }}
                </Button>
              </div>
            </div>
          </div>
        </div>
        <div
          v-else
          class="p-6 text-center text-sm text-muted-foreground"
        >
          {{ t('coreReadiness.noStabilityRecords') }}
        </div>
      </Card>

      <Card class="overflow-hidden">
        <SectionHeader
          :title="t('coreReadiness.issuesTitle')"
          :description="report.issues.length ? t('coreReadiness.issuesSummary') : t('coreReadiness.noBlockingIssues')"
        />
        <div
          v-if="report.issues.length === 0"
          class="flex items-center gap-3 p-5 text-sm text-muted-foreground"
        >
          <CheckCircle2 class="h-5 w-5 text-primary" />
          {{ t('coreReadiness.noIssues') }}
        </div>
        <div
          v-else
          class="divide-y divide-border/60"
        >
          <div
            v-for="issue in report.issues"
            :key="issue.code"
            class="flex items-start gap-3 p-5"
          >
            <component
              :is="issueIcon(issue.severity)"
              class="mt-0.5 h-5 w-5 shrink-0"
              :class="issueIconClass(issue.severity)"
            />
            <div>
              <div class="flex flex-wrap items-center gap-2">
                <p class="font-medium">
                  {{ issue.title }}
                </p>
                <Badge :variant="issue.severity === 'error' ? 'destructive' : 'secondary'">
                  {{ severityLabel(issue.severity) }}
                </Badge>
              </div>
              <p class="mt-1 text-sm text-muted-foreground">
                {{ issue.message }}
              </p>
            </div>
          </div>
        </div>
      </Card>

      <Card
        v-if="!legacyAudit"
        class="p-5 border-border/70"
      >
        <div class="flex flex-col gap-4 lg:flex-row lg:items-start lg:justify-between">
          <div class="space-y-1">
            <div class="flex flex-wrap items-center gap-2">
              <h3 class="font-semibold">
                {{ t('coreReadiness.legacyAuditTitle') }}
              </h3>
              <Badge variant="outline">
                {{ t('coreReadiness.manualReadonly') }}
              </Badge>
            </div>
            <p class="max-w-3xl text-sm text-muted-foreground">
              {{ t('coreReadiness.legacyAuditPendingHint') }}
            </p>
            <p
              v-if="legacyAuditError"
              class="text-sm text-destructive"
            >
              {{ legacyAuditError }}
            </p>
          </div>
          <RefreshButton
            :loading="legacyAuditLoading"
            @click="loadLegacyAudit"
          />
        </div>
      </Card>

      <Card
        v-else
        class="overflow-hidden"
      >
        <div class="flex flex-col gap-3 border-b border-border/60 px-5 py-4 lg:flex-row lg:items-start lg:justify-between">
          <div>
            <h3 class="font-semibold">
              {{ t('coreReadiness.legacyAuditTitle') }}
            </h3>
            <p class="mt-1 text-sm text-muted-foreground">
              {{ t('coreReadiness.legacyAuditLoadedHint') }}
            </p>
          </div>
          <RefreshButton
            :loading="legacyAuditLoading"
            @click="loadLegacyAudit"
          />
        </div>
        <div class="grid gap-4 p-5 md:grid-cols-2 xl:grid-cols-3">
          <MetricCard
            :title="t('coreReadiness.userKeyRestrictions')"
            :value="String(legacyAudit.summary.user_key_restrictions_in_page)"
            :description="legacyAudit.has_more_user_keys ? t('coreReadiness.moreUserKeys') : t('coreReadiness.userKeysComplete')"
            :tone="legacyAudit.summary.user_key_restrictions_in_page ? 'warning' : 'success'"
          />
          <MetricCard
            :title="t('coreReadiness.groupLegacyRules')"
            :value="String(legacyAudit.summary.user_group_policy_items)"
            :description="t('coreReadiness.groupLegacyRulesHint')"
            :tone="legacyAudit.summary.user_group_policy_items ? 'warning' : 'success'"
          />
          <MetricCard
            :title="t('coreReadiness.providerKeyRestrictions')"
            :value="String(legacyAudit.summary.provider_key_restriction_items)"
            :description="t('coreReadiness.providerKeyRestrictionsHint')"
            :tone="legacyAudit.summary.provider_key_restriction_items ? 'warning' : 'success'"
          />
          <MetricCard
            :title="t('coreReadiness.legacyPriceDependencies')"
            :value="String(legacyAudit.summary.provider_model_price_dependency_items)"
            :description="t('coreReadiness.legacyPriceDependenciesHint')"
            :tone="legacyAudit.summary.provider_model_price_dependency_items ? 'warning' : 'success'"
          />
          <MetricCard
            :title="t('coreReadiness.legacyWriteEntrypoints')"
            :value="String(legacyAudit.summary.legacy_write_entrypoints)"
            :description="t('coreReadiness.legacyWriteEntrypointsHint')"
            tone="warning"
          />
          <MetricCard
            :title="t('coreReadiness.legacyReadPaths')"
            :value="String(legacyAudit.summary.runtime_read_dependencies)"
            :description="t('coreReadiness.legacyReadPathsHint')"
            tone="warning"
          />
        </div>

        <div class="grid gap-4 border-t border-border/60 p-5 xl:grid-cols-2">
          <ListCard
            :title="t('coreReadiness.auditNotes')"
            :description="t('coreReadiness.auditPagination', { offset: legacyAudit.offset, limit: legacyAudit.limit })"
            :items="legacyAuditNoteItems"
            :empty-text="t('coreReadiness.noAdditionalNotes')"
          />
          <ListCard
            :title="t('coreReadiness.userKeyRestrictions')"
            :description="t('coreReadiness.userKeyRestrictionsHint')"
            :items="legacyUserKeyRestrictionItems"
            :empty-text="t('coreReadiness.noUserKeyRestrictions')"
          />
          <ListCard
            :title="t('coreReadiness.userGroupLegacyRules')"
            :description="t('coreReadiness.userGroupLegacyRulesHint')"
            :items="legacyGroupPolicyItems"
            :empty-text="t('coreReadiness.noUserGroupLegacyRules')"
          />
          <ListCard
            :title="t('coreReadiness.providerKeyRestrictions')"
            :description="t('coreReadiness.providerKeyRestrictionsListHint')"
            :items="legacyProviderKeyRestrictionItems"
            :empty-text="t('coreReadiness.noProviderKeyRestrictions')"
          />
          <ListCard
            :title="t('coreReadiness.providerModelPriceDependencies')"
            :description="t('coreReadiness.providerModelPriceDependenciesHint')"
            :items="legacyProviderModelPriceItems"
            :empty-text="t('coreReadiness.noLegacyPriceDependencies')"
          />
          <ListCard
            :title="t('coreReadiness.legacyWriteEntrypoints')"
            :description="t('coreReadiness.legacyWriteEntrypointsListHint')"
            :items="legacyWriteEntrypointItems"
            :empty-text="t('coreReadiness.noLegacyWriteEntrypoints')"
          />
          <ListCard
            :title="t('coreReadiness.legacyRuntimeReadPaths')"
            :description="t('coreReadiness.legacyRuntimeReadPathsHint')"
            :items="legacyRuntimeReadItems"
            :empty-text="t('coreReadiness.noLegacyReadPaths')"
          />
        </div>
      </Card>

      <div class="grid gap-4 xl:grid-cols-2">
        <Card class="overflow-hidden">
          <SectionHeader
            :title="t('coreReadiness.shadowTableStatus')"
            :description="t('coreReadiness.databaseDriver', { driver: report.shadow_tables.database_driver || t('coreReadiness.notConfigured') })"
          />
          <CompactTable
            :rows="shadowTableRows"
            :empty-text="t('coreReadiness.noShadowTableResults')"
          />
        </Card>

        <Card class="overflow-hidden">
          <SectionHeader
          :title="t('coreReadiness.accountStatusMapping')"
          :description="t('coreReadiness.accountStatusMappingHint')"
          />
          <CompactTable
            :rows="accountStatusRows"
          :empty-text="t('coreReadiness.noAccountData')"
          />
        </Card>
      </div>

      <div class="grid gap-4 xl:grid-cols-2">
        <ListCard
          :title="t('coreReadiness.disabledProviderReferences')"
          :description="t('coreReadiness.disabledProviderReferencesHint')"
          :items="disabledProviderItems"
          :empty-text="t('coreReadiness.noDisabledProviderReferences')"
        />
        <ListCard
          :title="t('coreReadiness.keyRestrictions')"
          :description="t('coreReadiness.keyRestrictionsHint')"
          :items="keyResidueItems"
          :empty-text="t('coreReadiness.noKeyRestrictions')"
        />
        <ListCard
          :title="t('coreReadiness.groupPolicyGaps')"
          :description="t('coreReadiness.groupPolicyGapsHint')"
          :items="groupGapItems"
          :empty-text="t('coreReadiness.noGroupPolicyGaps')"
        />
        <ListCard
          :title="t('coreReadiness.pricingGaps')"
          :description="t('coreReadiness.pricingGapsHint')"
          :items="priceGapItems"
          :empty-text="t('coreReadiness.noPricingGaps')"
        />
      </div>

      <Card class="overflow-hidden">
        <SectionHeader
          :title="t('coreReadiness.recentRequestAnomalies')"
          :description="t('coreReadiness.recentRequestAnomaliesHint')"
        />
        <Table class="hidden lg:table">
          <TableHeader>
            <TableRow>
              <TableHead>{{ t('coreReadiness.request') }}</TableHead>
              <TableHead>{{ t('coreReadiness.model') }}</TableHead>
              <TableHead>Provider</TableHead>
              <TableHead>{{ t('coreReadiness.billingSnapshot') }}</TableHead>
              <TableHead>{{ t('coreReadiness.status') }}</TableHead>
              <TableHead>{{ t('coreReadiness.diagnosis') }}</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            <TableRow
              v-for="item in report.recent_usage_anomalies"
              :key="item.usage_id"
            >
              <TableCell class="max-w-[220px]">
                <div class="truncate font-mono text-xs">
                  {{ item.request_id }}
                </div>
                <div class="text-xs text-muted-foreground">
                  {{ formatTime(item.created_at_unix_secs) }}
                </div>
              </TableCell>
              <TableCell class="text-sm">
                {{ item.model }}
              </TableCell>
              <TableCell class="max-w-[220px] text-sm">
                <div class="truncate">
                  {{ item.provider_display_name || item.provider_name || t('coreReadiness.unselectedUpstream') }}
                </div>
                <div
                  v-if="item.provider_account_label || item.provider_api_key_name"
                  class="truncate text-xs text-muted-foreground"
                >
                  {{ item.provider_account_label || item.provider_api_key_name }}
                </div>
              </TableCell>
              <TableCell class="text-sm">
                <div class="tabular-nums">
                  {{ t('coreReadiness.walletDebit', { amount: formatUsd(item.wallet_debit_usd) }) }}
                </div>
                <div class="tabular-nums text-xs text-muted-foreground">
                  {{ t('coreReadiness.packageDebit', { amount: formatUsd(item.package_debit_usd) }) }}
                </div>
              </TableCell>
              <TableCell>
                <Badge variant="outline">
                  {{ item.status }} / {{ item.billing_status }}
                </Badge>
              </TableCell>
              <TableCell class="max-w-[360px] text-sm text-muted-foreground">
                <div class="font-medium text-foreground">
                  {{ item.anomaly_label }}
                </div>
                <div>{{ item.diagnosis }}</div>
                <div class="mt-1">
                  {{ t('coreReadiness.recommendation', { text: item.recommended_action }) }}
                </div>
              </TableCell>
            </TableRow>
            <TableRow v-if="report.recent_usage_anomalies.length === 0">
              <TableCell
                colspan="6"
                class="py-8 text-center text-sm text-muted-foreground"
              >
                {{ t('coreReadiness.noRequestAnomalies') }}
              </TableCell>
            </TableRow>
          </TableBody>
        </Table>
        <div class="divide-y divide-border/60 lg:hidden">
          <div
            v-for="item in report.recent_usage_anomalies"
            :key="item.usage_id"
            class="space-y-2 p-4"
          >
            <div class="flex items-center justify-between gap-2">
              <span class="truncate font-mono text-xs">{{ item.request_id }}</span>
              <Badge variant="outline">
                {{ item.status }}
              </Badge>
            </div>
            <p class="text-sm">
              {{ item.model }} · {{ item.provider_display_name || item.provider_name || t('coreReadiness.unselectedUpstream') }}
            </p>
            <p
              v-if="item.provider_account_label || item.provider_api_key_name"
              class="text-xs text-muted-foreground"
            >
              {{ t('coreReadiness.account', { value: item.provider_account_label || item.provider_api_key_name }) }}
            </p>
            <p class="text-xs text-muted-foreground">
              {{ t('coreReadiness.walletAndPackage', { wallet: formatUsd(item.wallet_debit_usd), package: formatUsd(item.package_debit_usd) }) }}
            </p>
            <p class="text-sm text-muted-foreground">
              {{ item.diagnosis }}
            </p>
            <p class="text-sm text-muted-foreground">
              {{ t('coreReadiness.recommendation', { text: item.recommended_action }) }}
            </p>
          </div>
          <div
            v-if="report.recent_usage_anomalies.length === 0"
            class="p-6 text-center text-sm text-muted-foreground"
          >
            {{ t('coreReadiness.noRequestAnomalies') }}
          </div>
        </div>
      </Card>

      <ListCard
        :title="t('coreReadiness.routeSkipReasons')"
        :description="t('coreReadiness.routeSkipReasonsHint')"
        :items="routeSkipItems"
        :empty-text="t('coreReadiness.noRouteSkipReasons')"
      />

      <Card class="overflow-hidden">
        <SectionHeader
          :title="t('coreReadiness.routeSkipSamples')"
          :description="t('coreReadiness.routeSkipSamplesHint')"
        />
        <Table class="hidden lg:table">
          <TableHeader>
            <TableRow>
              <TableHead>{{ t('coreReadiness.request') }}</TableHead>
              <TableHead>Provider</TableHead>
              <TableHead>{{ t('coreReadiness.accountLabel') }}</TableHead>
              <TableHead>{{ t('coreReadiness.skipReason') }}</TableHead>
              <TableHead>{{ t('coreReadiness.recommendationLabel') }}</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            <TableRow
              v-for="item in routeSkipSamples"
              :key="`${item.request_id}-${item.provider_id || 'provider'}-${item.key_id || 'key'}-${item.reason}`"
            >
              <TableCell class="max-w-[220px]">
                <div class="truncate font-mono text-xs">
                  {{ item.request_id }}
                </div>
                <div class="text-xs text-muted-foreground">
                  {{ formatTime(item.created_at_unix_secs) }}
                </div>
              </TableCell>
              <TableCell class="text-sm">
                {{ item.provider_name || item.provider_id || t('coreReadiness.unselectedUpstream') }}
              </TableCell>
              <TableCell class="text-sm">
                {{ item.account_label || item.key_name || item.key_id || t('coreReadiness.unselectedAccount') }}
              </TableCell>
              <TableCell class="max-w-[240px] text-sm">
                <div class="font-medium">
                  {{ item.label }}
                </div>
                <div class="font-mono text-xs text-muted-foreground">
                  {{ item.reason }}
                </div>
              </TableCell>
              <TableCell class="max-w-[360px] text-sm text-muted-foreground">
                {{ item.recommended_action }}
              </TableCell>
            </TableRow>
            <TableRow v-if="routeSkipSamples.length === 0">
              <TableCell
                colspan="5"
                class="py-8 text-center text-sm text-muted-foreground"
              >
                {{ t('coreReadiness.noRouteSkipSamples') }}
              </TableCell>
            </TableRow>
          </TableBody>
        </Table>
        <div class="divide-y divide-border/60 lg:hidden">
          <div
            v-for="item in routeSkipSamples"
            :key="`${item.request_id}-${item.provider_id || 'provider'}-${item.key_id || 'key'}-${item.reason}`"
            class="space-y-2 p-4"
          >
            <div class="flex items-center justify-between gap-2">
              <span class="truncate font-mono text-xs">{{ item.request_id }}</span>
              <Badge variant="secondary">
                {{ item.label }}
              </Badge>
            </div>
            <p class="text-sm">
              {{ item.provider_name || item.provider_id || t('coreReadiness.unselectedUpstream') }} · {{ item.account_label || item.key_name || item.key_id || t('coreReadiness.unselectedAccount') }}
            </p>
            <p class="text-sm text-muted-foreground">
              {{ t('coreReadiness.recommendation', { text: item.recommended_action }) }}
            </p>
          </div>
          <div
            v-if="routeSkipSamples.length === 0"
            class="p-6 text-center text-sm text-muted-foreground"
          >
            {{ t('coreReadiness.noRouteSkipSamples') }}
          </div>
        </div>
      </Card>
    </template>
  </div>
</template>

<script setup lang="ts">
import { computed, defineComponent, h, onMounted, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'

const { t } = useI18n()
import axios from 'axios'
import {
  AlertCircle,
  CheckCircle2,
  Info,
  Loader2,
  TriangleAlert
} from 'lucide-vue-next'
import {
  Badge,
  Button,
  Card,
  Input,
  Label,
  RefreshButton,
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
  Textarea
} from '@/components/ui'
import { useToast } from '@/composables/useToast'
import {
  getNifflerRollbackDrillEvidence,
  getNifflerCoreReadiness,
  getNifflerLegacyDependencyAudit,
  listNifflerStabilityObservations,
  updateNifflerRollbackDrillEvidence,
  type NifflerCoreReadinessReport,
  type NifflerLegacyDependencyAuditReport,
  type NifflerRollbackDrillEvidencePayload,
  type NifflerRollbackDrillStatus,
  type NifflerReadinessSeverity,
  type NifflerStabilityObservation
} from '@/api/niffler-core'
import {
  STABILITY_OBSERVATION_FETCH_LIMIT,
  STABILITY_REQUIRED_PASS_DAYS,
  getStabilityGateState,
  sortStabilityObservations
} from './niffler-stability-gate'
import {
  getRollbackEvidenceHint,
  getRollbackEvidenceMissingLabels
} from './niffler-rollback-drill'

const { success: showSuccess, error: showError } = useToast()
const recentDays = ref('7')
const loading = ref(false)
const legacyAuditLoading = ref(false)
const stabilityLoading = ref(false)
const rollbackEvidenceLoading = ref(false)
const rollbackEvidenceSaving = ref(false)
const error = ref('')
const legacyAuditError = ref('')
const stabilityError = ref('')
const rollbackEvidenceError = ref('')
const report = ref<NifflerCoreReadinessReport | null>(null)
const legacyAudit = ref<NifflerLegacyDependencyAuditReport | null>(null)
const stabilityObservations = ref<NifflerStabilityObservation[]>([])
const rollbackEvidence = ref<NifflerRollbackDrillEvidencePayload | null>(null)
const rollbackEvidenceForm = ref({
  status: 'not_recorded' as NifflerRollbackDrillStatus,
  backup_reference: '',
  rollback_image_tag: '',
  drill_summary: ''
})

async function loadReadinessPage() {
  await Promise.all([
    loadReport(),
    loadStabilityObservations(),
    loadRollbackDrillEvidence()
  ])
}

async function loadReport() {
  loading.value = true
  error.value = ''
  try {
    report.value = await getNifflerCoreReadiness({
      recent_days: Number(recentDays.value)
    })
  } catch (err) {
    error.value = errorMessage(err)
  } finally {
    loading.value = false
  }
}

async function loadLegacyAudit() {
  legacyAuditLoading.value = true
  legacyAuditError.value = ''
  try {
    legacyAudit.value = await getNifflerLegacyDependencyAudit({
      offset: 0,
      limit: 50
    })
  } catch (err) {
    legacyAuditError.value = errorMessage(err)
  } finally {
    legacyAuditLoading.value = false
  }
}

async function loadStabilityObservations() {
  stabilityLoading.value = true
  stabilityError.value = ''
  try {
    const page = await listNifflerStabilityObservations({
      offset: 0,
      limit: STABILITY_OBSERVATION_FETCH_LIMIT
    })
    stabilityObservations.value = page.items
  } catch (err) {
    stabilityError.value = errorMessage(err)
  } finally {
    stabilityLoading.value = false
  }
}

async function loadRollbackDrillEvidence() {
  rollbackEvidenceLoading.value = true
  rollbackEvidenceError.value = ''
  try {
    const payload = await getNifflerRollbackDrillEvidence()
    rollbackEvidence.value = payload
    rollbackEvidenceForm.value = {
      status: normalizeRollbackDrillStatus(payload.evidence?.status || payload.status),
      backup_reference: payload.evidence?.backup_reference || '',
      rollback_image_tag: payload.evidence?.rollback_image_tag || '',
      drill_summary: payload.evidence?.drill_summary || ''
    }
  } catch (err) {
    rollbackEvidenceError.value = errorMessage(err)
  } finally {
    rollbackEvidenceLoading.value = false
  }
}

async function saveRollbackDrillEvidence() {
  if (rollbackEvidenceSubmitDisabled.value) return
  rollbackEvidenceSaving.value = true
  rollbackEvidenceError.value = ''
  try {
    await updateNifflerRollbackDrillEvidence({
      status: rollbackEvidenceForm.value.status,
      backup_reference: rollbackEvidenceForm.value.backup_reference,
      rollback_image_tag: rollbackEvidenceForm.value.rollback_image_tag,
      drill_summary: rollbackEvidenceForm.value.drill_summary
    })
    showSuccess(t('coreReadiness.evidenceSaved'))
    await Promise.all([
      loadRollbackDrillEvidence(),
      loadStabilityObservations()
    ])
  } catch (err) {
    const detail = errorMessage(err)
    rollbackEvidenceError.value = detail
    showError(detail)
  } finally {
    rollbackEvidenceSaving.value = false
  }
}

function errorMessage(err: unknown): string {
  if (axios.isAxiosError(err)) {
    const detail = err.response?.data?.detail
    if (typeof detail === 'string' && detail.trim()) {
      return detail
    }
    return err.message
  }
  return err instanceof Error ? err.message : t('coreReadiness.unknownError')
}

const shadowTableRows = computed(() => {
  return (report.value?.shadow_tables.tables ?? []).map((table) => ({
    title: table.table_name,
    value: table.exists ? t('coreReadiness.created') : t('coreReadiness.missing'),
    tone: (table.exists ? 'success' : 'danger') as Tone
  }))
})

const accountStatusRows = computed(() => {
  return Object.entries(report.value?.account_status_counts ?? {}).map(([status, count]) => ({
    title: statusLabel(status),
    value: String(count),
    tone: (status === 'available' ? 'success' : 'warning') as Tone
  }))
})

const disabledProviderItems = computed(() => {
  return (report.value?.disabled_provider_references ?? []).map((item) => ({
    title: t('coreReadiness.providerReferenceTitle', { plan: item.product_plan_name, provider: item.provider_name }),
    description: joinParts([
      t('coreReadiness.source', { value: item.source_field_label || item.source_field }),
      item.reason,
      t('coreReadiness.impact', { value: item.impact }),
      t('coreReadiness.recommendation', { text: item.recommended_action })
    ])
  }))
})

const keyResidueItems = computed(() => {
  return (report.value?.key_scope_residue ?? []).map((item) => ({
    title: item.display_name || item.account_label || item.key_name || item.key_id,
    description: joinParts([
      item.provider_name ? t('coreReadiness.provider', { value: item.provider_name }) : '',
      t('coreReadiness.configFields', { value: (item.field_labels?.length ? item.field_labels : item.residue_fields).join('、') }),
      item.reason,
      t('coreReadiness.impact', { value: item.impact }),
      t('coreReadiness.recommendation', { text: item.recommended_action })
    ])
  }))
})

const groupGapItems = computed(() => {
  return (report.value?.group_policy_gaps ?? []).map((item) => ({
    title: `${item.product_plan_name} · ${item.gap_label || item.gap_kind}`,
    description: joinParts([
      item.message,
      t('coreReadiness.impact', { value: item.impact }),
      t('coreReadiness.recommendation', { text: item.recommended_action })
    ])
  }))
})

const priceGapItems = computed(() => {
  return (report.value?.price_gaps ?? []).map((item) => ({
    title: item.provider_name ? `${item.provider_name} / ${item.model_name}` : item.model_name,
    description: joinParts([
      t('coreReadiness.scope', { value: item.scope_label || item.scope }),
      t('coreReadiness.missingFields', { fields: item.missing_fields.join('、') }),
      item.reason,
      t('coreReadiness.impact', { value: item.impact }),
      t('coreReadiness.recommendation', { text: item.recommended_action })
    ])
  }))
})

const routeSkipItems = computed(() => {
  return (report.value?.route_skip_reasons ?? []).map((item) => ({
    title: t('coreReadiness.routeSkipTitle', { label: item.label || item.reason, count: item.count }),
    description: joinParts([
      t('coreReadiness.category', { value: item.category || t('coreReadiness.uncategorized') }),
      t('coreReadiness.rawCode', { value: item.reason }),
      t('coreReadiness.impact', { value: item.impact }),
      t('coreReadiness.recommendation', { text: item.recommended_action })
    ])
  }))
})

const legacyAuditNoteItems = computed(() => {
  return (legacyAudit.value?.notes ?? []).map((note, index) => ({
    title: t('coreReadiness.noteNumber', { number: index + 1 }),
    description: note
  }))
})

const legacyUserKeyRestrictionItems = computed(() => {
  return (legacyAudit.value?.user_key_legacy_restrictions ?? []).map((item) => ({
    title: item.key_name || item.key_id,
    description: joinParts([
      item.owner_label,
      item.group_name ? t('coreReadiness.group', { value: item.group_name }) : '',
      t('coreReadiness.fields', { value: (item.field_labels.length ? item.field_labels : item.field_names).join('、') }),
      item.reason,
      t('coreReadiness.impact', { value: item.impact }),
      t('coreReadiness.recommendation', { text: item.recommended_action })
    ])
  }))
})

const legacyGroupPolicyItems = computed(() => {
  return (legacyAudit.value?.user_group_legacy_policies ?? []).map((item) => ({
    title: `${item.group_name} · ${item.field_label}`,
    description: joinParts([
      t('coreReadiness.mode', { value: legacyPolicyModeLabel(item.mode) }),
      t('coreReadiness.count', { value: item.item_count }),
      item.reason,
      t('coreReadiness.impact', { value: item.impact }),
      t('coreReadiness.recommendation', { text: item.recommended_action })
    ])
  }))
})

const legacyProviderKeyRestrictionItems = computed(() => {
  return (legacyAudit.value?.provider_key_legacy_restrictions ?? []).map((item) => ({
    title: item.display_name || item.account_label || item.key_name || item.key_id,
    description: joinParts([
      item.provider_name ? t('coreReadiness.provider', { value: item.provider_name }) : '',
      t('coreReadiness.configFields', { value: (item.field_labels.length ? item.field_labels : item.residue_fields).join('、') }),
      item.reason,
      t('coreReadiness.impact', { value: item.impact }),
      t('coreReadiness.recommendation', { text: item.recommended_action })
    ])
  }))
})

const legacyProviderModelPriceItems = computed(() => {
  return (legacyAudit.value?.provider_model_price_dependencies ?? []).map((item) => ({
    title: item.provider_name ? `${item.provider_name} / ${item.model_name}` : item.model_name,
    description: joinParts([
      item.dependency_label || item.dependency_kind,
      item.reason,
      t('coreReadiness.impact', { value: item.impact }),
      t('coreReadiness.recommendation', { text: item.recommended_action })
    ])
  }))
})

const legacyWriteEntrypointItems = computed(() => {
  return (legacyAudit.value?.legacy_write_entrypoints ?? []).map((item) => ({
    title: `${item.area} · ${item.current_status}`,
    description: joinParts([
      item.method ? t('coreReadiness.method', { value: item.method }) : '',
      t('coreReadiness.location', { value: item.path }),
      item.reason,
      t('coreReadiness.nextStep', { value: item.next_action })
    ])
  }))
})

const legacyRuntimeReadItems = computed(() => {
  return (legacyAudit.value?.runtime_read_dependencies ?? []).map((item) => ({
    title: `${item.area} · ${item.current_status}`,
    description: joinParts([
      item.label,
      t('coreReadiness.location', { value: item.path }),
      item.reason,
      t('coreReadiness.nextStep', { value: item.next_action })
    ])
  }))
})

const routeSkipSamples = computed(() => report.value?.route_skip_samples ?? [])

const latestStabilityObservation = computed(() => sortedStabilityObservations.value[0] ?? null)

const sortedStabilityObservations = computed(() => {
  return sortStabilityObservations(stabilityObservations.value)
})

const stabilityGateState = computed(() => getStabilityGateState(stabilityObservations.value))

const stabilityConsecutivePassDays = computed(() => stabilityGateState.value.consecutivePassDays)

const stabilityReadyForLegacyRemoval = computed(() => stabilityGateState.value.ready)

const stabilityGateDescription = computed(() => stabilityGateState.value.description)

const stabilityBlockerItems = computed(() => {
  return (latestStabilityObservation.value?.blocker_codes ?? []).map((code) => ({
    code,
    title: stabilityBlockerLabel(code),
    description: stabilityBlockerDescription(code)
  }))
})

const rollbackEvidenceMissingLabels = computed(() => (
  getRollbackEvidenceMissingLabels(rollbackEvidence.value)
))

const rollbackEvidenceHint = computed(() => (
  getRollbackEvidenceHint(
    rollbackEvidence.value,
    latestStabilityObservation.value?.blocker_codes ?? []
  )
))

const rollbackEvidenceSubmitDisabled = computed(() => {
  if (rollbackEvidenceSaving.value) return true
  const form = rollbackEvidenceForm.value
  if (form.status === 'passed') {
    return !form.backup_reference.trim()
      || !form.rollback_image_tag.trim()
      || !form.drill_summary.trim()
  }
  if (form.status === 'failed') {
    return !form.drill_summary.trim()
  }
  return false
})

const rollbackEvidenceStatusText = computed(() => {
  if (!rollbackEvidence.value) return t('coreReadiness.notRead')
  return rollbackEvidence.value.evidence_complete ? t('coreReadiness.evidenceComplete') : t('coreReadiness.evidenceIncomplete')
})

function joinParts(parts: Array<string | null | undefined>): string {
  return parts
    .map((part) => part?.trim())
    .filter((part): part is string => Boolean(part))
    .join('。')
}

function legacyPolicyModeLabel(mode: string): string {
  const labels: Record<string, string> = {
    inherit: t('coreReadiness.inherit'),
    unrestricted: t('coreReadiness.unrestricted'),
    specific: t('coreReadiness.specificList'),
    deny_all: t('coreReadiness.denyAll'),
    configured: t('coreReadiness.configured')
  }
  return labels[mode] ?? mode
}

function issueIcon(severity: NifflerReadinessSeverity) {
  if (severity === 'error') return AlertCircle
  if (severity === 'warning') return TriangleAlert
  return Info
}

function issueIconClass(severity: NifflerReadinessSeverity): string {
  if (severity === 'error') return 'text-destructive'
  if (severity === 'warning') return 'text-warning'
  return 'text-muted-foreground'
}

function severityLabel(severity: NifflerReadinessSeverity): string {
  if (severity === 'error') return t('coreReadiness.severityBlocking')
  if (severity === 'warning') return t('coreReadiness.severityReview')
  return t('coreReadiness.severityInfo')
}

function stabilityStatusLabel(status: string): string {
  const labels: Record<string, string> = {
    pass: t('coreReadiness.pass'),
    pending: t('coreReadiness.waitingEvidence'),
    reset_required: t('coreReadiness.resetRequired')
  }
  return labels[status] ?? status
}

function stabilityStatusTone(status: string): Tone {
  if (status === 'pass') return 'success'
  if (status === 'reset_required') return 'danger'
  return 'warning'
}

function rollbackDrillLabel(status: string): string {
  const labels: Record<string, string> = {
    passed: t('coreReadiness.recorded'),
    failed: t('coreReadiness.failed'),
    not_recorded: t('coreReadiness.notRecorded')
  }
  return labels[status] ?? status
}

function normalizeRollbackDrillStatus(status?: string): NifflerRollbackDrillStatus {
  if (status === 'passed' || status === 'failed' || status === 'not_recorded') {
    return status
  }
  return 'not_recorded'
}

function rollbackDrillTone(status: string): Tone {
  if (status === 'passed') return 'success'
  if (status === 'failed') return 'danger'
  return 'warning'
}

function stabilityBlockerLabel(code: string): string {
  const label = t(`coreReadiness.blockerLabels.${code}`)
  return label === `coreReadiness.blockerLabels.${code}` ? code : label
}

function stabilityBlockerDescription(code: string): string {
  const key = `coreReadiness.blockerDescriptions.${code}`
  const description = t(key)
  return description === key ? t('coreReadiness.unknownBlocker') : description
}

function statusLabel(status: string): string {
  const key = `coreReadiness.statusLabels.${status}`
  const label = t(key)
  return label === key ? status : label
}

function formatWindow(startUnixMs: number, endUnixMs: number): string {
  return `${formatUnixMs(startUnixMs)} - ${formatUnixMs(endUnixMs)}`
}

function formatUnixMs(unixMs: number): string {
  if (!Number.isFinite(unixMs) || unixMs <= 0) {
    return '-'
  }
  return new Date(unixMs).toLocaleString()
}

function formatTime(unixSecs: number): string {
  if (!Number.isFinite(unixSecs) || unixSecs <= 0) {
    return '-'
  }
  return new Date(unixSecs * 1000).toLocaleString()
}

function formatUsd(value?: number | null): string {
  if (typeof value !== 'number' || !Number.isFinite(value)) {
    return '-'
  }
  return `$${value.toFixed(6)}`
}

watch(recentDays, () => {
  void loadReport()
})

onMounted(() => {
  void loadReadinessPage()
})

type Tone = 'success' | 'warning' | 'danger' | 'neutral'

function toneClass(tone?: Tone): string {
  if (tone === 'success') return 'text-primary'
  if (tone === 'warning') return 'text-warning'
  if (tone === 'danger') return 'text-destructive'
  return 'text-foreground'
}

const rollbackEvidenceHintIcon = computed(() => {
  if (rollbackEvidenceHint.value.tone === 'success') return CheckCircle2
  if (rollbackEvidenceHint.value.tone === 'danger') return AlertCircle
  return TriangleAlert
})

const rollbackEvidenceHintClass = computed(() => {
  if (rollbackEvidenceHint.value.tone === 'success') {
    return 'border-primary/20 bg-primary/5'
  }
  if (rollbackEvidenceHint.value.tone === 'danger') {
    return 'border-destructive/30 bg-destructive/5'
  }
  return 'border-warning/30 bg-warning/10'
})

const MetricCard = defineComponent({
  name: 'MetricCard',
  props: {
    title: { type: String, required: true },
    value: { type: String, required: true },
    description: { type: String, required: true },
    tone: { type: String as () => Tone, default: 'neutral' }
  },
  setup(props) {
    return () => h(Card, { class: 'p-4' }, () => [
      h('p', { class: 'text-sm text-muted-foreground' }, props.title),
      h('p', { class: `mt-2 text-3xl font-semibold tabular-nums ${toneClass(props.tone)}` }, props.value),
      h('p', { class: 'mt-1 text-xs text-muted-foreground' }, props.description)
    ])
  }
})

const SectionHeader = defineComponent({
  name: 'SectionHeader',
  props: {
    title: { type: String, required: true },
    description: { type: String, required: true }
  },
  setup(props) {
    return () => h('div', { class: 'border-b border-border/60 px-5 py-4' }, [
      h('h3', { class: 'font-semibold' }, props.title),
      h('p', { class: 'mt-1 text-sm text-muted-foreground' }, props.description)
    ])
  }
})

const CompactTable = defineComponent({
  name: 'CompactTable',
  props: {
    rows: { type: Array as () => Array<{ title: string; value: string; tone?: Tone }>, required: true },
    emptyText: { type: String, required: true }
  },
  setup(props) {
    return () => h('div', { class: 'divide-y divide-border/60' }, props.rows.length
      ? props.rows.map((row) => h('div', { class: 'flex items-center justify-between gap-4 px-5 py-3 text-sm' }, [
        h('span', { class: 'truncate text-muted-foreground' }, row.title),
        h('span', { class: `font-medium ${toneClass(row.tone)}` }, row.value)
      ]))
      : h('div', { class: 'p-6 text-center text-sm text-muted-foreground' }, props.emptyText))
  }
})

const ListCard = defineComponent({
  name: 'ListCard',
  props: {
    title: { type: String, required: true },
    description: { type: String, required: true },
    items: { type: Array as () => Array<{ title: string; description: string }>, required: true },
    emptyText: { type: String, required: true }
  },
  setup(props) {
    return () => h(Card, { class: 'overflow-hidden' }, () => [
      h(SectionHeader, { title: props.title, description: props.description }),
      props.items.length
        ? h('div', { class: 'divide-y divide-border/60' }, props.items.map((item) => h('div', { class: 'space-y-1 p-5' }, [
          h('div', { class: 'flex items-center gap-2' }, [
            h('p', { class: 'font-medium' }, item.title),
            h(Badge, { variant: 'secondary' }, () => t('coreReadiness.sample'))
          ]),
          h('p', { class: 'text-sm text-muted-foreground' }, item.description)
        ])))
        : h('div', { class: 'p-6 text-center text-sm text-muted-foreground' }, props.emptyText)
    ])
  }
})
</script>
