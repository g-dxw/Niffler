<template>
  <div class="space-y-6 pb-8">
    <Card class="p-5 border-border/70 bg-card/95">
      <div class="flex flex-col gap-4 lg:flex-row lg:items-start lg:justify-between">
        <div class="space-y-2">
          <div class="flex flex-wrap items-center gap-2">
            <h2 class="text-xl font-semibold">
              Niffler 核心对账
            </h2>
            <Badge variant="outline">
              只读检查
            </Badge>
          </div>
          <p class="max-w-3xl text-sm text-muted-foreground">
            这里只读取旧 Provider、上游账号、分组、价格和请求记录，检查它们能否映射到新的 Niffler 核心模型。这个页面不会修改任何数据。
          </p>
        </div>
        <div class="flex items-center gap-2">
          <Select v-model="recentDays">
            <SelectTrigger class="h-9 w-32">
              <SelectValue placeholder="时间范围" />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="7">
                最近 7 天
              </SelectItem>
              <SelectItem value="30">
                最近 30 天
              </SelectItem>
              <SelectItem value="90">
                最近 90 天
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
            读取失败
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
        正在读取只读对账报告...
      </p>
    </div>

    <template v-else-if="report">
      <div class="grid gap-4 md:grid-cols-2 xl:grid-cols-5">
        <MetricCard
          title="影子表"
          :value="`${report.shadow_tables.existing_tables}/${report.shadow_tables.expected_tables}`"
          :description="report.shadow_tables.all_present ? '结构完整' : '缺少表，需要先跑迁移'"
          :tone="report.shadow_tables.all_present ? 'success' : 'danger'"
        />
        <MetricCard
          title="Provider 映射"
          :value="`${report.provider_mapping.mapped_count}/${report.provider_mapping.legacy_count}`"
          :description="`${report.provider_mapping.blocked_count} 个停用，不能进入新策略`"
          :tone="report.provider_mapping.blocked_count ? 'warning' : 'success'"
        />
        <MetricCard
          title="账号映射"
          :value="`${report.account_mapping.mapped_count}/${report.account_mapping.legacy_count}`"
          :description="`${report.account_mapping.blocked_count} 个不可直接调度`"
          :tone="report.account_mapping.blocked_count ? 'warning' : 'success'"
        />
        <MetricCard
          title="产品策略映射"
          :value="`${report.product_plan_mapping.mapped_count}/${report.product_plan_mapping.legacy_count}`"
          :description="`${report.summary.product_plans_public} 个公开，${report.summary.product_plans_total - report.summary.product_plans_public} 个内部`"
          :tone="report.product_plan_mapping.blocked_count ? 'warning' : 'success'"
        />
        <MetricCard
          title="请求记录异常"
          :value="String(report.summary.recent_problem_usage_sample_count)"
          :description="`最近 ${report.recent_days} 天样本`"
          :tone="report.summary.recent_problem_usage_sample_count ? 'warning' : 'success'"
        />
      </div>

      <Card class="overflow-hidden">
        <div class="flex flex-col gap-3 border-b border-border/60 px-5 py-4 lg:flex-row lg:items-start lg:justify-between">
          <div>
            <div class="flex flex-wrap items-center gap-2">
              <h3 class="font-semibold">
                稳定观察
              </h3>
              <Badge variant="outline">
                最近 5 条
              </Badge>
            </div>
            <p class="mt-1 text-sm text-muted-foreground">
              第 5 批第五片的上线观察结果。只有连续 14 天通过，才能继续删除旧逻辑。
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
          正在读取稳定观察...
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
          <div class="grid gap-4 md:grid-cols-2 xl:grid-cols-4">
            <MetricCard
              title="观察状态"
              :value="stabilityStatusLabel(latestStabilityObservation.status)"
              :description="formatWindow(latestStabilityObservation.window_start_unix_ms, latestStabilityObservation.window_end_unix_ms)"
              :tone="stabilityStatusTone(latestStabilityObservation.status)"
            />
            <MetricCard
              title="回滚演练"
              :value="rollbackDrillLabel(latestStabilityObservation.rollback_drill_status)"
              description="配置键 niffler_stability_rollback_drill_status"
              :tone="rollbackDrillTone(latestStabilityObservation.rollback_drill_status)"
            />
            <MetricCard
              title="未知上游"
              :value="String(latestStabilityObservation.unknown_upstream_count)"
              description="只统计已尝试上游但缺少服务或账号的记录"
              :tone="latestStabilityObservation.unknown_upstream_count ? 'danger' : 'success'"
            />
            <MetricCard
              title="对账异常"
              :value="String(latestStabilityObservation.consistency_issue_count)"
              :description="`${latestStabilityObservation.consistency_checked_count} 条检查样本`"
              :tone="latestStabilityObservation.consistency_issue_count ? 'danger' : 'success'"
            />
          </div>

          <div class="grid gap-4 xl:grid-cols-2">
            <div class="rounded-lg border border-border/60">
              <div class="border-b border-border/60 px-4 py-3">
                <p class="font-medium">
                  阻断原因
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
                当前观察没有阻断原因。
              </div>
            </div>

            <div class="rounded-lg border border-border/60">
              <div class="border-b border-border/60 px-4 py-3">
                <p class="font-medium">
                  最近观察
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
                    <p>阻断 {{ item.blocker_codes.length }}</p>
                    <p>未知上游 {{ item.unknown_upstream_count }}</p>
                  </div>
                </div>
              </div>
            </div>
          </div>

          <p class="text-xs text-muted-foreground">
            回滚演练状态只能在确认可回滚镜像、近期数据库备份和演练记录都存在后记录为 passed；本页不提供写入操作。
          </p>
        </div>
        <div
          v-else
          class="p-6 text-center text-sm text-muted-foreground"
        >
          还没有稳定观察记录。
        </div>
      </Card>

      <Card class="overflow-hidden">
        <SectionHeader
          title="需要处理的问题"
          :description="report.issues.length ? '按迁移风险汇总' : '没有发现阻塞问题'"
        />
        <div
          v-if="report.issues.length === 0"
          class="flex items-center gap-3 p-5 text-sm text-muted-foreground"
        >
          <CheckCircle2 class="h-5 w-5 text-emerald-600" />
          当前只读检查没有发现需要处理的问题。
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
                旧依赖下线稽核
              </h3>
              <Badge variant="outline">
                手动只读
              </Badge>
            </div>
            <p class="max-w-3xl text-sm text-muted-foreground">
              第 5 批第二片检查旧 Key 限制、旧分组规则、Provider Key 限制、旧价格、旧写入口冻结和旧运行时读路径。为避免后台默认打开页面时重复读取旧表，这块需要手动读取。
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
              旧依赖下线稽核
            </h3>
            <p class="mt-1 text-sm text-muted-foreground">
              第 5 批第二片已接入旧写入口冻结机制。已迁移对象的旧入口返回 409 并提示去 Niffler Core 修改；未迁移对象继续旧逻辑。
            </p>
          </div>
          <RefreshButton
            :loading="legacyAuditLoading"
            @click="loadLegacyAudit"
          />
        </div>
        <div class="grid gap-4 p-5 md:grid-cols-2 xl:grid-cols-3">
          <MetricCard
            title="独立 Key 旧限制"
            :value="String(legacyAudit.summary.user_key_restrictions_in_page)"
            :description="legacyAudit.has_more_user_keys ? '当前页有样本，后面还有独立 Key' : '当前页已读完'"
            :tone="legacyAudit.summary.user_key_restrictions_in_page ? 'warning' : 'success'"
          />
          <MetricCard
            title="分组旧规则"
            :value="String(legacyAudit.summary.user_group_policy_items)"
            description="旧分组仍表达模型、Provider、格式或倍率"
            :tone="legacyAudit.summary.user_group_policy_items ? 'warning' : 'success'"
          />
          <MetricCard
            title="Provider Key 旧限制"
            :value="String(legacyAudit.summary.provider_key_restriction_items)"
            description="旧上游账号仍保存模型、格式或优先级限制"
            :tone="legacyAudit.summary.provider_key_restriction_items ? 'warning' : 'success'"
          />
          <MetricCard
            title="旧价格依赖"
            :value="String(legacyAudit.summary.provider_model_price_dependency_items)"
            description="旧 Provider 模型价格仍可能参与成本展示"
            :tone="legacyAudit.summary.provider_model_price_dependency_items ? 'warning' : 'success'"
          />
          <MetricCard
            title="旧写入口"
            :value="String(legacyAudit.summary.legacy_write_entrypoints)"
            description="下一片需要冻结或跳转的旧入口"
            tone="warning"
          />
          <MetricCard
            title="旧读路径"
            :value="String(legacyAudit.summary.runtime_read_dependencies)"
            description="第三片需要切换的运行时读源"
            tone="warning"
          />
        </div>

        <div class="grid gap-4 border-t border-border/60 p-5 xl:grid-cols-2">
          <ListCard
            title="稽核说明"
            :description="`当前页 offset=${legacyAudit.offset}，limit=${legacyAudit.limit}`"
            :items="legacyAuditNoteItems"
            empty-text="没有额外说明"
          />
          <ListCard
            title="独立 Key 旧限制"
            description="只展示当前分页样本，不扫描普通用户 Key"
            :items="legacyUserKeyRestrictionItems"
            empty-text="当前页没有发现独立 Key 旧限制"
          />
          <ListCard
            title="用户分组旧规则"
            description="这些字段后续应迁到产品策略"
            :items="legacyGroupPolicyItems"
            empty-text="没有发现用户分组旧规则"
          />
          <ListCard
            title="Provider Key 旧限制"
            description="这些字段后续应迁到账号能力或调度策略"
            :items="legacyProviderKeyRestrictionItems"
            empty-text="没有发现 Provider Key 旧限制"
          />
          <ListCard
            title="Provider 模型价格依赖"
            description="这些价格后续应迁到基础价、成本倍率或账号成本倍率"
            :items="legacyProviderModelPriceItems"
            empty-text="没有发现旧价格依赖样本"
          />
          <ListCard
            title="旧写入口"
            description="第二片要冻结或跳转的旧管理入口"
            :items="legacyWriteEntrypointItems"
            empty-text="没有旧写入口"
          />
          <ListCard
            title="旧运行时读路径"
            description="第三片要切换到新模型的读源"
            :items="legacyRuntimeReadItems"
            empty-text="没有旧读路径"
          />
        </div>
      </Card>

      <div class="grid gap-4 xl:grid-cols-2">
        <Card class="overflow-hidden">
          <SectionHeader
            title="影子表状态"
            :description="`数据库：${report.shadow_tables.database_driver || '未配置'}`"
          />
          <CompactTable
            :rows="shadowTableRows"
            empty-text="没有影子表检查结果"
          />
        </Card>

        <Card class="overflow-hidden">
          <SectionHeader
            title="账号状态映射"
            description="按旧 Key 字段能确定的状态统计"
          />
          <CompactTable
            :rows="accountStatusRows"
            empty-text="没有账号数据"
          />
        </Card>
      </div>

      <div class="grid gap-4 xl:grid-cols-2">
        <ListCard
          title="停用 Provider 引用"
          description="这些分组仍引用了已停用的 Provider"
          :items="disabledProviderItems"
          empty-text="没有发现停用 Provider 引用"
        />
        <ListCard
          title="Key 独立限制"
          description="这些限制后续应归入账号能力或调度策略"
          :items="keyResidueItems"
          empty-text="没有发现 Key 独立限制"
        />
        <ListCard
          title="分组策略缺口"
          description="迁移为产品策略前需要确认"
          :items="groupGapItems"
          empty-text="没有发现分组策略缺口"
        />
        <ListCard
          title="价格缺口"
          description="迁移计费前需要补齐"
          :items="priceGapItems"
          empty-text="没有发现价格缺口"
        />
      </div>

      <Card class="overflow-hidden">
        <SectionHeader
          title="最近请求记录异常"
          description="只展示有限样本，不包含请求体和密钥内容"
        />
        <Table class="hidden lg:table">
          <TableHeader>
            <TableRow>
              <TableHead>请求</TableHead>
              <TableHead>模型</TableHead>
              <TableHead>Provider</TableHead>
              <TableHead>扣费快照</TableHead>
              <TableHead>状态</TableHead>
              <TableHead>判断</TableHead>
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
                  {{ item.provider_display_name || item.provider_name || '未选定上游' }}
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
                  钱包 {{ formatUsd(item.wallet_debit_usd) }}
                </div>
                <div class="tabular-nums text-xs text-muted-foreground">
                  套餐 {{ formatUsd(item.package_debit_usd) }}
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
                  建议：{{ item.recommended_action }}
                </div>
              </TableCell>
            </TableRow>
            <TableRow v-if="report.recent_usage_anomalies.length === 0">
              <TableCell
                colspan="6"
                class="py-8 text-center text-sm text-muted-foreground"
              >
                没有发现请求记录异常
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
              {{ item.model }} · {{ item.provider_display_name || item.provider_name || '未选定上游' }}
            </p>
            <p
              v-if="item.provider_account_label || item.provider_api_key_name"
              class="text-xs text-muted-foreground"
            >
              账号：{{ item.provider_account_label || item.provider_api_key_name }}
            </p>
            <p class="text-xs text-muted-foreground">
              钱包 {{ formatUsd(item.wallet_debit_usd) }} · 套餐 {{ formatUsd(item.package_debit_usd) }}
            </p>
            <p class="text-sm text-muted-foreground">
              {{ item.diagnosis }}
            </p>
            <p class="text-sm text-muted-foreground">
              建议：{{ item.recommended_action }}
            </p>
          </div>
          <div
            v-if="report.recent_usage_anomalies.length === 0"
            class="p-6 text-center text-sm text-muted-foreground"
          >
            没有发现请求记录异常
          </div>
        </div>
      </Card>

      <ListCard
        title="路由跳过原因"
        description="最近路由尝试里记录到的跳过原因"
        :items="routeSkipItems"
        empty-text="没有路由跳过原因样本"
      />

      <Card class="overflow-hidden">
        <SectionHeader
          title="路由跳过样本"
          description="展示最近被跳过的具体服务和账号，便于定位为什么没有被调度"
        />
        <Table class="hidden lg:table">
          <TableHeader>
            <TableRow>
              <TableHead>请求</TableHead>
              <TableHead>Provider</TableHead>
              <TableHead>账号</TableHead>
              <TableHead>跳过原因</TableHead>
              <TableHead>建议</TableHead>
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
                {{ item.provider_name || item.provider_id || '未选定上游' }}
              </TableCell>
              <TableCell class="text-sm">
                {{ item.account_label || item.key_name || item.key_id || '未选定账号' }}
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
                没有路由跳过样本
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
              {{ item.provider_name || item.provider_id || '未选定上游' }} · {{ item.account_label || item.key_name || item.key_id || '未选定账号' }}
            </p>
            <p class="text-sm text-muted-foreground">
              建议：{{ item.recommended_action }}
            </p>
          </div>
          <div
            v-if="routeSkipSamples.length === 0"
            class="p-6 text-center text-sm text-muted-foreground"
          >
            没有路由跳过样本
          </div>
        </div>
      </Card>
    </template>
  </div>
</template>

<script setup lang="ts">
import { computed, defineComponent, h, onMounted, ref, watch } from 'vue'
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
  Card,
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
  TableRow
} from '@/components/ui'
import {
  getNifflerCoreReadiness,
  getNifflerLegacyDependencyAudit,
  listNifflerStabilityObservations,
  type NifflerCoreReadinessReport,
  type NifflerLegacyDependencyAuditReport,
  type NifflerReadinessSeverity,
  type NifflerStabilityObservation
} from '@/api/niffler-core'

const recentDays = ref('7')
const loading = ref(false)
const legacyAuditLoading = ref(false)
const stabilityLoading = ref(false)
const error = ref('')
const legacyAuditError = ref('')
const stabilityError = ref('')
const report = ref<NifflerCoreReadinessReport | null>(null)
const legacyAudit = ref<NifflerLegacyDependencyAuditReport | null>(null)
const stabilityObservations = ref<NifflerStabilityObservation[]>([])

async function loadReadinessPage() {
  await Promise.all([
    loadReport(),
    loadStabilityObservations()
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
      limit: 5
    })
    stabilityObservations.value = page.items
  } catch (err) {
    stabilityError.value = errorMessage(err)
  } finally {
    stabilityLoading.value = false
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
  return err instanceof Error ? err.message : '未知错误'
}

const shadowTableRows = computed(() => {
  return (report.value?.shadow_tables.tables ?? []).map((table) => ({
    title: table.table_name,
    value: table.exists ? '已创建' : '缺失',
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
    title: `${item.product_plan_name} 引用了 ${item.provider_name}`,
    description: joinParts([
      `来源：${item.source_field_label || item.source_field}`,
      item.reason,
      `影响：${item.impact}`,
      `建议：${item.recommended_action}`
    ])
  }))
})

const keyResidueItems = computed(() => {
  return (report.value?.key_scope_residue ?? []).map((item) => ({
    title: item.display_name || item.account_label || item.key_name || item.key_id,
    description: joinParts([
      item.provider_name ? `Provider：${item.provider_name}` : '',
      `限制：${(item.field_labels?.length ? item.field_labels : item.residue_fields).join('、')}`,
      item.reason,
      `影响：${item.impact}`,
      `建议：${item.recommended_action}`
    ])
  }))
})

const groupGapItems = computed(() => {
  return (report.value?.group_policy_gaps ?? []).map((item) => ({
    title: `${item.product_plan_name} · ${item.gap_label || item.gap_kind}`,
    description: joinParts([
      item.message,
      `影响：${item.impact}`,
      `建议：${item.recommended_action}`
    ])
  }))
})

const priceGapItems = computed(() => {
  return (report.value?.price_gaps ?? []).map((item) => ({
    title: item.provider_name ? `${item.provider_name} / ${item.model_name}` : item.model_name,
    description: joinParts([
      `范围：${item.scope_label || item.scope}`,
      `缺少字段：${item.missing_fields.join('、')}`,
      item.reason,
      `影响：${item.impact}`,
      `建议：${item.recommended_action}`
    ])
  }))
})

const routeSkipItems = computed(() => {
  return (report.value?.route_skip_reasons ?? []).map((item) => ({
    title: `${item.label || item.reason} · ${item.count} 次`,
    description: joinParts([
      `分类：${item.category || '未归类'}`,
      `原始代码：${item.reason}`,
      `影响：${item.impact}`,
      `建议：${item.recommended_action}`
    ])
  }))
})

const legacyAuditNoteItems = computed(() => {
  return (legacyAudit.value?.notes ?? []).map((note, index) => ({
    title: `说明 ${index + 1}`,
    description: note
  }))
})

const legacyUserKeyRestrictionItems = computed(() => {
  return (legacyAudit.value?.user_key_legacy_restrictions ?? []).map((item) => ({
    title: item.key_name || item.key_id,
    description: joinParts([
      item.owner_label,
      item.group_name ? `分组：${item.group_name}` : '',
      `字段：${(item.field_labels.length ? item.field_labels : item.field_names).join('、')}`,
      item.reason,
      `影响：${item.impact}`,
      `建议：${item.recommended_action}`
    ])
  }))
})

const legacyGroupPolicyItems = computed(() => {
  return (legacyAudit.value?.user_group_legacy_policies ?? []).map((item) => ({
    title: `${item.group_name} · ${item.field_label}`,
    description: joinParts([
      `模式：${legacyPolicyModeLabel(item.mode)}`,
      `数量：${item.item_count}`,
      item.reason,
      `影响：${item.impact}`,
      `建议：${item.recommended_action}`
    ])
  }))
})

const legacyProviderKeyRestrictionItems = computed(() => {
  return (legacyAudit.value?.provider_key_legacy_restrictions ?? []).map((item) => ({
    title: item.display_name || item.account_label || item.key_name || item.key_id,
    description: joinParts([
      item.provider_name ? `Provider：${item.provider_name}` : '',
      `字段：${(item.field_labels.length ? item.field_labels : item.residue_fields).join('、')}`,
      item.reason,
      `影响：${item.impact}`,
      `建议：${item.recommended_action}`
    ])
  }))
})

const legacyProviderModelPriceItems = computed(() => {
  return (legacyAudit.value?.provider_model_price_dependencies ?? []).map((item) => ({
    title: item.provider_name ? `${item.provider_name} / ${item.model_name}` : item.model_name,
    description: joinParts([
      item.dependency_label || item.dependency_kind,
      item.reason,
      `影响：${item.impact}`,
      `建议：${item.recommended_action}`
    ])
  }))
})

const legacyWriteEntrypointItems = computed(() => {
  return (legacyAudit.value?.legacy_write_entrypoints ?? []).map((item) => ({
    title: `${item.area} · ${item.current_status}`,
    description: joinParts([
      item.method ? `方法：${item.method}` : '',
      `位置：${item.path}`,
      item.reason,
      `下一步：${item.next_action}`
    ])
  }))
})

const legacyRuntimeReadItems = computed(() => {
  return (legacyAudit.value?.runtime_read_dependencies ?? []).map((item) => ({
    title: `${item.area} · ${item.current_status}`,
    description: joinParts([
      item.label,
      `位置：${item.path}`,
      item.reason,
      `下一步：${item.next_action}`
    ])
  }))
})

const routeSkipSamples = computed(() => report.value?.route_skip_samples ?? [])

const latestStabilityObservation = computed(() => stabilityObservations.value[0] ?? null)

const stabilityBlockerItems = computed(() => {
  return (latestStabilityObservation.value?.blocker_codes ?? []).map((code) => ({
    code,
    title: stabilityBlockerLabel(code),
    description: stabilityBlockerDescription(code)
  }))
})

function joinParts(parts: Array<string | null | undefined>): string {
  return parts
    .map((part) => part?.trim())
    .filter((part): part is string => Boolean(part))
    .join('。')
}

function legacyPolicyModeLabel(mode: string): string {
  const labels: Record<string, string> = {
    inherit: '继承',
    unrestricted: '不限制',
    specific: '指定列表',
    deny_all: '全部拒绝',
    configured: '已配置'
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
  if (severity === 'warning') return 'text-amber-600'
  return 'text-muted-foreground'
}

function severityLabel(severity: NifflerReadinessSeverity): string {
  if (severity === 'error') return '阻塞'
  if (severity === 'warning') return '需确认'
  return '提示'
}

function stabilityStatusLabel(status: string): string {
  const labels: Record<string, string> = {
    pass: '通过',
    pending: '等待证据',
    reset_required: '需要重算稳定期'
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
    passed: '已记录',
    failed: '失败',
    not_recorded: '未记录'
  }
  return labels[status] ?? status
}

function rollbackDrillTone(status: string): Tone {
  if (status === 'passed') return 'success'
  if (status === 'failed') return 'danger'
  return 'warning'
}

function stabilityBlockerLabel(code: string): string {
  const labels: Record<string, string> = {
    rollback_drill_not_recorded: '还没有回滚演练记录',
    rollback_drill_failed: '回滚演练失败',
    p0_incident_recorded: '记录过 P0 事故',
    p1_incident_recorded: '记录过 P1 事故',
    incident_status_unknown: '事故状态不明确',
    legacy_write_audit_unavailable: '旧写入口审计不可读',
    request_candidate_audit_unavailable: '路由尝试审计不可读',
    consistency_sample_limit_reached: '对账样本达到读取上限',
    consistency_issue: '发现对账不一致',
    unknown_upstream: '发现未知上游记录',
    legacy_write_call: '仍有旧写入口调用',
    billing_reservation_exception: '预占计费有异常',
    referral_exception: '邀请返利有异常'
  }
  return labels[code] ?? code
}

function stabilityBlockerDescription(code: string): string {
  const descriptions: Record<string, string> = {
    rollback_drill_not_recorded: '缺少可回滚镜像、近期数据库备份或演练记录的确认。',
    rollback_drill_failed: '回滚演练失败会重置稳定期，修复演练问题后再重新观察。',
    p0_incident_recorded: '稳定窗口内出现 P0 事故，不能计入 14 天稳定期。',
    p1_incident_recorded: '稳定窗口内出现 P1 事故，不能计入 14 天稳定期。',
    incident_status_unknown: '事故状态配置不是 none、p0 或 p1，需要先修正配置。',
    legacy_write_audit_unavailable: '后台无法读取旧写入口审计，不能确认旧入口是否仍被调用。',
    request_candidate_audit_unavailable: '后台无法读取路由尝试审计，不能确认 unknown 上游是否归零。',
    consistency_sample_limit_reached: '对账样本太多，当前窗口可能没有完整读取完。',
    consistency_issue: '稳定窗口内仍有计费、结算或路由对账不一致。',
    unknown_upstream: '稳定窗口内仍有已尝试上游但服务或账号缺失的记录。',
    legacy_write_call: '稳定窗口内仍有人调用被冻结的旧写入口。',
    billing_reservation_exception: '稳定窗口内仍有预占计费异常，需要先处理。',
    referral_exception: '稳定窗口内仍有邀请返利失败记录，需要先处理。'
  }
  return descriptions[code] ?? '未识别的阻断代码，需要查看稳定观察生成逻辑。'
}

function statusLabel(status: string): string {
  const labels: Record<string, string> = {
    available: '可用',
    disabled: '停用',
    invalid: '失效',
    active: '启用'
  }
  return labels[status] ?? status
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
  if (tone === 'success') return 'text-emerald-600'
  if (tone === 'warning') return 'text-amber-600'
  if (tone === 'danger') return 'text-destructive'
  return 'text-foreground'
}

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
            h(Badge, { variant: 'secondary' }, () => '样本')
          ]),
          h('p', { class: 'text-sm text-muted-foreground' }, item.description)
        ])))
        : h('div', { class: 'p-6 text-center text-sm text-muted-foreground' }, props.emptyText)
    ])
  }
})
</script>
