<template>
  <section class="space-y-3 rounded-lg border border-border/60 p-4">
    <div class="flex items-start justify-between gap-4">
      <div class="min-w-0 space-y-1">
        <h3 class="text-sm font-medium">
          {{ t('userGroupManagedInstructions.title') }}
        </h3>
        <p class="text-xs text-muted-foreground">
          {{ t('userGroupManagedInstructions.hint') }}
        </p>
      </div>
      <Switch
        :model-value="enabled"
        :disabled="loading || (!profiles.length && !enabled)"
        :aria-label="t('userGroupManagedInstructions.title')"
        @update:model-value="setEnabled"
      />
    </div>

    <div
      v-if="loadError"
      data-testid="managed-instructions-error"
      class="flex items-center justify-between gap-3 rounded-lg border border-destructive/30 bg-destructive/5 px-3 py-2"
    >
      <p class="text-xs text-destructive">
        {{ loadError }}
      </p>
      <Button
        type="button"
        variant="ghost"
        size="sm"
        class="h-7 shrink-0 px-2 text-xs"
        @click="loadProfiles"
      >
        {{ t('common.retry') }}
      </Button>
    </div>

    <div
      v-else-if="loading"
      data-testid="managed-instructions-loading"
      class="grid grid-cols-2 gap-3"
    >
      <Skeleton class="h-16" />
      <Skeleton class="h-16" />
    </div>

    <template v-else>
      <div class="grid grid-cols-2 gap-3">
        <div class="space-y-1.5">
          <Label class="text-xs">{{ t('userGroupManagedInstructions.profile') }}</Label>
          <Select
            data-testid="managed-profile-select"
            :model-value="profileId"
            :disabled="profiles.length === 0"
            @update:model-value="setProfileId"
          >
            <SelectTrigger class="h-9 rounded-lg px-3 shadow-none">
              <SelectValue :placeholder="t('userGroupManagedInstructions.profilePlaceholder')" />
            </SelectTrigger>
            <SelectContent :searchable="false">
              <SelectItem
                v-for="profile in profiles"
                :key="profile.profile_id"
                :value="profile.profile_id"
              >
                {{ profile.display_name }}
              </SelectItem>
            </SelectContent>
          </Select>
        </div>

        <div class="space-y-1.5">
          <Label class="text-xs">{{ t('userGroupManagedInstructions.mergeMode') }}</Label>
          <Select
            :model-value="mergeMode"
            :disabled="!enabled"
            @update:model-value="setMergeMode"
          >
            <SelectTrigger class="h-9 rounded-lg px-3 shadow-none">
              <SelectValue />
            </SelectTrigger>
            <SelectContent :searchable="false">
              <SelectItem value="prepend">
                {{ t('userGroupManagedInstructions.mergePrepend') }}
              </SelectItem>
              <SelectItem value="if_missing">
                {{ t('userGroupManagedInstructions.mergeIfMissing') }}
              </SelectItem>
            </SelectContent>
          </Select>
        </div>
      </div>

      <p
        v-if="!modelValue"
        data-testid="managed-instructions-unconfigured"
        class="text-xs text-muted-foreground"
      >
        {{ t('userGroupManagedInstructions.notConfigured') }}
      </p>

      <p
        v-if="configurationError"
        class="text-xs text-destructive"
      >
        {{ configurationError }}
      </p>

      <div
        v-if="selectedProfile"
        data-testid="managed-instructions-summary"
        class="space-y-2 rounded-lg border border-border/60 bg-muted/20 px-3 py-2.5"
        :class="{ 'opacity-60': !enabled }"
      >
        <div class="flex flex-wrap items-center gap-2 text-xs">
          <Badge variant="outline">
            {{ selectedProfile.core_version }}
          </Badge>
          <Badge
            v-if="selectedProfile.domain_version"
            variant="outline"
          >
            {{ selectedProfile.domain_version }}
          </Badge>
          <span class="text-muted-foreground">
            {{ enabled ? mergeModeDescription : t('userGroupManagedInstructions.disabled') }}
          </span>
        </div>
        <p class="break-all font-mono text-[11px] text-muted-foreground">
          SHA-256 {{ selectedProfile.profile_sha256 }}
        </p>
        <div class="flex flex-wrap items-center gap-1.5 text-[11px] text-muted-foreground">
          <span class="rounded-md bg-background px-2 py-1 text-foreground">
            {{ t('userGroupManagedInstructions.orderManaged') }}
          </span>
          <span aria-hidden="true">→</span>
          <span class="rounded-md bg-background px-2 py-1 text-foreground">
            {{ t('userGroupManagedInstructions.orderClient') }}
          </span>
          <span aria-hidden="true">→</span>
          <span class="rounded-md bg-background px-2 py-1 text-foreground">
            {{ t('userGroupManagedInstructions.orderImage') }}
          </span>
        </div>
      </div>
    </template>
  </section>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import {
  Badge,
  Button,
  Label,
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
  Skeleton,
  Switch,
} from '@/components/ui'
import {
  type ManagedInstructionsProfile,
  type ManagedInstructionsConfig,
  type ManagedInstructionsMergeMode,
  usersApi,
} from '@/api/users'
import { parseApiError } from '@/utils/errorParser'

const props = defineProps<{
  modelValue?: ManagedInstructionsConfig | null
}>()

const emit = defineEmits<{
  'update:modelValue': [value: ManagedInstructionsConfig]
}>()

const { t } = useI18n()
const profiles = ref<ManagedInstructionsProfile[]>([])
const loading = ref(true)
const loadError = ref('')

const enabled = computed(() => props.modelValue?.enabled === true)
const profileId = computed(() => props.modelValue?.profile_id || '')
const mergeMode = computed<ManagedInstructionsMergeMode>(
  () => props.modelValue?.merge_mode || 'prepend'
)
const selectedProfile = computed(
  () => profiles.value.find(profile => profile.profile_id === profileId.value) ?? null
)
const configurationError = computed(() => {
  if (!props.modelValue?.profile_id || selectedProfile.value || profiles.value.length === 0) return ''
  return t('userGroupManagedInstructions.unknownProfile', { profile: profileId.value })
})
const mergeModeDescription = computed(() => (
  mergeMode.value === 'if_missing'
    ? t('userGroupManagedInstructions.ifMissingSummary')
    : t('userGroupManagedInstructions.prependSummary')
))

function emitConfig(patch: Partial<ManagedInstructionsConfig>) {
  emit('update:modelValue', {
    enabled: enabled.value,
    profile_id: profileId.value,
    merge_mode: mergeMode.value,
    ...patch,
  })
}

function setEnabled(value: boolean) {
  if (value && profiles.value.length === 0) return
  if (value && !props.modelValue) {
    const defaultProfileId = profiles.value[0]?.profile_id
    if (!defaultProfileId) return
    emit('update:modelValue', {
      enabled: true,
      profile_id: defaultProfileId,
      merge_mode: 'prepend',
    })
    return
  }
  emitConfig({ enabled: value })
}

function setProfileId(value: string) {
  emitConfig({ profile_id: value })
}

function setMergeMode(value: string) {
  if (value !== 'prepend' && value !== 'if_missing') return
  emitConfig({ merge_mode: value })
}

async function loadProfiles() {
  loading.value = true
  loadError.value = ''
  try {
    const response = await usersApi.getManagedInstructionProfiles()
    profiles.value = response.profiles
  } catch (error) {
    profiles.value = []
    loadError.value = parseApiError(
      error,
      t('userGroupManagedInstructions.loadFailed')
    )
  } finally {
    loading.value = false
  }
}

onMounted(loadProfiles)
</script>
