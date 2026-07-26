<template>
  <Card class="overflow-hidden">
    <div class="border-b border-border/60 px-5 py-4">
      <div class="flex items-center gap-2">
        <WandSparkles class="h-4 w-4 text-primary" />
        <h2 class="font-semibold">
          {{ t('imageStudio.formTitle') }}
        </h2>
      </div>
    </div>

    <form
      class="space-y-5 p-5"
      @submit.prevent="emit('submit')"
    >
      <div class="space-y-2">
        <div class="flex items-center justify-between gap-2">
          <Label>{{ t('imageStudio.apiKey') }}</Label>
          <button
            type="button"
            class="text-xs text-primary hover:underline"
            @click="emit('refresh')"
          >
            {{ t('imageStudio.refreshResources') }}
          </button>
        </div>
        <Select
          :model-value="settings.selectedKeyId"
          @update:model-value="updateSettings({ selectedKeyId: $event })"
        >
          <SelectTrigger><SelectValue :placeholder="t('imageStudio.chooseEnabledKey')" /></SelectTrigger>
          <SelectContent>
            <SelectItem
              v-for="key in apiKeys"
              :key="key.id"
              :value="key.id"
            >
              {{ key.name }} · {{ key.display }}
            </SelectItem>
          </SelectContent>
        </Select>
        <p
          v-if="!loading && apiKeys.length === 0"
          class="text-xs text-destructive"
        >
          {{ t('imageStudio.noKeys') }}
        </p>
      </div>

      <div class="space-y-2">
        <Label>{{ t('imageStudio.imageModel') }}</Label>
        <Select
          :model-value="settings.model"
          @update:model-value="updateSettings({ model: $event })"
        >
          <SelectTrigger><SelectValue :placeholder="t('imageStudio.chooseImageModel')" /></SelectTrigger>
          <SelectContent>
            <SelectItem
              v-for="model in models"
              :key="model.id"
              :value="model.name"
              :text-value="`${model.displayName} ${model.name}`"
            >
              {{ model.displayName }}
              <span
                v-if="model.displayName !== model.name"
                class="ml-1 text-xs text-muted-foreground"
              >{{ model.name }}</span>
            </SelectItem>
          </SelectContent>
        </Select>
        <p
          v-if="!loading && models.length === 0"
          class="text-xs text-destructive"
        >
          {{ t('imageStudio.noImageModels') }}
        </p>
      </div>

      <div class="space-y-2">
        <Label for="image-prompt">{{ t('imageStudio.prompt') }}</Label>
        <Textarea
          id="image-prompt"
          :model-value="form.prompt"
          class="min-h-32 resize-y"
          :placeholder="t('imageStudio.promptPlaceholder')"
          @update:model-value="updateForm({ prompt: $event })"
        />
      </div>

      <div class="grid grid-cols-2 gap-3">
        <div class="space-y-2">
          <Label for="image-count">{{ t('imageStudio.count') }}</Label>
          <Input
            id="image-count"
            :model-value="form.count"
            type="number"
            min="1"
            max="8"
            @update:model-value="updateCount($event)"
          />
        </div>
        <div class="space-y-2">
          <Label for="image-concurrency">{{ t('imageStudio.concurrency') }}</Label>
          <Input
            id="image-concurrency"
            :model-value="settings.concurrency"
            type="number"
            min="1"
            max="8"
            @update:model-value="updateConcurrency($event)"
          />
        </div>
      </div>

      <div class="space-y-2">
        <Label>{{ t('imageStudio.size') }}</Label>
        <div>
          <p class="mb-2 text-xs text-muted-foreground">
            {{ t('imageStudio.aspectRatio') }}
          </p>
          <div class="grid grid-cols-5 gap-2">
            <button
              v-for="preset in IMAGE_ASPECT_RATIO_PRESETS"
              :key="preset.ratio"
              type="button"
              class="rounded-xl border px-2 py-2 text-xs font-medium transition"
              :class="selectedAspectRatio === preset.ratio ? 'border-primary bg-primary/10 text-primary' : 'border-border/60 hover:border-primary/50'"
              :title="`${preset.ratio} · ${preset.size}`"
              @click="applyAspectRatio(preset)"
            >
              {{ preset.ratio }}
            </button>
          </div>
        </div>
        <p class="text-xs text-muted-foreground">
          {{ t('imageStudio.commonResolutions') }}
        </p>
        <div class="grid grid-cols-3 gap-2">
          <button
            v-for="size in IMAGE_SIZE_PRESETS"
            :key="size"
            type="button"
            class="rounded-xl border px-2 py-2 text-xs transition"
            :class="form.size === size ? 'border-primary bg-primary/10 text-primary' : 'border-border/60 hover:border-primary/50'"
            @click="updateForm({ size })"
          >
            {{ size.replace('x', '×') }}
          </button>
        </div>
        <Input
          :model-value="form.size"
          :placeholder="t('imageStudio.customSizePlaceholder')"
          @update:model-value="updateForm({ size: String($event) })"
        />
      </div>

      <div class="grid grid-cols-2 gap-3">
        <div class="space-y-2">
          <Label>{{ t('imageStudio.quality') }}</Label>
          <select
            :value="form.quality"
            class="native-select"
            @change="updateForm({ quality: selectValue($event) })"
          >
            <option value="auto">
              {{ t('imageStudio.auto') }}
            </option><option value="low">
              {{ t('imageStudio.low') }}
            </option><option value="medium">
              {{ t('imageStudio.medium') }}
            </option><option value="high">
              {{ t('imageStudio.high') }}
            </option>
          </select>
        </div>
        <div class="space-y-2">
          <Label>{{ t('imageStudio.background') }}</Label>
          <select
            :value="form.background"
            class="native-select"
            @change="updateForm({ background: selectValue($event) })"
          >
            <option value="auto">
              {{ t('imageStudio.auto') }}
            </option><option value="opaque">
              {{ t('imageStudio.opaque') }}
            </option><option value="transparent">
              {{ t('imageStudio.transparent') }}
            </option>
          </select>
        </div>
        <div class="space-y-2">
          <Label>{{ t('imageStudio.format') }}</Label>
          <select
            :value="form.outputFormat"
            class="native-select"
            @change="updateForm({ outputFormat: selectValue($event) })"
          >
            <option value="auto">
              {{ t('imageStudio.auto') }}
            </option><option value="png">
              PNG
            </option><option value="jpeg">
              JPEG
            </option><option value="webp">
              WebP
            </option>
          </select>
        </div>
        <div class="space-y-2">
          <Label>{{ t('imageStudio.response') }}</Label>
          <select
            :value="settings.responseFormat"
            class="native-select"
            @change="updateSettings({ responseFormat: selectValue($event) as ImageStudioSettings['responseFormat'] })"
          >
            <option value="url">
              URL
            </option><option value="b64_json">
              Base64
            </option>
          </select>
        </div>
      </div>

      <div class="space-y-3 rounded-2xl border border-dashed border-border/70 bg-muted/20 p-4">
        <div class="flex items-center justify-between gap-2">
          <div>
            <p class="text-sm font-medium">
              {{ t('imageStudio.referenceImage') }}
            </p>
            <p class="text-xs text-muted-foreground">
              {{ t('imageStudio.referenceHint') }}
            </p>
          </div>
          <label class="cursor-pointer rounded-lg border border-border/60 px-3 py-2 text-xs font-medium hover:border-primary/60 hover:text-primary">
            <Upload class="mr-1 inline h-3.5 w-3.5" />{{ t('imageStudio.addImage') }}
            <input
              class="sr-only"
              type="file"
              :accept="accept"
              multiple
              @change="handleReferenceFiles"
            >
          </label>
        </div>
        <div
          v-if="form.inputImages.length"
          class="space-y-2"
        >
          <div
            v-for="(file, index) in form.inputImages"
            :key="`${file.name}-${index}`"
            class="flex items-center justify-between rounded-lg bg-background/70 px-3 py-2 text-xs"
          >
            <span class="min-w-0 truncate">{{ file.name }}</span>
            <button
              type="button"
              class="ml-2 text-muted-foreground hover:text-destructive"
              @click="removeReference(index)"
            >
              <X class="h-3.5 w-3.5" />
            </button>
          </div>
          <label class="flex cursor-pointer items-center justify-center rounded-lg border border-border/60 px-3 py-2 text-xs hover:border-primary/60">
            <ScanLine class="mr-1.5 h-3.5 w-3.5" />{{ form.maskImage ? t('imageStudio.maskFile', { name: form.maskImage.name }) : t('imageStudio.optionalMask') }}
            <input
              class="sr-only"
              type="file"
              :accept="accept"
              @change="handleMaskFile"
            >
          </label>
        </div>
      </div>

      <details class="group rounded-2xl border border-border/60">
        <summary class="cursor-pointer list-none px-4 py-3 text-sm font-medium">
          {{ t('imageStudio.advancedJson') }}
        </summary>
        <div class="border-t border-border/60 p-3">
          <Textarea
            :model-value="form.advancedJson"
            class="min-h-28 font-mono text-xs"
            placeholder="{ &quot;input_fidelity&quot;: &quot;high&quot; }"
            @update:model-value="updateForm({ advancedJson: $event })"
          />
        </div>
      </details>

      <Button
        type="submit"
        class="w-full"
        :disabled="disabled || loading"
      >
        <Loader2
          v-if="loading"
          class="mr-2 h-4 w-4 animate-spin"
        />
        <WandSparkles
          v-else
          class="mr-2 h-4 w-4"
        />
        {{ t('imageStudio.startGeneration') }}
      </Button>
    </form>
  </Card>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { Loader2, ScanLine, Upload, WandSparkles, X } from 'lucide-vue-next'
import { Button, Card, Input, Label, Select, SelectContent, SelectItem, SelectTrigger, SelectValue, Textarea } from '@/components/ui'
import { IMAGE_ASPECT_RATIO_PRESETS, IMAGE_SIZE_PRESETS } from '../constants'
import type { ImageApiKeyOption, ImageGenerationForm, ImageModelOption, ImageStudioSettings } from '../types'
import { imageFileAccept, validateInputImages } from '../utils/image-input'
import { resizeByWidthForAspectRatio } from '../utils/image-sizing'

const props = defineProps<{
  settings: ImageStudioSettings
  form: ImageGenerationForm
  apiKeys: ImageApiKeyOption[]
  models: ImageModelOption[]
  loading: boolean
}>()
const { t } = useI18n()

const emit = defineEmits<{
  submit: []
  refresh: []
  error: [message: string]
  'update:settings': [value: ImageStudioSettings]
  'update:form': [value: ImageGenerationForm]
}>()

const accept = imageFileAccept()
const disabled = computed(() => !props.settings.selectedKeyId || !props.settings.model || !props.form.prompt.trim())
const selectedAspectRatio = computed(() => {
  const match = props.form.size.trim().toLowerCase().match(/^(\d+)x(\d+)$/)
  if (!match) return ''
  const width = Number(match[1])
  const height = Number(match[2])
  if (!width || !height) return ''
  const currentRatio = width / height
  let closest = ''
  let closestDistance = Number.POSITIVE_INFINITY
  for (const preset of IMAGE_ASPECT_RATIO_PRESETS) {
    const [ratioWidth, ratioHeight] = preset.ratio.split(':').map(Number)
    const distance = Math.abs(Math.log(currentRatio / (ratioWidth / ratioHeight)))
    if (distance < closestDistance) {
      closest = preset.ratio
      closestDistance = distance
    }
  }
  return closestDistance <= 0.03 ? closest : ''
})

function updateSettings(value: Partial<ImageStudioSettings>) {
  emit('update:settings', { ...props.settings, ...value })
}

function updateForm(value: Partial<ImageGenerationForm>) {
  emit('update:form', { ...props.form, ...value })
}

function applyAspectRatio(preset: typeof IMAGE_ASPECT_RATIO_PRESETS[number]) {
  updateForm({
    size: resizeByWidthForAspectRatio(props.form.size, preset.ratio, preset.size),
  })
}

function updateCount(value: string | number | null) {
  updateForm({ count: Math.min(8, Math.max(1, Number(value) || 1)) })
}

function updateConcurrency(value: string | number | null) {
  updateSettings({ concurrency: Math.min(8, Math.max(1, Number(value) || 1)) })
}

function selectValue(event: Event) {
  return (event.target as HTMLSelectElement).value
}

function readFiles(event: Event) {
  const input = event.target as HTMLInputElement
  const files = Array.from(input.files || [])
  input.value = ''
  return files
}

function handleReferenceFiles(event: Event) {
  try {
    updateForm({ inputImages: [...props.form.inputImages, ...validateInputImages(readFiles(event))] })
  } catch (error) {
    emit('error', error instanceof Error ? error.message : t('imageStudio.invalidReference'))
  }
}

function handleMaskFile(event: Event) {
  try {
    updateForm({ maskImage: validateInputImages(readFiles(event))[0] || null })
  } catch (error) {
    emit('error', error instanceof Error ? error.message : t('imageStudio.invalidMask'))
  }
}

function removeReference(index: number) {
  const inputImages = props.form.inputImages.filter((_, current) => current !== index)
  updateForm({ inputImages, maskImage: inputImages.length ? props.form.maskImage : null })
}
</script>

<style scoped>
.native-select {
  @apply h-11 w-full rounded-xl border border-border bg-muted px-3 text-sm text-foreground outline-none transition focus:border-primary focus:ring-2 focus:ring-primary;
}
</style>
