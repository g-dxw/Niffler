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
            :loading="loading"
            @click="loadReport"
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
      <div class="grid gap-4 md:grid-cols-2 xl:grid-cols-4">
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
          title="请求记录异常"
          :value="String(report.summary.recent_problem_usage_sample_count)"
          :description="`最近 ${report.recent_days} 天样本`"
          :tone="report.summary.recent_problem_usage_sample_count ? 'warning' : 'success'"
        />
      </div>

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
                  {{ formatTime(item.created_at_unix_ms) }}
                </div>
              </TableCell>
              <TableCell class="text-sm">
                {{ item.model }}
              </TableCell>
              <TableCell class="text-sm">
                {{ item.provider_name || 'unknown' }}
              </TableCell>
              <TableCell>
                <Badge variant="outline">
                  {{ item.status }} / {{ item.billing_status }}
                </Badge>
              </TableCell>
              <TableCell class="max-w-[360px] text-sm text-muted-foreground">
                {{ item.diagnosis }}
              </TableCell>
            </TableRow>
            <TableRow v-if="report.recent_usage_anomalies.length === 0">
              <TableCell
                colspan="5"
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
              {{ item.model }} · {{ item.provider_name || 'unknown' }}
            </p>
            <p class="text-sm text-muted-foreground">
              {{ item.diagnosis }}
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
  type NifflerCoreReadinessReport,
  type NifflerReadinessSeverity
} from '@/api/niffler-core'

const recentDays = ref('7')
const loading = ref(false)
const error = ref('')
const report = ref<NifflerCoreReadinessReport | null>(null)

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
    tone: table.exists ? 'success' : 'danger'
  }))
})

const accountStatusRows = computed(() => {
  return Object.entries(report.value?.account_status_counts ?? {}).map(([status, count]) => ({
    title: statusLabel(status),
    value: String(count),
    tone: status === 'available' ? 'success' : 'warning'
  }))
})

const disabledProviderItems = computed(() => {
  return (report.value?.disabled_provider_references ?? []).map((item) => ({
    title: `${item.routing_group_name} 引用了 ${item.provider_name}`,
    description: `来源字段：${item.source_field}`
  }))
})

const keyResidueItems = computed(() => {
  return (report.value?.key_scope_residue ?? []).map((item) => ({
    title: item.key_name || item.key_id,
    description: `${item.residue_fields.join('、')}。${item.impact}`
  }))
})

const groupGapItems = computed(() => {
  return (report.value?.group_policy_gaps ?? []).map((item) => ({
    title: item.routing_group_name,
    description: item.message
  }))
})

const priceGapItems = computed(() => {
  return (report.value?.price_gaps ?? []).map((item) => ({
    title: item.provider_name ? `${item.provider_name} / ${item.model_name}` : item.model_name,
    description: `缺少字段：${item.missing_fields.join('、')}`
  }))
})

const routeSkipItems = computed(() => {
  return (report.value?.route_skip_reasons ?? []).map((item) => ({
    title: item.reason,
    description: `${item.count} 次`
  }))
})

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

function statusLabel(status: string): string {
  const labels: Record<string, string> = {
    available: '可用',
    disabled: '停用',
    invalid: '失效',
    active: '启用'
  }
  return labels[status] ?? status
}

function formatTime(unixMs: number): string {
  if (!Number.isFinite(unixMs) || unixMs <= 0) {
    return '-'
  }
  return new Date(unixMs).toLocaleString()
}

watch(recentDays, () => {
  void loadReport()
})

onMounted(() => {
  void loadReport()
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
