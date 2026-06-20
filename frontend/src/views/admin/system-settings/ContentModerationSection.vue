<template>
  <CardSection
    title="内容审查 / 账号保护"
    description="在请求进入指定上游前先做内容审查"
  >
    <template #actions>
      <Button
        size="sm"
        :disabled="loading || !hasChanges"
        @click="$emit('save')"
      >
        {{ loading ? '保存中...' : '保存' }}
      </Button>
    </template>

    <div class="space-y-5">
      <div class="flex items-center justify-between gap-4 rounded-xl border border-border/60 bg-card/60 px-4 py-3">
        <div class="min-w-0">
          <p class="text-sm font-medium text-foreground">
            启用前置审查
          </p>
          <p class="mt-0.5 text-xs text-muted-foreground">
            审查失败时放行请求，不产生审查扣费。
          </p>
        </div>
        <Switch
          :model-value="config.enabled"
          :disabled="loading"
          @update:model-value="updateField('enabled', $event)"
        />
      </div>

      <div class="grid grid-cols-1 gap-5 md:grid-cols-2">
        <div>
          <Label
            for="content-moderation-level"
            class="block text-sm font-medium"
          >
            审查级别
          </Label>
          <Select
            :model-value="config.level"
            @update:model-value="updateLevel"
          >
            <SelectTrigger
              id="content-moderation-level"
              class="mt-1"
            >
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="off">
                关闭
              </SelectItem>
              <SelectItem value="latest_user_input">
                只审最新用户输入
              </SelectItem>
              <SelectItem value="all_user_inputs">
                审本次请求所有用户输入
              </SelectItem>
              <SelectItem value="full_request">
                审完整请求
              </SelectItem>
            </SelectContent>
          </Select>
        </div>

        <div>
          <Label
            for="content-moderation-model"
            class="block text-sm font-medium"
          >
            审查模型
          </Label>
          <Input
            id="content-moderation-model"
            :model-value="config.model"
            placeholder="omni-moderation-latest"
            class="mt-1"
            @update:model-value="updateField('model', String($event))"
          />
        </div>

        <div>
          <Label
            for="content-moderation-api-keys"
            class="block text-sm font-medium"
          >
            审查 API Keys
          </Label>
          <div
            v-if="savedApiKeyCount > 0"
            class="mt-2 rounded-lg border border-border/60 bg-muted/30 px-3 py-2"
          >
            <p class="text-xs text-muted-foreground">
              已保存 {{ savedApiKeyCount }} 个 Key
            </p>
            <div
              v-if="savedApiKeyMasks.length > 0"
              class="mt-2 flex flex-wrap gap-1.5"
            >
              <span
                v-for="mask in savedApiKeyMasks"
                :key="mask"
                class="rounded-md bg-background px-2 py-1 font-mono text-[11px] text-muted-foreground"
              >
                {{ mask }}
              </span>
            </div>
            <Button
              size="sm"
              variant="outline"
              class="mt-2"
              :disabled="loading"
              @click="clearSavedApiKeys"
            >
              清空已保存 Key
            </Button>
          </div>
          <Textarea
            id="content-moderation-api-keys"
            v-model="apiKeysText"
            class="mt-1 min-h-24 font-mono text-xs"
            placeholder="粘贴新 Key，每行一个；留空保持已保存 Key 不变"
          />
          <p class="mt-1 text-xs text-muted-foreground">
            每行一个 Key，保存时会自动去空行和重复项；填写后会整体替换已保存 Key。
            <span v-if="pendingApiKeyCount > 0">
              保存后将替换为 {{ pendingApiKeyCount }} 个 Key。
            </span>
            <span v-else-if="config.api_keys_clear">
              保存后会清空已保存 Key。
            </span>
          </p>
        </div>

        <div>
          <Label
            for="content-moderation-base-url"
            class="block text-sm font-medium"
          >
            Base URL
          </Label>
          <Input
            id="content-moderation-base-url"
            :model-value="config.base_url"
            placeholder="https://api.openai.com/v1"
            class="mt-1"
            @update:model-value="updateField('base_url', String($event))"
          />
        </div>

        <div>
          <Label
            for="content-moderation-timeout"
            class="block text-sm font-medium"
          >
            超时时间 (ms)
          </Label>
          <Input
            id="content-moderation-timeout"
            :model-value="config.timeout_ms"
            type="number"
            min="500"
            max="60000"
            class="mt-1"
            @update:model-value="updateNumber('timeout_ms', $event)"
          />
        </div>

        <div>
          <Label
            for="content-moderation-retention"
            class="block text-sm font-medium"
          >
            证据留存天数
          </Label>
          <Input
            id="content-moderation-retention"
            :model-value="config.evidence_retention_days"
            type="number"
            min="1"
            max="365"
            class="mt-1"
            @update:model-value="updateNumber('evidence_retention_days', $event)"
          />
        </div>

        <div>
          <Label
            for="content-moderation-input-price"
            class="block text-sm font-medium"
          >
            输入价格 / 100 万 tokens
          </Label>
          <Input
            id="content-moderation-input-price"
            :model-value="config.input_price_per_1m"
            type="number"
            min="0"
            step="0.000001"
            class="mt-1"
            @update:model-value="updateNumber('input_price_per_1m', $event)"
          />
        </div>

        <div>
          <Label
            for="content-moderation-output-price"
            class="block text-sm font-medium"
          >
            输出价格 / 100 万 tokens
          </Label>
          <Input
            id="content-moderation-output-price"
            :model-value="config.output_price_per_1m"
            type="number"
            min="0"
            step="0.000001"
            class="mt-1"
            @update:model-value="updateNumber('output_price_per_1m', $event)"
          />
        </div>
      </div>

      <div class="grid grid-cols-1 gap-5 md:grid-cols-3">
        <div>
          <Label
            for="content-moderation-provider-targets"
            class="block text-sm font-medium"
          >
            Provider IDs
          </Label>
          <Textarea
            id="content-moderation-provider-targets"
            v-model="providerTargetText"
            class="mt-1 min-h-24 font-mono text-xs"
            placeholder="provider-id-1, provider-id-2"
          />
        </div>

        <div>
          <Label
            for="content-moderation-service-targets"
            class="block text-sm font-medium"
          >
            上游服务 IDs
          </Label>
          <Textarea
            id="content-moderation-service-targets"
            v-model="upstreamServiceTargetText"
            class="mt-1 min-h-24 font-mono text-xs"
            placeholder="endpoint-id-1"
          />
        </div>

        <div>
          <Label
            for="content-moderation-account-targets"
            class="block text-sm font-medium"
          >
            上游账号 IDs
          </Label>
          <Textarea
            id="content-moderation-account-targets"
            v-model="upstreamAccountTargetText"
            class="mt-1 min-h-24 font-mono text-xs"
            placeholder="provider-key-id-1"
          />
        </div>
      </div>
    </div>
  </CardSection>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import Button from '@/components/ui/button.vue'
import Input from '@/components/ui/input.vue'
import Label from '@/components/ui/label.vue'
import Select from '@/components/ui/select.vue'
import SelectTrigger from '@/components/ui/select-trigger.vue'
import SelectValue from '@/components/ui/select-value.vue'
import SelectContent from '@/components/ui/select-content.vue'
import SelectItem from '@/components/ui/select-item.vue'
import Switch from '@/components/ui/switch.vue'
import Textarea from '@/components/ui/textarea.vue'
import { CardSection } from '@/components/layout'
import type {
  ContentModerationAccountProtectionConfig,
  ContentModerationLevel,
  ContentModerationTargetKind,
} from './composables/useSystemConfig'

type NumericConfigKey =
  | 'timeout_ms'
  | 'input_price_per_1m'
  | 'output_price_per_1m'
  | 'evidence_retention_days'

const props = defineProps<{
  config: ContentModerationAccountProtectionConfig
  loading: boolean
  hasChanges: boolean
}>()

const emit = defineEmits<{
  save: []
  'update:config': [value: ContentModerationAccountProtectionConfig]
}>()

const targetKindOrder: Record<ContentModerationTargetKind, number> = {
  provider: 0,
  upstream_service: 1,
  upstream_account: 2,
}

function updateConfig(patch: Partial<ContentModerationAccountProtectionConfig>) {
  emit('update:config', {
    ...props.config,
    ...patch,
  })
}

function updateField<Key extends keyof ContentModerationAccountProtectionConfig>(
  key: Key,
  value: ContentModerationAccountProtectionConfig[Key],
) {
  updateConfig({ [key]: value } as Partial<ContentModerationAccountProtectionConfig>)
}

function updateNumber(key: NumericConfigKey, value: string | number) {
  const nextValue = Number(value)
  updateConfig({
    [key]: Number.isFinite(nextValue) ? nextValue : 0,
  } as Pick<ContentModerationAccountProtectionConfig, NumericConfigKey>)
}

function updateLevel(value: string) {
  const allowed = new Set<ContentModerationLevel>([
    'off',
    'latest_user_input',
    'all_user_inputs',
    'full_request',
  ])
  updateField('level', allowed.has(value as ContentModerationLevel)
    ? value as ContentModerationLevel
    : 'all_user_inputs')
}

function parseTargetIds(value: string): string[] {
  const seen = new Set<string>()
  return value
    .split(/[\n,]+/)
    .map(item => item.trim())
    .filter((item) => {
      if (!item || seen.has(item)) return false
      seen.add(item)
      return true
    })
}

function parseApiKeys(value: string): string[] {
  const seen = new Set<string>()
  return value
    .split(/\n+/)
    .map(item => item.trim())
    .filter((item) => {
      if (!item || seen.has(item)) return false
      seen.add(item)
      return true
    })
}

const apiKeysText = computed({
  get: () => props.config.api_keys.join('\n'),
  set: value => updateConfig({ api_keys: parseApiKeys(value), api_keys_clear: false }),
})

const savedApiKeyCount = computed(() => props.config.api_key_count)

const savedApiKeyMasks = computed(() => props.config.api_key_masks)

const pendingApiKeyCount = computed(() => props.config.api_keys.length)

function clearSavedApiKeys() {
  updateConfig({
    api_keys: [],
    api_keys_clear: true,
    api_key_count: 0,
    api_key_masks: [],
  })
}

function targetText(kind: ContentModerationTargetKind): string {
  return props.config.targets
    .filter(target => target.kind === kind)
    .map(target => target.id)
    .join('\n')
}

function updateTargetText(kind: ContentModerationTargetKind, value: string) {
  const nextTargets = [
    ...props.config.targets.filter(target => target.kind !== kind),
    ...parseTargetIds(value).map(id => ({ kind, id })),
  ].sort((a, b) => targetKindOrder[a.kind] - targetKindOrder[b.kind])
  updateConfig({ targets: nextTargets })
}

const providerTargetText = computed({
  get: () => targetText('provider'),
  set: value => updateTargetText('provider', value),
})

const upstreamServiceTargetText = computed({
  get: () => targetText('upstream_service'),
  set: value => updateTargetText('upstream_service', value),
})

const upstreamAccountTargetText = computed({
  get: () => targetText('upstream_account'),
  set: value => updateTargetText('upstream_account', value),
})
</script>
