<template>
  <PageContainer>
    <PageHeader
      :title="t('routingProfiles.title')"
      :description="t('routingProfiles.description')"
    >
      <template #actions>
        <Button
          variant="outline"
          :disabled="loading"
          @click="refreshPage"
        >
          <RefreshCw
            class="mr-2 h-4 w-4"
            :class="{ 'animate-spin': loading || loadingGlobalModels }"
          />
          {{ t('routingProfiles.refresh') }}
        </Button>
        <Button @click="startCreate">
          <Plus class="mr-2 h-4 w-4" />
          {{ t('routingProfiles.create') }}
        </Button>
      </template>
    </PageHeader>

    <div class="mt-6 grid gap-5 xl:grid-cols-[320px_minmax(0,1fr)]">
      <Card class="overflow-hidden">
        <div class="border-b border-border/60 px-5 py-4">
          <div class="flex items-center justify-between gap-3">
            <div>
              <h2 class="text-sm font-semibold">
                {{ t('routingProfiles.groups') }}
              </h2>
              <p class="mt-1 text-xs text-muted-foreground">
                {{ t('routingProfiles.total') }} {{ groups.length }}
              </p>
            </div>
            <SlidersHorizontal class="h-4 w-4 text-muted-foreground" />
          </div>
        </div>

        <div class="max-h-[calc(100vh-18rem)] overflow-y-auto p-3">
          <div
            v-if="loading"
            class="py-10 text-center text-sm text-muted-foreground"
          >
            {{ t('routingProfiles.loading') }}
          </div>
          <div
            v-else-if="groups.length === 0"
            class="rounded-lg border border-dashed border-border/70 px-4 py-8 text-center"
          >
            <p class="text-sm font-medium">
              {{ t('routingProfiles.empty') }}
            </p>
            <p class="mt-1 text-xs text-muted-foreground">
              {{ t('routingProfiles.emptyHint') }}
            </p>
          </div>
          <button
            v-for="group in groups"
            v-else
            :key="group.id"
            type="button"
            class="mb-2 w-full rounded-lg border px-4 py-3 text-left transition-colors"
            :class="group.id === selectedGroupId
              ? 'border-primary/60 bg-primary/10'
              : 'border-border/60 bg-background hover:border-primary/40 hover:bg-muted/50'"
            @click="selectGroup(group)"
          >
            <div class="flex items-start justify-between gap-3">
              <div class="min-w-0">
                <p class="truncate text-sm font-medium">
                  {{ group.name }}
                </p>
                <p class="mt-1 line-clamp-2 text-xs text-muted-foreground">
                  {{ group.description || t('routingProfiles.unconfigured') }}
                </p>
              </div>
              <Badge
                :variant="group.enabled ? 'default' : 'secondary'"
                class="shrink-0"
              >
                {{ group.enabled ? t('routingProfiles.enabled') : t('routingProfiles.disabled') }}
              </Badge>
            </div>
            <div class="mt-3 flex flex-wrap items-center gap-2 text-xs text-muted-foreground">
              <span>v{{ group.version }}</span>
              <span v-if="group.is_system_default">{{ t('routingProfiles.systemDefault') }}</span>
              <span>{{ group.config_json.allowed_models.length || t('routingProfiles.allModels') }} {{ t('routingProfiles.modelScope') }}</span>
            </div>
          </button>
        </div>
      </Card>

      <Card
        v-if="draft"
        class="overflow-hidden"
      >
        <div class="border-b border-border/60 px-5 py-4">
          <div class="flex flex-col gap-3 lg:flex-row lg:items-start lg:justify-between">
            <div>
              <div class="flex flex-wrap items-center gap-2">
                <h2 class="text-base font-semibold">
                  {{ isCreating ? t('routingProfiles.create') : draft.name || t('routingProfiles.unnamed') }}
                </h2>
                <Badge
                  v-if="!isCreating"
                  variant="outline"
                >
                  v{{ draft.version }}
                </Badge>
                <Badge
                  v-if="draft.is_system_default"
                  variant="secondary"
                >
                  {{ t('routingProfiles.systemDefault') }}
                </Badge>
              </div>
              <p class="mt-1 text-xs text-muted-foreground">
                {{ t('routingProfiles.updated') }} {{ formatUnixSeconds(draft.updated_at) }}
              </p>
            </div>
            <div class="flex flex-wrap items-center gap-2">
              <Button
                variant="outline"
                size="sm"
                :class="draft.is_system_default ? 'border-primary/50 bg-primary/10 text-primary' : ''"
                @click="draft.is_system_default = !draft.is_system_default"
              >
                <Star class="mr-2 h-4 w-4" />
                {{ draft.is_system_default ? t('routingProfiles.systemDefault') : t('routingProfiles.setDefault') }}
              </Button>
              <Button
                variant="outline"
                size="sm"
                :disabled="!canSaveDraft"
                @click="saveDraft"
              >
                <Save
                  class="mr-2 h-4 w-4"
                  :class="{ 'animate-pulse': saving }"
                />
                {{ t('routingProfiles.save') }}
              </Button>
              <Button
                v-if="!isCreating"
                variant="destructive"
                size="sm"
                :disabled="deleting"
                @click="deleteDraft"
              >
                <Trash2 class="mr-2 h-4 w-4" />
                {{ t('routingProfiles.delete') }}
              </Button>
            </div>
          </div>
        </div>

        <div class="space-y-6 p-5">
          <div class="grid gap-3 lg:grid-cols-[240px_minmax(0,1fr)_160px]">
            <label class="space-y-1 text-sm">
              <span class="text-muted-foreground">{{ t('routingProfiles.name') }}</span>
              <Input
                v-model="draft.name"
                :placeholder="t('routingProfiles.unnamed')"
              />
            </label>
            <label class="space-y-1 text-sm">
              <span class="text-muted-foreground">{{ t('routingProfiles.descriptionLabel') }}</span>
              <Input
                v-model="draft.description"
                :placeholder="t('routingProfiles.description')"
              />
            </label>
            <div class="flex items-center justify-between gap-3 rounded-lg border border-border/60 px-3 py-2 text-sm">
              <span>{{ t('routingProfiles.enabledLabel') }}</span>
              <Switch v-model="draft.enabled" />
            </div>
          </div>

          <ManagedInstructionsConfigSection
            :model-value="draft.config_json.managed_instructions ?? null"
            @update:model-value="updateManagedInstructions"
          />

          <section class="space-y-3 rounded-lg border border-border/60 p-4">
            <div>
              <h3 class="text-sm font-medium">
                {{ t('routingProfiles.sortScope') }}
              </h3>
              <p class="mt-1 text-xs text-muted-foreground">
                {{ t('routingProfiles.sortHint') }}
              </p>
            </div>
            <div class="grid grid-cols-2 gap-1 rounded-lg bg-muted/40 p-1 sm:max-w-[320px]">
              <button
                type="button"
                class="h-9 rounded-md px-3 text-sm font-medium transition-colors"
                :class="sortingScope === 'unified'
                  ? 'bg-background text-foreground shadow-sm'
                  : 'text-muted-foreground hover:bg-background/60 hover:text-foreground'"
                @click="setSortingScope('unified')"
              >
                {{ t('routingProfiles.unifiedSort') }}
              </button>
              <button
                type="button"
                class="h-9 rounded-md px-3 text-sm font-medium transition-colors"
                :class="sortingScope === 'per_model'
                  ? 'bg-background text-foreground shadow-sm'
                  : 'text-muted-foreground hover:bg-background/60 hover:text-foreground'"
                @click="setSortingScope('per_model')"
              >
                {{ t('routingProfiles.perModelSort') }}
              </button>
            </div>
          </section>

          <section
            v-if="sortingScope === 'unified'"
            class="space-y-4"
          >
            <div class="space-y-3 rounded-lg border border-border/60 p-4">
              <div>
                <h3 class="text-sm font-medium">
                  {{ t('routingProfiles.priorityRoutingTitle') }}
                </h3>
                <p class="mt-1 text-xs text-muted-foreground">
                  {{ t('routingProfiles.unifiedScopeHint') }}
                </p>
              </div>
              <div class="grid gap-3 lg:grid-cols-2">
                <div class="space-y-1 text-sm">
                  <span class="text-muted-foreground">{{ t('routingProfiles.priority') }}</span>
                  <div class="grid grid-cols-2 gap-1 rounded-lg bg-muted/40 p-1">
                    <button
                      type="button"
                      class="flex h-9 items-center justify-center gap-2 rounded-md px-3 text-sm font-medium transition-colors"
                      :class="firstStepPriorityMode === 'provider'
                        ? 'bg-background text-foreground shadow-sm'
                        : 'text-muted-foreground hover:bg-background/60 hover:text-foreground'"
                      @click="updateFirstStepPriorityMode('provider')"
                    >
                      <Layers class="h-4 w-4" />
                      Provider
                    </button>
                    <button
                      type="button"
                      class="flex h-9 items-center justify-center gap-2 rounded-md px-3 text-sm font-medium transition-colors"
                      :class="firstStepPriorityMode === 'global_key'
                        ? 'bg-background text-foreground shadow-sm'
                        : 'text-muted-foreground hover:bg-background/60 hover:text-foreground'"
                      @click="updateFirstStepPriorityMode('global_key')"
                    >
                      <Key class="h-4 w-4" />
                      Key
                    </button>
                  </div>
                </div>

                <div class="space-y-1 text-sm">
                  <span class="text-muted-foreground">{{ t('routingProfiles.routing') }}</span>
                  <div class="grid grid-cols-3 gap-1 rounded-lg bg-muted/40 p-1">
                    <button
                      v-for="mode in schedulingModes"
                      :key="mode.value"
                      type="button"
                      class="h-9 rounded-md px-3 text-sm font-medium transition-colors"
                      :class="firstStepSchedulingMode === mode.value
                        ? 'bg-background text-foreground shadow-sm'
                        : 'text-muted-foreground hover:bg-background/60 hover:text-foreground'"
                      @click="updateFirstStepSchedulingMode(mode.value)"
                    >
                      {{ mode.label }}
                    </button>
                  </div>
                </div>
              </div>
            </div>

            <RoutingPriorityPolicyEditor
              :config="draft.config_json"
              :model="DEFAULT_ROUTING_POLICY_MODEL"
              :show-priority-mode="false"
              :show-scheduling-mode="false"
              :subtitle="t('routingProfiles.unifiedScopeHint')"
              @update:config="updateDraftConfig"
            />
          </section>

          <section
            v-else
            class="space-y-4"
          >
            <div class="grid gap-4 lg:grid-cols-[320px_minmax(0,1fr)]">
              <div class="flex flex-col gap-3 rounded-lg border border-border/60 p-3">
                <div class="flex items-center justify-between gap-3">
                  <span class="text-sm font-medium">{{ t('routingProfiles.globalModels') }}</span>
                  <Badge variant="outline">
                    {{ filteredGlobalModels.length }}
                  </Badge>
                </div>
                <Input
                  v-model="globalModelSearch"
                  :placeholder="t('routingProfiles.searchModel')"
                />
                <div class="grid grid-cols-3 gap-1 rounded-lg bg-muted/40 p-1">
                  <button
                    v-for="filter in modelFilterOptions"
                    :key="filter.value"
                    type="button"
                    class="h-7 rounded-md px-2 text-xs font-medium transition-colors"
                    :class="modelFilter === filter.value
                      ? 'bg-background text-foreground shadow-sm'
                      : 'text-muted-foreground hover:bg-background/60 hover:text-foreground'"
                    @click="modelFilter = filter.value"
                  >
                    {{ filter.label }}
                  </button>
                </div>
                <div
                  v-if="loadingGlobalModels"
                  class="rounded-md border border-dashed border-border/70 px-3 py-4 text-center text-xs text-muted-foreground"
                >
                  {{ t('routingProfiles.loadingModels') }}
                </div>
                <div
                  v-else-if="globalModelsError"
                  class="rounded-md border border-destructive/30 bg-destructive/5 px-3 py-2 text-xs text-destructive"
                >
                  {{ globalModelsError }}
                </div>
                <div
                  v-else-if="globalModels.length === 0"
                  class="rounded-md border border-dashed border-border/70 px-3 py-4 text-center text-xs text-muted-foreground"
                >
                  {{ t('routingProfiles.noModels') }}
                </div>
                <div
                  v-else-if="filteredGlobalModels.length === 0"
                  class="rounded-md border border-dashed border-border/70 px-3 py-4 text-center text-xs text-muted-foreground"
                >
                  {{ t('routingProfiles.noMatch') }}
                </div>
                <div
                  v-else
                  class="max-h-[640px] overflow-y-auto"
                >
                  <button
                    v-for="model in filteredGlobalModels"
                    :key="model.id"
                    type="button"
                    class="mb-1 flex w-full items-center gap-3 rounded-md px-3 py-2 text-left text-sm transition-colors"
                    :class="activePerModelPolicy?.model === model.name
                      ? 'bg-primary/10 text-foreground'
                      : 'hover:bg-muted/60'"
                    @click="selectGlobalModel(model.name)"
                  >
                    <span
                      class="h-2 w-2 shrink-0 rounded-full"
                      :class="hasModelPolicy(model.name)
                        ? 'bg-primary'
                        : 'bg-muted-foreground/20'"
                      :title="hasModelPolicy(model.name) ? t('routingProfiles.configured') : t('routingProfiles.unconfigured')"
                    />
                    <span class="min-w-0 flex-1">
                      <span class="block truncate font-medium">{{ model.display_name || model.name }}</span>
                      <span class="block truncate text-xs text-muted-foreground">{{ model.name }}</span>
                    </span>
                  </button>
                </div>
              </div>

              <div class="min-w-0 rounded-lg border border-border/60 p-4">
                <template v-if="activePerModelPolicy">
                  <div class="mb-4 flex flex-col gap-3 md:flex-row md:items-center md:justify-between">
                    <div class="min-w-0 space-y-1 text-sm">
                      <span class="text-muted-foreground">{{ t('routingProfiles.currentModel') }}</span>
                      <div class="truncate text-sm font-medium">
                        {{ globalModelLabel(activePerModelPolicy.model) }}
                      </div>
                      <div class="truncate text-xs text-muted-foreground">
                        {{ activePerModelPolicy.model }}
                      </div>
                    </div>
                    <div class="flex flex-wrap items-center gap-2">
                      <DropdownMenu>
                        <DropdownMenuTrigger as-child>
                          <Button
                            type="button"
                            variant="outline"
                            size="sm"
                            :disabled="copySourceCandidates.length === 0"
                          >
                            <Copy class="mr-2 h-4 w-4" />
                            {{ t('routingProfiles.loadConfig') }}
                          </Button>
                        </DropdownMenuTrigger>
                        <DropdownMenuContent
                          align="end"
                          class="max-h-[320px] overflow-y-auto"
                        >
                          <DropdownMenuItem
                            v-for="source in copySourceCandidates"
                            :key="source.model"
                            @select="copyModelConfig(source.model)"
                          >
                            <span class="min-w-0">
                              <span class="block truncate text-sm font-medium">{{ source.label }}</span>
                              <span class="block truncate text-xs text-muted-foreground">{{ source.model }}</span>
                            </span>
                          </DropdownMenuItem>
                        </DropdownMenuContent>
                      </DropdownMenu>
                      <Button
                        type="button"
                        variant="outline"
                        size="sm"
                        :disabled="!canSaveCurrentModel"
                        @click="saveCurrentModel"
                      >
                        <Save
                          class="mr-2 h-4 w-4"
                        />
                        {{ t('routingProfiles.saveDraftModel') }}
                      </Button>
                      <Button
                        v-if="hasModelPolicy(activePerModelPolicy.model)"
                        type="button"
                        :variant="canRemoveCurrentModel ? 'destructive' : 'outline'"
                        size="sm"
                        :class="canRemoveCurrentModel ? 'shadow-sm' : 'text-muted-foreground'"
                        :disabled="!canRemoveCurrentModel"
                        :title="canRemoveCurrentModel ? t('routingProfiles.removeCurrentTitle') : t('routingProfiles.removeUnsavedTitle')"
                        @click="removePerModelPolicy(activePerModelPolicy.model)"
                      >
                        <Trash2 class="mr-2 h-4 w-4" />
                        {{ t('routingProfiles.remove') }}
                      </Button>
                    </div>
                  </div>

                  <div class="mb-4 space-y-3 rounded-lg border border-border/60 p-4">
                    <div>
                      <h3 class="text-sm font-medium">
                        {{ t('routingProfiles.priorityRoutingTitle') }}
                      </h3>
                      <p class="mt-1 text-xs text-muted-foreground">
                        {{ t('routingProfiles.currentModelScopeHint') }}
                      </p>
                    </div>
                    <div class="grid gap-3 lg:grid-cols-2">
                      <div class="space-y-1 text-sm">
                        <span class="text-muted-foreground">{{ t('routingProfiles.priority') }}</span>
                        <div class="grid grid-cols-2 gap-1 rounded-lg bg-muted/40 p-1">
                          <button
                            type="button"
                            class="flex h-9 items-center justify-center gap-2 rounded-md px-3 text-sm font-medium transition-colors"
                            :class="modelPriorityMode(activePerModelPolicy.model) === 'provider'
                              ? 'bg-background text-foreground shadow-sm'
                              : 'text-muted-foreground hover:bg-background/60 hover:text-foreground'"
                            @click="updateModelPriorityMode(activePerModelPolicy.model, 'provider')"
                          >
                            <Layers class="h-4 w-4" />
                            Provider
                          </button>
                          <button
                            type="button"
                            class="flex h-9 items-center justify-center gap-2 rounded-md px-3 text-sm font-medium transition-colors"
                            :class="modelPriorityMode(activePerModelPolicy.model) === 'global_key'
                              ? 'bg-background text-foreground shadow-sm'
                              : 'text-muted-foreground hover:bg-background/60 hover:text-foreground'"
                            @click="updateModelPriorityMode(activePerModelPolicy.model, 'global_key')"
                          >
                            <Key class="h-4 w-4" />
                            Key
                          </button>
                        </div>
                      </div>

                      <div class="space-y-1 text-sm">
                        <span class="text-muted-foreground">{{ t('routingProfiles.routing') }}</span>
                        <div class="grid grid-cols-3 gap-1 rounded-lg bg-muted/40 p-1">
                          <button
                            v-for="mode in schedulingModes"
                            :key="mode.value"
                            type="button"
                            class="h-9 rounded-md px-3 text-sm font-medium transition-colors"
                            :class="modelSchedulingMode(activePerModelPolicy.model) === mode.value
                              ? 'bg-background text-foreground shadow-sm'
                              : 'text-muted-foreground hover:bg-background/60 hover:text-foreground'"
                            @click="updateModelSchedulingMode(activePerModelPolicy.model, mode.value)"
                          >
                            {{ mode.label }}
                          </button>
                        </div>
                      </div>
                    </div>
                  </div>

                  <RoutingPriorityPolicyEditor
                    :config="activeConfigForReading"
                    :model="activePerModelName"
                    :priority-mode="modelPriorityMode(activePerModelName)"
                    :scheduling-mode="modelSchedulingMode(activePerModelName)"
                    :show-priority-mode="false"
                    :show-scheduling-mode="false"
                    :subtitle="t('routingProfiles.modelScopeHint', { model: activePerModelName })"
                    @update:config="updateEditingConfig"
                    @update:priority-mode="mode => updateModelPriorityMode(activePerModelName, mode)"
                    @update:scheduling-mode="mode => updateModelSchedulingMode(activePerModelName, mode)"
                  />
                </template>
                <div
                  v-else
                  class="rounded-lg border border-dashed border-border/70 px-4 py-8 text-center text-sm text-muted-foreground"
                >
                  {{ t('routingProfiles.addModelHint') }}
                </div>
              </div>
            </div>
          </section>
        </div>
      </Card>

      <Card
        v-else
        class="flex min-h-[360px] items-center justify-center p-8 text-center"
      >
        <div>
          <SlidersHorizontal class="mx-auto h-8 w-8 text-muted-foreground" />
          <p class="mt-3 text-sm font-medium">
            {{ t('routingProfiles.choosePolicy') }}
          </p>
        </div>
      </Card>
    </div>

    <AlertDialog
      v-model="switchModelDialogOpen"
      type="warning"
      :title="t('routingProfiles.switchModel')"
      :description="t('routingProfiles.switchModelDescription')"
      :confirm-text="t('routingProfiles.continue')"
      @confirm="confirmSwitchModel"
      @cancel="cancelSwitchModel"
    />

    <AlertDialog
      v-model="deleteDialogOpen"
      type="destructive"
      :title="t('routingProfiles.deleteTitle')"
      :description="t('routingProfiles.deleteDescription', { name: draft?.name ?? '' })"
      :confirm-text="t('routingProfiles.delete')"
      :loading="deleting"
      @confirm="confirmDeleteDraft"
    />
  </PageContainer>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useI18n } from 'vue-i18n'

const { t, locale } = useI18n()
import { Copy, Key, Layers, Plus, RefreshCw, Save, SlidersHorizontal, Star, Trash2 } from 'lucide-vue-next'

import { PageContainer, PageHeader } from '@/components/layout'
import { Badge, Button, Card, Input, Switch } from '@/components/ui'
import { DropdownMenu, DropdownMenuTrigger, DropdownMenuContent, DropdownMenuItem } from '@/components/ui/dropdown-menu'
import { AlertDialog } from '@/components/common'
import {
  DEFAULT_ROUTING_POLICY_MODEL,
  createEmptyModelPolicy,
  createEmptyRoutingGroupConfig,
  getModelPolicy,
  getModelScheduling,
  isGeneratedModelSchedulingRule,
  modelSchedulingRuleId,
  normalizeRoutingGroupConfig,
  removeGeneratedModelSchedulingRules,
  removeModelPolicy,
  removeModelSchedulingRule,
  setManagedInstructionsConfig,
  upsertModelPolicy,
  upsertModelSchedulingRule,
  type RoutingGroupConfig,
  type ManagedInstructionsConfig,
  type RoutingPriorityMode,
  type RoutingSchedulingMode,
} from '@/features/routing/utils/routingPolicy'
import {
  ManagedInstructionsConfigSection,
  RoutingPriorityPolicyEditor,
} from '@/features/routing/components'
import {
  createRoutingGroup,
  deleteRoutingGroup,
  listRoutingGroups,
  updateRoutingGroup,
  type RoutingGroupRecord,
} from '@/api/routing-profiles'
import { getGlobalModels, type GlobalModelResponse } from '@/api/global-models'
import { useToast } from '@/composables/useToast'
import { parseApiError } from '@/utils/errorParser'
import { log } from '@/utils/logger'

interface RoutingGroupDraft {
  id?: string
  name: string
  description: string
  enabled: boolean
  is_system_default: boolean
  config_json: RoutingGroupConfig
  version: number
  updated_at?: number | null
}

type SortingScope = 'unified' | 'per_model'
type ModelFilter = 'all' | 'configured' | 'unconfigured'

const { success, error: showError } = useToast()

const schedulingModes = computed<Array<{ value: RoutingSchedulingMode; label: string }>>(() => [
  { value: 'cache_affinity', label: t('routingProfiles.cacheAffinity') },
  { value: 'load_balance', label: t('routingProfiles.loadBalance') },
  { value: 'fixed_order', label: t('routingProfiles.fixedOrder') },
])

const modelFilterOptions = computed<Array<{ value: ModelFilter; label: string }>>(() => [
  { value: 'all', label: t('routingProfiles.all') },
  { value: 'configured', label: t('routingProfiles.configured') },
  { value: 'unconfigured', label: t('routingProfiles.unconfigured') },
])

const groups = ref<RoutingGroupRecord[]>([])
const selectedGroupId = ref<string | null>(null)
const draft = ref<RoutingGroupDraft | null>(null)
const savedDraftSnapshot = ref<string | null>(null)
const sortingScope = ref<SortingScope>('unified')
const selectedPerModelName = ref<string | null>(null)
const editingConfig = ref<RoutingGroupConfig | null>(null)
const globalModelSearch = ref('')
const modelFilter = ref<ModelFilter>('all')
const globalModels = ref<GlobalModelResponse[]>([])
const loadingGlobalModels = ref(false)
const globalModelsError = ref<string | null>(null)

const loading = ref(false)
const saving = ref(false)
const deleting = ref(false)
const isCreating = ref(false)

const switchModelTarget = ref<string | null>(null)
const switchModelDialogOpen = ref(false)
const deleteDialogOpen = ref(false)

const selectedGroup = computed(() => groups.value.find(group => group.id === selectedGroupId.value) ?? null)
const perModelPolicies = computed(() => {
  return draft.value?.config_json.model_policies
    .filter(policy => policy.model !== DEFAULT_ROUTING_POLICY_MODEL)
    ?? []
})
const activePerModelPolicy = computed(() => {
  if (!selectedPerModelName.value) return null
  const existing = perModelPolicies.value.find(policy => policy.model === selectedPerModelName.value)
  if (existing) return existing
  return createEmptyModelPolicy(selectedPerModelName.value)
})
const activePerModelName = computed(() => activePerModelPolicy.value?.model ?? '')
const firstStepPriorityMode = computed<RoutingPriorityMode>(() => {
  if (sortingScope.value === 'per_model' && activePerModelPolicy.value) {
    return modelPriorityMode(activePerModelPolicy.value.model)
  }
  return draft.value?.config_json.default_policy.priority_mode ?? 'provider'
})
const firstStepSchedulingMode = computed<RoutingSchedulingMode>(() => {
  if (sortingScope.value === 'per_model' && activePerModelPolicy.value) {
    return modelSchedulingMode(activePerModelPolicy.value.model)
  }
  return draft.value?.config_json.default_policy.scheduling_mode ?? 'cache_affinity'
})

const filteredGlobalModels = computed(() => {
  const query = globalModelSearch.value.trim().toLowerCase()
  const filter = modelFilter.value
  const models = [...globalModels.value].sort((left, right) =>
    left.name.localeCompare(right.name)
  )
  return models.filter(model => {
    if (query
      && !model.name.toLowerCase().includes(query)
      && !model.display_name?.toLowerCase().includes(query)) {
      return false
    }
    if (filter === 'configured' && !hasModelPolicy(model.name)) return false
    if (filter === 'unconfigured' && hasModelPolicy(model.name)) return false
    return true
  })
})

function normalizeRecord(group: RoutingGroupRecord): RoutingGroupRecord {
  return {
    ...group,
    config_json: normalizeRoutingGroupConfig(group.config_json),
  }
}

function cloneConfig(config: RoutingGroupConfig): RoutingGroupConfig {
  return normalizeRoutingGroupConfig(JSON.parse(JSON.stringify(config)) as Partial<RoutingGroupConfig>)
}

function draftSnapshotValue(value: RoutingGroupDraft): string {
  return JSON.stringify({
    name: value.name.trim(),
    description: value.description.trim() || null,
    enabled: value.enabled,
    is_system_default: value.is_system_default,
    config_json: cloneConfig(value.config_json),
  })
}

function buildDraft(group: RoutingGroupRecord): RoutingGroupDraft {
  return {
    id: group.id,
    name: group.name,
    description: group.description ?? '',
    enabled: group.enabled,
    is_system_default: group.is_system_default,
    config_json: cloneConfig(group.config_json),
    version: group.version,
    updated_at: group.updated_at,
  }
}

function selectGroup(group: RoutingGroupRecord): void {
  const normalized = normalizeRecord(group)
  isCreating.value = false
  selectedGroupId.value = normalized.id
  draft.value = buildDraft(normalized)
  savedDraftSnapshot.value = draftSnapshotValue(draft.value)
  syncEditorStateFromConfig(draft.value.config_json)
  resetEditingConfig()
}

function startCreate(): void {
  isCreating.value = true
  selectedGroupId.value = null
  draft.value = {
    name: t('routingProfiles.newName'),
    description: '',
    enabled: true,
    is_system_default: groups.value.length === 0,
    config_json: createEmptyRoutingGroupConfig(),
    version: 1,
    updated_at: null,
  }
  savedDraftSnapshot.value = null
  syncEditorStateFromConfig(draft.value.config_json)
  resetEditingConfig()
}

function updateDraftConfig(value: RoutingGroupConfig): void {
  if (!draft.value) return
  draft.value.config_json = normalizeRoutingGroupConfig(value)
  syncSelectedPerModelPolicy()
}

function updateManagedInstructions(value: ManagedInstructionsConfig): void {
  if (!draft.value) return
  updateDraftConfig(setManagedInstructionsConfig(draft.value.config_json, value))
  if (editingConfig.value) {
    editingConfig.value = setManagedInstructionsConfig(editingConfig.value, value)
  }
}

function resetEditingConfig(): void {
  if (!draft.value) {
    editingConfig.value = null
    return
  }
  editingConfig.value = cloneConfig(draft.value.config_json)
}

function updateEditingConfig(value: RoutingGroupConfig): void {
  editingConfig.value = normalizeRoutingGroupConfig(value)
}

const editingDirty = computed(() => {
  if (!editingConfig.value || !draft.value) return false
  return JSON.stringify(editingConfig.value) !== JSON.stringify(draft.value.config_json)
})

const draftDirty = computed(() => {
  if (!draft.value) return false
  if (isCreating.value) return true
  return savedDraftSnapshot.value !== draftSnapshotValue(draft.value)
})

const canSaveDraft = computed(() => {
  const hasPendingCurrentModel = perModelEditingActive.value
    && Boolean(activePerModelPolicy.value)
    && (editingDirty.value || !currentModelPersisted.value)
  return Boolean(draft.value)
    && !saving.value
    && draftDirty.value
    && !hasPendingCurrentModel
    && !(perModelEditingActive.value && perModelPolicies.value.length === 0)
})

const currentModelPersisted = computed(() => {
  const model = activePerModelPolicy.value?.model
  return model ? hasModelPolicy(model) : false
})

const canSaveCurrentModel = computed(() => {
  return Boolean(activePerModelPolicy.value)
    && !saving.value
    && (editingDirty.value || !currentModelPersisted.value)
})

const canRemoveCurrentModel = computed(() => {
  return Boolean(activePerModelPolicy.value)
    && currentModelPersisted.value
    && !saving.value
    && !editingDirty.value
    && !draftDirty.value
})

function syncEditorStateFromConfig(config: RoutingGroupConfig): void {
  const normalized = normalizeRoutingGroupConfig(config)
  sortingScope.value = hasPerModelSorting(normalized) ? 'per_model' : 'unified'
  syncSelectedPerModelPolicy()
}

function hasPerModelSorting(config: RoutingGroupConfig): boolean {
  return config.model_policies.some(policy => policy.model !== DEFAULT_ROUTING_POLICY_MODEL)
    || config.rules.some(isGeneratedModelSchedulingRule)
}

function setSortingScope(scope: SortingScope): void {
  if (!draft.value) return
  sortingScope.value = scope
  if (scope === 'unified') {
    const next = removeGeneratedModelSchedulingRules(draft.value.config_json)
    next.model_policies = next.model_policies.filter(policy => policy.model === DEFAULT_ROUTING_POLICY_MODEL)
    next.allowed_models = []
    updateDraftConfig(next)
    resetEditingConfig()
    return
  }
  resetEditingConfig()
}

function updateFirstStepPriorityMode(mode: RoutingPriorityMode): void {
  if (!draft.value) return
  if (sortingScope.value === 'per_model' && activePerModelPolicy.value) {
    updateModelPriorityMode(activePerModelPolicy.value.model, mode)
    return
  }
  updateDraftConfig({
    ...draft.value.config_json,
    default_policy: {
      ...draft.value.config_json.default_policy,
      priority_mode: mode,
    },
  })
}

function updateFirstStepSchedulingMode(mode: RoutingSchedulingMode): void {
  if (!draft.value) return
  if (sortingScope.value === 'per_model' && activePerModelPolicy.value) {
    updateModelSchedulingMode(activePerModelPolicy.value.model, mode)
    return
  }
  updateDraftConfig({
    ...draft.value.config_json,
    default_policy: {
      ...draft.value.config_json.default_policy,
      scheduling_mode: mode,
    },
  })
}

function removePerModelPolicy(model: string): void {
  if (!draft.value) return
  if (perModelEditingActive.value && (editingDirty.value || draftDirty.value)) {
    showError(t('routingProfiles.saveBeforeRemove'))
    return
  }
  let next = removeModelPolicy(draft.value.config_json, model)
  next = removeModelSchedulingRule(next, model)
  next.allowed_models = next.allowed_models.filter(item => item !== model)
  if (selectedPerModelName.value === model) {
    selectedPerModelName.value = null
  }
  updateDraftConfig(next)
  resetEditingConfig()
}

function selectGlobalModel(model: string): void {
  if (!model) return
  if (model === selectedPerModelName.value) return
  if (perModelEditingActive.value && editingDirty.value) {
    switchModelTarget.value = model
    switchModelDialogOpen.value = true
    return
  }
  selectedPerModelName.value = model
}

function confirmSwitchModel(): void {
  const target = switchModelTarget.value
  if (target) {
    resetEditingConfig()
    selectedPerModelName.value = target
  }
  switchModelTarget.value = null
  switchModelDialogOpen.value = false
}

function cancelSwitchModel(): void {
  switchModelTarget.value = null
}

function hasModelPolicy(model: string): boolean {
  if (perModelPolicies.value.some(policy => policy.model === model)) return true
  const ruleId = modelSchedulingRuleId(model)
  return draft.value?.config_json.rules.some(rule => rule.id === ruleId) ?? false
}

const copySourceCandidates = computed(() => {
  if (!draft.value) return []
  const current = selectedPerModelName.value
  return perModelPolicies.value
    .filter(policy => policy.model !== current)
    .map(policy => ({
      model: policy.model,
      label: globalModelLabel(policy.model),
    }))
})

function copyModelConfig(sourceModel: string): void {
  if (!draft.value || !editingConfig.value) return
  const target = selectedPerModelName.value
  if (!target || target === sourceModel) return
  const sourcePolicy = getModelPolicy(draft.value.config_json, sourceModel)
  const sourceScheduling = getModelScheduling(draft.value.config_json, sourceModel)
  let next = upsertModelPolicy(editingConfig.value, {
    ...sourcePolicy,
    model: target,
  })
  next = upsertModelSchedulingRule(next, target, {
    priority_mode: sourceScheduling.priority_mode,
    scheduling_mode: sourceScheduling.scheduling_mode,
  })
  if (!next.allowed_models.includes(target)) {
    next = { ...next, allowed_models: [...next.allowed_models, target] }
  }
  updateEditingConfig(next)
  success(t('routingProfiles.configLoaded', { model: globalModelLabel(sourceModel) }))
}

function syncSelectedPerModelPolicy(): void {
  if (selectedPerModelName.value) return
  const firstConfigured = perModelPolicies.value[0]?.model
  selectedPerModelName.value = firstConfigured ?? null
}

const perModelEditingActive = computed(() => sortingScope.value === 'per_model')

const activeConfigForReading = computed<RoutingGroupConfig>(() => {
  if (perModelEditingActive.value && editingConfig.value) return editingConfig.value
  return draft.value?.config_json ?? createEmptyRoutingGroupConfig()
})

function modelPriorityMode(model: string): RoutingPriorityMode {
  return getModelScheduling(activeConfigForReading.value, model).priority_mode
}

function modelSchedulingMode(model: string): RoutingSchedulingMode {
  return getModelScheduling(activeConfigForReading.value, model).scheduling_mode
}

function updateModelPriorityMode(model: string, mode: RoutingPriorityMode): void {
  if (!draft.value) return
  const baseConfig = perModelEditingActive.value && editingConfig.value
    ? editingConfig.value
    : draft.value.config_json
  const current = getModelScheduling(baseConfig, model)
  const next = upsertModelSchedulingRule(baseConfig, model, {
    priority_mode: mode,
    scheduling_mode: current.scheduling_mode,
  })
  if (perModelEditingActive.value) {
    updateEditingConfig(next)
    return
  }
  updateDraftConfig(next)
}

function updateModelSchedulingMode(model: string, mode: RoutingSchedulingMode): void {
  if (!draft.value) return
  const baseConfig = perModelEditingActive.value && editingConfig.value
    ? editingConfig.value
    : draft.value.config_json
  const current = getModelScheduling(baseConfig, model)
  const next = upsertModelSchedulingRule(baseConfig, model, {
    priority_mode: current.priority_mode,
    scheduling_mode: mode,
  })
  if (perModelEditingActive.value) {
    updateEditingConfig(next)
    return
  }
  updateDraftConfig(next)
}

function globalModelLabel(modelName: string): string {
  const model = globalModels.value.find(item => item.name === modelName)
  if (!model) return modelName
  if (!model.display_name || model.display_name === model.name) return model.name
  return `${model.display_name} (${model.name})`
}

function replaceGroup(group: RoutingGroupRecord): void {
  const normalized = normalizeRecord(group)
  const index = groups.value.findIndex(item => item.id === normalized.id)
  if (index >= 0) {
    groups.value[index] = normalized
  } else {
    groups.value.unshift(normalized)
  }
  selectGroup(normalized)
}

function refreshPage(): void {
  void fetchGroups()
  void loadGlobalModels()
}

async function fetchGroups(): Promise<void> {
  loading.value = true
  try {
    const response = await listRoutingGroups()
    groups.value = response.items.map(normalizeRecord)
    const nextSelected = selectedGroup.value ?? groups.value[0] ?? null
    if (nextSelected) {
      selectGroup(nextSelected)
    } else if (!draft.value) {
      startCreate()
    }
  } catch (err) {
    showError(parseApiError(err, t('routingProfiles.loadFailed')))
    log.error('加载调度策略失败:', err)
  } finally {
    loading.value = false
  }
}

async function loadGlobalModels(options: { cacheTtlMs?: number } = {}): Promise<void> {
  loadingGlobalModels.value = true
  globalModelsError.value = null
  try {
    const response = await getGlobalModels(
      { limit: 1000, is_active: true },
      { cacheTtlMs: options.cacheTtlMs ?? 0 },
    )
    globalModels.value = response.models ?? []
  } catch (err) {
    globalModels.value = []
    globalModelsError.value = parseApiError(err, t('routingProfiles.loadModelsFailed'))
    log.error('加载全局模型失败:', err)
  } finally {
    loadingGlobalModels.value = false
  }
}

async function saveDraft(): Promise<void> {
  if (!draft.value) return
  const name = draft.value.name.trim()
  if (!name) {
    showError(t('routingProfiles.nameRequired'))
    return
  }
  const config = cloneConfig(draft.value.config_json)
  if (sortingScope.value === 'per_model' && perModelPolicies.value.length === 0) {
    showError(t('routingProfiles.modelRequired'))
    return
  }

  saving.value = true
  try {
    const payload = {
      name,
      description: draft.value.description.trim() || null,
      enabled: draft.value.enabled,
      is_system_default: draft.value.is_system_default,
      config_json: config,
    }
    const saved = isCreating.value || !draft.value.id
      ? await createRoutingGroup(payload)
      : await updateRoutingGroup(draft.value.id, payload)
    isCreating.value = false
    replaceGroup(saved)
    success(t('routingProfiles.saved'))
  } catch (err) {
    showError(parseApiError(err, t('routingProfiles.saveFailed')))
    log.error('保存调度策略失败:', err)
  } finally {
    saving.value = false
  }
}

function saveCurrentModel(): void {
  if (!draft.value || !editingConfig.value) return
  const model = selectedPerModelName.value
  if (!model) {
    showError(t('routingProfiles.selectModel'))
    return
  }
  let next = editingConfig.value
  if (!next.model_policies.some(policy => policy.model === model)) {
    next = upsertModelPolicy(next, createEmptyModelPolicy(model))
  }
  if (!next.allowed_models.includes(model)) {
    next = { ...next, allowed_models: [...next.allowed_models, model] }
  }
  updateDraftConfig(next)
  resetEditingConfig()
  success(t('routingProfiles.modelDraftSaved'))
}

function deleteDraft(): void {
  if (!draft.value?.id) return
  deleteDialogOpen.value = true
}

async function confirmDeleteDraft(): Promise<void> {
  if (!draft.value?.id) return

  deleting.value = true
  try {
    const deletedId = draft.value.id
    await deleteRoutingGroup(deletedId)
    groups.value = groups.value.filter(group => group.id !== deletedId)
    selectedGroupId.value = null
    draft.value = null
    if (groups.value.length > 0) {
      selectGroup(groups.value[0])
    } else {
      startCreate()
    }
    success(t('routingProfiles.deleted'))
    deleteDialogOpen.value = false
  } catch (err) {
    showError(parseApiError(err, t('routingProfiles.deleteFailed')))
    log.error('删除调度策略失败:', err)
  } finally {
    deleting.value = false
  }
}

function formatUnixSeconds(value?: number | null): string {
  if (!value) return '-'
  return new Date(value * 1000).toLocaleString(locale.value)
}

onMounted(() => {
  void fetchGroups()
  void loadGlobalModels({ cacheTtlMs: 60_000 })
})
</script>
