<template>
  <div class="min-h-screen overflow-x-hidden bg-background text-foreground literary-grid literary-paper">
    <main class="relative z-10">
      <section class="mx-auto max-w-[1480px] px-4 pb-10 pt-8 sm:px-6 sm:pt-12 lg:px-8 lg:pb-14 lg:pt-16">
        <div class="grid overflow-hidden border border-border/80 bg-background/75 shadow-sm backdrop-blur-sm lg:grid-cols-[minmax(0,1.55fr)_minmax(340px,0.75fr)]">
          <div class="relative border-b border-border/70 p-7 sm:p-10 lg:border-b-0 lg:border-r lg:p-14">
            <div class="absolute inset-y-0 left-0 w-1 bg-primary" />
            <div class="flex items-center gap-3 text-[11px] font-bold uppercase tracking-[0.2em] text-primary">
              <span class="h-px w-8 bg-primary" />
              {{ t('home.heroEyebrow') }}
            </div>
            <h1 class="mt-7 max-w-4xl font-serif text-5xl font-semibold leading-[0.94] tracking-[-0.045em] sm:text-7xl lg:text-[5.6rem]">
              {{ t('home.heroTitleLine1') }}<br>
              <span class="text-primary">{{ t('home.heroTitleLine2') }}</span>
            </h1>
            <p class="mt-7 max-w-2xl text-base leading-8 text-muted-foreground sm:text-lg">
              {{ t('home.heroDescription') }}
            </p>
            <div class="mt-9 flex flex-col gap-3 sm:flex-row">
              <RouterLink
                :to="authStore.isAuthenticated ? dashboardPath : '/models'"
                class="inline-flex h-12 items-center justify-center gap-2 bg-primary px-6 text-sm font-semibold text-primary-foreground transition hover:bg-primary/90"
              >
                {{ authStore.isAuthenticated ? t('nav.dashboard') : t('home.exploreModels') }}
                <ArrowRight class="h-4 w-4" />
              </RouterLink>
              <RouterLink
                to="/guide"
                class="inline-flex h-12 items-center justify-center gap-2 border border-border bg-background/70 px-6 text-sm font-semibold transition hover:border-primary/50 hover:text-primary"
              >
                <BookOpen class="h-4 w-4" />
                {{ t('home.readDocs') }}
              </RouterLink>
            </div>
            <div class="mt-10 flex flex-wrap items-center gap-x-6 gap-y-3 border-t border-border/60 pt-6 text-xs text-muted-foreground">
              <span class="flex items-center gap-2"><CheckCircle2 class="h-4 w-4 text-emerald-500" />{{ t('home.openaiCompatible') }}</span>
              <span class="flex items-center gap-2"><CheckCircle2 class="h-4 w-4 text-emerald-500" />{{ t('home.noSdkChanges') }}</span>
              <span class="flex items-center gap-2"><CheckCircle2 class="h-4 w-4 text-emerald-500" />{{ t('home.selfHosted') }}</span>
            </div>
          </div>

          <aside class="bg-muted/15 p-7 sm:p-10 lg:p-12">
            <div class="flex items-center gap-3">
              <span class="h-2.5 w-2.5 bg-primary" />
              <h2 class="font-semibold">{{ t('home.integrationSteps') }}</h2>
            </div>
            <ol class="mt-7 divide-y divide-border/60 border-y border-border/60">
              <li v-for="(step, index) in integrationSteps" :key="step" class="flex gap-5 py-6">
                <span class="flex h-8 w-8 shrink-0 items-center justify-center bg-primary font-mono text-xs font-bold text-primary-foreground">
                  {{ String(index + 1).padStart(2, '0') }}
                </span>
                <p class="pt-1 text-sm leading-6 text-muted-foreground">{{ step }}</p>
              </li>
            </ol>
            <div class="mt-7 grid grid-cols-3 divide-x divide-border/60 border border-border/60 bg-background/60 text-center">
              <div class="px-2 py-4">
                <div class="font-serif text-2xl font-semibold">{{ modelTotalLabel }}</div>
                <div class="mt-1 text-[10px] uppercase tracking-wider text-muted-foreground">{{ t('home.modelsStat') }}</div>
              </div>
              <div class="px-2 py-4">
                <div class="font-serif text-2xl font-semibold">3</div>
                <div class="mt-1 text-[10px] uppercase tracking-wider text-muted-foreground">{{ t('home.protocolsStat') }}</div>
              </div>
              <div class="px-2 py-4">
                <div class="font-serif text-2xl font-semibold">1</div>
                <div class="mt-1 text-[10px] uppercase tracking-wider text-muted-foreground">{{ t('home.gatewayStat') }}</div>
              </div>
            </div>
          </aside>
        </div>

        <div class="grid border-x border-b border-border/80 bg-background/70 sm:grid-cols-2 lg:grid-cols-4">
          <article v-for="(feature, index) in heroFeatures" :key="feature.title" class="border-b border-border/70 p-6 last:border-b-0 sm:[&:nth-child(odd)]:border-r lg:border-b-0 lg:border-r lg:last:border-r-0">
            <div class="text-[10px] font-bold tracking-[0.18em] text-primary">A{{ String(index + 1).padStart(2, '0') }}</div>
            <h3 class="mt-4 font-semibold">{{ feature.title }}</h3>
            <p class="mt-2 text-sm leading-6 text-muted-foreground">{{ feature.description }}</p>
          </article>
        </div>
      </section>

      <section class="border-y border-border/70 bg-background/55 py-10">
        <div class="mx-auto max-w-[1480px] px-4 sm:px-6 lg:px-8">
          <div class="flex flex-col justify-between gap-4 sm:flex-row sm:items-end">
            <div>
              <div class="section-eyebrow">{{ t('home.modelsEyebrow') }}</div>
              <h2 class="mt-3 font-serif text-3xl font-semibold sm:text-4xl">{{ t('home.modelsTitle') }}</h2>
            </div>
            <RouterLink to="/models" class="inline-flex items-center gap-2 text-sm font-semibold text-primary hover:underline">
              {{ t('home.viewAllModels') }} <ArrowRight class="h-4 w-4" />
            </RouterLink>
          </div>

          <div class="mt-8 grid gap-3 sm:grid-cols-2 lg:grid-cols-4">
            <RouterLink
              v-for="model in featuredModels"
              :key="model.id"
              to="/models"
              class="group flex items-center gap-4 border border-border/80 bg-background/75 p-4 transition hover:-translate-y-0.5 hover:border-primary/50 hover:shadow-sm"
            >
              <span
                class="flex h-10 w-10 shrink-0 items-center justify-center rounded-full border bg-background font-serif text-lg font-semibold"
                :class="modelBadgeClass(model.name)"
                :aria-label="`${modelFamily(model.name)} model`"
              >
                <img
                  v-if="modelIcon(model.name)"
                  :src="modelIcon(model.name) || undefined"
                  :alt="`${modelFamily(model.name)} icon`"
                  class="h-5 w-5 object-contain"
                >
                <span v-else>{{ modelInitial(model.name) }}</span>
              </span>
              <div class="min-w-0">
                <div class="truncate text-sm font-semibold">{{ model.display_name || model.name }}</div>
                <div class="mt-1 truncate font-mono text-[10px] text-muted-foreground">{{ model.name }}</div>
              </div>
              <ArrowUpRight class="ml-auto h-4 w-4 shrink-0 text-muted-foreground transition group-hover:text-primary" />
            </RouterLink>
          </div>
          <div v-if="modelsLoading" class="mt-8 grid gap-3 sm:grid-cols-2 lg:grid-cols-4">
            <div v-for="index in 4" :key="index" class="h-[74px] animate-pulse border border-border/70 bg-muted/30" />
          </div>
        </div>
      </section>

      <section id="gateway" class="mx-auto max-w-[1480px] scroll-mt-24 px-4 py-16 sm:px-6 lg:px-8 lg:py-24">
        <div class="grid gap-10 lg:grid-cols-[0.82fr_1.18fr] lg:items-start">
          <div class="lg:sticky lg:top-28">
            <div class="section-eyebrow">{{ t('home.gatewayEyebrow') }}</div>
            <h2 class="mt-4 max-w-xl font-serif text-4xl font-semibold leading-tight sm:text-5xl">{{ t('home.gatewayTitle') }}</h2>
            <p class="mt-5 max-w-xl text-base leading-8 text-muted-foreground">{{ t('home.gatewayDescription') }}</p>
            <div class="mt-8 grid grid-cols-3 gap-3">
              <div v-for="metric in gatewayMetrics" :key="metric.label" class="border border-border/70 bg-background/70 p-4">
                <div class="font-serif text-2xl font-semibold text-primary">{{ metric.value }}</div>
                <div class="mt-1 text-[10px] uppercase tracking-wider text-muted-foreground">{{ metric.label }}</div>
              </div>
            </div>
          </div>

          <div class="border border-border/80 bg-background/75 p-5 shadow-sm sm:p-8">
            <div class="flex items-center justify-between border-b border-border/60 pb-5">
              <div class="flex items-center gap-3">
                <Activity class="h-5 w-5 text-primary" />
                <span class="font-semibold">{{ t('home.requestFlow') }}</span>
              </div>
              <span class="flex items-center gap-2 text-xs text-emerald-600 dark:text-emerald-400">
                <span class="h-2 w-2 animate-pulse rounded-full bg-emerald-500" />{{ t('home.live') }}
              </span>
            </div>
            <div class="mt-6 space-y-3">
              <div v-for="(layer, index) in gatewayLayers" :key="layer.title" class="grid grid-cols-[42px_minmax(0,1fr)] gap-4">
                <div class="flex flex-col items-center">
                  <span class="flex h-10 w-10 items-center justify-center border border-primary/30 bg-primary/10 text-xs font-bold text-primary">{{ index + 1 }}</span>
                  <span v-if="index < gatewayLayers.length - 1" class="my-2 h-full min-h-8 w-px bg-border" />
                </div>
                <div class="mb-3 border border-border/70 bg-muted/15 p-5">
                  <div class="flex items-center justify-between gap-4">
                    <div class="flex items-center gap-3">
                      <component :is="layer.icon" class="h-5 w-5 text-primary" />
                      <h3 class="font-semibold">{{ layer.title }}</h3>
                    </div>
                    <span class="font-mono text-[10px] text-muted-foreground">{{ layer.code }}</span>
                  </div>
                  <p class="mt-3 text-sm leading-6 text-muted-foreground">{{ layer.description }}</p>
                </div>
              </div>
            </div>
          </div>
        </div>
      </section>

      <section class="border-y border-border/70 bg-[#26231f] py-16 text-[#f7f3ea] dark:bg-[#11100e] lg:py-20">
        <div class="mx-auto grid max-w-[1480px] gap-10 px-4 sm:px-6 lg:grid-cols-[0.75fr_1.25fr] lg:px-8">
          <div>
            <div class="text-[11px] font-bold uppercase tracking-[0.2em] text-[#d4a27f]">{{ t('home.quickstartEyebrow') }}</div>
            <h2 class="mt-4 font-serif text-4xl font-semibold sm:text-5xl">{{ t('home.quickstartTitle') }}</h2>
            <p class="mt-5 max-w-lg text-sm leading-7 text-[#c9c3b4]">{{ t('home.quickstartDescription') }}</p>
            <div class="mt-8 flex flex-wrap gap-2">
              <button
                v-for="option in protocolOptions"
                :key="option.id"
                class="border px-4 py-2 text-sm font-medium transition"
                :class="activeProtocol === option.id ? 'border-[#d4a27f] bg-[#d4a27f] text-[#26231f]' : 'border-white/15 text-[#c9c3b4] hover:border-[#d4a27f]/60'"
                @click="activeProtocol = option.id"
              >
                {{ option.label }}
              </button>
            </div>
          </div>

          <div class="overflow-hidden border border-white/15 bg-[#181613] shadow-2xl">
            <div class="flex items-center justify-between border-b border-white/10 px-5 py-3">
              <div class="flex items-center gap-2">
                <span class="h-2.5 w-2.5 rounded-full bg-[#e06c5f]" />
                <span class="h-2.5 w-2.5 rounded-full bg-[#d8b45f]" />
                <span class="h-2.5 w-2.5 rounded-full bg-[#75a56b]" />
              </div>
              <button class="flex items-center gap-2 text-xs text-[#c9c3b4] transition hover:text-white" @click="copyProtocolConfig">
                <Check v-if="copied" class="h-3.5 w-3.5 text-emerald-400" />
                <Copy v-else class="h-3.5 w-3.5" />
                {{ copied ? t('common.copied') : t('common.copy') }}
              </button>
            </div>
            <pre class="min-h-[290px] overflow-x-auto p-6 text-[12px] leading-7 text-[#e8ddc5] sm:p-8 sm:text-sm"><code>{{ activeConfig }}</code></pre>
          </div>
        </div>
      </section>

      <section class="mx-auto max-w-[1480px] px-4 py-16 sm:px-6 lg:px-8 lg:py-24">
        <div class="flex flex-col items-start justify-between gap-8 border border-border/80 bg-background/75 p-8 shadow-sm sm:p-12 lg:flex-row lg:items-center">
          <div>
            <div class="section-eyebrow">{{ t('home.ctaEyebrow') }}</div>
            <h2 class="mt-3 font-serif text-3xl font-semibold sm:text-4xl">{{ t('home.ctaTitle') }}</h2>
            <p class="mt-3 max-w-2xl text-sm leading-7 text-muted-foreground">{{ t('home.ctaDescription') }}</p>
          </div>
          <div class="flex w-full shrink-0 flex-col gap-3 sm:w-auto sm:flex-row">
            <RouterLink to="/models" class="inline-flex h-12 items-center justify-center gap-2 border border-border px-5 text-sm font-semibold hover:border-primary/50 hover:text-primary">
              {{ t('home.exploreModels') }}
            </RouterLink>
            <button class="inline-flex h-12 items-center justify-center gap-2 bg-primary px-5 text-sm font-semibold text-primary-foreground" @click="openPrimaryAction">
              {{ authStore.isAuthenticated ? t('nav.dashboard') : t('home.startNow') }} <ArrowRight class="h-4 w-4" />
            </button>
          </div>
        </div>
      </section>
    </main>

  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { RouterLink, useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import {
  Activity,
  ArrowRight,
  ArrowUpRight,
  BookOpen,
  Check,
  CheckCircle2,
  Copy,
  KeyRound,
  Route,
  ShieldCheck,
} from 'lucide-vue-next'
import { useAuthStore } from '@/stores/auth'
import { useClipboard } from '@/composables/useClipboard'
import { usePublicLoginDialog } from '@/composables/usePublicLoginDialog'
import { getPublicGlobalModels, type PublicGlobalModel } from '@/api/public-models'

type ProtocolId = 'openai' | 'anthropic' | 'gemini'

const router = useRouter()
const { t } = useI18n()
const authStore = useAuthStore()
const { copyToClipboard } = useClipboard()

const { showLoginDialog } = usePublicLoginDialog()
const models = ref<PublicGlobalModel[]>([])
const modelsLoading = ref(true)
const activeProtocol = ref<ProtocolId>('openai')
const copied = ref(false)
let copyTimer: ReturnType<typeof setTimeout> | undefined

const dashboardPath = computed(() => authStore.canAccessAdmin ? '/admin/dashboard' : '/dashboard')
const modelTotalLabel = computed(() => modelsLoading.value ? '—' : `${models.value.length}+`)
const featuredModels = computed(() => {
  const groups = new Map<string, PublicGlobalModel[]>()
  for (const model of models.value) {
    const family = modelFamily(model.name)
    const group = groups.get(family) || []
    group.push(model)
    groups.set(family, group)
  }
  const selected: PublicGlobalModel[] = []
  let index = 0
  while (selected.length < 8 && selected.length < models.value.length) {
    let added = false
    for (const group of groups.values()) {
      if (group[index]) {
        selected.push(group[index])
        added = true
        if (selected.length === 8) break
      }
    }
    if (!added) break
    index += 1
  }
  return selected
})

const integrationSteps = computed(() => [
  t('home.stepCreateKey'),
  t('home.stepChooseModels'),
  t('home.stepConnect'),
])

const heroFeatures = computed(() => [
  { title: t('home.featureOneKeyTitle'), description: t('home.featureOneKeyDescription') },
  { title: t('home.featureRoutingTitle'), description: t('home.featureRoutingDescription') },
  { title: t('home.featureQuotaTitle'), description: t('home.featureQuotaDescription') },
  { title: t('home.featureTraceTitle'), description: t('home.featureTraceDescription') },
])

const gatewayMetrics = computed(() => [
  { value: '3', label: t('home.protocolsStat') },
  { value: modelTotalLabel.value, label: t('home.modelsStat') },
  { value: '100%', label: t('home.traceableStat') },
])

const gatewayLayers = computed(() => [
  { icon: KeyRound, code: 'AUTH', title: t('home.layerAuthTitle'), description: t('home.layerAuthDescription') },
  { icon: Route, code: 'ROUTE', title: t('home.layerRouteTitle'), description: t('home.layerRouteDescription') },
  { icon: ShieldCheck, code: 'TRACE', title: t('home.layerTraceTitle'), description: t('home.layerTraceDescription') },
])

const protocolOptions = computed(() => [
  { id: 'openai' as const, label: 'OpenAI' },
  { id: 'anthropic' as const, label: 'Claude' },
  { id: 'gemini' as const, label: 'Gemini' },
])

const baseUrl = 'https://niffler.org'
const protocolConfigs = computed<Record<ProtocolId, string>>(() => ({
  openai: `from openai import OpenAI\n\nclient = OpenAI(\n    api_key="YOUR_NIFFLER_KEY",\n    base_url="${baseUrl}/v1"\n)\n\nresponse = client.chat.completions.create(\n    model="gpt-5.4",\n    messages=[{"role": "user", "content": "Hello"}]\n)`,
  anthropic: `export ANTHROPIC_AUTH_TOKEN="YOUR_NIFFLER_KEY"\nexport ANTHROPIC_BASE_URL="${baseUrl}"\n\nclaude`,
  gemini: `export GEMINI_API_KEY="YOUR_NIFFLER_KEY"\nexport GOOGLE_GEMINI_BASE_URL="${baseUrl}"\nexport GEMINI_MODEL="gemini-3-pro"\n\ngemini`,
}))
const activeConfig = computed(() => protocolConfigs.value[activeProtocol.value])

function modelInitial(name: string) {
  const family = modelFamily(name)
  if (family === 'claude' || family === 'codex') return 'C'
  if (family === 'gpt' || family === 'image') return 'G'
  if (family === 'gemini') return '✦'
  if (family === 'deepseek') return 'D'
  if (family === 'qwen') return 'Q'
  if (family === 'embedding') return 'E'
  if (family === 'rerank') return 'R'
  return name.slice(0, 1).toUpperCase()
}

function modelBadgeClass(name: string) {
  const family = modelFamily(name)
  if (family === 'claude' || family === 'codex') return 'border-[#d97757]/35 bg-[#d97757]/10 text-[#c65f3d]'
  if (family === 'gpt' || family === 'image') return 'border-[#10a37f]/35 bg-[#10a37f]/10 text-[#087f63]'
  if (family === 'gemini') return 'border-[#4285f4]/35 bg-[#4285f4]/10 text-[#3574d3]'
  if (family === 'deepseek') return 'border-[#4b8bea]/35 bg-[#4b8bea]/10 text-[#3675c9]'
  if (family === 'qwen') return 'border-[#6155d9]/35 bg-[#6155d9]/10 text-[#5145bf]'
  return 'border-primary/25 bg-primary/10 text-primary'
}

function modelIcon(name: string): string | null {
  const family = modelFamily(name)
  if (family === 'claude') return '/claude-color.svg'
  if (family === 'gemini') return '/gemini-color.svg'
  if (family === 'gpt' || family === 'image' || family === 'codex') return '/openai.svg'
  if (family === 'deepseek') return '/deepseek.svg'
  if (family === 'doubao') return '/doubao.svg'
  if (family === 'glm') return '/glm.svg'
  if (family === 'grok') return '/grok.svg'
  if (family === 'kimi') return '/kimi.svg'
  if (family === 'mimo') return '/mimo.svg'
  if (family === 'minimax') return '/minimax.svg'
  if (family === 'qwen') return '/qwen.svg'
  if (family === 'wenxin') return '/wenxin.svg'
  return null
}

function modelFamily(name: string) {
  const normalized = name.toLowerCase()
  if (normalized.startsWith('claude')) return 'claude'
  if (normalized.startsWith('codex')) return 'codex'
  if (normalized.startsWith('gpt-image')) return 'image'
  if (normalized.startsWith('gpt') || normalized.startsWith('o1') || normalized.startsWith('o3')) return 'gpt'
  if (normalized.startsWith('gemini')) return 'gemini'
  if (normalized.startsWith('deepseek')) return 'deepseek'
  if (normalized.startsWith('doubao')) return 'doubao'
  if (normalized.startsWith('glm') || normalized.startsWith('chatglm') || normalized.startsWith('zhipu')) return 'glm'
  if (normalized.startsWith('grok')) return 'grok'
  if (normalized.startsWith('kimi') || normalized.startsWith('moonshot')) return 'kimi'
  if (normalized.startsWith('mimo') || normalized.startsWith('xiaomi')) return 'mimo'
  if (normalized.startsWith('minimax')) return 'minimax'
  if (normalized.startsWith('qwen')) return 'qwen'
  if (normalized.startsWith('wenxin') || normalized.startsWith('ernie') || normalized.startsWith('baidu')) return 'wenxin'
  if (normalized.includes('embedding')) return 'embedding'
  if (normalized.includes('rerank') || normalized.startsWith('bge')) return 'rerank'
  return normalized.split(/[-/:]/)[0] || normalized
}

async function loadModels() {
  modelsLoading.value = true
  try {
    const firstPage = await getPublicGlobalModels({ skip: 0, limit: 1000, is_active: true })
    const collected = [...(firstPage.models || [])]
    while (collected.length < firstPage.total) {
      const page = await getPublicGlobalModels({ skip: collected.length, limit: 1000, is_active: true })
      if (!page.models?.length) break
      const knownIds = new Set(collected.map(model => model.id))
      const additions = page.models.filter(model => !knownIds.has(model.id))
      if (!additions.length) break
      collected.push(...additions)
    }
    models.value = collected
  } catch {
    models.value = []
  } finally {
    modelsLoading.value = false
  }
}

async function copyProtocolConfig() {
  await copyToClipboard(activeConfig.value)
  copied.value = true
  if (copyTimer) clearTimeout(copyTimer)
  copyTimer = setTimeout(() => { copied.value = false }, 1600)
}

function openPrimaryAction() {
  if (authStore.isAuthenticated) void router.push(dashboardPath.value)
  else showLoginDialog.value = true
}

onMounted(loadModels)
</script>

<style scoped>
.nav-link {
  border-radius: 0.5rem;
  padding: 0.5rem 0.75rem;
  color: hsl(var(--muted-foreground));
  font-size: 0.875rem;
  font-weight: 500;
  transition: color 150ms ease, background-color 150ms ease;
}
.nav-link:hover { color: hsl(var(--foreground)); background: hsl(var(--muted) / 0.5); }
.nav-link-active { color: hsl(var(--primary)); background: hsl(var(--primary) / 0.1); }
.section-eyebrow {
  color: hsl(var(--primary));
  font-size: 0.6875rem;
  font-weight: 700;
  letter-spacing: 0.2em;
  text-transform: uppercase;
}
pre, code { font-family: var(--font-mono, ui-monospace, SFMono-Regular, Menlo, monospace); }
</style>
