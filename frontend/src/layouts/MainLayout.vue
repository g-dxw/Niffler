<template>
  <AppShell
    :show-notice="showAuthError"
    :main-class="mainClasses"
    :sidebar-class="sidebarClasses"
    :content-class="contentClasses"
    :mobile-sidebar-open="mobileMenuOpen"
    @close-mobile-sidebar="mobileMenuOpen = false"
  >
    <!-- GLOBAL TEXTURE (Paper Noise) -->
    <div
      class="absolute inset-0 pointer-events-none z-0 opacity-[0.03] mix-blend-multiply fixed"
      :style="{ backgroundImage: `url(\&quot;data:image/svg+xml,%3Csvg viewBox='0 0 200 200' xmlns='http://www.w3.org/2000/svg'%3E%3Cfilter id='noise'%3E%3CfeTurbulence type='fractalNoise' baseFrequency='0.8' numOctaves='3' stitchTiles='stitch'/%3E%3C/filter%3E%3Crect width='100%25' height='100%25' filter='url(%23noise)'/%3E%3C/svg%3E\&quot;)` }"
    />

    <template #notice>
      <div class="flex w-full max-w-3xl items-center justify-between rounded-3xl bg-orange-500 px-6 py-3 text-white shadow-2xl ring-1 ring-white/30">
        <div class="flex items-center gap-3">
          <AlertTriangle class="h-5 w-5" />
          <span>{{ t('console.authExpired') }}</span>
        </div>
        <Button
          variant="outline"
          size="sm"
          class="border-white/60 text-white hover:bg-white/10"
          @click="handleRelogin"
        >
          {{ t('console.relogin') }}
        </Button>
      </div>
    </template>

    <template #sidebar>
      <!-- HEADER (Brand) -->
      <div class="shrink-0 flex items-center justify-between px-6 h-20">
        <RouterLink
          to="/"
          class="flex items-center gap-3 group transition-opacity hover:opacity-80"
        >
          <HeaderLogo
            size="h-9 w-9"
            class-name="text-[#191919] dark:text-white"
          />
          <div class="flex flex-col justify-center">
            <h1 class="text-lg font-bold text-[#191919] dark:text-white leading-none">
              {{ siteName }}
            </h1>
            <span class="text-[10px] text-[#91918d] dark:text-muted-foreground leading-none mt-1.5 font-medium tracking-wide">{{ siteSubtitle }}</span>
          </div>
        </RouterLink>
        <button
          type="button"
          class="flex h-9 w-9 items-center justify-center rounded-lg text-muted-foreground hover:bg-muted/50 hover:text-foreground lg:hidden"
          :aria-label="t('layout.closeSidebar')"
          @click="mobileMenuOpen = false"
        >
          <X class="h-5 w-5" />
        </button>
      </div>

      <!-- NAVIGATION -->
      <div class="flex-1 overflow-y-auto py-2 scrollbar-none">
        <SidebarNav
          :items="navigation"
          :is-active="isNavActive"
          @prefetch="prefetchNavigationItem"
        />
      </div>

    </template>

    <template #header>
      <!-- Responsive application header: one shared header slot for desktop and mobile -->
      <header class="relative z-40">
      <!-- Mobile Header -->
      <div class="hidden">
        <div class="mx-auto max-w-7xl px-6 py-4">
          <div class="flex items-center justify-between">
            <!-- Mobile navigation trigger -->
            <button
              class="flex h-9 w-9 items-center justify-center rounded-lg text-muted-foreground transition hover:bg-muted/50 hover:text-foreground"
              :aria-label="t('layout.openNavigation')"
              @click="mobileMenuOpen = !mobileMenuOpen"
            >
              <div class="relative h-5 w-5">
                <Transition
                  enter-active-class="transition-all duration-200 ease-out"
                  enter-from-class="opacity-0 rotate-90 scale-75"
                  enter-to-class="opacity-100 rotate-0 scale-100"
                  leave-active-class="transition-all duration-150 ease-in absolute inset-0"
                  leave-from-class="opacity-100 rotate-0 scale-100"
                  leave-to-class="opacity-0 -rotate-90 scale-75"
                  mode="out-in"
                >
                  <Menu
                    v-if="!mobileMenuOpen"
                    class="h-5 w-5"
                  />
                  <X
                    v-else
                    class="h-5 w-5"
                  />
                </Transition>
              </div>
            </button>

            <!-- Mobile account summary -->
            <div class="flex min-w-0 items-center gap-2.5">
              <button
                class="flex h-9 w-9 items-center justify-center rounded-lg text-muted-foreground hover:text-foreground hover:bg-muted/50 transition"
              :title="themeMode === 'system' ? t('common.themeSystem') : themeMode === 'dark' ? t('common.themeDark') : t('common.themeLight')"
                @click="toggleDarkMode"
              >
                <SunMoon
                  v-if="themeMode === 'system'"
                  class="h-4 w-4"
                />
                <SunMedium
                  v-else-if="themeMode === 'light'"
                  class="h-4 w-4"
                />
                <Moon
                  v-else
                  class="h-4 w-4"
                />
              </button>
              <div class="flex min-w-0 items-center gap-2.5">
                <div class="flex min-w-0 flex-col items-end justify-center">
                  <RouterLink
                    to="/dashboard/settings"
                    class="max-w-24 truncate text-sm font-semibold leading-none text-[#191919] transition hover:text-[#a8533a] dark:text-white dark:hover:text-[#d4a27f]"
                  >
                    {{ authStore.user?.username || t('layout.user') }}
                  </RouterLink>
                  <RouterLink
                    to="/dashboard/wallet"
                    class="mt-1.5 flex items-center gap-1 text-[11px] leading-none text-muted-foreground transition hover:text-[#a8533a] dark:hover:text-[#d4a27f]"
                  >
                    <Wallet class="h-3 w-3 shrink-0" />
                    <span class="shrink-0">{{ t('console.accountBalance') }}</span>
                    <span class="max-w-20 truncate font-semibold tabular-nums text-foreground">{{ billingSummaryTotalLabel }}</span>
                  </RouterLink>
                </div>
                <RouterLink
                  to="/dashboard/settings"
                  class="flex h-9 w-9 shrink-0 items-center justify-center rounded-full border border-black/5 bg-[#f0f0eb] text-xs font-bold text-[#3d3929] transition hover:bg-[#e8e6df] dark:border-white/10 dark:bg-white/10 dark:text-[#d4a27f] dark:hover:bg-white/15"
                  :aria-label="t('console.personalSettings')"
                >
                  {{ authStore.user?.username?.slice(0, 2).toUpperCase() || 'U' }}
                </RouterLink>
              </div>
            </div>
          </div>
        </div>

        <!-- Mobile Dropdown Menu -->
        <Transition
          enter-active-class="transition-all duration-300 ease-out overflow-hidden"
          enter-from-class="opacity-0 max-h-0"
          enter-to-class="opacity-100 max-h-[500px]"
          leave-active-class="transition-all duration-200 ease-in overflow-hidden"
          leave-from-class="opacity-100 max-h-[500px]"
          leave-to-class="opacity-0 max-h-0"
        >
          <div
            v-if="mobileMenuOpen"
            class="hidden border-t border-[var(--shell-border)] bg-[var(--shell-glass)] backdrop-blur-xl"
          >
            <div class="mx-auto max-w-7xl px-6 py-4">
              <!-- Navigation Groups -->
              <div class="space-y-4">
                <div
                  v-for="group in navigation"
                  :key="group.title"
                >
                  <div
                    v-if="group.title"
                    class="text-[10px] font-semibold text-[#91918d] dark:text-muted-foreground uppercase tracking-wider mb-2"
                  >
                    {{ group.title }}
                  </div>
                  <div class="grid grid-cols-2 gap-2">
                    <template
                      v-for="item in group.items"
                      :key="item.href"
                    >
                      <a
                        v-if="item.external"
                        :href="item.href"
                        target="_blank"
                        rel="noopener noreferrer"
                        class="flex items-center gap-2.5 px-3 py-2.5 rounded-xl text-sm font-medium transition-all text-[#666663] dark:text-muted-foreground hover:bg-black/5 dark:hover:bg-white/5 hover:text-[#191919] dark:hover:text-white"
                        @click="mobileMenuOpen = false"
                      >
                        <component
                          :is="item.icon"
                          class="h-4 w-4 shrink-0"
                        />
                        <span class="truncate">{{ item.name }}</span>
                      </a>
                      <RouterLink
                        v-else
                        :to="item.href"
                        class="flex items-center gap-2.5 px-3 py-2.5 rounded-xl text-sm font-medium transition-all"
                        :class="isNavActive(item.href)
                          ? 'bg-[#cc785c]/10 dark:bg-[#cc785c]/20 text-[#cc785c] dark:text-[#d4a27f]'
                          : 'text-[#666663] dark:text-muted-foreground hover:bg-black/5 dark:hover:bg-white/5 hover:text-[#191919] dark:hover:text-white'"
                        @mouseenter="prefetchNavigationItem(item.href)"
                        @focus="prefetchNavigationItem(item.href)"
                        @pointerdown="prefetchNavigationItem(item.href)"
                        @click="mobileMenuOpen = false"
                      >
                        <component
                          :is="item.icon"
                          class="h-4 w-4 shrink-0"
                        />
                        <span class="truncate">{{ item.name }}</span>
                      </RouterLink>
                    </template>
                  </div>
                </div>
              </div>

              <!-- User Section -->
              <div class="mt-4 pt-4 border-t border-[#cc785c]/10 dark:border-[rgba(227,224,211,0.12)]">
                <RouterLink
                  to="/dashboard/wallet"
                  class="mb-4 block rounded-2xl border border-[#cc785c]/15 bg-white/70 p-4 text-[#3d3929] shadow-sm dark:border-white/10 dark:bg-white/[0.04] dark:text-[#d4a27f]"
                  @click="mobileMenuOpen = false"
                >
                  <div class="flex items-center justify-between gap-2">
                    <span class="text-[10px] font-semibold uppercase tracking-[0.16em] text-[#91918d] dark:text-muted-foreground">{{ t('console.totalAvailable') }}</span>
                    <span
                      v-if="billingSummaryStatusLabel"
                      class="text-[10px]"
                      :class="billingSummaryStatusClass"
                    >
                      {{ billingSummaryStatusLabel }}
                    </span>
                  </div>
                  <div class="mt-1 text-xl font-semibold tabular-nums">
                    {{ billingSummaryTotalLabel }}
                  </div>
                  <div class="mt-3 grid grid-cols-2 gap-3 text-xs text-muted-foreground">
                    <div>
                      <div>{{ t('console.walletBalance') }}</div>
                      <div class="mt-0.5 font-medium text-[#191919] tabular-nums dark:text-white">
                        {{ billingSummaryWalletLabel }}
                      </div>
                    </div>
                    <div>
                      <div>{{ t('console.packageQuota') }}</div>
                      <div class="mt-0.5 font-medium text-[#191919] tabular-nums dark:text-white">
                        {{ billingSummaryPackageLabel }}
                      </div>
                    </div>
                  </div>
                  <div class="mt-3 border-t border-[#cc785c]/10 pt-2 text-xs text-muted-foreground dark:border-white/10">
                    {{ t('console.planExpiry') }} <span class="font-medium text-[#191919] dark:text-white">{{ nearestPlanExpiryLabel }}</span>
                  </div>
                </RouterLink>
                <div class="flex items-center justify-between">
                  <div class="flex items-center gap-3 min-w-0">
                    <div class="w-8 h-8 rounded-full bg-[#f0f0eb] dark:bg-white/10 border border-black/5 flex items-center justify-center text-xs font-bold text-[#3d3929] dark:text-[#d4a27f] shrink-0">
                      {{ authStore.user?.username?.substring(0, 2).toUpperCase() }}
                    </div>
                    <div class="flex flex-col min-w-0">
                      <span class="text-sm font-semibold leading-none truncate text-[#191919] dark:text-white">{{ authStore.user?.username }}</span>
                      <span class="text-[10px] text-[#91918d] dark:text-muted-foreground leading-none mt-1">{{ currentRoleLabel }}</span>
                    </div>
                  </div>
                  <div class="flex items-center gap-1">
                    <RouterLink
                      to="/dashboard/settings"
                      class="p-2 hover:bg-muted/50 rounded-lg text-muted-foreground hover:text-foreground transition-colors"
                      @click="mobileMenuOpen = false"
                    >
                      <Settings class="w-4 h-4" />
                    </RouterLink>
                    <button
                      class="p-2 rounded-lg text-muted-foreground hover:text-red-500 transition-colors"
                      @click="handleLogout"
                    >
                      <LogOut class="w-4 h-4" />
                    </button>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </Transition>
      </div>

      <!-- Desktop Page Header -->
      <div class="sticky top-0 z-40 flex h-16 shrink-0 items-center justify-between border-b border-[#3d3929]/5 bg-[#faf9f5]/90 px-4 backdrop-blur-md dark:border-white/5 dark:bg-[#191714]/90 sm:px-6 lg:px-8">
        <div class="flex min-w-0 items-center gap-3">
          <button
            type="button"
            class="flex h-9 w-9 shrink-0 items-center justify-center rounded-lg text-muted-foreground transition hover:bg-muted/50 hover:text-foreground lg:hidden"
              :aria-label="t('layout.openNavigation')"
            @click="mobileMenuOpen = !mobileMenuOpen"
          >
            <Menu class="h-5 w-5" />
          </button>
          <div class="hidden min-w-0 items-center gap-2 text-sm text-muted-foreground lg:flex">
            <template
              v-for="(crumb, index) in breadcrumbs"
              :key="index"
            >
              <template v-if="index > 0">
                <ChevronRight class="w-3 h-3 opacity-50" />
              </template>
              <RouterLink
                v-if="crumb.href && index < breadcrumbs.length - 1"
                :to="crumb.href"
                class="hover:text-foreground transition-colors"
              >
                {{ crumb.label }}
              </RouterLink>
              <span
                v-else
                :class="index === breadcrumbs.length - 1 ? 'text-foreground font-medium' : ''"
              >
                {{ crumb.label }}
              </span>
            </template>
            <!-- 页面级操作插入点 -->
            <div id="breadcrumb-actions" />
          </div>
        </div>

        <!-- Demo Mode Badge (center) -->
        <div
          v-if="isDemo"
          class="flex items-center gap-2 px-3 py-1.5 rounded-full bg-amber-100 dark:bg-amber-900/30 text-amber-700 dark:text-amber-400 text-xs font-medium"
        >
          <AlertTriangle class="w-3.5 h-3.5" />
          <span>{{ t('console.demoMode') }}</span>
        </div>

        <div class="flex items-center gap-2">
          <!-- Page-level header actions (right side) -->
          <div
            id="header-actions-right"
            class="flex items-center"
          />
          <!-- Public shortcuts (mirrors the public site navigation) -->
          <nav
            class="mr-2 hidden items-center gap-1 border-r border-[#3d3929]/10 pr-3 text-sm dark:border-white/10 xl:flex"
            :aria-label="t('layout.quickNavigation')"
          >
            <RouterLink
              to="/"
              class="rounded-lg px-2.5 py-1.5 text-muted-foreground transition-colors hover:bg-muted/50 hover:text-foreground"
            >
              {{ t('nav.home') }}
            </RouterLink>
            <RouterLink
              to="/models"
              class="rounded-lg px-2.5 py-1.5 text-muted-foreground transition-colors hover:bg-muted/50 hover:text-foreground"
            >
              {{ t('nav.models') }}
            </RouterLink>
            <RouterLink
              to="/guide"
              class="rounded-lg px-2.5 py-1.5 text-muted-foreground transition-colors hover:bg-muted/50 hover:text-foreground"
            >
              {{ t('nav.docs') }}
            </RouterLink>
          </nav>
          <VersionButton
            v-if="isAdmin"
            :status="versionStatus"
            :loading="loadingVersionStatus"
            @refresh="handleVersionRefresh"
            @open-release="openVersionReleasePage"
          />
          <TopBarActions show-public-account />
          <!--
              <div class="mt-2 rounded-lg bg-black/[0.03] px-2.5 py-2 text-xs text-muted-foreground dark:bg-white/[0.04]">
                {{ t('console.planExpiry') }}：<span class="font-medium text-foreground">{{ nearestPlanExpiryLabel }}</span>
              </div>
              <RouterLink
                to="/dashboard/wallet"
                class="mt-2 flex items-center justify-center rounded-lg px-3 py-2 text-xs font-medium text-[#a8533a] hover:bg-[#cc785c]/10 dark:text-[#d4a27f]"
              >
                {{ t('console.viewWallet') }}
              </RouterLink>
            </div>
          </div> -->
        </div>
      </div>
      </header>
    </template>

    <RouterView />

    <Dialog
      v-model="requiredAnnouncementOpen"
      persistent
      size="lg"
      :title="t('console.requiredNotice')"
      :description="t('console.requiredNoticeHint')"
    >
      <div
        v-if="currentRequiredAnnouncement"
        class="space-y-4"
      >
        <div>
          <h3 class="text-lg font-semibold text-foreground">
            {{ currentRequiredAnnouncement.title }}
          </h3>
          <p class="mt-1 text-xs text-muted-foreground">
            {{ formatRequiredAnnouncementDate(currentRequiredAnnouncement.created_at) }}
          </p>
        </div>
        <!-- eslint-disable vue/no-v-html -->
        <div
          class="prose prose-sm dark:prose-invert max-h-[50vh] max-w-none overflow-y-auto"
          v-html="renderRequiredAnnouncement(currentRequiredAnnouncement.content)"
        />
        <!-- eslint-enable vue/no-v-html -->
      </div>
      <template #footer>
        <Button
          type="button"
          :disabled="acknowledgingRequiredAnnouncement"
          @click="acknowledgeRequiredAnnouncement"
        >
          {{ acknowledgingRequiredAnnouncement ? t('console.confirming') : t('console.confirmRead') }}
        </Button>
      </template>
    </Dialog>

    <!-- 更新提示弹窗 -->
    <UpdateDialog
      v-if="updateInfo"
      v-model="showUpdateDialog"
      :current-version="updateInfo.current_version"
      :latest-version="updateInfo.latest_version || ''"
      :release-url="updateInfo.release_url"
      :release-notes="updateInfo.release_notes"
      :published-at="updateInfo.published_at"
    />
  </AppShell>
</template>

<script setup lang="ts">
/* global __APP_VERSION__ */
import { computed, ref, watch, onMounted, onUnmounted } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { marked } from 'marked'
import { useAuthStore } from '@/stores/auth'
import { useModuleStore } from '@/stores/modules'
import { useDarkMode } from '@/composables/useDarkMode'
import { useSiteInfo } from '@/composables/useSiteInfo'
import { isDemoMode } from '@/config/demo'
import { adminApi, type CheckUpdateResponse } from '@/api/admin'
import { announcementApi, type Announcement } from '@/api/announcements'
import Button from '@/components/ui/button.vue'
import { Dialog } from '@/components/ui'
import AppShell from '@/components/layout/AppShell.vue'
import SidebarNav from '@/components/layout/SidebarNav.vue'
import HeaderLogo from '@/components/HeaderLogo.vue'
import UpdateDialog from '@/components/common/UpdateDialog.vue'
import TopBarActions from '@/components/common/TopBarActions.vue'
import VersionButton from '@/components/common/VersionButton.vue'
import { buildUpdateErrorStatus } from '@/utils/updateStatus'
import { getInfiniteCanvasUrl } from '@/utils/infiniteCanvasUrl'
import {
  Home,
  Users,
  Key,
  KeyRound,
  BarChart3,
  Cog,
  Settings,
  Activity,
  Shield,
  AlertTriangle,
  SunMedium,
  Moon,
  Gauge,
  Layers,
  FolderTree,
  Database,
  Box,
  LogOut,
  SunMoon,
  ChevronRight,
  Megaphone,
  Wallet,
  CreditCard,
  Package,
  Gift,
  Menu,
  Maximize2,
  X,
  Puzzle,
  Zap,
  FileUp,
  Server,
  SlidersHorizontal,
  type LucideIcon,
} from 'lucide-vue-next'

import GithubIcon from '@/components/icons/GithubIcon.vue'
import { BUILTIN_TOOL_BREADCRUMBS } from '@/config/builtin-tools'
import { prefetchAdminNavigationTarget } from '@/utils/adminNavigationPrefetch'
import { useBillingSummary } from '@/composables/useBillingSummary'
import { sanitizeMarkdown } from '@/utils/sanitize'

const router = useRouter()
const route = useRoute()
const authStore = useAuthStore()
const { t } = useI18n()
const moduleStore = useModuleStore()
const { themeMode, toggleDarkMode } = useDarkMode()
const { siteName, siteSubtitle } = useSiteInfo()
const isDemo = computed(() => isDemoMode())
const isAdmin = computed(() => authStore.user?.role === 'admin')

const showAuthError = ref(false)
const mobileMenuOpen = ref(false)
const requiredAnnouncements = ref<Announcement[]>([])
const acknowledgingRequiredAnnouncement = ref(false)
const {
  loading: billingSummaryLoading,
  walletError: walletBalanceError,
  planError: planEntitlementsError,
  hasError: hasBillingSummaryError,
  statusLabel: billingSummaryStatusLabel,
  totalLabel: billingSummaryTotalLabel,
  walletLabel: billingSummaryWalletLabel,
  packageLabel: billingSummaryPackageLabel,
  nearestPlanExpiryLabel,
  refresh: loadBillingSummary,
} = useBillingSummary()
const requiredAnnouncementOpen = computed({
  get: () => requiredAnnouncements.value.length > 0,
  set: (value) => {
    if (value) void loadRequiredAnnouncements()
  }
})
const currentRequiredAnnouncement = computed(() => requiredAnnouncements.value[0] ?? null)
const billingSummaryStatusClass = computed(() =>
  hasBillingSummaryError.value ? 'text-rose-500' : 'text-muted-foreground'
)

// 更新检查相关
const showUpdateDialog = ref(false)
const updateInfo = ref<CheckUpdateResponse | null>(null)
const versionStatus = ref<CheckUpdateResponse | null>(null)
const loadingVersionStatus = ref(false)
let versionStatusLoadPromise: Promise<CheckUpdateResponse | null> | null = null

// 路由变化时自动关闭移动端菜单
watch(() => route.path, () => {
  mobileMenuOpen.value = false
  if (route.path.startsWith('/dashboard/wallet') || route.path.startsWith('/dashboard/billing')) {
    void loadBillingSummary()
  }
})

async function loadVersionStatus() {
  if (!isAdmin.value) return null
  if (versionStatusLoadPromise) return versionStatusLoadPromise

  loadingVersionStatus.value = true
  versionStatusLoadPromise = (async () => {
    try {
      versionStatus.value = await adminApi.checkUpdate()
      return versionStatus.value
    } catch (error) {
      versionStatus.value = buildUpdateErrorStatus(versionStatus.value, error)
      return versionStatus.value
    } finally {
      loadingVersionStatus.value = false
      versionStatusLoadPromise = null
    }
  })()

  return versionStatusLoadPromise
}

function handleVersionRefresh() {
  void loadVersionStatus()
}

function openVersionReleasePage() {
  if (versionStatus.value?.release_url) {
    window.open(versionStatus.value.release_url, '_blank', 'noopener,noreferrer')
  }
}

function showDebugUpdateDialog() {
  const currentVersion = versionStatus.value?.current_version || __APP_VERSION__ || '0.7.0-rc28'
  updateInfo.value = {
    current_version: currentVersion,
    latest_version: 'v0.7.0-rc99',
    has_update: true,
    release_url: 'https://github.com/ryfineZ/Niffler/releases',
    release_notes: [
      "### What's Changed",
      `- ${t('layout.updateNote1')}`,
      `- ${t('layout.updateNote2')}`,
      `- ${t('layout.updateNote3')}`,
    ].join('\n'),
    published_at: new Date().toISOString(),
    error: null,
  }
  showUpdateDialog.value = true
}

function showDebugVersionStatus(hasUpdate = true) {
  const currentVersion = versionStatus.value?.current_version || __APP_VERSION__ || '0.7.0-rc28'
  versionStatus.value = {
    current_version: currentVersion,
    latest_version: hasUpdate ? 'v0.7.0-rc99' : currentVersion,
    has_update: hasUpdate,
    release_url: hasUpdate ? 'https://github.com/ryfineZ/Niffler/releases' : null,
    release_notes: hasUpdate
      ? [
        "### What's Changed",
        `- ${t('layout.updateNote1')}`,
        `- ${t('layout.updateNote2')}`,
        `- ${t('layout.updateNote3')}`,
      ].join('\n')
      : null,
    published_at: hasUpdate ? new Date().toISOString() : null,
    error: null,
  }
}

function syncAuthNotice() {
  authStore.syncToken()
  showAuthError.value = !!authStore.user && !authStore.token
}

function handleStorageChange(event: StorageEvent) {
  if (event.key === null || event.key === 'access_token') {
    syncAuthNotice()
  }
}

function handleVisibilityChange() {
  if (!document.hidden) {
    syncAuthNotice()
  }
}

watch(
  () => [authStore.user, authStore.token] as const,
  () => {
    showAuthError.value = !!authStore.user && !authStore.token
    if (authStore.user && authStore.token) {
      void loadRequiredAnnouncements()
    } else {
      requiredAnnouncements.value = []
    }
  },
  { immediate: true }
)

async function loadRequiredAnnouncements() {
  if (!authStore.user || !authStore.token) return
  try {
    const response = await announcementApi.getRequiredUnreadAnnouncements()
    requiredAnnouncements.value = response.items.filter(item => item.requires_ack && !item.is_read)
  } catch {
    requiredAnnouncements.value = []
  }
}


/*
    log.error('加载侧边栏钱包余额失败:', balanceResult.reason)
  }

  if (entitlementResult.status === 'fulfilled') {
    planEntitlements.value = entitlementResult.value.items
    planEntitlementsError.value = false
  } else {
    planEntitlementsError.value = true
    log.error('加载侧边栏套餐权益失败:', entitlementResult.reason)
  }

  billingSummaryLoading.value = false
}
*/

function renderRequiredAnnouncement(content: string): string {
  return sanitizeMarkdown(marked(content || '') as string)
}


function formatRequiredAnnouncementDate(value: string): string {
  return new Date(value).toLocaleString('zh-CN')
}

async function acknowledgeRequiredAnnouncement() {
  const announcement = currentRequiredAnnouncement.value
  if (!announcement) return
  acknowledgingRequiredAnnouncement.value = true
  try {
    await announcementApi.markAsRead(announcement.id)
    requiredAnnouncements.value = requiredAnnouncements.value.slice(1)
  } finally {
    acknowledgingRequiredAnnouncement.value = false
  }
}

onMounted(() => {
  window.addEventListener('storage', handleStorageChange)
  document.addEventListener('visibilitychange', handleVisibilityChange)
  syncAuthNotice()

  // 管理员预加载模块状态（路由守卫会按需加载，这里提前加载以避免菜单闪烁）
  if (authStore.canAccessAdmin && !moduleStore.loaded && !moduleStore.loading) {
    moduleStore.fetchModules()
  }
  void loadRequiredAnnouncements()

  if (import.meta.env.DEV) {
    window.__aetherShowUpdateDialog = showDebugUpdateDialog
    window.__aetherMockVersionStatus = showDebugVersionStatus
  }
})

onUnmounted(() => {
  window.removeEventListener('storage', handleStorageChange)
  document.removeEventListener('visibilitychange', handleVisibilityChange)
  if (import.meta.env.DEV && window.__aetherShowUpdateDialog === showDebugUpdateDialog) {
    delete window.__aetherShowUpdateDialog
  }
  if (import.meta.env.DEV && window.__aetherMockVersionStatus === showDebugVersionStatus) {
    delete window.__aetherMockVersionStatus
  }
})

async function handleRelogin() {
  showAuthError.value = false
  await authStore.logout()
  await router.push('/')
}

async function handleLogout() {
  await authStore.logout()
  await router.push('/')
}

function isNavActive(href: string) {
  if (href === '/dashboard' || href === '/admin/dashboard') {
    return route.path === href
  }
  return route.path === href || route.path.startsWith(`${href}/`)
}

function prefetchNavigationItem(href: string) {
  prefetchAdminNavigationTarget(href)
}

// Navigation Data
const infiniteCanvasCanvasUrl = getInfiniteCanvasUrl('canvas')

const navigation = computed(() => {
  const baseNavigation = [
    {
      title: t('console.nav.overview'),
      items: [
        { name: t('console.nav.dashboard'), href: '/dashboard', icon: Home },
        { name: t('console.nav.healthMonitor'), href: '/dashboard/endpoint-status', icon: Activity },
      ]
    },
    {
      title: t('console.nav.resources'),
      items: [
        { name: t('console.nav.modelCatalog'), href: '/dashboard/models', icon: Box },
        { name: t('console.nav.apiKeys'), href: '/dashboard/api-keys', icon: Key },
      ]
    },
      {
        title: t('console.nav.tools'),
        items: [
          { name: t('console.nav.imageStudio'), href: '/dashboard/image-studio', icon: Zap },
          { name: t('console.nav.infiniteCanvas'), href: infiniteCanvasCanvasUrl, icon: Maximize2, external: true },
        ]
      },
    {
      title: t('console.nav.account'),
      items: [
         { name: t('console.nav.wallet'), href: '/dashboard/wallet', icon: Wallet },
         { name: t('console.nav.billing'), href: '/dashboard/billing', icon: Package },
         { name: t('console.nav.referrals'), href: '/dashboard/referral', icon: Gift },
         { name: t('console.nav.usage'), href: '/dashboard/usage', icon: BarChart3 },
      ]
    }
  ]

  // 系统菜单项（静态部分）
  const systemItems: { name: string; href: string; icon: LucideIcon }[] = [
    { name: t('console.nav.announcements'), href: '/admin/announcements', icon: Megaphone },
    { name: t('console.nav.core'), href: '/admin/niffler-core', icon: Gauge },
    { name: t('console.nav.migration'), href: '/admin/niffler-migration', icon: Gauge },
    { name: t('console.nav.cache'), href: '/admin/cache-monitoring', icon: Gauge },
  ]

  // 动态添加已激活模块的菜单项
  // 图标映射
  const iconMap: Record<string, LucideIcon> = {
    Key,
    KeyRound,
    FileUp,
    Shield,
    Puzzle,
    Server,
    SlidersHorizontal,
  }

  // 添加模块菜单项（按 admin_menu_order 排序，只显示已激活的）
  const moduleMenuItems = Object.values(moduleStore.modules)
    .filter(m => m.active && m.admin_route && m.admin_menu_group === 'system')
    .sort((a, b) => a.admin_menu_order - b.admin_menu_order)
    .map(m => ({
      name: m.display_name,
      href: m.admin_route ?? '',
      icon: iconMap[m.admin_menu_icon || ''] || Puzzle
    }))

  systemItems.push(...moduleMenuItems)

  // 模块管理和系统设置放在最后
  systemItems.push({ name: t('console.nav.modules'), href: '/admin/modules', icon: Puzzle })
  systemItems.push({ name: t('console.nav.settings'), href: '/admin/system', icon: Cog })

  const adminNavigation = [
     {
      title: t('console.nav.overview'),
      items: [
        { name: t('console.nav.dashboard'), href: '/admin/dashboard', icon: Home },
        { name: t('console.nav.healthMonitor'), href: '/admin/health-monitor', icon: Activity },
        { name: t('console.nav.userStats'), href: '/admin/user-stats', icon: BarChart3 },
        { name: t('console.nav.costAnalysis'), href: '/admin/cost-analysis', icon: Gauge },
        { name: t('console.nav.performanceAnalysis'), href: '/admin/performance-analysis', icon: Activity },
      ]
    },
    {
      title: t('console.nav.management'),
      items: [
        { name: t('console.nav.users'), href: '/admin/users', icon: Users },
        { name: t('console.nav.upstreams'), href: '/admin/niffler-upstreams', icon: Server },
        { name: t('console.nav.productPlans'), href: '/admin/niffler-product-plans', icon: Package },
        { name: t('console.nav.errorMessages'), href: '/admin/niffler-error-messages', icon: AlertTriangle },
        { name: t('console.nav.providers'), href: '/admin/providers', icon: FolderTree },
        { name: t('console.nav.models'), href: '/admin/models', icon: Layers },
        { name: t('console.nav.routing'), href: '/admin/routing', icon: SlidersHorizontal },
        { name: t('console.nav.pool'), href: '/admin/pool', icon: Database },
        { name: t('console.nav.keys'), href: '/admin/keys', icon: Key },
        { name: t('console.nav.wallets'), href: '/admin/wallets', icon: Wallet },
        { name: t('console.nav.payment'), href: '/admin/payment-gateways', icon: CreditCard },
        { name: t('console.nav.billingPlans'), href: '/admin/billing-plans', icon: Package },
        { name: t('console.nav.referrals'), href: '/admin/referrals', icon: Gift },
        { name: t('console.nav.asyncTasks'), href: '/admin/async-tasks', icon: Zap },
        { name: t('console.nav.usage'), href: '/admin/usage', icon: BarChart3 },
      ]
    },
      {
        title: t('console.nav.tools'),
        items: [
          { name: t('console.nav.imageStudio'), href: '/admin/image-studio', icon: Zap },
          { name: t('console.nav.infiniteCanvas'), href: infiniteCanvasCanvasUrl, icon: Maximize2, external: true },
        ]
      },
    {
      title: t('console.nav.system'),
      items: systemItems
    }
  ]

  return authStore.canAccessAdmin ? adminNavigation : baseNavigation
})

const currentRoleLabel = computed(() => {
  if (authStore.isAdmin) return t('console.roles.admin')
  if (authStore.isAuditAdmin) return t('console.roles.auditAdmin')
  return t('console.roles.user')
})

// Breadcrumbs
interface BreadcrumbItem {
  label: string
  href?: string
}

const breadcrumbs = computed((): BreadcrumbItem[] => {
  // Special case: personal settings page accessed by admin
  if (route.path === '/dashboard/settings') {
    return [
      { label: t('console.nav.account') },
      { label: t('console.breadcrumbs.settings') }
    ]
  }

  // Special case: module config pages (e.g., /admin/ldap)
  if (route.meta?.module) {
    const moduleName = route.meta.module as string
    const moduleStatus = moduleStore.modules[moduleName]
    const displayName = moduleStatus?.display_name || moduleName
    return [
      { label: t('console.nav.system') },
      { label: t('console.breadcrumbs.modules'), href: '/admin/modules' },
      { label: displayName }
    ]
  }

  // Special case: built-in tools under module management
  if (BUILTIN_TOOL_BREADCRUMBS[route.path]) {
    return [
      { label: t('console.nav.system') },
      { label: t('console.breadcrumbs.modules'), href: '/admin/modules' },
      { label: BUILTIN_TOOL_BREADCRUMBS[route.path] }
    ]
  }

  // Find section and page from navigation
  for (const group of navigation.value) {
    const activeItem = group.items.find(item => isNavActive(item.href))
    if (activeItem) {
      return [
        { label: group.title || '' },
        { label: activeItem.name }
      ]
    }
  }

  // Special case: module pages not in navigation (module not active)
  // Check if current path matches a module's admin_route
  const currentModule = Object.values(moduleStore.modules).find(
    m => m.admin_route && route.path === m.admin_route
  )
  if (currentModule) {
    return [
      { label: t('console.breadcrumbs.modules'), href: '/admin/modules' },
      { label: currentModule.display_name }
    ]
  }

  return [{ label: t('console.nav.dashboard') }]
})

// Styling Classes (Editorial)
const sidebarClasses = computed(() => {
    const mobilePosition = mobileMenuOpen.value ? 'translate-x-0' : '-translate-x-full'
    return `fixed inset-y-0 left-0 z-[60] w-[280px] flex flex-col border-r border-[#3d3929]/5 bg-[#faf9f5] shadow-2xl transition-transform duration-300 ease-out dark:border-white/5 dark:bg-[#1e1c19] ${mobilePosition} lg:sticky lg:top-0 lg:z-auto lg:w-[260px] lg:translate-x-0 lg:shadow-none`
})

const contentClasses = computed(() => {
    return `flex-1 min-w-0 bg-[#faf9f5] dark:bg-[#191714] text-[#3d3929] dark:text-[#d4a27f]`
})

const mainClasses = computed(() => {
    // 统一使用 sticky Header，不再为旧的移动端 fixed Header 预留额外高度
    return `pt-4 lg:pt-6`
})

</script>

<style scoped>
.scrollbar-none::-webkit-scrollbar { display: none; }
.scrollbar-none { -ms-overflow-style: none; scrollbar-width: none; }
</style>
